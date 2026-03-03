use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use uuid::Uuid;

use crate::models::{
    attachment::Attachment,
    note::{Note, NoteMeta},
    notebook::Notebook,
};

use super::repo::StorageRepo;

pub struct FsRepo {
    vault_path: PathBuf,
}

impl FsRepo {
    /// Crea el repo y asegura que existen los directorios del vault.
    pub fn new(vault_path: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(vault_path.join("notebooks"))?;
        std::fs::create_dir_all(vault_path.join("notes"))?;
        std::fs::create_dir_all(vault_path.join("attachments"))?;
        Ok(Self { vault_path })
    }

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
}

#[async_trait]
impl StorageRepo for FsRepo {
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
        let path = self.notebook_path(id);
        let content = tokio::fs::read_to_string(&path)
            .await
            .with_context(|| format!("Notebook {id} not found"))?;
        Ok(serde_json::from_str(&content)?)
    }

    async fn save_notebook(&self, nb: &Notebook) -> Result<()> {
        let path = self.notebook_path(nb.id);
        tokio::fs::write(&path, serde_json::to_string_pretty(nb)?).await?;
        Ok(())
    }

    async fn delete_notebook(&self, id: Uuid) -> Result<()> {
        tokio::fs::remove_file(self.notebook_path(id))
            .await
            .with_context(|| format!("Notebook {id} not found"))?;
        Ok(())
    }

    async fn list_notes(&self, notebook_id: Uuid) -> Result<Vec<NoteMeta>> {
        let dir = self.vault_path.join("notes");
        let mut entries = tokio::fs::read_dir(&dir)
            .await
            .context("Failed to read notes directory")?;

        let mut notes = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let content = tokio::fs::read_to_string(&path).await?;
                let note: Note = serde_json::from_str(&content)?;
                if note.notebook_id == notebook_id {
                    notes.push(note.to_meta());
                }
            }
        }

        // Pinned primero, luego por updated_at desc
        notes.sort_by(|a, b| {
            b.is_pinned
                .cmp(&a.is_pinned)
                .then_with(|| b.updated_at.cmp(&a.updated_at))
        });
        Ok(notes)
    }

    async fn get_note(&self, id: Uuid) -> Result<Note> {
        let path = self.note_path(id);
        let content = tokio::fs::read_to_string(&path)
            .await
            .with_context(|| format!("Note {id} not found"))?;
        Ok(serde_json::from_str(&content)?)
    }

    async fn save_note(&self, note: &Note) -> Result<()> {
        let path = self.note_path(note.id);
        tokio::fs::write(&path, serde_json::to_string_pretty(note)?).await?;
        Ok(())
    }

    async fn delete_note(&self, id: Uuid) -> Result<()> {
        tokio::fs::remove_file(self.note_path(id))
            .await
            .with_context(|| format!("Note {id} not found"))?;
        Ok(())
    }

    async fn save_attachment(&self, att: &Attachment, data: &[u8]) -> Result<()> {
        tokio::fs::write(
            self.attachment_meta_path(att.id),
            serde_json::to_string_pretty(att)?,
        )
        .await?;
        tokio::fs::write(self.attachment_data_path(att.id), data).await?;
        Ok(())
    }

    async fn get_attachment_data(&self, id: Uuid) -> Result<Vec<u8>> {
        tokio::fs::read(self.attachment_data_path(id))
            .await
            .with_context(|| format!("Attachment {id} not found"))
    }

    async fn delete_attachment(&self, id: Uuid) -> Result<()> {
        // Silencioso si alguno no existe
        let _ = tokio::fs::remove_file(self.attachment_meta_path(id)).await;
        let _ = tokio::fs::remove_file(self.attachment_data_path(id)).await;
        Ok(())
    }
}
