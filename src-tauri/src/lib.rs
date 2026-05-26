mod commands;
mod crypto;
mod models;
mod storage;
mod sync;

use storage::fs_repo::FsRepo;
use sync::engine::SyncEngine;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data = app.path().app_data_dir()?;
            let vault_path = app_data.join("vault");
            let repo = FsRepo::new(vault_path.clone())?;
            app.manage(repo);
            let sync_engine = SyncEngine::new(vault_path, app_data.join("sync_config.json"));
            app.manage(sync_engine);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Vault
            commands::vault_create,
            commands::vault_unlock,
            commands::vault_lock,
            commands::vault_change_password,
            commands::vault_status,
            // Notebooks
            commands::notebooks_list,
            commands::notebook_get,
            commands::notebook_create,
            commands::notebook_update,
            commands::notebook_delete,
            // Notes
            commands::notes_list,
            commands::note_get,
            commands::note_create,
            commands::note_update,
            commands::note_set_sort_order,
            commands::note_set_pinned,
            commands::note_delete,
            // Trash
            commands::trash_list,
            commands::trash_restore,
            commands::trash_purge,
            commands::trash_empty,
            commands::search_notes,
            // Attachments
            commands::attachment_save,
            commands::attachment_get,
            commands::attachment_delete,
            // Sync
            commands::sync_configure,
            commands::sync_get_config,
            commands::sync_clear_config,
            commands::sync_run,
            commands::sync_webdav_test,
            // Dev (debug only)
            #[cfg(debug_assertions)]
            commands::seed::dev_seed,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
