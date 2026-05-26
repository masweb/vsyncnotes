use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultMeta {
    pub version: u32,
    pub salt: String,            // base64 random Argon2 salt (16 bytes)
    pub key_check: String,       // base64 AES-GCM encrypted known plaintext
    pub key_check_nonce: String, // base64 nonce
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_meta_serialization_roundtrip() {
        let meta = VaultMeta {
            version: 1,
            salt: "c2FsdA==".to_string(),
            key_check: "a2V5Y2hlY2s=".to_string(),
            key_check_nonce: "bm9uY2U=".to_string(),
        };
        let json = serde_json::to_string(&meta).unwrap();
        let deserialized: VaultMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.version, meta.version);
        assert_eq!(deserialized.salt, meta.salt);
        assert_eq!(deserialized.key_check, meta.key_check);
        assert_eq!(deserialized.key_check_nonce, meta.key_check_nonce);
    }
}
