# Pinia Stores — vsyncnotes

vsyncnotes uses three Pinia stores for reactive state management. All stores are auto-imported in Vue components.

## Import

```typescript
// In Vue components — auto-imported, no explicit import needed
const appStore = useAppStore()
const notebookStore = useNotebookStore()
const noteStore = useNoteStore()

// In plain TypeScript files — explicit import required
import { useAppStore } from '@/stores/appStore'
import { useNotebookStore } from '@/stores/notebookStore'
import { useNoteStore } from '@/stores/noteStore'
```

## Capabilities

### `useAppStore` — Application Navigation

Controls top-level navigation state: current view and selected notebook/note IDs.

```typescript { .api }
const useAppStore: () => {
  /** Current application view. 'unlock' shows UnlockView, 'main' shows MainView. */
  currentView: Ref<'unlock' | 'main'>
  /** UUID of currently selected notebook, or null */
  selectedNotebookId: Ref<string | null>
  /** UUID of currently selected note, or null */
  selectedNoteId: Ref<string | null>
  /**
   * Switch between views.
   * @param view - 'unlock' | 'main'
   */
  setView(view: 'unlock' | 'main'): void
  /**
   * Select a notebook and clear the selected note.
   * @param id - Notebook UUID, or null to deselect
   */
  selectNotebook(id: string | null): void
  /**
   * Select a note without changing the selected notebook.
   * @param id - Note UUID, or null to deselect
   */
  selectNote(id: string | null): void
}
```

**Usage example:**

```typescript
const appStore = useAppStore()

// After vault unlock — switch to main view
await api.vaultUnlock(password)
appStore.setView('main')

// Navigate to a notebook and note
appStore.selectNotebook(notebookId)
appStore.selectNote(noteId)

// Lock and return to unlock screen
await api.vaultLock()
appStore.setView('unlock')
appStore.selectNotebook(null)
```

### `useNotebookStore` — Notebook State

Manages the flat list of notebooks with CRUD actions and a computed tree.

```typescript { .api }
const useNotebookStore: () => {
  /** Flat list of all notebooks from backend */
  notebooks: Ref<Notebook[]>
  /**
   * Computed hierarchical tree of notebooks.
   * Root nodes have parent_id = null.
   * Each node has children: NotebookNode[] recursively.
   */
  tree: ComputedRef<NotebookNode[]>
  /** true while loadNotebooks() is fetching */
  loading: Ref<boolean>
  /** Last error string, or null */
  error: Ref<string | null>
  /**
   * Fetch all notebooks from backend and populate notebooks[].
   * Sets loading/error accordingly.
   */
  loadNotebooks(): Promise<void>
  /**
   * Create a notebook and append to notebooks[].
   * @param title - Notebook name
   * @param parentId - UUID of parent notebook, or null/undefined for root-level
   */
  createNotebook(title: string, parentId?: string | null): Promise<Notebook>
  /**
   * Update a notebook and sync to notebooks[].
   */
  updateNotebook(notebook: Notebook): Promise<void>
  /**
   * Delete a notebook and remove from notebooks[].
   */
  deleteNotebook(id: string): Promise<void>
}
```

**Usage example:**

```typescript
const notebookStore = useNotebookStore()

// Load after vault unlock
await notebookStore.loadNotebooks()

// Access as flat list
console.log(notebookStore.notebooks) // Notebook[]

// Access as tree (for sidebar rendering)
function renderTree(nodes: NotebookNode[]) {
  for (const node of nodes) {
    console.log(node.title)
    renderTree(node.children) // recursive
  }
}
renderTree(notebookStore.tree)

// Create nested notebook
const parent = await notebookStore.createNotebook('Work')
const child = await notebookStore.createNotebook('Projects', parent.id)

// Rename
await notebookStore.updateNotebook({ ...parent, title: 'Work Notes' })

// Delete
await notebookStore.deleteNotebook(parent.id)
```

### `useNoteStore` — Note State

Manages the list of notes (metadata) for the currently selected notebook.

```typescript { .api }
const useNoteStore: () => {
  /** Notes (metadata only) in the currently loaded notebook */
  notes: Ref<NoteMeta[]>
  /**
   * Sorted notes: pinned notes first, then by sort_order ascending.
   */
  sortedNotes: ComputedRef<NoteMeta[]>
  /** true while loadNotes() is fetching */
  loading: Ref<boolean>
  /** Last error string, or null */
  error: Ref<string | null>
  /**
   * Fetch notes for a notebook and populate notes[].
   * Replaces any previously loaded notes.
   */
  loadNotes(notebookId: string): Promise<void>
  /**
   * Create a note and append to notes[].
   * @returns Full Note with body (not just metadata)
   */
  createNote(notebookId: string, title: string): Promise<Note>
  /**
   * Delete a note and remove from notes[].
   */
  deleteNote(id: string): Promise<void>
  /**
   * Rename a note: fetches full note, updates title, saves.
   * Also updates title in the local notes[] metadata.
   */
  renameNote(id: string, title: string): Promise<void>
  /**
   * Clear notes[] (e.g. on vault lock or notebook deselect).
   */
  clear(): void
}
```

**Usage example:**

```typescript
const noteStore = useNoteStore()
const appStore = useAppStore()

// Load notes when notebook is selected
watch(() => appStore.selectedNotebookId, async (id) => {
  if (id) {
    await noteStore.loadNotes(id)
  } else {
    noteStore.clear()
  }
})

// Access sorted notes for display (pinned first)
const notes = noteStore.sortedNotes // NoteMeta[]

// Create a note
const note = await noteStore.createNote(notebookId, 'New Note')
// note.body is available (full Note returned)
appStore.selectNote(note.id)

// Rename
await noteStore.renameNote(noteId, 'Updated Title')

// Delete
await noteStore.deleteNote(noteId)
if (appStore.selectedNoteId === noteId) {
  appStore.selectNote(null)
}
```
