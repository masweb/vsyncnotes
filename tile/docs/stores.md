# Pinia State Stores

VSyncNotes uses three Pinia stores for reactive state management. All stores are auto-imported in Vue components.

## Import

```typescript
import { useAppStore } from '@/stores/appStore'
import { useNotebookStore } from '@/stores/notebookStore'
import { useNoteStore } from '@/stores/noteStore'
```

In components with auto-imports, use directly without explicit import.

---

## Capabilities

### App Store (`useAppStore`)

Controls the active view, selected notebook, and selected note. This is the core navigation state.

```typescript { .api }
const useAppStore: () => {
  // State
  currentView: Ref<'unlock' | 'main'>
  selectedNotebookId: Ref<string | null>
  selectedNoteId: Ref<string | null>

  // Actions
  /**
   * Switches the active view.
   * @param view - 'unlock' (auth screen) or 'main' (editor layout)
   */
  setView(view: 'unlock' | 'main'): void

  /**
   * Selects a notebook and clears the selectedNoteId.
   * @param id - Notebook UUID, or null to deselect
   */
  selectNotebook(id: string | null): void

  /**
   * Selects a note.
   * @param id - Note UUID, or null to deselect
   */
  selectNote(id: string | null): void
}
```

**Usage Example:**

```typescript
const appStore = useAppStore()

// After successful vault unlock
appStore.setView('main')

// When user clicks a notebook
appStore.selectNotebook(notebookId)

// When user clicks a note
appStore.selectNote(noteId)

// Watch for note selection changes
watch(() => appStore.selectedNoteId, (id) => {
  if (id) loadNote(id)
})
```

---

### Notebook Store (`useNotebookStore`)

Manages the flat list of notebooks and provides a computed tree representation. Calls tauriApi functions for all backend operations and keeps local state in sync.

```typescript { .api }
const useNotebookStore: () => {
  // State
  notebooks: Ref<Notebook[]>      // Flat list of all notebooks
  tree: ComputedRef<NotebookNode[]> // Nested tree (built from parent_id relationships)
  loading: Ref<boolean>
  error: Ref<string | null>

  // Actions
  /**
   * Loads all notebooks from the backend into `notebooks`.
   * Sets loading/error state.
   */
  loadNotebooks(): Promise<void>

  /**
   * Creates a notebook via backend and appends to local state.
   * @param title - Notebook title
   * @param parentId - Optional parent notebook ID (null for root)
   * @returns The created Notebook
   */
  createNotebook(title: string, parentId?: string | null): Promise<Notebook>

  /**
   * Updates a notebook via backend and syncs local state.
   * @param notebook - Full Notebook object with updated values
   */
  updateNotebook(notebook: Notebook): Promise<void>

  /**
   * Deletes a notebook via backend and removes from local state.
   * @param id - UUID of notebook to delete
   */
  deleteNotebook(id: string): Promise<void>
}
```

**Tree Building:**

The `tree` computed property builds a nested `NotebookNode[]` from the flat `notebooks` array using `parent_id` references. Orphaned nodes (parent not in list) become root nodes.

```typescript
interface NotebookNode extends Notebook {
  children: NotebookNode[]
}
```

**Usage Example:**

```typescript
const notebookStore = useNotebookStore()

// Load on mount
onMounted(() => notebookStore.loadNotebooks())

// Create root notebook
const nb = await notebookStore.createNotebook('Work')

// Create child notebook
const child = await notebookStore.createNotebook('Projects', nb.id)

// Rename
const updated = { ...nb, title: 'Work Notes' }
await notebookStore.updateNotebook(updated)

// Delete
await notebookStore.deleteNotebook(nb.id)

// Iterate tree
for (const root of notebookStore.tree) {
  console.log(root.title, root.children)
}
```

---

### Note Store (`useNoteStore`)

Manages the list of note metadata for the currently selected notebook. Does not store the full note body; use `noteGet` from tauriApi to load a full note.

```typescript { .api }
const useNoteStore: () => {
  // State
  notes: Ref<NoteMeta[]>              // Note metadata list for current notebook
  sortedNotes: ComputedRef<NoteMeta[]> // Sorted: pinned first, then by updated_at desc
  loading: Ref<boolean>
  error: Ref<string | null>

  // Actions
  /**
   * Loads note metadata for a notebook from the backend.
   * Replaces current notes list.
   * @param notebookId - UUID of the notebook
   */
  loadNotes(notebookId: string): Promise<void>

  /**
   * Creates a note via backend and pushes to local state.
   * @param notebookId - UUID of the parent notebook
   * @param title - Note title
   * @returns The full created Note (includes empty body)
   */
  createNote(notebookId: string, title: string): Promise<Note>

  /**
   * Deletes a note via backend and removes from local state.
   * @param id - UUID of the note to delete
   */
  deleteNote(id: string): Promise<void>

  /**
   * Clears the notes list (e.g., when no notebook is selected).
   */
  clear(): void
}
```

**Note:** `noteUpdate` is not part of this store — call `noteUpdate(note)` from `tauriApi` directly for saves. The store is used for list management only.

**Usage Example:**

```typescript
const noteStore = useNoteStore()
const appStore = useAppStore()

// Load notes when notebook is selected
watch(() => appStore.selectedNotebookId, async (id) => {
  if (id) await noteStore.loadNotes(id)
  else noteStore.clear()
})

// Create a new note
const newNote = await noteStore.createNote(appStore.selectedNotebookId!, 'Untitled')
appStore.selectNote(newNote.id)

// Display sorted notes
for (const meta of noteStore.sortedNotes) {
  console.log(meta.title, meta.is_pinned, meta.updated_at)
}

// Update note metadata in the local list (without calling backend)
const meta = noteStore.notes.find(n => n.id === noteId)
if (meta) meta.title = 'New Title'

// Delete
await noteStore.deleteNote(noteId)
```
