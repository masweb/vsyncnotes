use sha2::{Digest, Sha256};
use tauri::State;
use uuid::Uuid;

use crate::models::{
    attachment::Attachment,
    note::{Note, NoteMeta},
    notebook::Notebook,
};
use crate::storage::{fs_repo::FsRepo, repo::StorageRepo};

// ── Vault ─────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn vault_create(repo: State<'_, FsRepo>, password: String) -> Result<(), String> {
    repo.vault_create(&password).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn vault_unlock(repo: State<'_, FsRepo>, password: String) -> Result<(), String> {
    repo.vault_unlock(&password).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn vault_lock(repo: State<'_, FsRepo>) -> Result<(), String> {
    repo.vault_lock().await;
    Ok(())
}

#[tauri::command]
pub async fn vault_change_password(
    repo: State<'_, FsRepo>,
    old_password: String,
    new_password: String,
) -> Result<(), String> {
    repo.vault_change_password(&old_password, &new_password)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn vault_status(repo: State<'_, FsRepo>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "exists": repo.vault_exists().await,
        "locked": repo.is_locked().await,
    }))
}

// ── Notebooks ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn notebooks_list(repo: State<'_, FsRepo>) -> Result<Vec<Notebook>, String> {
    repo.list_notebooks().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn notebook_get(repo: State<'_, FsRepo>, id: Uuid) -> Result<Notebook, String> {
    repo.get_notebook(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn notebook_create(
    repo: State<'_, FsRepo>,
    title: String,
    parent_id: Option<Uuid>,
) -> Result<Notebook, String> {
    let nb = Notebook::new(title, parent_id);
    repo.save_notebook(&nb).await.map_err(|e| e.to_string())?;
    Ok(nb)
}

#[tauri::command]
pub async fn notebook_update(repo: State<'_, FsRepo>, notebook: Notebook) -> Result<(), String> {
    repo.save_notebook(&notebook).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn notebook_delete(repo: State<'_, FsRepo>, id: Uuid) -> Result<(), String> {
    repo.delete_notebook(id).await.map_err(|e| e.to_string())
}

// ── Notes ─────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn notes_list(
    repo: State<'_, FsRepo>,
    notebook_id: Uuid,
) -> Result<Vec<NoteMeta>, String> {
    repo.list_notes(notebook_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn note_get(repo: State<'_, FsRepo>, id: Uuid) -> Result<Note, String> {
    repo.get_note(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn note_create(
    repo: State<'_, FsRepo>,
    notebook_id: Uuid,
    title: String,
) -> Result<Note, String> {
    let note = Note::new(notebook_id, title);
    repo.save_note(&note).await.map_err(|e| e.to_string())?;
    Ok(note)
}

#[tauri::command]
pub async fn note_update(repo: State<'_, FsRepo>, note: Note) -> Result<(), String> {
    repo.save_note(&note).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn note_delete(repo: State<'_, FsRepo>, id: Uuid) -> Result<(), String> {
    repo.delete_note(id).await.map_err(|e| e.to_string())
}

// ── Attachments ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn attachment_save(
    repo: State<'_, FsRepo>,
    note_id: Uuid,
    filename: String,
    mime: String,
    data: Vec<u8>,
) -> Result<Attachment, String> {
    let hash = format!("{:x}", Sha256::digest(&data));
    let att = Attachment::new(note_id, filename, mime, data.len() as u64, hash);
    repo.save_attachment(&att, &data)
        .await
        .map_err(|e| e.to_string())?;
    Ok(att)
}

#[tauri::command]
pub async fn attachment_get(repo: State<'_, FsRepo>, id: Uuid) -> Result<Vec<u8>, String> {
    repo.get_attachment_data(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn attachment_delete(repo: State<'_, FsRepo>, id: Uuid) -> Result<(), String> {
    repo.delete_attachment(id).await.map_err(|e| e.to_string())
}
