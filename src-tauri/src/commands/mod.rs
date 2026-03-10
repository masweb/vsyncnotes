pub mod seed;

use sha2::{Digest, Sha256};
use tauri::State;
use uuid::Uuid;

use crate::models::{
    attachment::Attachment,
    note::{DeletedNoteMeta, Note, NoteMeta, NoteSearchResult},
    notebook::Notebook,
};
use crate::storage::{fs_repo::FsRepo, repo::StorageRepo};
use crate::sync::engine::{SyncConfig, SyncEngine, SyncResult};

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
    let all = repo.list_notebooks().await.map_err(|e| e.to_string())?;
    let next_order = all
        .iter()
        .filter(|nb| nb.parent_id == parent_id)
        .map(|nb| nb.sort_order)
        .max()
        .unwrap_or(-1)
        + 1;
    let mut nb = Notebook::new(title, parent_id);
    nb.sort_order = next_order;
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
    let siblings = repo.list_notes(notebook_id).await.map_err(|e| e.to_string())?;
    let next_order = siblings.iter().map(|n| n.sort_order).max().unwrap_or(-1) + 1;
    let mut note = Note::new(notebook_id, title);
    note.sort_order = next_order;
    repo.save_note(&note).await.map_err(|e| e.to_string())?;
    Ok(note)
}

#[tauri::command]
pub async fn note_update(repo: State<'_, FsRepo>, note: Note) -> Result<(), String> {
    repo.save_note(&note).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn note_set_sort_order(
    repo: State<'_, FsRepo>,
    id: Uuid,
    sort_order: i32,
) -> Result<(), String> {
    repo.set_note_sort_order(id, sort_order)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn note_set_pinned(
    repo: State<'_, FsRepo>,
    id: Uuid,
    pinned: bool,
) -> Result<(), String> {
    repo.set_note_pinned(id, pinned)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn note_delete(repo: State<'_, FsRepo>, id: Uuid) -> Result<(), String> {
    repo.delete_note(id).await.map_err(|e| e.to_string())
}

// ── Trash ─────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn trash_list(repo: State<'_, FsRepo>) -> Result<Vec<DeletedNoteMeta>, String> {
    repo.list_deleted_notes().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn trash_restore(repo: State<'_, FsRepo>, id: Uuid) -> Result<(), String> {
    repo.restore_note(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn trash_purge(repo: State<'_, FsRepo>, id: Uuid) -> Result<(), String> {
    repo.purge_note(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn trash_empty(repo: State<'_, FsRepo>) -> Result<(), String> {
    repo.trash_empty().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_notes(
    repo: State<'_, FsRepo>,
    query: String,
) -> Result<Vec<NoteSearchResult>, String> {
    repo.search_notes(&query).await.map_err(|e| e.to_string())
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

// ── Sync ──────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn sync_configure(
    engine: State<'_, SyncEngine>,
    provider: String,
    auto_sync_interval_secs: Option<u64>,
    target_path: Option<String>,
    webdav_url: Option<String>,
    webdav_username: Option<String>,
    webdav_password: Option<String>,
) -> Result<(), String> {
    let config = SyncConfig {
        provider,
        target_path,
        webdav_url,
        webdav_username,
        webdav_password,
        auto_sync_interval_secs: auto_sync_interval_secs.unwrap_or(300),
    };
    engine.save_config(&config).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_get_config(
    engine: State<'_, SyncEngine>,
) -> Result<Option<SyncConfig>, String> {
    Ok(engine.load_config().await)
}

#[tauri::command]
pub async fn sync_clear_config(engine: State<'_, SyncEngine>) -> Result<(), String> {
    engine.clear_config().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_run(engine: State<'_, SyncEngine>) -> Result<SyncResult, String> {
    engine.run().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_webdav_test(
    webdav_url: String,
    webdav_username: Option<String>,
    webdav_password: Option<String>,
) -> Result<(), String> {
    use crate::sync::webdav::WebDavClient;
    let client = WebDavClient::new(
        &webdav_url,
        webdav_username.as_deref().unwrap_or(""),
        webdav_password.as_deref().unwrap_or(""),
    )
    .map_err(|e| e.to_string())?;
    client.test_connection().await.map_err(|e| e.to_string())
}
