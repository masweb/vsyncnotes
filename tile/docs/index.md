# vsyncnotes

vsyncnotes is a cross-platform desktop notes application built with Tauri 2 (Rust backend) and Vue 3 + TypeScript (frontend). It implements a 3-column layout (notebook tree, note list, WYSIWYG editor) with end-to-end encryption (AES-256-GCM + Argon2id) for all notes and attachments stored on the local filesystem. The Rust backend exposes Tauri IPC commands for full CRUD on notebooks, notes, and attachments, as well as vault lifecycle management.

## Package Information

- **Package Name**: vsyncnotes
- **Package Type**: npm
- **Language**: TypeScript + Rust (Tauri 2 desktop application)
- **Installation**: `pnpm install` then `pnpm tauri dev` (development) or `pnpm tauri build` (production)

## Core Imports

```typescript
// Tauri IPC service layer (all backend communication)
import * as api from '@/services/tauriApi'
import type { Notebook, NoteMeta, Note, Attachment, VaultStatus } from '@/types/models'

// Pinia stores (auto-imported in Vue components)
import { useAppStore } from '@/stores/appStore'
import { useNotebookStore } from '@/stores/notebookStore'
import { useNoteStore } from '@/stores/noteStore'

// Composables (auto-imported in Vue components)
import { useTheme } from '@/composables/useTheme'
import { useLocale } from '@/composables/useLocale'
```

## Basic Usage

```typescript
import * as api from '@/services/tauriApi'

// Check vault status and unlock
const status = await api.vaultStatus()
if (!status.exists) {
  await api.vaultCreate('mypassword')
} else if (status.locked) {
  await api.vaultUnlock('mypassword')
}

// Manage notebooks
const notebooks = await api.notebooksList()
const nb = await api.notebookCreate('My Notebook')
const child = await api.notebookCreate('Sub Notebook', nb.id)

// Manage notes
const notes = await api.notesList(nb.id)
const note = await api.noteCreate(nb.id, 'My Note')
const full = await api.noteGet(note.id)
await api.noteUpdate({ ...full, body: { type: 'doc', content: [] } })

// Attachments (image/binary data as number array)
const bytes = Array.from(new Uint8Array(imageBuffer))
const att = await api.attachmentSave(note.id, 'photo.png', 'image/png', bytes)
const data = await api.attachmentGet(att.id) // number[]
```

## Architecture

vsyncnotes is organized around:

- **Rust Backend** (`src-tauri/`): Filesystem storage, AES-256-GCM encryption, Tauri command handlers. Data stored under `$APP_DATA/vsyncnotes/vault/`.
- **TypeScript Service Layer** (`src/services/tauriApi.ts`): Typed wrappers around `invoke()` for all backend commands.
- **Pinia Stores** (`src/stores/`): Reactive state for notebooks, notes, and app-level navigation.
- **Vue Components** (`src/components/`, `src/views/`): 3-column layout, notebook tree, note list, Tiptap WYSIWYG editor.
- **Composables** (`src/composables/`): Theme, locale, and validation utilities.

## Capabilities

### Data Types and Interfaces

Core TypeScript interfaces for all domain objects: `Notebook`, `NoteMeta`, `Note`, `Attachment`, `VaultStatus`, `SeedResult`.

```typescript { .api }
interface Notebook {
  id: string
  parent_id: string | null
  title: string
  sort_order: number
  created_at: string  // ISO 8601
  updated_at: string  // ISO 8601
}

interface Note extends NoteMeta {
  body: unknown        // Tiptap JSON document
  body_format: string  // Always "tiptap-json"
}
```

[Data Types](./data-types.md)

### Vault Management

Lifecycle management for the encrypted vault: create, unlock, lock, change password, and check status. All notes/attachments are inaccessible until the vault is unlocked.

```typescript { .api }
function vaultCreate(password: string): Promise<void>
function vaultUnlock(password: string): Promise<void>
function vaultLock(): Promise<void>
function vaultStatus(): Promise<VaultStatus>
```

[Tauri Commands](./tauri-commands.md)

### Notebook & Note CRUD

Full CRUD operations for notebooks (with nesting support) and notes (with encrypted body storage).

```typescript { .api }
function notebookCreate(title: string, parentId?: string): Promise<Notebook>
function noteCreate(notebookId: string, title: string): Promise<Note>
function noteUpdate(note: Note): Promise<void>
```

[Tauri Commands](./tauri-commands.md)

### Attachment Management

Store, retrieve, and delete encrypted binary attachments (images, files) associated with notes. Images use the `vsync://attachment/{uuid}` URI scheme within editor content.

```typescript { .api }
function attachmentSave(noteId: string, filename: string, mime: string, data: number[]): Promise<Attachment>
function attachmentGet(id: string): Promise<number[]>
```

[Tauri Commands](./tauri-commands.md)

### Pinia Stores

Reactive state stores for application navigation and data management.

```typescript { .api }
// App navigation
const appStore = useAppStore()
appStore.setView('main')
appStore.selectNotebook(notebookId)
appStore.selectNote(noteId)

// Notebook state
const notebookStore = useNotebookStore()
await notebookStore.loadNotebooks()
const tree = notebookStore.tree  // NotebookNode[]

// Note state
const noteStore = useNoteStore()
await noteStore.loadNotes(notebookId)
const sorted = noteStore.sortedNotes  // pinned first
```

[Pinia Stores](./stores.md)

### Vue Composables

Theme, locale, and vee-validate rule composables.

```typescript { .api }
const { currentTheme, setTheme, toggleTheme, isDark } = useTheme()
const { currentLocale, availableLocales, setLocale } = useLocale()
```

[Composables](./composables.md)

### NoteEditor (Tiptap WYSIWYG)

Full-featured WYSIWYG editor built on Tiptap with rich formatting (bold, italic, underline, highlight, color), tables, syntax-highlighted code blocks (12 languages), checklists, encrypted image attachments via `vsync://attachment/{uuid}` URI scheme, 1.5s auto-save, spellcheck, and a native Tauri context menu. The component auto-loads whichever note is set in `appStore.selectedNoteId`.

```typescript { .api }
// Key editor commands (via editor.chain().focus())
editor.chain().focus().toggleBold().run()
editor.chain().focus().toggleItalic().run()
editor.chain().focus().toggleStrike().run()
editor.chain().focus().toggleUnderline().run()
editor.chain().focus().toggleHighlight().run()
editor.chain().focus().toggleHeading({ level: 1 | 2 | 3 }).run()
editor.chain().focus().toggleBulletList().run()
editor.chain().focus().toggleOrderedList().run()
editor.chain().focus().toggleTaskList().run()
editor.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run()
editor.chain().focus().setCodeBlock({ language: 'typescript' }).run()
editor.chain().focus().setImage({ src: 'vsync://attachment/{uuid}', vsyncFilename: string }).run()
editor.chain().focus().setColor('#ef4444').run()
editor.getJSON()                              // Returns Tiptap JSON body for noteUpdate()
editor.storage.characterCount.characters()   // number
editor.storage.characterCount.words()        // number
```

[NoteEditor](./editor.md)
