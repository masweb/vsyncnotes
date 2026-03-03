use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use crate::models::{
    attachment::Attachment,
    note::{Note, NoteMeta},
    notebook::Notebook,
};

#[async_trait]
pub trait StorageRepo: Send + Sync {
    // Notebooks
    async fn list_notebooks(&self) -> Result<Vec<Notebook>>;
    async fn get_notebook(&self, id: Uuid) -> Result<Notebook>;
    async fn save_notebook(&self, nb: &Notebook) -> Result<()>;
    async fn delete_notebook(&self, id: Uuid) -> Result<()>;

    // Notes
    async fn list_notes(&self, notebook_id: Uuid) -> Result<Vec<NoteMeta>>;
    async fn get_note(&self, id: Uuid) -> Result<Note>;
    async fn save_note(&self, note: &Note) -> Result<()>;
    async fn delete_note(&self, id: Uuid) -> Result<()>;

    // Attachments
    async fn save_attachment(&self, att: &Attachment, data: &[u8]) -> Result<()>;
    async fn get_attachment_data(&self, id: Uuid) -> Result<Vec<u8>>;
    async fn delete_attachment(&self, id: Uuid) -> Result<()>;
}
