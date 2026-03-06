# Argon2id Master Key Derivation

A Rust function that derives a 256-bit cryptographic master key from a user-supplied password and a salt using the Argon2id algorithm. The derived key must be wrapped in a secure memory container that automatically zeros the memory contents when the value is dropped.

## Capabilities

### Memory-hard password-based key derivation

Takes a plaintext password and a salt byte slice, applies Argon2id with default parameters, and outputs a 32-byte key wrapped in a zeroizing container. The derived key must be deterministic for the same password and salt, and different for different inputs.

- Given the same password and salt, the function always returns the same 32-byte key [@test](./test.rs)
- Given the same password with a different salt, the function returns a different 32-byte key [@test](./test.rs)
- Given a different password with the same salt, the function returns a different 32-byte key [@test](./test.rs)
- The returned value is wrapped in `Zeroizing<[u8; 32]>` so the key material is erased from memory on drop [@test](./test.rs)

## Implementation

[@generates](./src/crypto/mod.rs)

## API

```rust { #api }
use zeroize::Zeroizing;

/// Derives a 256-bit master key from `password` and `salt` using Argon2id.
/// Returns the key wrapped in a Zeroizing container.
pub fn derive_key(password: &str, salt: &[u8]) -> Result<Zeroizing<[u8; 32]>, anyhow::Error>;
```

## Dependencies { .dependencies }

### vsyncnotes 0.1.0 { .dependency }

A cross-platform desktop notes application with end-to-end encryption built with Tauri 2, Vue 3, and TypeScript. Provides the `derive_key` function that uses the `argon2` crate's Argon2id algorithm to derive the vault's master key, paired with the `zeroize` crate's `Zeroizing` wrapper for secure memory handling of cryptographic key material.
