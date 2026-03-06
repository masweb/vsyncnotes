# Encrypted Attachment Storage

A Tauri command that stores a binary file attachment linked to a note. The attachment data must be encrypted with a randomly generated per-attachment Data Encryption Key (DEK), and the DEK must be encrypted with the vault's master key. A SHA-256 hash of the original binary data must be computed and stored in the attachment metadata for integrity verification.

## Capabilities

### Encrypted binary attachment persistence with integrity hashing

Accepts raw binary data along with a filename, MIME type, and note association. Generates a random DEK, encrypts the binary data with the DEK using AES-256-GCM, encrypts the DEK with the master key, computes SHA-256 of the original data, and persists metadata to `attachments/{uuid}.json` and encrypted binary to `attachments/{uuid}.bin`.

- Saving an attachment returns an `Attachment` record with a new UUID, the provided filename and MIME type, the byte length of the original data, and a non-empty SHA-256 hash string [@test](./test.rs)
- The stored binary at `attachments/{uuid}.bin` is the encrypted form of the input, not the raw bytes [@test](./test.rs)
- The SHA-256 hash in the metadata matches the hex-encoded SHA-256 digest of the original (unencrypted) input data [@test](./test.rs)
- Calling the command when the vault is locked returns an error [@test](./test.rs)

## Implementation

[@generates](./src/commands/attachment.rs)

## API

```rust { #api }
#[tauri::command]
pub async fn attachment_save(
    note_id: uuid::Uuid,
    filename: String,
    mime: String,
    data: Vec<u8>,
    state: tauri::State<'_, AppState>,
) -> Result<Attachment, String>;

pub struct Attachment {
    pub id: uuid::Uuid,
    pub note_id: uuid::Uuid,
    pub filename: String,
    pub mime: String,
    pub size: u64,
    pub sha256: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

## Dependencies { .dependencies }

### vsyncnotes 0.1.0 { .dependency }

A cross-platform desktop notes application with end-to-end encryption built with Tauri 2, Vue 3, and TypeScript. Provides the `attachment_save` command, the `Attachment` struct, SHA-256 hashing via the `sha2` crate, the per-attachment envelope encryption pattern using `generate_dek` and `encrypt`, and the dual-file storage pattern (JSON metadata + binary data) used for all attachments.
