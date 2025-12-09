//! Content-related data models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Content type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Comic,
    Novel,
}

/// A content item (comic or novel).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Content {
    pub id: i64,
    pub library_id: i64,
    pub scan_path_id: i64,
    pub content_type: ContentType,
    pub title: String,
    pub folder_path: String,
    pub chapter_count: i32,
    #[sqlx(default)]
    pub thumbnail: Option<Vec<u8>>,
    #[sqlx(default)]
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A chapter within a content item.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Chapter {
    pub id: i64,
    pub content_id: i64,
    pub title: String,
    pub file_path: String,
    pub sort_order: i32,
}
