# Tauri Commands — vsyncnotes

vsyncnotes exposes all backend functionality through Tauri IPC commands. There are two ways to call them:

1. **TypeScript service layer** (`src/services/tauriApi.ts`) — Typed wrapper functions (recommended)
2. **Direct `invoke()`** — Raw Tauri IPC calls

All commands are async and throw a `string` error message on failure.

## Import

```typescript
// Recommended: typed wrapper functions
import * as api from '@/services/tauriApi'
import type { Notebook, NoteMeta, Note, Attachment, VaultStatus } from '@/types/models'

// Alternative: direct invoke (lower level)
import { invoke } from '@tauri-apps/api/core'
```

## Capabilities

### Vault Management

The vault must be created and unlocked before any notebook/note/attachment operations. The vault stores the encryption salt and key-check in `$APP_DATA/vsyncnotes/vault/vault.json`.

```typescript { .api }
/**
 * Initialize a new vault with the given master password.
 * Creates $APP_DATA/vsyncnotes/vault/ directory structure.
 * Throws if vault already exists.
 */
function vaultCreate(password: string): Promise<void>

/**
 * Unlock the vault by deriving the master key from the password.
 * Must be called before any CRUD operations.
 * Throws if password is incorrect or vault doesn't exist.
 */
function vaultUnlock(password: string): Promise<void>

/**
 * Lock the vault by zeroizing the master key from memory.
 * After this call, all CRUD operations will fail until unlocked again.
 */
function vaultLock(): Promise<void>

/**
 * Get the current vault state.
 * Does not require vault to be unlocked.
 */
function vaultStatus(): Promise<VaultStatus>
```

**Direct invoke equivalents:**

```typescript { .api }
// vault_change_password — not exposed in TypeScript service layer
// Re-encrypts all DEKs. Does NOT re-encrypt data.
invoke<void>('vault_change_password', { oldPassword: string, newPassword: string }): Promise<void>
```

**Usage example:**

```typescript
const status = await api.vaultStatus()

if (!status.exists) {
  // First run: create vault
  await api.vaultCreate('securepassword')
} else if (status.locked) {
  // Subsequent runs: unlock
  try {
    await api.vaultUnlock('securepassword')
  } catch (e) {
    console.error('Wrong password:', e)
  }
}

// ... do work ...

// Lock on app close
await api.vaultLock()

// Change password (direct invoke — not in service layer)
await invoke('vault_change_password', {
  oldPassword: 'securepassword',
  newPassword: 'newpassword'
})
```

### Notebook CRUD

Notebooks are stored as plaintext JSON in `vault/notebooks/{uuid}.json`. They support arbitrary nesting via `parent_id`.

```typescript { .api }
/**
 * List all notebooks. Returns flat array — build tree with notebookStore.tree.
 * Requires unlocked vault.
 */
function notebooksList(): Promise<Notebook[]>

/**
 * Get a single notebook by ID.
 * Only available via direct invoke (no TypeScript wrapper).
 */
invoke<Notebook>('notebook_get', { id: string }): Promise<Notebook>

/**
 * Create a new notebook.
 * @param title - Notebook title
 * @param parentId - UUID of parent notebook for nesting, undefined for root-level
 * @returns Created notebook with generated UUIDv7 id
 */
function notebookCreate(title: string, parentId?: string): Promise<Notebook>

/**
 * Update notebook metadata (title, sort_order, etc.).
 * @param notebook - Full Notebook object with updated fields
 */
function notebookUpdate(notebook: Notebook): Promise<void>

/**
 * Delete notebook by ID. Does NOT cascade-delete child notebooks or notes.
 */
function notebookDelete(id: string): Promise<void>
```

**Usage example:**

```typescript
// Load all notebooks
const notebooks = await api.notebooksList()

// Create root notebook
const root = await api.notebookCreate('Personal')

// Create nested notebook
const journal = await api.notebookCreate('Journal', root.id)

// Update
await api.notebookUpdate({ ...journal, title: 'Daily Journal', sort_order: 1 })

// Delete
await api.notebookDelete(journal.id)
```

### Note CRUD

Notes are stored encrypted in `vault/notes/{uuid}.json`. The `body` field (Tiptap JSON) and `title` are both encrypted. The `notebook_id`, `sort_order`, `is_pinned`, `created_at`, and `updated_at` are stored unencrypted for indexing.

```typescript { .api }
/**
 * List notes in a notebook. Returns NoteMeta (no body) for efficient listing.
 * Use noteGet() to retrieve the full body.
 * Requires unlocked vault.
 */
function notesList(notebookId: string): Promise<NoteMeta[]>

/**
 * Get a full note with body content.
 * Body is decrypted and returned as Tiptap JSON.
 */
function noteGet(id: string): Promise<Note>

/**
 * Create a new empty note.
 * Body is initialized as { type: "doc", content: [] } with body_format "tiptap-json".
 * @returns Created note with body
 */
function noteCreate(notebookId: string, title: string): Promise<Note>

/**
 * Save a full note (title + body + all fields).
 * Body is encrypted before storage.
 * Always send the complete Note object — partial updates are not supported.
 */
function noteUpdate(note: Note): Promise<void>

/**
 * Delete note by ID.
 * Does NOT delete associated attachments.
 */
function noteDelete(id: string): Promise<void>
```

**Usage example:**

```typescript
// Create and update a note
const note = await api.noteCreate(notebookId, 'Meeting Notes')
const full = await api.noteGet(note.id)

// Update with new Tiptap JSON body
const updatedBody = {
  type: 'doc',
  content: [
    { type: 'heading', attrs: { level: 1 }, content: [{ type: 'text', text: 'Meeting Notes' }] },
    { type: 'paragraph', content: [{ type: 'text', text: 'Discussion points...' }] },
  ]
}
await api.noteUpdate({ ...full, body: updatedBody, updated_at: new Date().toISOString() })

// List notes (metadata only)
const metas = await api.notesList(notebookId)
// metas[0].body is NOT present — use noteGet() for body

// Rename a note
const existing = await api.noteGet(noteId)
await api.noteUpdate({ ...existing, title: 'New Title' })
```

### Attachment CRUD

Attachments are encrypted and stored in `vault/attachments/{uuid}.json` (metadata) and `vault/attachments/{uuid}.bin` (binary data). In editor content, images reference attachments via `vsync://attachment/{uuid}` URIs.

```typescript { .api }
/**
 * Encrypt and store an attachment.
 * @param noteId - UUID of the associated note
 * @param filename - Original filename (e.g. "photo.png")
 * @param mime - MIME type (e.g. "image/png", "image/jpeg", "image/gif", "image/webp")
 * @param data - Binary data as a plain number array (0-255 values)
 * @returns Attachment metadata with generated id
 */
function attachmentSave(
  noteId: string,
  filename: string,
  mime: string,
  data: number[]
): Promise<Attachment>

/**
 * Retrieve decrypted attachment binary data.
 * @returns Plain number array (0-255 values) — convert to Uint8Array or Blob as needed
 */
function attachmentGet(id: string): Promise<number[]>

/**
 * Delete attachment metadata and binary data.
 */
function attachmentDelete(id: string): Promise<void>
```

**Usage example:**

```typescript
// Save an image attachment from a File object
const file: File = // ... from input or drop event
const buffer = await file.arrayBuffer()
const bytes = Array.from(new Uint8Array(buffer))
const att = await api.attachmentSave(noteId, file.name, file.type, bytes)

// The attachment id can be embedded in Tiptap JSON
const imageNode = {
  type: 'image',
  attrs: {
    src: `vsync://attachment/${att.id}`,
    vsyncFilename: file.name
  }
}

// Retrieve and render attachment data
const data = await api.attachmentGet(att.id)
const blob = new Blob([new Uint8Array(data)], { type: att.mime })
const blobUrl = URL.createObjectURL(blob)
// Use blobUrl as <img src> — revoke when done

// Delete attachment
await api.attachmentDelete(att.id)
```

### Development Seeding

```typescript { .api }
/**
 * Seed the vault with sample data for development.
 * No-op (returns skipped: true) if vault already exists.
 * Creates vault with password "dev123", 7 notebooks, and 9 notes.
 * Only available in dev builds.
 */
function devSeed(): Promise<SeedResult>
```

## Error Handling

All commands throw a `string` error message on failure. Common failure modes:

- Vault commands: wrong password, vault doesn't exist, vault already exists
- CRUD commands: vault locked, item not found, storage I/O error

```typescript
try {
  await api.vaultUnlock(password)
} catch (e: unknown) {
  // e is a string error message from the Rust backend
  console.error('Vault unlock failed:', e)
}
```

## Direct `invoke()` Reference

For commands without TypeScript wrappers:

```typescript { .api }
import { invoke } from '@tauri-apps/api/core'

// vault_change_password
invoke<void>('vault_change_password', {
  oldPassword: string,
  newPassword: string
}): Promise<void>

// notebook_get (no TypeScript wrapper exists)
invoke<Notebook>('notebook_get', { id: string }): Promise<Notebook>
```

**Note on camelCase/snake_case**: `invoke()` parameter keys must match the Rust command's parameter names with Tauri's automatic camelCase→snake_case conversion. The TypeScript service layer handles this automatically.
