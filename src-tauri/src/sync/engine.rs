use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SyncConfig {
    pub target_path: String,
    pub auto_sync_interval_secs: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncResult {
    pub pushed: u32,
    pub pulled: u32,
    pub skipped: u32,
    pub errors: Vec<String>,
}

pub struct SyncEngine {
    vault_path: PathBuf,
    config_path: PathBuf,
    syncing: Mutex<bool>,
}

impl SyncEngine {
    pub fn new(vault_path: PathBuf, config_path: PathBuf) -> Self {
        Self {
            vault_path,
            config_path,
            syncing: Mutex::new(false),
        }
    }

    pub async fn load_config(&self) -> Option<SyncConfig> {
        let bytes = fs::read(&self.config_path).await.ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    pub async fn save_config(&self, config: &SyncConfig) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(config)?;
        fs::write(&self.config_path, &bytes).await?;
        Ok(())
    }

    pub async fn clear_config(&self) -> Result<()> {
        if self.config_path.exists() {
            fs::remove_file(&self.config_path).await?;
        }
        Ok(())
    }

    pub async fn run(&self) -> Result<SyncResult> {
        {
            let mut guard = self.syncing.lock().await;
            if *guard {
                return Err(anyhow!("Sync already in progress"));
            }
            *guard = true;
        }
        let result = self.do_sync().await;
        *self.syncing.lock().await = false;
        result
    }

    async fn do_sync(&self) -> Result<SyncResult> {
        let config = self
            .load_config()
            .await
            .ok_or_else(|| anyhow!("Sync not configured"))?;

        let target = PathBuf::from(&config.target_path);
        for dir in &["notebooks", "notes", "attachments"] {
            fs::create_dir_all(target.join(dir)).await?;
        }

        let mut result = SyncResult {
            pushed: 0,
            pulled: 0,
            skipped: 0,
            errors: vec![],
        };

        // Sync vault.json (uses mtime, not updated_at)
        if let Err(e) = self.sync_vault_json(&target, &mut result).await {
            result.errors.push(format!("vault.json: {e}"));
        }

        for subdir in &["notebooks", "notes", "attachments"] {
            if let Err(e) = self.sync_dir(subdir, &target, &mut result).await {
                result.errors.push(format!("{subdir}: {e}"));
            }
        }

        Ok(result)
    }

    async fn sync_vault_json(&self, target: &Path, result: &mut SyncResult) -> Result<()> {
        let local = self.vault_path.join("vault.json");
        let remote = target.join("vault.json");

        match (local.exists(), remote.exists()) {
            (true, false) => {
                fs::copy(&local, &remote).await?;
                result.pushed += 1;
            }
            (false, true) => {
                // Bootstrap: new device has no vault yet → pull remote vault
                fs::create_dir_all(&self.vault_path).await?;
                fs::copy(&remote, &local).await?;
                result.pulled += 1;
            }
            (true, true) => {
                // Compare mtime; remote wins if newer (vault_change_password propagation)
                let local_mtime = mtime(&local).await?;
                let remote_mtime = mtime(&remote).await?;
                if remote_mtime > local_mtime {
                    fs::copy(&remote, &local).await?;
                    result.pulled += 1;
                } else if local_mtime > remote_mtime {
                    fs::copy(&local, &remote).await?;
                    result.pushed += 1;
                } else {
                    result.skipped += 1;
                }
            }
            (false, false) => {} // nothing to do
        }
        Ok(())
    }

    async fn sync_dir(
        &self,
        subdir: &str,
        target: &Path,
        result: &mut SyncResult,
    ) -> Result<()> {
        let local_dir = self.vault_path.join(subdir);
        let remote_dir = target.join(subdir);

        // Push: local → remote
        let mut local_rd = match fs::read_dir(&local_dir).await {
            Ok(rd) => rd,
            Err(_) => return Ok(()),
        };
        while let Some(entry) = local_rd.next_entry().await? {
            let path = entry.path();
            let fname = path.file_name().unwrap().to_string_lossy().to_string();
            if fname.ends_with(".bin") {
                continue; // handled as sidecar of .json
            }
            let remote_path = remote_dir.join(&fname);
            match compare_timestamps(&path, &remote_path).await {
                TimestampCmp::LocalNewer | TimestampCmp::RemoteMissing => {
                    if let Err(e) = fs::copy(&path, &remote_path).await {
                        result.errors.push(format!("push {fname}: {e}"));
                    } else {
                        result.pushed += 1;
                        sync_bin_sidecar(&local_dir, &remote_dir, &fname, Direction::Push).await;
                    }
                }
                TimestampCmp::RemoteNewer => {
                    if let Err(e) = fs::copy(&remote_path, &path).await {
                        result.errors.push(format!("pull {fname}: {e}"));
                    } else {
                        result.pulled += 1;
                        sync_bin_sidecar(&local_dir, &remote_dir, &fname, Direction::Pull).await;
                    }
                }
                TimestampCmp::Equal => {
                    result.skipped += 1;
                }
                TimestampCmp::Error(e) => {
                    result.errors.push(e);
                }
            }
        }

        // Pull: remote-only files → local
        let mut remote_rd = match fs::read_dir(&remote_dir).await {
            Ok(rd) => rd,
            Err(_) => return Ok(()),
        };
        while let Some(entry) = remote_rd.next_entry().await? {
            let path = entry.path();
            let fname = path.file_name().unwrap().to_string_lossy().to_string();
            if fname.ends_with(".bin") {
                continue;
            }
            let local_path = local_dir.join(&fname);
            if !local_path.exists() {
                if let Err(e) = fs::copy(&path, &local_path).await {
                    result.errors.push(format!("pull-new {fname}: {e}"));
                } else {
                    result.pulled += 1;
                    sync_bin_sidecar(&local_dir, &remote_dir, &fname, Direction::Pull).await;
                }
            }
        }

        Ok(())
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

enum Direction {
    Push,
    Pull,
}

async fn sync_bin_sidecar(local_dir: &Path, remote_dir: &Path, json_fname: &str, dir: Direction) {
    if !json_fname.ends_with(".json") {
        return;
    }
    let bin_name = json_fname.replace(".json", ".bin");
    let (src, dst) = match dir {
        Direction::Push => (local_dir.join(&bin_name), remote_dir.join(&bin_name)),
        Direction::Pull => (remote_dir.join(&bin_name), local_dir.join(&bin_name)),
    };
    if src.exists() {
        let _ = fs::copy(src, dst).await;
    }
}

enum TimestampCmp {
    LocalNewer,
    RemoteNewer,
    Equal,
    RemoteMissing,
    Error(String),
}

async fn compare_timestamps(local: &Path, remote: &Path) -> TimestampCmp {
    if !remote.exists() {
        return TimestampCmp::RemoteMissing;
    }
    match (read_updated_at(local).await, read_updated_at(remote).await) {
        (Ok(l), Ok(r)) => {
            if l > r {
                TimestampCmp::LocalNewer
            } else if r > l {
                TimestampCmp::RemoteNewer
            } else {
                TimestampCmp::Equal
            }
        }
        (Err(e), _) | (_, Err(e)) => TimestampCmp::Error(e.to_string()),
    }
}

async fn mtime(path: &Path) -> Result<std::time::SystemTime> {
    let meta = fs::metadata(path).await?;
    Ok(meta.modified()?)
}

async fn read_updated_at(path: &Path) -> Result<DateTime<Utc>> {
    let bytes = fs::read(path).await?;
    let v: Value = serde_json::from_slice(&bytes)?;
    let ts_str = v["updated_at"]
        .as_str()
        .ok_or_else(|| anyhow!("missing updated_at in {}", path.display()))?;
    Ok(ts_str.parse::<DateTime<Utc>>()?)
}
