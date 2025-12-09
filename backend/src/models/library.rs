//! Library-related data models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A content library that can contain multiple scan paths.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Library {
    pub id: i64,
    pub name: String,
    pub scan_interval: i32,
    pub watch_mode: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A scan path associated with a library.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScanPath {
    pub id: i64,
    pub library_id: i64,
    pub path: String,
    pub created_at: DateTime<Utc>,
}

/// Library with computed statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryWithStats {
    #[serde(flatten)]
    pub library: Library,
    pub path_count: i64,
    pub content_count: i64,
}

/// Request to create a new library.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateLibraryRequest {
    pub name: String,
    pub scan_interval: Option<i32>,
    pub watch_mode: Option<bool>,
}

/// Request to update an existing library.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateLibraryRequest {
    pub name: Option<String>,
    pub scan_interval: Option<i32>,
    pub watch_mode: Option<bool>,
}
