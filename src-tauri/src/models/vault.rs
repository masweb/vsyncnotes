use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultMeta {
    pub version: u32,
    pub salt: String,            // base64 random Argon2 salt (16 bytes)
    pub key_check: String,       // base64 AES-GCM encrypted known plaintext
    pub key_check_nonce: String, // base64 nonce
}
