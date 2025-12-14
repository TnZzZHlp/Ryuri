//! Library-related data models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A content library that can contain multiple scan paths.
///
/// Libraries are the top-level organizational unit for content.
/// Each library can have multiple scan paths and contains content
/// of any type (comics or novels).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct Library {
    /// Unique identifier for the library.
    pub id: i64,
    /// Display name of the library.
    pub name: String,
    /// Automatic scan interval in minutes. 0 means disabled.
    pub scan_interval: i32,
    /// Whether file system watching is enabled for real-time updates.
    pub watch_mode: bool,
    /// Timestamp when the library was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the library was last updated.
    pub updated_at: DateTime<Utc>,
}

impl Library {
    /// Creates a new Library instance for insertion (without id and timestamps).
    pub fn create(name: String, scan_interval: i32, watch_mode: bool) -> NewLibrary {
        NewLibrary {
            name,
            scan_interval,
            watch_mode,
        }
    }
}

/// Data for creating a new library (without auto-generated fields).
#[derive(Debug, Clone)]
pub struct NewLibrary {
    pub name: String,
    pub scan_interval: i32,
    pub watch_mode: bool,
}

/// A scan path associated with a library.
///
/// Scan paths are directories on the file system that are scanned
/// for content to import into the library.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct ScanPath {
    /// Unique identifier for the scan path.
    pub id: i64,
    /// ID of the library this path belongs to.
    pub library_id: i64,
    /// File system path to scan.
    pub path: String,
    /// Timestamp when the scan path was added.
    pub created_at: DateTime<Utc>,
}

impl ScanPath {
    /// Creates a new ScanPath instance for insertion.
    pub fn create(library_id: i64, path: String) -> NewScanPath {
        NewScanPath { library_id, path }
    }
}

/// Data for creating a new scan path.
#[derive(Debug, Clone)]
pub struct NewScanPath {
    pub library_id: i64,
    pub path: String,
}

/// Library with computed statistics.
///
/// This struct is used when displaying library information
/// along with counts of associated paths and content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct LibraryWithStats {
    /// The library data.
    #[serde(flatten)]
    pub library: Library,
    /// Number of scan paths associated with this library.
    pub path_count: i64,
    /// Number of content items in this library.
    pub content_count: i64,
}

/// Request to create a new library.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct CreateLibraryRequest {
    /// Name for the new library.
    pub name: String,
    /// Optional scan interval in minutes (defaults to 0).
    pub scan_interval: Option<i32>,
    /// Optional watch mode setting (defaults to false).
    pub watch_mode: Option<bool>,
}

/// Request to update an existing library.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct UpdateLibraryRequest {
    /// New name for the library.
    pub name: Option<String>,
    /// New scan interval in minutes.
    pub scan_interval: Option<i32>,
    /// New watch mode setting.
    pub watch_mode: Option<bool>,
}
