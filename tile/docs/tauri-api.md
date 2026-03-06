# Tauri IPC API

VSyncNotes communicates between the Vue frontend and Rust backend via Tauri's IPC mechanism. The TypeScript wrapper is in `src/services/tauriApi.ts` and uses `invoke()` from `@tauri-apps/api/core`.

## Import

```typescript
import {
  vaultCreate, vaultUnlock, vaultLock, vaultStatus,
  notebooksList, notebookCreate, notebookUpdate, notebookDelete,
  notesList, noteGet, noteCreate, noteUpdate, noteDelete,
  attachmentSave, attachmentGet, attachmentDelete,
  devSeed,
} from '@/services/tauriApi'
```

Or use `invoke` directly:

```typescript
import { invoke } from '@tauri-apps/api/core'
const result = await invoke<Notebook>('notebook_create', { title: 'My Notebook', parentId: null })
```

## Error Handling

All functions return Promises that reject with a string error message on failure (Tauri convention: `Result<T, String>` in Rust). Use try/catch:

```typescript
try {
  await vaultUnlock(password)
} catch (e) {
  console.error('Unlock failed:', String(e))
}
```

---

## Capabilities

### Vault Management

The vault holds all encrypted data. Must be created once and unlocked on each app start before accessing notebooks/notes.

```typescript { .api }
/**
 * Creates a new encrypted vault. Must be called only once (when no vault exists).
 * Generates Argon2id salt and AES-GCM key_check. Throws if vault already exists.
 * @param password - Master password used to derive encryption key via Argon2id
 */
function vaultCreate(password: string): Promise<void>;

/**
 * Unlocks the vault by deriving the master key from the password.
 * Verifies against the stored key_check. Master key held in memory until lock.
 * @param password - Master password
 * @throws If password is incorrect or vault does not exist
 */
function vaultUnlock(password: string): Promise<void>;

/**
 * Locks the vault and zeroes (Zeroizing) the master key from memory.
 * All subsequent note/attachment operations will fail until unlocked again.
 */
function vaultLock(): Promise<void>;

/**
 * Returns vault existence and lock state.
 */
function vaultStatus(): Promise<VaultStatus>;
```

**`vault_change_password` is registered as a Tauri command but is NOT wrapped in `tauriApi.ts`. Call it directly via invoke:**

```typescript { .api }
/**
 * Changes the vault password. Re-encrypts all Data Encryption Keys (DEKs)
 * with the new master key. Does NOT re-encrypt note data itself.
 * @param oldPassword - Current password for verification
 * @param newPassword - New password to set
 * @throws If old_password is incorrect
 */
async function changeVaultPassword(oldPassword: string, newPassword: string): Promise<void> {
  await invoke('vault_change_password', { oldPassword, newPassword })
}
```

**Usage Example:**

```typescript
import { vaultStatus, vaultCreate, vaultUnlock } from '@/services/tauriApi'

// On app startup
const status = await vaultStatus()
if (!status.exists) {
  // First run: create vault
  await vaultCreate('my-secure-password')
}
await vaultUnlock('my-secure-password')
// Vault is now unlocked; notebooks/notes are accessible
```

---

### Notebook CRUD

Notebooks are stored as plaintext JSON files. They support a recursive parent/child hierarchy (flat list returned; tree built on frontend).

```typescript { .api }
/**
 * Returns all notebooks as a flat list. Build the tree on the frontend
 * using parent_id references.
 */
function notebooksList(): Promise<Notebook[]>;

/**
 * Creates a new notebook.
 * @param title - Notebook title
 * @param parentId - Optional parent notebook UUID for nesting (omit or undefined for root)
 * @returns The created Notebook with a UUIDv7 id
 */
function notebookCreate(title: string, parentId?: string): Promise<Notebook>;

/**
 * Updates an existing notebook (replaces the stored JSON).
 * @param notebook - Full Notebook object with updated fields
 */
function notebookUpdate(notebook: Notebook): Promise<void>;

/**
 * Deletes a notebook by ID.
 * @param id - UUID of the notebook to delete
 */
function notebookDelete(id: string): Promise<void>;
```

**Usage Example:**

```typescript
import { notebooksList, notebookCreate } from '@/services/tauriApi'

const notebooks = await notebooksList()

// Create root notebook
const rootNb = await notebookCreate('Work')

// Create nested notebook under rootNb
const childNb = await notebookCreate('Projects', rootNb.id)
```

---

### Note CRUD

Notes are stored encrypted (AES-256-GCM) on disk. Titles and bodies are individually encrypted with per-note DEKs. The `NoteMeta` type omits the body for efficient list rendering.

```typescript { .api }
/**
 * Returns lightweight note metadata (no body) for all notes in a notebook.
 * Sorted by backend in storage order; frontend sorts by pinned+updated_at.
 * @param notebookId - UUID of the notebook
 */
function notesList(notebookId: string): Promise<NoteMeta[]>;

/**
 * Returns a full Note including the decrypted Tiptap JSON body.
 * @param id - UUID of the note
 */
function noteGet(id: string): Promise<Note>;

/**
 * Creates a new empty note with an empty Tiptap document body.
 * @param notebookId - UUID of the parent notebook
 * @param title - Note title
 * @returns The created Note with id and empty body
 */
function noteCreate(notebookId: string, title: string): Promise<Note>;

/**
 * Saves (overwrites) a note including its body. Use for both content updates
 * and metadata updates (title, is_pinned, sort_order).
 * @param note - Full Note object; body must be valid Tiptap JSON or empty string
 */
function noteUpdate(note: Note): Promise<void>;

/**
 * Deletes a note by ID.
 * @param id - UUID of the note to delete
 */
function noteDelete(id: string): Promise<void>;
```

**Usage Example:**

```typescript
import { noteCreate, noteGet, noteUpdate, notesList } from '@/services/tauriApi'

// List notes (metadata only)
const metas = await notesList(notebookId)

// Create and load a note
const note = await noteCreate(notebookId, 'My Note')

// Update note content (Tiptap JSON body)
const updatedNote = {
  ...note,
  body: { type: 'doc', content: [{ type: 'paragraph', content: [{ type: 'text', text: 'Hello' }] }] },
  updated_at: new Date().toISOString(),
}
await noteUpdate(updatedNote)

// Pin a note
note.is_pinned = true
await noteUpdate(note)
```

---

### Attachment Management

Attachments are binary files (images, etc.) stored encrypted on disk. The typical flow is: read file → convert to byte array → save → embed `vsync://attachment/{id}` in Tiptap JSON.

```typescript { .api }
/**
 * Saves an attachment file for a note (encrypted on disk).
 * SHA-256 hash is computed by the backend.
 * @param noteId - UUID of the parent note
 * @param filename - Original filename (e.g., "photo.png")
 * @param mime - MIME type (e.g., "image/png")
 * @param data - File contents as an array of byte values (0-255)
 * @returns Attachment metadata including the generated UUID id
 */
function attachmentSave(noteId: string, filename: string, mime: string, data: number[]): Promise<Attachment>;

/**
 * Retrieves the decrypted binary data of an attachment.
 * @param id - UUID of the attachment
 * @returns File contents as an array of byte values (0-255)
 */
function attachmentGet(id: string): Promise<number[]>;

/**
 * Deletes an attachment (both metadata JSON and binary data files).
 * @param id - UUID of the attachment to delete
 */
function attachmentDelete(id: string): Promise<void>;
```

**Usage Example:**

```typescript
import { attachmentSave, attachmentGet } from '@/services/tauriApi'

// Save a file as attachment
const file: File = // ... from input or drop
const buffer = await file.arrayBuffer()
const bytes = Array.from(new Uint8Array(buffer))
const att = await attachmentSave(noteId, file.name, file.type, bytes)
// att.id can now be embedded in Tiptap JSON as: vsync://attachment/{att.id}

// Load and display an attachment
const bytes = await attachmentGet(att.id)
const blob = new Blob([new Uint8Array(bytes)], { type: att.mime })
const url = URL.createObjectURL(blob)
// Use url in <img src="...">
```

---

### Dev Seeding

```typescript { .api }
/**
 * Development-only command. Seeds the vault with sample notebooks and notes.
 * No-op (returns skipped: true) if vault already exists.
 * Uses hardcoded password "dev123".
 */
function devSeed(): Promise<SeedResult>;
```

**Usage:**

```typescript
const result = await devSeed()
if (result.skipped) {
  console.log('Vault already exists')
} else {
  await vaultUnlock('dev123')
  console.log(`Seeded ${result.notebooks} notebooks, ${result.notes} notes`)
}
```

---

## Tauri Command Reference Table

All commands map directly to Tauri IPC names (snake_case):

| TypeScript Function | Tauri Command | Parameters | Return |
|---------------------|--------------|-----------|--------|
| `vaultCreate` | `vault_create` | `{ password }` | `void` |
| `vaultUnlock` | `vault_unlock` | `{ password }` | `void` |
| `vaultLock` | `vault_lock` | — | `void` |
| `vaultStatus` | `vault_status` | — | `VaultStatus` |
| *(invoke directly)* | `vault_change_password` | `{ oldPassword, newPassword }` | `void` |
| `notebooksList` | `notebooks_list` | — | `Notebook[]` |
| *(invoke directly)* | `notebook_get` | `{ id }` | `Notebook` |
| `notebookCreate` | `notebook_create` | `{ title, parentId }` | `Notebook` |
| `notebookUpdate` | `notebook_update` | `{ notebook }` | `void` |
| `notebookDelete` | `notebook_delete` | `{ id }` | `void` |
| `notesList` | `notes_list` | `{ notebookId }` | `NoteMeta[]` |
| `noteGet` | `note_get` | `{ id }` | `Note` |
| `noteCreate` | `note_create` | `{ notebookId, title }` | `Note` |
| `noteUpdate` | `note_update` | `{ note }` | `void` |
| `noteDelete` | `note_delete` | `{ id }` | `void` |
| `attachmentSave` | `attachment_save` | `{ noteId, filename, mime, data }` | `Attachment` |
| `attachmentGet` | `attachment_get` | `{ id }` | `number[]` |
| `attachmentDelete` | `attachment_delete` | `{ id }` | `void` |
| `devSeed` | `dev_seed` | — | `SeedResult` |

Notes:
- `vault_change_password` and `notebook_get` are registered Tauri commands but not wrapped in `tauriApi.ts`; use `invoke()` directly.
- All commands are registered in `src-tauri/src/lib.rs` `invoke_handler`.
