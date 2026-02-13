//! Content management service.
//!
//! This module provides the business logic for content operations including
//! retrieval, listing, searching, deletion, and chapter management.

use rust_i18n::t;
use sqlx::{Pool, Sqlite};
use std::path::Path;

use crate::error::{AppError, Result};
use crate::extractors::{ComicArchiveExtractor, NovelArchiveExtractor, PdfExtractor};
use crate::models::{Chapter, Content};
use crate::repository::content::{ChapterRepository, ContentRepository};

/// Service for content management operations.
pub struct ContentService;

impl ContentService {
    /// Get a content by ID.
    pub async fn get_content(pool: &Pool<Sqlite>, id: i64) -> Result<Content> {
        ContentRepository::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(t!("content.id_not_found", id = id).to_string()))
    }

    /// List all contents for a library.
    pub async fn list_contents(pool: &Pool<Sqlite>, library_id: i64) -> Result<Vec<Content>> {
        ContentRepository::list_by_library(pool, library_id).await
    }

    /// Search contents by title within a library.
    pub async fn search_contents(
        pool: &Pool<Sqlite>,
        library_id: i64,
        query: &str,
    ) -> Result<Vec<Content>> {
        ContentRepository::search_by_title(pool, library_id, query).await
    }

    /// Delete a content by ID.
    /// This will cascade delete all associated chapters due to database constraints.
    pub async fn delete_content(pool: &Pool<Sqlite>, id: i64) -> Result<()> {
        // First verify the content exists
        let _content = Self::get_content(pool, id).await?;

        // Delete the content (chapters are cascade deleted by the database)
        ContentRepository::delete(pool, id).await
    }

    /// List all chapters for a content.
    pub async fn list_chapters(pool: &Pool<Sqlite>, content_id: i64) -> Result<Vec<Chapter>> {
        // First verify the content exists
        let _content = Self::get_content(pool, content_id).await?;

        let chapters = ChapterRepository::list_by_content(pool, content_id).await?;

        Ok(chapters)
    }

    /// Get a specific page image from an image-based chapter.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `content_id` - ID of the content
    /// * `chapter_id` - ID of the chapter
    /// * `page_index` - 0-based index of the page within the chapter
    ///
    /// # Returns
    /// The raw image bytes for the requested page.
    pub async fn get_page(
        pool: &Pool<Sqlite>,
        content_id: i64,
        chapter_id: i64,
        page_index: i64,
    ) -> Result<Vec<u8>> {
        // Get the content to verify it exists
        let _content = Self::get_content(pool, content_id).await?;

        // Get the chapters
        let chapters = ChapterRepository::list_by_content(pool, content_id).await?;

        // Find the chapter by id
        let chapter = chapters
            .iter()
            .find(|c| c.id == chapter_id)
            .ok_or_else(|| {
                AppError::NotFound(t!("content.chapter_not_found", id = chapter_id).to_string())
            })?;

        // Verify this is an image-based or text-based chapter
        if !chapter.is_image_based() && !chapter.is_text_based() {
            return Err(AppError::BadRequest(
                "Cannot get page from non-image-based or non-text-based chapter".to_string(),
            ));
        }

        let archive_path = Path::new(&chapter.file_path);

        // List files/images/sections in the archive
        let files = if chapter.is_text_based() {
            NovelArchiveExtractor::list_files(archive_path)?
        } else if PdfExtractor::is_supported(archive_path) {
            PdfExtractor::list_files(archive_path)?
        } else {
            ComicArchiveExtractor::list_files(archive_path)?
        };

        // Validate page index
        if page_index < 0 || page_index as usize >= files.len() {
            return Err(AppError::NotFound(
                t!("komga.page_not_found", page = page_index).to_string(),
            ));
        }

        let file_name = &files[page_index as usize];

        // Extract and return the content
        if chapter.is_text_based() {
            let text = NovelArchiveExtractor::extract_file(archive_path, file_name)?;
            Ok(text.into_bytes())
        } else if PdfExtractor::is_supported(archive_path) {
            PdfExtractor::extract_file(archive_path, file_name)
        } else {
            ComicArchiveExtractor::extract_file(archive_path, file_name)
        }
    }

    /// Get the text content of a novel chapter.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `content_id` - ID of the content
    /// * `chapter_index` - 0-based index of the chapter
    ///
    /// # Returns
    /// The text content of the chapter.
    pub async fn get_chapter_text(
        pool: &Pool<Sqlite>,
        content_id: i64,
        chapter_index: i32,
    ) -> Result<String> {
        // Get the content
        let _content = Self::get_content(pool, content_id).await?;

        // Get the chapters
        let chapters = ChapterRepository::list_by_content(pool, content_id).await?;

        // Validate chapter index
        if chapter_index < 0 || chapter_index as usize >= chapters.len() {
            let chapter_id = chapter_index as i64; // Approximation for error message
            return Err(AppError::NotFound(
                t!("content.chapter_not_found", id = chapter_id).to_string(),
            ));
        }

        let chapter = &chapters[chapter_index as usize];

        // Verify this is a text-based chapter
        if !chapter.is_text_based() {
            return Err(AppError::BadRequest(
                "Cannot get text from non-text-based chapter".to_string(),
            ));
        }

        let archive_path = Path::new(&chapter.file_path);

        // Extract all text from the chapter archive
        NovelArchiveExtractor::extract_all_text(archive_path)
    }

    /// Get the page count for a specific chapter.
    pub async fn get_chapter_page_count(
        pool: &Pool<Sqlite>,
        content_id: i64,
        chapter_index: i32,
    ) -> Result<usize> {
        // Get the content
        let _content = Self::get_content(pool, content_id).await?;

        // Get the chapters
        let chapters = ChapterRepository::list_by_content(pool, content_id).await?;

        // Validate chapter index
        if chapter_index < 0 || chapter_index as usize >= chapters.len() {
            let chapter_id = chapter_index as i64; // Approximation for error message
            return Err(AppError::NotFound(
                t!("content.chapter_not_found", id = chapter_id).to_string(),
            ));
        }

        let chapter = &chapters[chapter_index as usize];
        let archive_path = Path::new(&chapter.file_path);

        // Branch based on file type
        if chapter.is_text_based() {
            NovelArchiveExtractor::chapter_count(archive_path)
        } else if PdfExtractor::is_supported(archive_path) {
            PdfExtractor::page_count(archive_path)
        } else {
            ComicArchiveExtractor::page_count(archive_path)
        }
    }

    /// Update content information.
    pub async fn update_content(
        pool: &Pool<Sqlite>,
        id: i64,
        title: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Content> {
        // First verify the content exists
        let _content = Self::get_content(pool, id).await?;

        // Handle thumbnail logic if metadata is updated
        let thumbnail_update = if let Some(meta) = &metadata {
            // If we have metadata with cover image, use it
            if let Some(cover_data) = meta
                .get("images")
                .and_then(|v| v.get("common"))
                .and_then(|s| s.as_str())
            {
                // New thumbnail found
                Some(crate::utils::download_image(cover_data).await.ok())
            } else {
                Some(None)
            }
        } else {
            // Metadata not updated -> Thumbnail not updated
            None
        };

        // Convert metadata to Option<Option<Value>> for the repository
        let metadata_update = metadata.map(Some);

        ContentRepository::update_info(pool, id, title, metadata_update, thumbnail_update).await
    }

    /// Get thumbnail for a content.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `content_id` - ID of the content
    ///
    /// # Returns
    /// The thumbnail image bytes if available.
    pub async fn get_thumbnail(pool: &Pool<Sqlite>, content_id: i64) -> Result<Vec<u8>> {
        let content = Self::get_content(pool, content_id).await?;

        content.thumbnail.ok_or_else(|| {
            AppError::NotFound(t!("content.thumbnail_not_found", id = content_id).to_string())
        })
    }
}
