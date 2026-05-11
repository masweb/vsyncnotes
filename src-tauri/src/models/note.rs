use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: Uuid,
    pub notebook_id: Uuid,
    pub title: String,
    pub body: Value,
    pub body_format: String,
    pub sort_order: i32,
    pub is_pinned: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Versión ligera de Note para listados (sin body completo).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteMeta {
    pub id: Uuid,
    pub notebook_id: Uuid,
    pub title: String,
    pub snippet: Option<String>,
    pub sort_order: i32,
    pub is_pinned: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Resultado de búsqueda por título (sin body).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteSearchResult {
    pub id: Uuid,
    pub notebook_id: Uuid,
    pub title: String,
    pub updated_at: DateTime<Utc>,
}

/// Nota en la papelera (soft-deleted).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedNoteMeta {
    pub id: Uuid,
    pub notebook_id: Uuid,
    pub title: String,
    pub deleted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Note {
    pub fn new(notebook_id: Uuid, title: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            notebook_id,
            title,
            body: serde_json::json!({ "type": "doc", "content": [] }),
            body_format: "tiptap-json".to_string(),
            sort_order: 0,
            is_pinned: false,
            created_at: now,
            updated_at: now,
        }
    }

    #[allow(dead_code)]
    pub fn to_meta(&self) -> NoteMeta {
        NoteMeta {
            id: self.id,
            notebook_id: self.notebook_id,
            title: self.title.clone(),
            snippet: None,
            sort_order: self.sort_order,
            is_pinned: self.is_pinned,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
