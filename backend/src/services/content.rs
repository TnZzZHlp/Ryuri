//! Content management service.
//!
//! This module provides the business logic for content operations including
//! retrieval, listing, searching, deletion, and chapter management.

use sqlx::{Pool, Sqlite};
use std::path::Path;

use crate::error::{AppError, Result};
use crate::extractors::{ComicArchiveExtractor, NovelArchiveExtractor};
use crate::models::{Chapter, Content, ContentType};
use crate::repository::content::{ChapterRepository, ContentRepository};

/// Service for content management operations.
pub struct ContentService;

impl ContentService {
    /// Get a content by ID.
    pub async fn get_content(pool: &Pool<Sqlite>, id: i64) -> Result<Content> {
        ContentRepository::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Content with id {} not found", id)))
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

    /// Get a specific page image from a comic chapter.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `content_id` - ID of the content
    /// * `chapter_index` - 0-based index of the chapter
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
        // Get the content and verify it's a comic
        let content = Self::get_content(pool, content_id).await?;

        if content.content_type != ContentType::Comic {
            return Err(AppError::BadRequest(
                "Cannot get page from non-comic content".to_string(),
            ));
        }

        // Get the chapters
        let chapters = ChapterRepository::list_by_content(pool, content_id).await?;

        // Validate chapter id
        if !chapters.iter().any(|chapter| chapter.id == chapter_id) {
            return Err(AppError::NotFound(format!(
                "Chapter {} not found for content {}",
                chapter_id, content_id
            )));
        }

        let chapter = &chapters.iter().find(|c| c.id == chapter_id).unwrap();
        let archive_path = Path::new(&chapter.file_path);

        // List images in the archive
        let images = ComicArchiveExtractor::list_files(archive_path)?;

        // Validate page index
        if page_index < 0 || page_index as usize >= images.len() {
            return Err(AppError::NotFound(format!(
                "Page {} not found in chapter {}",
                page_index, chapter_id
            )));
        }

        let image_name = &images[page_index as usize];

        // Extract and return the image
        ComicArchiveExtractor::extract_file(archive_path, image_name)
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
        // Get the content and verify it's a novel
        let content = Self::get_content(pool, content_id).await?;

        if content.content_type != ContentType::Novel {
            return Err(AppError::BadRequest(
                "Cannot get text from non-novel content".to_string(),
            ));
        }

        // Get the chapters
        let chapters = ChapterRepository::list_by_content(pool, content_id).await?;

        // Validate chapter index
        if chapter_index < 0 || chapter_index as usize >= chapters.len() {
            return Err(AppError::NotFound(format!(
                "Chapter {} not found for content {}",
                chapter_index, content_id
            )));
        }

        let chapter = &chapters[chapter_index as usize];
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
        let content = Self::get_content(pool, content_id).await?;

        // Get the chapters
        let chapters = ChapterRepository::list_by_content(pool, content_id).await?;

        // Validate chapter index
        if chapter_index < 0 || chapter_index as usize >= chapters.len() {
            return Err(AppError::NotFound(format!(
                "Chapter {} not found for content {}",
                chapter_index, content_id
            )));
        }

        let chapter = &chapters[chapter_index as usize];
        let archive_path = Path::new(&chapter.file_path);

        match content.content_type {
            ContentType::Comic => ComicArchiveExtractor::page_count(archive_path),
            ContentType::Novel => NovelArchiveExtractor::chapter_count(archive_path),
        }
    }

    /// Update content metadata.
    pub async fn update_metadata(
        pool: &Pool<Sqlite>,
        content_id: i64,
        metadata: Option<serde_json::Value>,
    ) -> Result<Content> {
        // First verify the content exists
        let _content = Self::get_content(pool, content_id).await?;

        ContentRepository::update_metadata(pool, content_id, metadata).await
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
            AppError::NotFound(format!("Thumbnail not found for content {}", content_id))
        })
    }
}
