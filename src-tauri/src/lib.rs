mod commands;
mod crypto;
mod models;
mod storage;

use storage::fs_repo::FsRepo;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let vault_path = app.path().app_data_dir()?.join("vault");
            let repo = FsRepo::new(vault_path)?;
            app.manage(repo);
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
            commands::note_delete,
            commands::search_notes,
            // Attachments
            commands::attachment_save,
            commands::attachment_get,
            commands::attachment_delete,
            // Dev
            commands::seed::dev_seed,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
