# Notebook Creator

A Tauri command that creates a new notebook entry, assigns it a unique identifier and timestamps, and immediately persists it to disk. Notebooks may optionally be nested under a parent notebook to support hierarchical organization.

## Capabilities

### Notebook creation with optional parent hierarchy

Creates a notebook record with a unique UUIDv7 identifier, a title, an optional parent notebook ID, creation and modification timestamps, and a default sort order. Persists the notebook immediately to disk as a JSON file.

- Creating a notebook with only a title produces a record with a valid UUID, matching title, null parent_id, equal created_at and updated_at timestamps, and a default sort_order of 0 [@test](./test.rs)
- Creating a notebook with a parent_id stores the parent reference correctly alongside the notebook data [@test](./test.rs)
- The persisted notebook JSON file is stored at the path `notebooks/{uuid}.json` relative to the vault directory [@test](./test.rs)

## Implementation

[@generates](./src/commands/notebook.rs)

## API

```rust { #api }
#[tauri::command]
pub async fn notebook_create(
    title: String,
    parent_id: Option<uuid::Uuid>,
    state: tauri::State<'_, AppState>,
) -> Result<Notebook, String>;

pub struct Notebook {
    pub id: uuid::Uuid,
    pub title: String,
    pub parent_id: Option<uuid::Uuid>,
    pub sort_order: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

## Dependencies { .dependencies }

### vsyncnotes 0.1.0 { .dependency }

A cross-platform desktop notes application with end-to-end encryption built with Tauri 2, Vue 3, and TypeScript. Provides the notebook management architecture including the `Notebook` struct, the `StorageRepo` trait for persistence, and the `FsRepo` filesystem implementation that stores each notebook as an individual JSON file.
