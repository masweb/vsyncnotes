# Note Pin Toggle

A frontend store action that toggles the pinned state of a note. Pinned notes must appear before unpinned notes when the note list is rendered, with secondary sorting by last-modified time descending.

## Capabilities

### Pin state toggle with sort-order enforcement

Toggling the pin state on a note flips its `is_pinned` boolean and persists the change. The computed sorted list must always show pinned notes first regardless of their modification time, and within each group notes are ordered by `updated_at` descending.

- Pinning an unpinned note sets `is_pinned` to `true` and moves it to the top of the sorted list [@test](./test.ts)
- Unpinning a pinned note sets `is_pinned` to `false` and moves it below all still-pinned notes [@test](./test.ts)
- When two notes are both pinned, they are ordered among each other by `updated_at` descending [@test](./test.ts)
- When a note has `is_pinned: true` and a very old `updated_at`, it still appears before any note with `is_pinned: false` [@test](./test.ts)

## Implementation

[@generates](./src/stores/noteStore.ts)

## API

```typescript { #api }
interface NoteMeta {
  id: string;
  notebook_id: string;
  title: string;
  is_pinned: boolean;
  updated_at: string; // ISO 8601 timestamp
  created_at: string;
}

// Pinia store
const useNoteStore: () => {
  notes: Ref<NoteMeta[]>;
  sortedNotes: ComputedRef<NoteMeta[]>;
  togglePin(id: string): Promise<void>;
};
```

## Dependencies { .dependencies }

### vsyncnotes 0.1.0 { .dependency }

A cross-platform desktop notes application with end-to-end encryption built with Tauri 2, Vue 3, and TypeScript. Provides the `NoteMeta` type, the Pinia-based note store, the Tauri IPC bridge for persisting note updates, and the pin-first sort ordering used throughout the notes list UI.
