# Note Pinning and Sort Order

A module that pins notes for priority display and retrieves them in the correct sorted order.

## Capabilities

### Pin and unpin notes

- Setting `is_pinned: true` on a note and saving it persists the pinned state [@test](./tests/01-pin-note.test.ts)
- Setting `is_pinned: false` on a previously pinned note unpins it [@test](./tests/02-unpin-note.test.ts)

### Retrieve notes in pinned-first order

- When listing notes, pinned notes appear before unpinned notes regardless of creation order [@test](./tests/03-pinned-first.test.ts)
- Among unpinned notes, the note most recently updated appears first [@test](./tests/04-unpinned-sort-by-updated.test.ts)

## Implementation

[@generates](./src/notePinning.ts)

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

/** Pins a note so it appears at the top of the note list. */
export async function pinNote(note: Note): Promise<void>;

/** Unpins a note so it returns to chronological sort order. */
export async function unpinNote(note: Note): Promise<void>;

/** Returns notes for a notebook sorted: pinned first, then by last-updated descending. */
export async function getSortedNotes(notebookId: string): Promise<NoteMeta[]>;
```

## Dependencies { .dependencies }

### vsyncnotes 0.1.0 { .dependency }

Offline-first encrypted notes desktop application built with Tauri 2 and Vue 3. Note objects have an `is_pinned` boolean field. The backend sorts returned notes with pinned notes first, then by `updated_at` descending.

[@satisfied-by](vsyncnotes)
