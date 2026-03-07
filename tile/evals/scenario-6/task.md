# Note Manager

A module providing full create, read, update, and delete operations for notes within a notebook.

## Capabilities

### Create and retrieve notes

- Creating a note in a notebook returns a full note object with id, title, body (Tiptap JSON), body_format, and timestamp fields [@test](./tests/01-create-note.test.ts)
- Listing notes for a notebook returns metadata objects without a body field [@test](./tests/02-list-notes-metadata.test.ts)
- Retrieving a note by ID returns the decrypted note including the body [@test](./tests/03-get-note-full.test.ts)

### Update and delete notes

- Updating a note's title and body persists changes when retrieved again [@test](./tests/04-update-note.test.ts)
- Deleting a note removes it from the notebook's note list [@test](./tests/05-delete-note.test.ts)

## Implementation

[@generates](./src/noteManager.ts)

## API

```typescript { #api }
export interface NoteMeta {
  id: string;
  notebook_id: string;
  title: string;
  sort_order: number;
  is_pinned: boolean;
  created_at: string;
  updated_at: string;
}

export interface Note extends NoteMeta {
  body: unknown;
  body_format: string;
}

export async function createNote(notebookId: string, title: string): Promise<Note>;
export async function listNotes(notebookId: string): Promise<NoteMeta[]>;
export async function getNote(id: string): Promise<Note>;
export async function updateNote(note: Note): Promise<void>;
export async function deleteNote(id: string): Promise<void>;
```

## Dependencies { .dependencies }

### vsyncnotes 0.1.0 { .dependency }

Offline-first encrypted notes desktop application built with Tauri 2 and Vue 3. Provides Tauri commands for full CRUD on notes. Notes are encrypted at rest using per-note data encryption keys. Requires an unlocked vault for all note operations.

[@satisfied-by](vsyncnotes)
