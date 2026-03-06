# Envelope Encryption for Note Storage

A set of Rust functions that implement envelope encryption for note content. Each note is encrypted with a randomly generated per-note Data Encryption Key (DEK). The DEK itself is then encrypted with the vault's master key. Both the encrypted note content and the encrypted DEK are stored together on disk.

## Capabilities

### Two-layer encryption and decryption for note data

Encrypting a note generates a fresh random DEK, encrypts the note title and body with the DEK using AES-256-GCM, and then encrypts the DEK itself with the master key. Decrypting reverses this: decrypt the DEK with the master key, then use the DEK to decrypt the note content.

- Encrypting a note with a master key produces an `EncryptedNote` that contains the note body ciphertext and the DEK ciphertext (not the raw DEK) [@test](./test.rs)
- Decrypting an `EncryptedNote` with the same master key returns the original plaintext title and body [@test](./test.rs)
- Attempting to decrypt with a different master key fails with an error rather than returning garbage data [@test](./test.rs)
- Each call to the encrypt function produces a different ciphertext even for identical inputs, due to random nonce generation [@test](./test.rs)

## Implementation

[@generates](./src/crypto/envelope.rs)

## API

```rust { #api }
pub struct EncryptedNote {
    pub id: uuid::Uuid,
    pub notebook_id: uuid::Uuid,
    pub title_ciphertext: String,
    pub title_nonce: String,
    pub body_ciphertext: String,
    pub body_nonce: String,
    pub dek_ciphertext: String,
    pub dek_nonce: String,
    pub is_pinned: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub fn encrypt_note(note: &Note, master_key: &[u8; 32]) -> Result<EncryptedNote, anyhow::Error>;
pub fn decrypt_note(enc: &EncryptedNote, master_key: &[u8; 32]) -> Result<Note, anyhow::Error>;
```

## Dependencies { .dependencies }

### vsyncnotes 0.1.0 { .dependency }

A cross-platform desktop notes application with end-to-end encryption built with Tauri 2, Vue 3, and TypeScript. Provides the `encrypt_note` and `decrypt_note` envelope encryption functions, the `EncryptedNote` struct, the `generate_dek` function for random DEK generation, and the `encrypt`/`decrypt` AES-256-GCM primitives that underpin all note encryption operations.
