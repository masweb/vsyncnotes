# Attachment Manager

A module that saves binary file attachments to a note and retrieves them in decrypted form.

## Capabilities

### Save an attachment to a note

- Saving a binary buffer with a filename and MIME type returns an attachment metadata object containing an id, filename, mime, size_bytes, and sha256 hash [@test](./tests/01-save-attachment.test.ts)
- The returned SHA256 hash matches the hash of the original input data [@test](./tests/02-hash-matches.test.ts)

### Retrieve and delete an attachment

- Retrieving an attachment by its ID returns the original binary data unchanged [@test](./tests/03-retrieve-attachment.test.ts)
- Deleting an attachment removes it such that subsequent retrieval throws an error [@test](./tests/04-delete-attachment.test.ts)

## Implementation

[@generates](./src/attachmentManager.ts)

## API

```typescript { #api }
export interface Attachment {
  id: string;
  note_id: string;
  filename: string;
  mime: string;
  size_bytes: number;
  hash_sha256: string;
  created_at: string;
  updated_at: string;
}

/** Saves a binary attachment to a note and returns its metadata. */
export async function saveAttachment(
  noteId: string,
  filename: string,
  mime: string,
  data: Uint8Array
): Promise<Attachment>;

/** Returns the decrypted binary data for an attachment. */
export async function getAttachment(id: string): Promise<Uint8Array>;

/** Deletes an attachment by ID. */
export async function deleteAttachment(id: string): Promise<void>;
```

## Dependencies { .dependencies }

### vsyncnotes 0.1.0 { .dependency }

Offline-first encrypted notes desktop application built with Tauri 2 and Vue 3. Provides Tauri commands for saving, retrieving, and deleting encrypted binary attachments. Attachments are encrypted with per-attachment data encryption keys and a SHA256 hash of the original data is stored in metadata. Requires an unlocked vault.

[@satisfied-by](vsyncnotes)
