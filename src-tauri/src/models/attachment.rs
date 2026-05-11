use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: Uuid,
    pub note_id: Uuid,
    pub filename: String,
    pub mime: String,
    pub size_bytes: u64,
    pub hash_sha256: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Attachment {
    pub fn new(
        note_id: Uuid,
        filename: String,
        mime: String,
        size_bytes: u64,
        hash_sha256: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            note_id,
            filename,
            mime,
            size_bytes,
            hash_sha256,
            created_at: now,
            updated_at: now,
        }
    }
}
