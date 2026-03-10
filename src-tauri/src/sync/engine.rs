use crate::sync::webdav::WebDavClient;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::Mutex;

fn default_fs() -> String {
    "fs".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SyncConfig {
    #[serde(default = "default_fs")]
    pub provider: String,
    #[serde(default)]
    pub target_path: Option<String>,
    #[serde(default)]
    pub webdav_url: Option<String>,
    #[serde(default)]
    pub webdav_username: Option<String>,
    #[serde(default)]
    pub webdav_password: Option<String>,
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

        match config.provider.as_str() {
            "webdav" | "nextcloud" => self.do_sync_webdav(&config).await,
            _ => self.do_sync_fs(&config).await,
        }
    }

    // ── Filesystem provider ────────────────────────────────────────────────────

    async fn do_sync_fs(&self, config: &SyncConfig) -> Result<SyncResult> {
        let target_path = config
            .target_path
            .as_deref()
            .ok_or_else(|| anyhow!("target_path not configured"))?;
        let target = PathBuf::from(target_path);

        for dir in &["notebooks", "notes", "attachments"] {
            fs::create_dir_all(self.vault_path.join(dir)).await?;
            fs::create_dir_all(target.join(dir)).await?;
        }

        let mut result = SyncResult {
            pushed: 0, pulled: 0, skipped: 0,
            errors: vec![], vault_updated: false, pulled_note_ids: vec![],
        };

        if let Err(e) = self.sync_vault_json_fs(&target, &mut result).await {
            result.errors.push(format!("vault.json: {}", friendly_error(&e)));
        }
        for subdir in &["notebooks", "notes", "attachments"] {
            if let Err(e) = self.sync_dir_fs(subdir, &target, &mut result).await {
                result.errors.push(format!("{subdir}: {}", friendly_error(&e)));
            }
        }
        Ok(result)
    }

    async fn sync_vault_json_fs(&self, target: &Path, result: &mut SyncResult) -> Result<()> {
        let local = self.vault_path.join("vault.json");
        let remote = target.join("vault.json");
        match (local.exists(), remote.exists()) {
            (true, false) => { copy_atomic(&local, &remote).await?; result.pushed += 1; }
            (false, true) => {
                fs::create_dir_all(&self.vault_path).await?;
                copy_atomic(&remote, &local).await?;
                result.pulled += 1; result.vault_updated = true;
            }
            (true, true) => {
                let local_bytes = fs::read(&local).await?;
                let remote_bytes = fs::read(&remote).await?;
                if local_bytes != remote_bytes {
                    copy_atomic(&remote, &local).await?;
                    result.pulled += 1; result.vault_updated = true;
                } else { result.skipped += 1; }
            }
            (false, false) => {}
        }
        Ok(())
    }

    async fn sync_dir_fs(&self, subdir: &str, target: &Path, result: &mut SyncResult) -> Result<()> {
        let local_dir = self.vault_path.join(subdir);
        let remote_dir = target.join(subdir);

        let mut local_rd = match fs::read_dir(&local_dir).await { Ok(rd) => rd, Err(_) => return Ok(()) };
        while let Some(entry) = local_rd.next_entry().await? {
            let path = entry.path();
            let fname = path.file_name().unwrap().to_string_lossy().to_string();
            if !fname.ends_with(".json") { continue; }
            let remote_path = remote_dir.join(&fname);
            match compare_timestamps(&path, &remote_path).await {
                TimestampCmp::LocalNewer | TimestampCmp::RemoteMissing => {
                    if let Err(e) = copy_atomic(&path, &remote_path).await {
                        result.errors.push(format!("{fname}: {}", friendly_error(&e)));
                    } else { result.pushed += 1; sync_bin_sidecar(&local_dir, &remote_dir, &fname, Direction::Push).await; }
                }
                TimestampCmp::RemoteNewer => {
                    if let Err(e) = copy_atomic(&remote_path, &path).await {
                        result.errors.push(format!("{fname}: {}", friendly_error(&e)));
                    } else {
                        result.pulled += 1;
                        sync_bin_sidecar(&local_dir, &remote_dir, &fname, Direction::Pull).await;
                        if subdir == "notes" { result.pulled_note_ids.push(fname.trim_end_matches(".json").to_string()); }
                    }
                }
                TimestampCmp::Equal => { result.skipped += 1; }
                TimestampCmp::Error(e) => { result.errors.push(e); }
            }
        }

        let mut remote_rd = match fs::read_dir(&remote_dir).await { Ok(rd) => rd, Err(_) => return Ok(()) };
        while let Some(entry) = remote_rd.next_entry().await? {
            let path = entry.path();
            let fname = path.file_name().unwrap().to_string_lossy().to_string();
            if !fname.ends_with(".json") { continue; }
            let local_path = local_dir.join(&fname);
            if !local_path.exists() {
                if let Err(e) = copy_atomic(&path, &local_path).await {
                    result.errors.push(format!("{fname}: {}", friendly_error(&e)));
                } else {
                    result.pulled += 1;
                    sync_bin_sidecar(&local_dir, &remote_dir, &fname, Direction::Pull).await;
                    if subdir == "notes" { result.pulled_note_ids.push(fname.trim_end_matches(".json").to_string()); }
                }
            }
        }
        Ok(())
    }

    // ── WebDAV provider ────────────────────────────────────────────────────────

    async fn do_sync_webdav(&self, config: &SyncConfig) -> Result<SyncResult> {
        let url = config.webdav_url.as_deref().ok_or_else(|| anyhow!("webdav_url not configured"))?;
        let username = config.webdav_username.as_deref().unwrap_or("");
        let password = config.webdav_password.as_deref().unwrap_or("");
        let client = WebDavClient::new(url, username, password)?;

        for dir in &["notebooks", "notes", "attachments"] {
            fs::create_dir_all(self.vault_path.join(dir)).await?;
            let _ = client.ensure_dir(dir).await;
        }

        let mut result = SyncResult {
            pushed: 0, pulled: 0, skipped: 0,
            errors: vec![], vault_updated: false, pulled_note_ids: vec![],
        };

        if let Err(e) = self.sync_vault_json_webdav(&client, &mut result).await {
            result.errors.push(format!("vault.json: {e}"));
        }
        for subdir in &["notebooks", "notes", "attachments"] {
            if let Err(e) = self.sync_dir_webdav(subdir, &client, &mut result).await {
                result.errors.push(format!("{subdir}: {e}"));
            }
        }
        Ok(result)
    }

    async fn sync_vault_json_webdav(&self, client: &WebDavClient, result: &mut SyncResult) -> Result<()> {
        let local = self.vault_path.join("vault.json");
        let remote_exists = client.file_exists("vault.json").await;
        match (local.exists(), remote_exists) {
            (true, false) => { client.write_bytes("vault.json", fs::read(&local).await?).await?; result.pushed += 1; }
            (false, true) => {
                let bytes = client.read_bytes("vault.json").await?;
                fs::create_dir_all(&self.vault_path).await?;
                copy_atomic_bytes(&local, bytes).await?;
                result.pulled += 1; result.vault_updated = true;
            }
            (true, true) => {
                let local_bytes = fs::read(&local).await?;
                let remote_bytes = client.read_bytes("vault.json").await?;
                if local_bytes != remote_bytes {
                    copy_atomic_bytes(&local, remote_bytes).await?;
                    result.pulled += 1; result.vault_updated = true;
                } else { result.skipped += 1; }
            }
            (false, false) => {}
        }
        Ok(())
    }

    async fn sync_dir_webdav(&self, subdir: &str, client: &WebDavClient, result: &mut SyncResult) -> Result<()> {
        let local_dir = self.vault_path.join(subdir);
        let remote_files: HashSet<String> = client.list_json_files(subdir).await.unwrap_or_default().into_iter().collect();
        let mut local_files: HashSet<String> = HashSet::new();

        let mut local_rd = match fs::read_dir(&local_dir).await { Ok(rd) => rd, Err(_) => return Ok(()) };
        while let Some(entry) = local_rd.next_entry().await? {
            let path = entry.path();
            let fname = path.file_name().unwrap().to_string_lossy().to_string();
            if !fname.ends_with(".json") { continue; }
            local_files.insert(fname.clone());
            let remote_path = format!("{}/{}", subdir, fname);

            if !remote_files.contains(&fname) {
                match fs::read(&path).await {
                    Ok(bytes) => {
                        if let Err(e) = client.write_bytes(&remote_path, bytes).await { result.errors.push(format!("{fname}: {e}")); }
                        else { result.pushed += 1; self.sync_bin_webdav(subdir, &fname, client, Direction::Push).await; }
                    }
                    Err(e) => result.errors.push(format!("{fname}: {e}")),
                }
            } else {
                let local_ts = match read_updated_at(&path).await {
                    Ok(t) => t,
                    Err(e) => { result.errors.push(format!("{fname}: {e}")); continue; }
                };
                let remote_ts = match client.read_updated_at(&remote_path).await {
                    Ok(t) => t,
                    Err(_) => {
                        match fs::read(&path).await {
                            Ok(bytes) => { if let Err(e) = client.write_bytes(&remote_path, bytes).await { result.errors.push(format!("{fname}: {e}")); } else { result.pushed += 1; } }
                            Err(e) => result.errors.push(format!("{fname}: {e}")),
                        }
                        continue;
                    }
                };

                if local_ts > remote_ts {
                    match fs::read(&path).await {
                        Ok(bytes) => {
                            if let Err(e) = client.write_bytes(&remote_path, bytes).await { result.errors.push(format!("{fname}: {e}")); }
                            else { result.pushed += 1; self.sync_bin_webdav(subdir, &fname, client, Direction::Push).await; }
                        }
                        Err(e) => result.errors.push(format!("{fname}: {e}")),
                    }
                } else if remote_ts > local_ts {
                    match client.read_bytes(&remote_path).await {
                        Ok(bytes) => {
                            if let Err(e) = copy_atomic_bytes(&path, bytes).await { result.errors.push(format!("{fname}: {e}")); }
                            else {
                                result.pulled += 1;
                                self.sync_bin_webdav(subdir, &fname, client, Direction::Pull).await;
                                if subdir == "notes" { result.pulled_note_ids.push(fname.trim_end_matches(".json").to_string()); }
                            }
                        }
                        Err(e) => result.errors.push(format!("{fname}: {e}")),
                    }
                } else { result.skipped += 1; }
            }
        }

        for fname in &remote_files {
            if !local_files.contains(fname) {
                let remote_path = format!("{}/{}", subdir, fname);
                match client.read_bytes(&remote_path).await {
                    Ok(bytes) => {
                        let local_path = local_dir.join(fname);
                        if let Err(e) = copy_atomic_bytes(&local_path, bytes).await { result.errors.push(format!("{fname}: {e}")); }
                        else {
                            result.pulled += 1;
                            self.sync_bin_webdav(subdir, fname, client, Direction::Pull).await;
                            if subdir == "notes" { result.pulled_note_ids.push(fname.trim_end_matches(".json").to_string()); }
                        }
                    }
                    Err(e) => result.errors.push(format!("{fname}: {e}")),
                }
            }
        }
        Ok(())
    }

    async fn sync_bin_webdav(&self, subdir: &str, json_fname: &str, client: &WebDavClient, dir: Direction) {
        let bin_name = json_fname.replace(".json", ".bin");
        let local_path = self.vault_path.join(subdir).join(&bin_name);
        let remote_path = format!("{}/{}", subdir, bin_name);
        match dir {
            Direction::Push => {
                if local_path.exists() {
                    if let Ok(bytes) = fs::read(&local_path).await { let _ = client.write_bytes(&remote_path, bytes).await; }
                }
            }
            Direction::Pull => {
                if client.file_exists(&remote_path).await {
                    if let Ok(bytes) = client.read_bytes(&remote_path).await { let _ = fs::write(&local_path, bytes).await; }
                }
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

enum Direction { Push, Pull }

async fn sync_bin_sidecar(local_dir: &Path, remote_dir: &Path, json_fname: &str, dir: Direction) {
    let bin_name = json_fname.replace(".json", ".bin");
    let (src, dst) = match dir {
        Direction::Push => (local_dir.join(&bin_name), remote_dir.join(&bin_name)),
        Direction::Pull => (remote_dir.join(&bin_name), local_dir.join(&bin_name)),
    };
    if src.exists() { let _ = fs::copy(src, dst).await; }
}

enum TimestampCmp { LocalNewer, RemoteNewer, Equal, RemoteMissing, Error(String) }

async fn compare_timestamps(local: &Path, remote: &Path) -> TimestampCmp {
    if !remote.exists() { return TimestampCmp::RemoteMissing; }
    let l = match read_updated_at(local).await { Ok(t) => t, Err(e) => return TimestampCmp::Error(e.to_string()) };
    let r = match read_updated_at(remote).await { Ok(t) => t, Err(_) => return TimestampCmp::RemoteMissing };
    if l > r { TimestampCmp::LocalNewer } else if r > l { TimestampCmp::RemoteNewer } else { TimestampCmp::Equal }
}

async fn read_updated_at(path: &Path) -> Result<DateTime<Utc>> {
    let bytes = fs::read(path).await?;
    let v: Value = serde_json::from_slice(&bytes)?;
    let ts_str = v["updated_at"].as_str().ok_or_else(|| anyhow!("missing updated_at in {}", path.display()))?;
    Ok(ts_str.parse::<DateTime<Utc>>()?)
}

fn friendly_error(e: &anyhow::Error) -> String {
    if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
        match io_err.kind() {
            std::io::ErrorKind::PermissionDenied => return "sin permisos de escritura en la carpeta destino".to_string(),
            std::io::ErrorKind::NotFound => return "carpeta destino no encontrada".to_string(),
            _ => {}
        }
    }
    e.to_string()
}

async fn copy_atomic(src: &Path, dst: &Path) -> Result<()> {
    let bytes = fs::read(src).await?;
    if bytes.is_empty() { return Err(anyhow!("source file is empty: {}", src.display())); }
    copy_atomic_bytes(dst, bytes).await
}

async fn copy_atomic_bytes(dst: &Path, bytes: Vec<u8>) -> Result<()> {
    if bytes.is_empty() { return Err(anyhow!("received empty content for {}", dst.display())); }
    let tmp = dst.with_extension("tmp");
    fs::write(&tmp, &bytes).await?;
    fs::rename(&tmp, dst).await?;
    Ok(())
}
