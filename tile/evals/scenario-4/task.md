# Notes List with Sort Order

A Tauri command that retrieves all notes belonging to a given notebook, decrypts their metadata (title only, not body), and returns them sorted so that pinned notes appear first, with secondary ordering by last-modified time descending within each group.

## Capabilities

### Note metadata decryption and sort-priority listing

Lists all notes for a notebook by reading their encrypted on-disk representations, decrypting only the title field using the active master key, and applying a two-criterion sort: pinned status descending, then `updated_at` descending.

- Calling the command with a valid notebook ID returns `NoteMeta` objects with decrypted titles and correct metadata fields [@test](./test.rs)
- Pinned notes appear before unpinned notes regardless of their modification times [@test](./test.rs)
- Among notes with equal pin status, more recently updated notes appear first [@test](./test.rs)
- Calling the command when the vault is locked returns an error [@test](./test.rs)

## Implementation

[@generates](./src/commands/note.rs)

## API

```rust { #api }
#[tauri::command]
pub async fn notes_list(
    notebook_id: uuid::Uuid,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<NoteMeta>, String>;

pub struct NoteMeta {
    pub id: uuid::Uuid,
    pub notebook_id: uuid::Uuid,
    pub title: String,
    pub is_pinned: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

## Dependencies { .dependencies }

### vsyncnotes 0.1.0 { .dependency }

A cross-platform desktop notes application with end-to-end encryption built with Tauri 2, Vue 3, and TypeScript. Provides the `NoteMeta` struct, the encrypted note storage format with per-note DEKs, the `decrypt_note_meta` function, the `AppState` mutex-protected master key, and the pin-first sort ordering used throughout the notes list.
