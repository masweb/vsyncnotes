# Vault Key Check

A pair of Rust functions used during vault creation and vault unlock to verify password correctness without storing the master key in plaintext. On vault creation the key-check material is generated; on unlock the stored material is verified against a candidate key.

## Capabilities

### Password validation via encrypted sentinel verification

Encrypts a fixed sentinel plaintext with the master key to produce key-check material. On unlock, re-derives a candidate master key and attempts to decrypt the stored material; if the decrypted result matches the sentinel, the password is correct.

- `make_key_check` encrypts the sentinel string and returns a ciphertext and nonce that can later be used to verify the same key [@test](./test.rs)
- `verify_key_check` returns `true` when given the ciphertext/nonce produced by `make_key_check` and the same 32-byte key [@test](./test.rs)
- `verify_key_check` returns `false` when given a different 32-byte key [@test](./test.rs)
- The sentinel string used is `"vsyncnotes-v1-key-check"` [@test](./test.rs)

## Implementation

[@generates](./src/crypto/mod.rs)

## API

```rust { #api }
/// Produces ciphertext and nonce by encrypting the known sentinel string with `key`.
pub fn make_key_check(key: &[u8; 32]) -> Result<(String, String), anyhow::Error>;

/// Returns true if decrypting `ciphertext_b64` with `key` and `nonce_b64`
/// yields the known sentinel string.
pub fn verify_key_check(
    ciphertext_b64: &str,
    nonce_b64: &str,
    key: &[u8; 32],
) -> Result<bool, anyhow::Error>;
```

## Dependencies { .dependencies }

### vsyncnotes 0.1.0 { .dependency }

A cross-platform desktop notes application with end-to-end encryption built with Tauri 2, Vue 3, and TypeScript. Provides the `encrypt` and `decrypt` AES-256-GCM primitives, the sentinel constant `"vsyncnotes-v1-key-check"`, and the key-check pattern used in `VaultMeta` to enable password validation without storing plaintext keys.
