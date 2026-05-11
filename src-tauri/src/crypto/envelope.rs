use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{anyhow, Context, Result};
use argon2::Argon2;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rand::{rngs::OsRng, RngCore};
use zeroize::Zeroizing;

const KEY_CHECK_PLAINTEXT: &[u8] = b"vsyncnotes-v1-key-check";

/// Encrypts `plaintext` with AES-256-GCM using a fresh random nonce.
/// Returns `(ciphertext_base64, nonce_base64)`.
pub fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<(String, String)> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| anyhow!("Encryption failed: {e}"))?;
    Ok((B64.encode(&ciphertext), B64.encode(nonce_bytes)))
}

/// Decrypts AES-256-GCM ciphertext (base64-encoded).
pub fn decrypt(ciphertext_b64: &str, nonce_b64: &str, key: &[u8; 32]) -> Result<Vec<u8>> {
    let ciphertext = B64
        .decode(ciphertext_b64)
        .context("base64 decode ciphertext")?;
    let nonce_bytes = B64.decode(nonce_b64).context("base64 decode nonce")?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(&nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| anyhow!("Decryption failed — wrong key or corrupted data"))
}

/// Generates a random 256-bit Data Encryption Key.
pub fn generate_dek() -> Zeroizing<[u8; 32]> {
    let mut dek = Zeroizing::new([0u8; 32]);
    OsRng.fill_bytes(dek.as_mut());
    dek
}

/// Derives a 256-bit master key from `password` + `salt` using Argon2id (default params).
pub fn derive_key(password: &str, salt: &[u8]) -> Result<Zeroizing<[u8; 32]>> {
    let mut key = Zeroizing::new([0u8; 32]);
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, key.as_mut())
        .map_err(|e| anyhow!("Key derivation failed: {e}"))?;
    Ok(key)
}

/// Produces an encrypted key-check blob to verify password correctness later.
/// Returns `(key_check_base64, nonce_base64)`.
pub fn make_key_check(master_key: &[u8; 32]) -> Result<(String, String)> {
    encrypt(KEY_CHECK_PLAINTEXT, master_key)
}

/// Returns `true` if `master_key` correctly decrypts the stored key-check.
pub fn verify_key_check(key_check_b64: &str, nonce_b64: &str, master_key: &[u8; 32]) -> bool {
    match decrypt(key_check_b64, nonce_b64, master_key) {
        Ok(plain) => plain == KEY_CHECK_PLAINTEXT,
        Err(_) => false,
    }
}
