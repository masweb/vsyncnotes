# VSyncNotes

VSyncNotes is a cross-platform desktop application for encrypted note-taking built with Tauri 2 (Rust backend) and Vue 3 + TypeScript (frontend). It implements end-to-end encryption using AES-256-GCM with Argon2id key derivation. Notes are organized into hierarchical notebooks with a resizable 3-column UI. The Tiptap rich-text editor supports headings, lists, tables, code blocks, images (stored as encrypted attachments), links, and text formatting.

## Package Information

- **Project Name**: vsyncnotes
- **Repository**: github.com/masweb/vsyncnotes
- **Language**: TypeScript (frontend) + Rust (backend)
- **Frontend Stack**: Vue 3, Pinia, Tiptap, vue-i18n, vee-validate, splitpanes, CoreUI
- **Backend Stack**: Tauri 2, Rust (aes-gcm, argon2, uuid, serde_json)
- **Build**: pnpm + Tauri CLI
- **Dev command**: `pnpm dev` (web only) or `pnpm tauri dev` (native)
- **Build command**: `pnpm build` or `pnpm tauri build`

## Core Imports

```typescript
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { i18n } from '@/i18n/i18n'
import App from './App.vue'

createApp(App).use(createPinia()).use(i18n).mount('#app')
```

Vite auto-imports: `ref`, `computed`, `watch`, `onMounted`, `onUnmounted`, `onBeforeUnmount`, `nextTick`, `useI18n`, `useAppStore`, `useNotebookStore`, `useNoteStore`, `useTheme`, `useLocale`, `useForm`, `useField`.

## Basic Usage

Typical workflow for working with the VSyncNotes application:

```typescript
import { vaultStatus, vaultCreate, vaultUnlock, notebookCreate, noteCreate, noteUpdate } from '@/services/tauriApi'

// 1. On app startup: check vault and unlock
const status = await vaultStatus()
if (!status.exists) {
  await vaultCreate('my-secure-password')
}
await vaultUnlock('my-secure-password')

// 2. Load notebooks (via store)
const notebookStore = useNotebookStore()
await notebookStore.loadNotebooks()

// 3. Create a notebook and a note
const nb = await notebookStore.createNotebook('Work')
const noteStore = useNoteStore()
const note = await noteStore.createNote(nb.id, 'My First Note')

// 4. Update note content (Tiptap JSON body)
await noteUpdate({
  ...note,
  body: { type: 'doc', content: [{ type: 'paragraph', content: [{ type: 'text', text: 'Hello!' }] }] },
  updated_at: new Date().toISOString(),
})

// 5. Lock vault when done
import { vaultLock } from '@/services/tauriApi'
await vaultLock()
```

## Architecture Overview

```
┌─────────────────────────────────────────────┐
│              Vue 3 Frontend                 │
│  UnlockView → vault auth → MainView         │
│  MainView: [NotebookTree | NoteList | NoteEditor] (splitpanes) │
│  Stores: appStore, notebookStore, noteStore │
│  Composables: useTheme, useLocale           │
│  Tiptap editor with vsync:// image URIs     │
├─────────────────────────────────────────────┤
│         Tauri IPC (invoke)                  │
│  src/services/tauriApi.ts                   │
├─────────────────────────────────────────────┤
│            Rust Backend                     │
│  commands/ → storage/ (FsRepo) → crypto/   │
│  Vault: vault.json (Argon2id salt + AES key_check) │
│  Storage: $APP_DATA/vault/{notebooks,notes,attachments}/ │
└─────────────────────────────────────────────┘
```

## Capabilities

### Tauri IPC API

The Rust backend exposes Tauri commands for vault management, notebook CRUD, note CRUD, and attachment management. The TypeScript wrapper module provides typed Promise-based access.

```typescript { .api }
// Vault (src/services/tauriApi.ts)
function vaultCreate(password: string): Promise<void>;
function vaultUnlock(password: string): Promise<void>;
function vaultLock(): Promise<void>;
function vaultStatus(): Promise<VaultStatus>;
// vault_change_password is registered but not wrapped in tauriApi.ts:
// invoke('vault_change_password', { oldPassword, newPassword })

// Notebooks
function notebooksList(): Promise<Notebook[]>;
function notebookCreate(title: string, parentId?: string): Promise<Notebook>;
function notebookUpdate(notebook: Notebook): Promise<void>;
function notebookDelete(id: string): Promise<void>;

// Notes
function notesList(notebookId: string): Promise<NoteMeta[]>;
function noteGet(id: string): Promise<Note>;
function noteCreate(notebookId: string, title: string): Promise<Note>;
function noteUpdate(note: Note): Promise<void>;
function noteDelete(id: string): Promise<void>;

// Attachments
function attachmentSave(noteId: string, filename: string, mime: string, data: number[]): Promise<Attachment>;
function attachmentGet(id: string): Promise<number[]>;
function attachmentDelete(id: string): Promise<void>;

// Dev-only
function devSeed(): Promise<SeedResult>;
```

[Tauri IPC API](./tauri-api.md)

### Pinia State Stores

Three Pinia stores manage reactive application state: `useAppStore` (navigation/view), `useNotebookStore` (notebooks with tree), `useNoteStore` (notes list).

```typescript { .api }
// App navigation store
const appStore = useAppStore()
appStore.currentView           // Ref<'unlock' | 'main'>
appStore.selectedNotebookId    // Ref<string | null>
appStore.selectedNoteId        // Ref<string | null>
appStore.setView(view: 'unlock' | 'main'): void
appStore.selectNotebook(id: string | null): void
appStore.selectNote(id: string | null): void
```

[State Stores](./stores.md)

### Composables

Provides `useTheme()` for dark/light theme management and `useLocale()` for i18n locale switching.

```typescript { .api }
const { currentTheme, setTheme, toggleTheme, isDark, isLight } = useTheme()
const { currentLocale, availableLocales, setLocale } = useLocale()
```

[Composables](./composables.md)

### Vue Components

The UI is split into views (`UnlockView`, `MainView`) and components for sidebar, note list, and editor. All components use auto-imports and Bootstrap/CoreUI for styling.

[Vue Components](./components.md)

### Tiptap Rich-Text Editor

`NoteEditor` wraps Tiptap with a full toolbar supporting bold/italic/strike/underline/code/highlight, headings (H1-H3), text alignment, lists (bullet/ordered/task), blockquote, code blocks with syntax highlighting, tables, links, text color, and images stored as encrypted attachments via `vsync://attachment/{uuid}` URIs. Auto-saves with 1500ms debounce.

[Tiptap Editor](./editor.md)

## Data Models

```typescript { .api }
interface Notebook {
  id: string            // UUIDv7
  parent_id: string | null
  title: string
  sort_order: number
  created_at: string    // ISO 8601
  updated_at: string    // ISO 8601
}

interface NotebookNode extends Notebook {
  children: NotebookNode[]
}

interface NoteMeta {
  id: string            // UUIDv7
  notebook_id: string
  title: string
  sort_order: number
  is_pinned: boolean
  created_at: string    // ISO 8601
  updated_at: string    // ISO 8601
}

interface Note extends NoteMeta {
  body: unknown         // Tiptap JSON document
  body_format: string   // always "tiptap-json"
}

interface Attachment {
  id: string            // UUIDv7
  note_id: string
  filename: string
  mime: string
  size_bytes: number
  hash_sha256: string
  created_at: string    // ISO 8601
  updated_at: string    // ISO 8601
}

interface VaultStatus {
  exists: boolean
  locked: boolean
}

interface SeedResult {
  skipped: boolean
  password: string
  notebooks: number
  notes: number
}
```
