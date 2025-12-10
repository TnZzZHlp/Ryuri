//! Content-related data models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Content type enumeration.
///
/// Distinguishes between comic (image-based) and novel (text-based) content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
#[sqlx(type_name = "TEXT")]
pub enum ContentType {
    /// Image-based content (manga, comics, etc.)
    Comic,
    /// Text-based content (novels, light novels, etc.)
    Novel,
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentType::Comic => write!(f, "Comic"),
            ContentType::Novel => write!(f, "Novel"),
        }
    }
}

impl ContentType {
    /// Returns the supported archive extensions for this content type.
    pub fn supported_extensions(&self) -> &'static [&'static str] {
        match self {
            ContentType::Comic => &["zip", "cbz", "cbr", "rar"],
            ContentType::Novel => &["zip", "epub", "txt"],
        }
    }
}

/// A content item (comic or novel).
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
    /// Type of content (comic or novel).
    pub content_type: ContentType,
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
    pub metadata: Option<serde_json::Value>,
    /// Timestamp when the content was imported.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the content was last updated.
    pub updated_at: DateTime<Utc>,
}

impl Content {
    /// Creates a new Content instance for insertion.
    pub fn new(
        library_id: i64,
        scan_path_id: i64,
        content_type: ContentType,
        title: String,
        folder_path: String,
    ) -> NewContent {
        NewContent {
            library_id,
            scan_path_id,
            content_type,
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
    pub content_type: ContentType,
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
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct Chapter {
    /// Unique identifier for the chapter.
    pub id: i64,
    /// ID of the content this chapter belongs to.
    pub content_id: i64,
    /// Display title of the chapter.
    pub title: String,
    /// Path to the chapter archive file.
    pub file_path: String,
    /// Sort order for displaying chapters.
    pub sort_order: i32,
}

impl Chapter {
    /// Creates a new Chapter instance for insertion.
    pub fn new(content_id: i64, title: String, file_path: String, sort_order: i32) -> NewChapter {
        NewChapter {
            content_id,
            title,
            file_path,
            sort_order,
        }
    }
}

/// Data for creating a new chapter.
#[derive(Debug, Clone)]
pub struct NewChapter {
    pub content_id: i64,
    pub title: String,
    pub file_path: String,
    pub sort_order: i32,
}

/// Response structure for content list API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct ContentResponse {
    pub id: i64,
    pub library_id: i64,
    pub content_type: ContentType,
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
            content_type: content.content_type,
            title: content.title,
            chapter_count: content.chapter_count,
            has_thumbnail: content.thumbnail.is_some(),
            metadata: content.metadata,
            created_at: content.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_serialize_comic() {
        let content_type = ContentType::Comic;
        let json = serde_json::to_string(&content_type).unwrap();
        assert_eq!(json, "\"Comic\"");
    }

    #[test]
    fn test_content_type_serialize_novel() {
        let content_type = ContentType::Novel;
        let json = serde_json::to_string(&content_type).unwrap();
        assert_eq!(json, "\"Novel\"");
    }

    #[test]
    fn test_content_type_deserialize_comic() {
        let json = "\"Comic\"";
        let content_type: ContentType = serde_json::from_str(json).unwrap();
        assert_eq!(content_type, ContentType::Comic);
    }

    #[test]
    fn test_content_type_deserialize_novel() {
        let json = "\"Novel\"";
        let content_type: ContentType = serde_json::from_str(json).unwrap();
        assert_eq!(content_type, ContentType::Novel);
    }

    #[test]
    fn test_content_type_display() {
        assert_eq!(ContentType::Comic.to_string(), "Comic");
        assert_eq!(ContentType::Novel.to_string(), "Novel");
    }

    #[test]
    fn test_content_type_supported_extensions() {
        let comic_exts = ContentType::Comic.supported_extensions();
        assert!(comic_exts.contains(&"zip"));
        assert!(comic_exts.contains(&"cbz"));
        assert!(comic_exts.contains(&"cbr"));
        assert!(comic_exts.contains(&"rar"));

        let novel_exts = ContentType::Novel.supported_extensions();
        assert!(novel_exts.contains(&"zip"));
        assert!(novel_exts.contains(&"epub"));
        assert!(novel_exts.contains(&"txt"));
    }
}
