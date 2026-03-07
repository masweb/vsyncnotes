# Notebook Manager

A module that provides full create, read, update, and delete operations for notebooks in the encrypted notes application.

## Capabilities

### Create and retrieve notebooks

- Creating a notebook with a title returns a notebook object with a generated UUID, the given title, and timestamp fields [@test](./tests/01-create-notebook.test.ts)
- Listing all notebooks returns all previously created notebooks [@test](./tests/02-list-notebooks.test.ts)
- Fetching a notebook by its UUID returns that specific notebook [@test](./tests/03-get-notebook.test.ts)

### Update and delete notebooks

- Updating a notebook's title persists the new title when retrieved again [@test](./tests/04-update-notebook.test.ts)
- Deleting a notebook removes it from the list [@test](./tests/05-delete-notebook.test.ts)

## Implementation

[@generates](./src/notebookManager.ts)

## API

```typescript { #api }
export interface Notebook {
  id: string;
  parent_id: string | null;
  title: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export async function createNotebook(title: string): Promise<Notebook>;
export async function listNotebooks(): Promise<Notebook[]>;
export async function getNotebook(id: string): Promise<Notebook>;
export async function updateNotebook(notebook: Notebook): Promise<void>;
export async function deleteNotebook(id: string): Promise<void>;
```

## Dependencies { .dependencies }

### vsyncnotes 0.1.0 { .dependency }

Offline-first encrypted notes desktop application built with Tauri 2 and Vue 3. Provides Tauri commands for full CRUD on notebooks. Notebook data is stored as plaintext JSON and does not require an unlocked vault.

[@satisfied-by](vsyncnotes)
