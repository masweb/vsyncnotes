# Note List vs Full Note Retrieval

A module that demonstrates the distinction between lightweight note metadata listing and full note retrieval, and uses them appropriately.

## Capabilities

### List notes as lightweight metadata

- Listing notes for a notebook returns objects that have `id`, `title`, `is_pinned`, and timestamp fields but do NOT have a `body` field [@test](./tests/01-list-returns-meta.test.ts)
- Listing notes is efficient for populating a note list UI without decrypting note bodies [@test](./tests/02-list-multiple-notes.test.ts)

### Retrieve full note with body

- Fetching a note by ID returns an object that includes both the metadata fields AND a `body` field containing the Tiptap JSON content [@test](./tests/03-get-includes-body.test.ts)
- The `body_format` field on a retrieved note is `"tiptap-json"` [@test](./tests/04-body-format.test.ts)

## Implementation

[@generates](./src/noteRetrieval.ts)

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

/** Returns lightweight note metadata for all notes in a notebook (no body). */
export async function listNoteMeta(notebookId: string): Promise<NoteMeta[]>;

/** Returns a full note including its decrypted body. */
export async function getFullNote(id: string): Promise<Note>;

/** Returns true if the object is a full Note (has a body field). */
export function isFullNote(note: NoteMeta | Note): note is Note;
```

## Dependencies { .dependencies }

### vsyncnotes 0.1.0 { .dependency }

Offline-first encrypted notes desktop application built with Tauri 2 and Vue 3. Exposes separate commands for listing note metadata (without body) and fetching a full note (with decrypted body in Tiptap JSON format). Requires an unlocked vault.

[@satisfied-by](vsyncnotes)
