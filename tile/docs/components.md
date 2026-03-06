# Vue Components

VSyncNotes is organized into views (top-level screens) and components (reusable UI pieces). All components use `<script lang="ts" setup>` (setup SFC), auto-imports, Bootstrap/CoreUI for styling, and no `<style>` blocks.

## App Entry (`App.vue`)

The root component renders `<component :is="currentView">` based on `appStore.currentView`:
- `'unlock'` → `UnlockView`
- `'main'` → `MainView`

---

## Views

### `UnlockView` (`src/views/UnlockView.vue`)

The vault authentication screen. Shown when `appStore.currentView === 'unlock'`.

**Behavior:**
1. On mount: calls `vaultStatus()` to check if vault exists
2. If vault exists: shows single password field → calls `vaultUnlock(password)`
3. If vault doesn't exist: shows password + confirm password fields → calls `vaultCreate(password)` then `vaultUnlock(password)`
4. On success: calls `appStore.setView('main')`
5. Dev Seed button: calls `devSeed()`, then unlocks with `'dev123'`

**No props. No emits.**

**Dependencies:** `vaultCreate`, `vaultUnlock`, `vaultStatus`, `devSeed` from `@/services/tauriApi`; `useAppStore`.

---

### `MainView` (`src/views/MainView.vue`)

The 3-column editor layout using `splitpanes`.

**Layout:**
```
┌─ AppNavbar (theme toggle) ────────────────────────────┐
├──────────┬────────────┬──────────────────────────────┤
│ Pane 1   │ Pane 2     │ Pane 3                       │
│ NotebookTree (sidebar) │ NoteList │ NoteEditor        │
│ (collapsible)          │          │                   │
└──────────┴────────────┴──────────────────────────────┘
```

**localStorage keys used:**
- `vsyncnotes:pane-sizes` — JSON `{ p1: number, p2: number, p3: number }` (percentages). Defaults: `{ p1: 15, p2: 25, p3: 60 }`
- `vsyncnotes:sidebar-open` — `'true'` / `'false'`

**Sidebar collapse/expand:**
- Collapsing: sets `p1 = 0`, adds recovered size to `p3`, saves `sidebar-open=false`
- Expanding: restores saved `p1` from snap, subtracts from `p3`, saves `sidebar-open=true`
- When collapsed, `NotebookTree` is hidden (`v-show`) and an expand button overlays the note list

**Props:** None. **Emits:** None.

---

## Sidebar Components

### `NotebookTree` (`src/components/sidebar/NotebookTree.vue`)

Renders the recursive notebook tree by iterating `useNotebookStore().tree`.

**Behavior:**
- Calls `notebookStore.loadNotebooks()` on mount
- Renders `<NotebookTreeItem>` for each root node in the tree
- Shows "No notebooks" message when tree is empty and not loading

**Emits:**
```typescript { .api }
// Emitted when the collapse button in SidebarActions is clicked
emit('collapse'): void
```

**Props:** None.

---

### `NotebookTreeItem` (`src/components/sidebar/NotebookTreeItem.vue`)

A single node in the notebook tree. Renders recursively for children.

**Props:**

```typescript { .api }
interface Props {
  node: NotebookNode  // The notebook node including children[]
  depth: number       // Nesting depth (0 = root), used for indentation
}
```

---

### `SidebarActions` (`src/components/sidebar/SidebarActions.vue`)

Action buttons in the sidebar header (create notebook, collapse).

**Emits:**
```typescript { .api }
emit('collapse'): void
```

**Props:** None.

---

## Note List Components

### `NoteList` (`src/components/notelist/NoteList.vue`)

Displays the list of notes for the currently selected notebook.

**Behavior:**
- Watches `useAppStore().selectedNotebookId`; calls `noteStore.loadNotes(id)` on change; calls `noteStore.clear()` when null
- Renders `<NoteListItem>` for each item in `noteStore.sortedNotes`

**Props:** None. **Emits:** None.

---

### `NoteListItem` (`src/components/notelist/NoteListItem.vue`)

A single item in the note list. Displays the note title, a relative date (today/yesterday/N days ago/formatted), and a pin icon if pinned. Clicking calls `appStore.selectNote(note.id)`.

**Props:**

```typescript { .api }
interface Props {
  note: NoteMeta  // The note metadata to display
}
```

---

## Editor Components

See [Tiptap Editor](./editor.md) for detailed `NoteEditor` documentation.

### `NoteEditor` (`src/components/editor/NoteEditor.vue`)

Full-featured Tiptap rich text editor with toolbar. See [Tiptap Editor](./editor.md).

**Props:** None (reads from `useAppStore().selectedNoteId`).

---

### `ImageNodeView` (`src/components/editor/ImageNodeView.vue`)

Custom Tiptap NodeView for resolving `vsync://attachment/{uuid}` image URIs. Used internally by `NoteEditor`.

**Behavior:**
- Receives a Tiptap image node with `src: "vsync://attachment/{uuid}"`
- Calls `attachmentGet(uuid)` to retrieve the encrypted attachment bytes
- Converts bytes → `Blob` → `URL.createObjectURL()` for display
- Revokes the object URL on unmount

---

## Navbar Component

### `AppNavbar` (`src/components/AppNavbar.vue`)

Simple top navigation bar with a single theme toggle button.

**Props:** None. **Emits:** None.

**Behavior:** Calls `useTheme().toggleTheme()` on button click. Shows `IconSun` when theme is dark, `IconMoon` otherwise.

---

## Component Dependencies Summary

| Component | Key Dependencies |
|-----------|-----------------|
| `UnlockView` | `vaultCreate`, `vaultUnlock`, `vaultStatus`, `devSeed`, `useAppStore` |
| `MainView` | `useTheme`, `splitpanes` (`Splitpanes`, `Pane`), `AppNavbar`, `NotebookTree`, `NoteList`, `NoteEditor` |
| `NotebookTree` | `useNotebookStore`, `SidebarActions`, `NotebookTreeItem` |
| `NotebookTreeItem` | `useNotebookStore`, `useAppStore` |
| `SidebarActions` | `useNotebookStore`, `useAppStore` |
| `NoteList` | `useNoteStore`, `useAppStore`, `NoteListItem` |
| `NoteListItem` | `useNoteStore`, `useAppStore` |
| `NoteEditor` | `useAppStore`, `useNoteStore`, `noteGet`, `noteUpdate`, `attachmentSave`, Tiptap, `ImageNodeView` |
| `ImageNodeView` | `attachmentGet` |
| `AppNavbar` | `useTheme` |
