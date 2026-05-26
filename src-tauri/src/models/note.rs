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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_note_new_generates_v7_uuid() {
        let notebook_id = Uuid::now_v7();
        let note = Note::new(notebook_id, "Test note".to_string());
        // UUID v7 is a time-ordered UUID; verify it's valid and not nil
        assert!(!note.id.is_nil());
        assert_eq!(note.notebook_id, notebook_id);
    }

    #[test]
    fn test_note_serialization_roundtrip() {
        let notebook_id = Uuid::now_v7();
        let note = Note::new(notebook_id, "Roundtrip test".to_string());
        let json = serde_json::to_string(&note).unwrap();
        let deserialized: Note = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, note.id);
        assert_eq!(deserialized.notebook_id, note.notebook_id);
        assert_eq!(deserialized.title, note.title);
        assert_eq!(deserialized.body, note.body);
        assert_eq!(deserialized.body_format, note.body_format);
        assert_eq!(deserialized.sort_order, note.sort_order);
        assert_eq!(deserialized.is_pinned, note.is_pinned);
        assert_eq!(deserialized.created_at, note.created_at);
        assert_eq!(deserialized.updated_at, note.updated_at);
    }

    #[test]
    fn test_note_meta_from_note() {
        let notebook_id = Uuid::now_v7();
        let note = Note::new(notebook_id, "Meta test".to_string());
        let meta = note.to_meta();
        assert_eq!(meta.id, note.id);
        assert_eq!(meta.notebook_id, note.notebook_id);
        assert_eq!(meta.title, note.title);
        assert_eq!(meta.snippet, None);
        assert_eq!(meta.sort_order, note.sort_order);
        assert_eq!(meta.is_pinned, note.is_pinned);
        assert_eq!(meta.created_at, note.created_at);
        assert_eq!(meta.updated_at, note.updated_at);
    }
}
