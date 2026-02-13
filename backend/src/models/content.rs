//! Content-related data models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A content item.
///
/// Content represents a single manga series, comic, or novel that has been
/// imported into a library. Each content item can have multiple chapters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Content {
    /// Unique identifier for the content.
    pub id: i64,
    /// ID of the library this content belongs to.
    pub library_id: i64,
    /// ID of the scan path this content was imported from.
    pub scan_path_id: i64,
    /// Title of the content (derived from folder name).
    pub title: String,
    /// Path to the content folder on the file system.
    pub folder_path: String,
    /// Number of chapters in this content.
    pub chapter_count: i32,
    /// Compressed thumbnail image data.
    #[sqlx(default)]
    pub thumbnail: Option<Vec<u8>>,
    /// Metadata from Bangumi API (stored as JSON blob).
    #[sqlx(default)]
    pub metadata: Option<Vec<u8>>,
    /// Timestamp when the content was imported.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the content was last updated.
    pub updated_at: DateTime<Utc>,
}

impl Content {
    /// Creates a new Content instance for insertion.
    pub fn create(
        library_id: i64,
        scan_path_id: i64,
        title: String,
        folder_path: String,
    ) -> NewContent {
        NewContent {
            library_id,
            scan_path_id,
            title,
            folder_path,
            chapter_count: 0,
            thumbnail: None,
            metadata: None,
        }
    }
}

/// Data for creating new content.
#[derive(Debug, Clone)]
pub struct NewContent {
    pub library_id: i64,
    pub scan_path_id: i64,
    pub title: String,
    pub folder_path: String,
    pub chapter_count: i32,
    pub thumbnail: Option<Vec<u8>>,
    pub metadata: Option<serde_json::Value>,
}

/// A chapter within a content item.
///
/// Chapters represent individual archive files (volumes, issues, etc.)
/// within a content folder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Chapter {
    /// Unique identifier for the chapter.
    pub id: i64,
    /// ID of the content this chapter belongs to.
    pub content_id: i64,
    /// Display title of the chapter.
    pub title: String,
    /// Path to the chapter archive file.
    pub file_path: String,
    /// File type (extension) of the chapter file (e.g. "cbz", "pdf", "epub").
    pub file_type: String,
    /// Sort order for displaying chapters.
    pub sort_order: i32,
    /// Number of pages/images in this chapter (0 if not yet calculated).
    #[sqlx(default)]
    pub page_count: i32,
    /// File size in bytes.
    #[sqlx(default)]
    pub size: i64,
}

impl Chapter {
    /// Creates a new Chapter instance for insertion.
    pub fn create(
        content_id: i64,
        title: String,
        file_path: String,
        file_type: String,
        sort_order: i32,
        page_count: i32,
        size: i64,
    ) -> NewChapter {
        NewChapter {
            content_id,
            title,
            file_path,
            file_type,
            sort_order,
            page_count,
            size,
        }
    }

    /// Returns true if this chapter is a text-based format (epub).
    pub fn is_text_based(&self) -> bool {
        self.file_type == "epub"
    }

    /// Returns true if this chapter is an image-based format (zip, cbz, cbr, rar, pdf).
    pub fn is_image_based(&self) -> bool {
        matches!(
            self.file_type.as_str(),
            "zip" | "cbz" | "cbr" | "rar" | "pdf"
        )
    }
}

/// Data for creating a new chapter.
#[derive(Debug, Clone)]
pub struct NewChapter {
    pub content_id: i64,
    pub title: String,
    pub file_path: String,
    pub file_type: String,
    pub sort_order: i32,
    pub page_count: i32,
    pub size: i64,
}

/// Response structure for content list API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentResponse {
    pub id: i64,
    pub library_id: i64,
    pub title: String,
    pub chapter_count: i32,
    pub has_thumbnail: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl From<Content> for ContentResponse {
    fn from(content: Content) -> Self {
        Self {
            id: content.id,
            library_id: content.library_id,
            title: content.title,
            chapter_count: content.chapter_count,
            has_thumbnail: content.thumbnail.is_some(),
            metadata: content
                .metadata
                .and_then(|bytes| serde_json::from_slice(&bytes).ok()),
            created_at: content.created_at,
        }
    }
}

/// Helper to extract file type (extension) from a path.
pub fn file_type_from_path(path: &std::path::Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default()
}

/// All supported archive extensions.
pub const ALL_SUPPORTED_EXTENSIONS: &[&str] = &["zip", "cbz", "cbr", "rar", "pdf", "epub"];
