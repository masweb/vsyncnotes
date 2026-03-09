use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use chrono::{DateTime, Utc};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::Mutex;
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::crypto::envelope::{
    decrypt, derive_key, encrypt, generate_dek, make_key_check, verify_key_check,
};
use crate::models::{
    attachment::Attachment,
    note::{Note, NoteMeta, NoteSearchResult},
    notebook::Notebook,
    vault::VaultMeta,
};

use super::repo::StorageRepo;

// ── On-disk encrypted structs (private to this module) ───────────────────────

#[derive(Serialize, Deserialize)]
struct EncryptedNote {
    id: Uuid,
    notebook_id: Uuid,       // plaintext for filtering
    title_encrypted: String,
    nonce_title: String,
    body_encrypted: String,
    nonce_body: String,
    dek_encrypted: String,   // DEK encrypted with master_key
    nonce_dek: String,
    body_format: String,
    sort_order: i32,
    is_pinned: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
struct EncryptedAttachmentMeta {
    id: Uuid,
    note_id: Uuid,
    filename: String,
    mime: String,
    size_bytes: u64,
    hash_sha256: String,
    dek_encrypted: String,
    nonce_dek: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

// ── Pure helpers ──────────────────────────────────────────────────────────────

fn encrypt_note(note: &Note, master_key: &[u8; 32]) -> Result<EncryptedNote> {
    let dek = generate_dek();
    let (title_encrypted, nonce_title) = encrypt(note.title.as_bytes(), &dek)?;
    let body_bytes = serde_json::to_vec(&note.body)?;
    let (body_encrypted, nonce_body) = encrypt(&body_bytes, &dek)?;
    let (dek_encrypted, nonce_dek) = encrypt(&*dek, master_key)?;
    Ok(EncryptedNote {
        id: note.id,
        notebook_id: note.notebook_id,
        title_encrypted,
        nonce_title,
        body_encrypted,
        nonce_body,
        dek_encrypted,
        nonce_dek,
        body_format: note.body_format.clone(),
        sort_order: note.sort_order,
        is_pinned: note.is_pinned,
        created_at: note.created_at,
        updated_at: note.updated_at,
    })
}

fn decrypt_note(enc: &EncryptedNote, master_key: &[u8; 32]) -> Result<Note> {
    let dek_bytes = decrypt(&enc.dek_encrypted, &enc.nonce_dek, master_key)?;
    let dek: [u8; 32] = dek_bytes
        .try_into()
        .map_err(|_| anyhow!("Invalid DEK length"))?;
    let title = String::from_utf8(decrypt(&enc.title_encrypted, &enc.nonce_title, &dek)?)?;
    let body: serde_json::Value =
        serde_json::from_slice(&decrypt(&enc.body_encrypted, &enc.nonce_body, &dek)?)?;
    Ok(Note {
        id: enc.id,
        notebook_id: enc.notebook_id,
        title,
        body,
        body_format: enc.body_format.clone(),
        sort_order: enc.sort_order,
        is_pinned: enc.is_pinned,
        created_at: enc.created_at,
        updated_at: enc.updated_at,
    })
}

fn decrypt_note_meta(enc: &EncryptedNote, master_key: &[u8; 32]) -> Result<NoteMeta> {
    let dek_bytes = decrypt(&enc.dek_encrypted, &enc.nonce_dek, master_key)?;
    let dek: [u8; 32] = dek_bytes
        .try_into()
        .map_err(|_| anyhow!("Invalid DEK length"))?;
    let title = String::from_utf8(decrypt(&enc.title_encrypted, &enc.nonce_title, &dek)?)?;
    Ok(NoteMeta {
        id: enc.id,
        notebook_id: enc.notebook_id,
        title,
        sort_order: enc.sort_order,
        is_pinned: enc.is_pinned,
        created_at: enc.created_at,
        updated_at: enc.updated_at,
    })
}

// ── FsRepo ────────────────────────────────────────────────────────────────────

pub struct FsRepo {
    vault_path: PathBuf,
    master_key: Mutex<Option<Zeroizing<[u8; 32]>>>,
    search_index: Mutex<HashMap<Uuid, NoteSearchResult>>,
}

impl FsRepo {
    pub fn new(vault_path: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(vault_path.join("notebooks"))?;
        std::fs::create_dir_all(vault_path.join("notes"))?;
        std::fs::create_dir_all(vault_path.join("attachments"))?;
        Ok(Self {
            vault_path,
            master_key: Mutex::new(None),
            search_index: Mutex::new(HashMap::new()),
        })
    }

    // ── Path helpers ──────────────────────────────────────────────────────────

    fn notebook_path(&self, id: Uuid) -> PathBuf {
        self.vault_path.join("notebooks").join(format!("{id}.json"))
    }
    fn note_path(&self, id: Uuid) -> PathBuf {
        self.vault_path.join("notes").join(format!("{id}.json"))
    }
    fn attachment_meta_path(&self, id: Uuid) -> PathBuf {
        self.vault_path.join("attachments").join(format!("{id}.json"))
    }
    fn attachment_data_path(&self, id: Uuid) -> PathBuf {
        self.vault_path.join("attachments").join(format!("{id}.bin"))
    }
    fn vault_meta_path(&self) -> PathBuf {
        self.vault_path.join("vault.json")
    }

    // ── Master key access ─────────────────────────────────────────────────────

    async fn get_master_key(&self) -> Result<Zeroizing<[u8; 32]>> {
        self.master_key
            .lock()
            .await
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow!("Vault is locked"))
    }

    // ── Vault operations ──────────────────────────────────────────────────────

    pub async fn vault_create(&self, password: &str) -> Result<()> {
        if self.vault_meta_path().exists() {
            return Err(anyhow!("Vault already exists — use vault_unlock"));
        }
        let mut salt_bytes = [0u8; 16];
        OsRng.fill_bytes(&mut salt_bytes);
        let master_key = derive_key(password, &salt_bytes)?;
        let (key_check, key_check_nonce) = make_key_check(&master_key)?;
        let meta = VaultMeta {
            version: 1,
            salt: B64.encode(salt_bytes),
            key_check,
            key_check_nonce,
        };
        tokio::fs::write(self.vault_meta_path(), serde_json::to_string_pretty(&meta)?).await?;
        *self.master_key.lock().await = Some(master_key);
        Ok(())
    }

    pub async fn vault_unlock(&self, password: &str) -> Result<()> {
        let content = tokio::fs::read_to_string(self.vault_meta_path())
            .await
            .context("Vault not found — call vault_create first")?;
        let meta: VaultMeta = serde_json::from_str(&content)?;
        let salt_bytes = B64.decode(&meta.salt).context("Invalid vault salt")?;
        let candidate_key = derive_key(password, &salt_bytes)?;
        if !verify_key_check(&meta.key_check, &meta.key_check_nonce, &candidate_key) {
            return Err(anyhow!("Wrong password"));
        }
        *self.master_key.lock().await = Some(candidate_key);
        let _ = self.build_search_index().await;
        Ok(())
    }

    pub async fn vault_lock(&self) {
        *self.master_key.lock().await = None;
        self.search_index.lock().await.clear();
    }

    pub async fn vault_change_password(
        &self,
        old_password: &str,
        new_password: &str,
    ) -> Result<()> {
        // Verify old password
        let content = tokio::fs::read_to_string(self.vault_meta_path()).await?;
        let meta: VaultMeta = serde_json::from_str(&content)?;
        let old_salt = B64.decode(&meta.salt)?;
        let old_key = derive_key(old_password, &old_salt)?;
        if !verify_key_check(&meta.key_check, &meta.key_check_nonce, &old_key) {
            return Err(anyhow!("Wrong current password"));
        }

        // Derive new key
        let mut new_salt_bytes = [0u8; 16];
        OsRng.fill_bytes(&mut new_salt_bytes);
        let new_key = derive_key(new_password, &new_salt_bytes)?;

        // Re-encrypt all note DEKs
        let notes_dir = self.vault_path.join("notes");
        let mut entries = tokio::fs::read_dir(&notes_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let raw = tokio::fs::read_to_string(&path).await?;
                let mut enc: EncryptedNote = serde_json::from_str(&raw)?;
                let dek_bytes = decrypt(&enc.dek_encrypted, &enc.nonce_dek, &old_key)?;
                let dek: [u8; 32] = dek_bytes
                    .try_into()
                    .map_err(|_| anyhow!("Invalid DEK length"))?;
                let (new_dek_enc, new_nonce_dek) = encrypt(&dek, &new_key)?;
                enc.dek_encrypted = new_dek_enc;
                enc.nonce_dek = new_nonce_dek;
                tokio::fs::write(&path, serde_json::to_string_pretty(&enc)?).await?;
            }
        }

        // Re-encrypt all attachment DEKs
        let atts_dir = self.vault_path.join("attachments");
        let mut entries = tokio::fs::read_dir(&atts_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let raw = tokio::fs::read_to_string(&path).await?;
                let mut enc: EncryptedAttachmentMeta = serde_json::from_str(&raw)?;
                let dek_bytes = decrypt(&enc.dek_encrypted, &enc.nonce_dek, &old_key)?;
                let dek: [u8; 32] = dek_bytes
                    .try_into()
                    .map_err(|_| anyhow!("Invalid DEK length"))?;
                let (new_dek_enc, new_nonce_dek) = encrypt(&dek, &new_key)?;
                enc.dek_encrypted = new_dek_enc;
                enc.nonce_dek = new_nonce_dek;
                tokio::fs::write(&path, serde_json::to_string_pretty(&enc)?).await?;
            }
        }

        // Update vault.json
        let (key_check, key_check_nonce) = make_key_check(&new_key)?;
        let new_meta = VaultMeta {
            version: meta.version,
            salt: B64.encode(new_salt_bytes),
            key_check,
            key_check_nonce,
        };
        tokio::fs::write(
            self.vault_meta_path(),
            serde_json::to_string_pretty(&new_meta)?,
        )
        .await?;

        *self.master_key.lock().await = Some(new_key);
        Ok(())
    }

    pub async fn is_locked(&self) -> bool {
        self.master_key.lock().await.is_none()
    }

    pub async fn vault_exists(&self) -> bool {
        self.vault_meta_path().exists()
    }

    // ── Search index ──────────────────────────────────────────────────────────

    async fn build_search_index(&self) -> Result<()> {
        let master_key = self.get_master_key().await?;
        let dir = self.vault_path.join("notes");
        let mut entries = tokio::fs::read_dir(&dir)
            .await
            .context("Failed to read notes directory")?;
        let mut index = self.search_index.lock().await;
        index.clear();
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    if let Ok(enc) = serde_json::from_str::<EncryptedNote>(&content) {
                        if let Ok(meta) = decrypt_note_meta(&enc, &master_key) {
                            index.insert(meta.id, NoteSearchResult {
                                id: meta.id,
                                notebook_id: meta.notebook_id,
                                title: meta.title,
                                updated_at: meta.updated_at,
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    }

}

// ── StorageRepo impl ──────────────────────────────────────────────────────────

#[async_trait]
impl StorageRepo for FsRepo {
    // ── Notebooks (plaintext — no sensitive content) ──────────────────────────

    async fn list_notebooks(&self) -> Result<Vec<Notebook>> {
        let dir = self.vault_path.join("notebooks");
        let mut entries = tokio::fs::read_dir(&dir)
            .await
            .context("Failed to read notebooks directory")?;
        let mut notebooks = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let content = tokio::fs::read_to_string(&path)
                    .await
                    .with_context(|| format!("Failed to read {path:?}"))?;
                let nb: Notebook = serde_json::from_str(&content)
                    .with_context(|| format!("Failed to parse {path:?}"))?;
                notebooks.push(nb);
            }
        }
        notebooks.sort_by_key(|nb| nb.sort_order);
        Ok(notebooks)
    }

    async fn get_notebook(&self, id: Uuid) -> Result<Notebook> {
        let content = tokio::fs::read_to_string(self.notebook_path(id))
            .await
            .with_context(|| format!("Notebook {id} not found"))?;
        Ok(serde_json::from_str(&content)?)
    }

    async fn save_notebook(&self, nb: &Notebook) -> Result<()> {
        tokio::fs::write(self.notebook_path(nb.id), serde_json::to_string_pretty(nb)?).await?;
        Ok(())
    }

    async fn delete_notebook(&self, id: Uuid) -> Result<()> {
        tokio::fs::remove_file(self.notebook_path(id))
            .await
            .with_context(|| format!("Notebook {id} not found"))?;
        Ok(())
    }

    // ── Notes (encrypted) ─────────────────────────────────────────────────────

    async fn list_notes(&self, notebook_id: Uuid) -> Result<Vec<NoteMeta>> {
        let master_key = self.get_master_key().await?;
        let dir = self.vault_path.join("notes");
        let mut entries = tokio::fs::read_dir(&dir)
            .await
            .context("Failed to read notes directory")?;
        let mut notes = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let content = tokio::fs::read_to_string(&path).await?;
                let enc: EncryptedNote = serde_json::from_str(&content)?;
                if enc.notebook_id == notebook_id {
                    notes.push(decrypt_note_meta(&enc, &master_key)?);
                }
            }
        }
        notes.sort_by(|a, b| {
            b.is_pinned
                .cmp(&a.is_pinned)
                .then_with(|| b.updated_at.cmp(&a.updated_at))
        });
        Ok(notes)
    }

    async fn get_note(&self, id: Uuid) -> Result<Note> {
        let master_key = self.get_master_key().await?;
        let content = tokio::fs::read_to_string(self.note_path(id))
            .await
            .with_context(|| format!("Note {id} not found"))?;
        let enc: EncryptedNote = serde_json::from_str(&content)?;
        decrypt_note(&enc, &master_key)
    }

    async fn save_note(&self, note: &Note) -> Result<()> {
        let master_key = self.get_master_key().await?;
        let enc = encrypt_note(note, &master_key)?;
        tokio::fs::write(self.note_path(note.id), serde_json::to_string_pretty(&enc)?).await?;
        self.search_index.lock().await.insert(note.id, NoteSearchResult {
            id: note.id,
            notebook_id: note.notebook_id,
            title: note.title.clone(),
            updated_at: note.updated_at,
        });
        Ok(())
    }

    async fn delete_note(&self, id: Uuid) -> Result<()> {
        tokio::fs::remove_file(self.note_path(id))
            .await
            .with_context(|| format!("Note {id} not found"))?;
        self.search_index.lock().await.remove(&id);
        Ok(())
    }

    async fn search_notes(&self, query: &str) -> Result<Vec<NoteSearchResult>> {
        let query_lower = query.to_lowercase();
        let index = self.search_index.lock().await;
        let mut results: Vec<NoteSearchResult> = index
            .values()
            .filter(|r| r.title.to_lowercase().contains(&query_lower))
            .cloned()
            .collect();
        results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        results.truncate(20);
        Ok(results)
    }

    // ── Attachments (binary data encrypted) ───────────────────────────────────

    async fn save_attachment(&self, att: &Attachment, data: &[u8]) -> Result<()> {
        let master_key = self.get_master_key().await?;
        let dek = generate_dek();
        let (data_encrypted, nonce_data) = encrypt(data, &dek)?;
        let (dek_encrypted, nonce_dek) = encrypt(&*dek, &master_key)?;

        // Binary: store as base64 in a .bin file (simplest for now)
        tokio::fs::write(self.attachment_data_path(att.id), data_encrypted).await?;

        let enc_meta = EncryptedAttachmentMeta {
            id: att.id,
            note_id: att.note_id,
            filename: att.filename.clone(),
            mime: att.mime.clone(),
            size_bytes: att.size_bytes,
            hash_sha256: att.hash_sha256.clone(),
            dek_encrypted,
            nonce_dek,
            created_at: att.created_at,
            updated_at: att.updated_at,
        };
        // Store nonce_data alongside the metadata so we can decrypt later
        #[derive(Serialize)]
        struct WithNonce<'a> {
            #[serde(flatten)]
            meta: &'a EncryptedAttachmentMeta,
            nonce_data: String,
        }
        let full = WithNonce { meta: &enc_meta, nonce_data: nonce_data.clone() };
        tokio::fs::write(
            self.attachment_meta_path(att.id),
            serde_json::to_string_pretty(&full)?,
        )
        .await?;
        Ok(())
    }

    async fn get_attachment_data(&self, id: Uuid) -> Result<Vec<u8>> {
        let master_key = self.get_master_key().await?;

        // Read metadata (includes nonce_data)
        #[derive(Deserialize)]
        struct AttMetaFull {
            #[serde(flatten)]
            inner: EncryptedAttachmentMeta,
            nonce_data: String,
        }
        let raw = tokio::fs::read_to_string(self.attachment_meta_path(id))
            .await
            .with_context(|| format!("Attachment {id} not found"))?;
        let meta: AttMetaFull = serde_json::from_str(&raw)?;

        // Decrypt DEK
        let dek_bytes = decrypt(&meta.inner.dek_encrypted, &meta.inner.nonce_dek, &master_key)?;
        let dek: [u8; 32] = dek_bytes
            .try_into()
            .map_err(|_| anyhow!("Invalid DEK length"))?;

        // Decrypt data
        let encrypted_data = tokio::fs::read_to_string(self.attachment_data_path(id)).await?;
        decrypt(&encrypted_data, &meta.nonce_data, &dek)
    }

    async fn delete_attachment(&self, id: Uuid) -> Result<()> {
        let _ = tokio::fs::remove_file(self.attachment_meta_path(id)).await;
        let _ = tokio::fs::remove_file(self.attachment_data_path(id)).await;
        Ok(())
    }
}
