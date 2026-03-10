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
    pub vault_updated: bool,
    pub pulled_note_ids: Vec<String>,
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
            fs::create_dir_all(self.vault_path.join(dir)).await?;
            fs::create_dir_all(target.join(dir)).await?;
        }

        let mut result = SyncResult {
            pushed: 0,
            pulled: 0,
            skipped: 0,
            errors: vec![],
            vault_updated: false,
            pulled_note_ids: vec![],
        };

        if let Err(e) = self.sync_vault_json(&target, &mut result).await {
            result.errors.push(format!("vault.json: {}", friendly_error(&e)));
        }

        for subdir in &["notebooks", "notes", "attachments"] {
            if let Err(e) = self.sync_dir(subdir, &target, &mut result).await {
                result.errors.push(format!("{subdir}: {}", friendly_error(&e)));
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
                fs::create_dir_all(&self.vault_path).await?;
                fs::copy(&remote, &local).await?;
                result.pulled += 1;
                result.vault_updated = true;
            }
            (true, true) => {
                let local_bytes = fs::read(&local).await?;
                let remote_bytes = fs::read(&remote).await?;
                if local_bytes != remote_bytes {
                    fs::copy(&remote, &local).await?;
                    result.pulled += 1;
                    result.vault_updated = true;
                } else {
                    result.skipped += 1;
                }
            }
            (false, false) => {}
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
            if !fname.ends_with(".json") {
                continue; // ignorar .DS_Store, .bin, .tmp, etc.
            }
            let remote_path = remote_dir.join(&fname);
            match compare_timestamps(&path, &remote_path).await {
                TimestampCmp::LocalNewer | TimestampCmp::RemoteMissing => {
                    if let Err(e) = copy_atomic(&path, &remote_path).await {
                        result.errors.push(format!("{fname}: {}", friendly_error(&e)));
                    } else {
                        result.pushed += 1;
                        sync_bin_sidecar(&local_dir, &remote_dir, &fname, Direction::Push).await;
                    }
                }
                TimestampCmp::RemoteNewer => {
                    if let Err(e) = copy_atomic(&remote_path, &path).await {
                        result.errors.push(format!("{fname}: {}", friendly_error(&e)));
                    } else {
                        result.pulled += 1;
                        sync_bin_sidecar(&local_dir, &remote_dir, &fname, Direction::Pull).await;
                        if subdir == "notes" {
                            let id = fname.trim_end_matches(".json").to_string();
                            result.pulled_note_ids.push(id);
                        }
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
            if !fname.ends_with(".json") {
                continue;
            }
            let local_path = local_dir.join(&fname);
            if !local_path.exists() {
                if let Err(e) = copy_atomic(&path, &local_path).await {
                    result.errors.push(format!("{fname}: {}", friendly_error(&e)));
                } else {
                    result.pulled += 1;
                    sync_bin_sidecar(&local_dir, &remote_dir, &fname, Direction::Pull).await;
                    if subdir == "notes" {
                        let id = fname.trim_end_matches(".json").to_string();
                        result.pulled_note_ids.push(id);
                    }
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
    let l = match read_updated_at(local).await {
        Ok(t) => t,
        Err(e) => return TimestampCmp::Error(e.to_string()),
    };
    // Si el remoto está vacío o corrupto, tratarlo como ausente → push local
    let r = match read_updated_at(remote).await {
        Ok(t) => t,
        Err(_) => return TimestampCmp::RemoteMissing,
    };
    if l > r {
        TimestampCmp::LocalNewer
    } else if r > l {
        TimestampCmp::RemoteNewer
    } else {
        TimestampCmp::Equal
    }
}

async fn read_updated_at(path: &Path) -> Result<DateTime<Utc>> {
    let bytes = fs::read(path).await?;
    let v: Value = serde_json::from_slice(&bytes)?;
    let ts_str = v["updated_at"]
        .as_str()
        .ok_or_else(|| anyhow!("missing updated_at in {}", path.display()))?;
    Ok(ts_str.parse::<DateTime<Utc>>()?)
}

/// Convierte un error anyhow a un mensaje legible por el usuario.
/// Detecta errores de permisos de SO (EPERM / EACCES) y los traduce.
fn friendly_error(e: &anyhow::Error) -> String {
    if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
        match io_err.kind() {
            std::io::ErrorKind::PermissionDenied => {
                return "sin permisos de escritura en la carpeta destino".to_string();
            }
            std::io::ErrorKind::NotFound => {
                return "carpeta destino no encontrada".to_string();
            }
            _ => {}
        }
    }
    e.to_string()
}

/// Copia src → dst de forma atómica: lee todo en memoria, escribe a .tmp, renombra.
/// Esto evita que un fallo a mitad deje el destino vacío (0 bytes).
async fn copy_atomic(src: &Path, dst: &Path) -> Result<()> {
    let bytes = fs::read(src).await?;
    if bytes.is_empty() {
        return Err(anyhow!("source file is empty: {}", src.display()));
    }
    let tmp = dst.with_extension("tmp");
    fs::write(&tmp, &bytes).await?;
    fs::rename(&tmp, dst).await?;
    Ok(())
}
