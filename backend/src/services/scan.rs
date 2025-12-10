//! Library scanning service.
//!
//! This module provides functionality to scan library paths for content,
//! detect content folders, identify chapters, and generate thumbnails.

use sqlx::{Pool, Sqlite};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, error, instrument, warn};

use crate::error::{AppError, Result};
use crate::extractors::{ComicArchiveExtractor, NovelArchiveExtractor, natural_sort_key};
use crate::models::{Content, ContentType, NewChapter, NewContent, ScanPath};
use crate::repository::content::{ChapterRepository, ContentRepository};
use crate::repository::library::ScanPathRepository;
use crate::services::bangumi::BangumiService;

/// Result of a library scan operation.
#[derive(Debug, Default)]
pub struct ScanResult {
    /// Newly added content items.
    pub added: Vec<Content>,
    /// IDs of content items that were removed (folder no longer exists).
    pub removed: Vec<i64>,
    /// Content items that failed metadata scraping, with error messages.
    pub failed_scrape: Vec<(Content, String)>,
}

/// Service for scanning library paths and importing content.
pub struct ScanService {
    pool: Pool<Sqlite>,
    bangumi_service: Option<Arc<BangumiService>>,
}

impl ScanService {
    /// Create a new scan service.
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self {
            pool,
            bangumi_service: None,
        }
    }

    /// Create a new scan service with Bangumi integration for auto-scraping.
    pub fn with_bangumi(pool: Pool<Sqlite>, bangumi_service: Arc<BangumiService>) -> Self {
        Self {
            pool,
            bangumi_service: Some(bangumi_service),
        }
    }

    /// Set the Bangumi service for auto-scraping.
    pub fn set_bangumi_service(&mut self, bangumi_service: Arc<BangumiService>) {
        self.bangumi_service = Some(bangumi_service);
    }

    /// Scan all paths in a library and import/update content.
    ///
    /// Requirements: 2.1
    #[instrument(skip(self), fields(library_id = library_id))]
    pub async fn scan_library(&self, library_id: i64) -> Result<ScanResult> {
        let scan_paths = ScanPathRepository::list_by_library(&self.pool, library_id).await?;

        let mut result = ScanResult::default();

        for scan_path in scan_paths {
            let path_result = self.scan_path(&scan_path).await?;
            result.added.extend(path_result.added);
            result.removed.extend(path_result.removed);
            result.failed_scrape.extend(path_result.failed_scrape);
        }

        Ok(result)
    }

    /// Scan a single scan path and import/update content.
    #[instrument(skip(self), fields(scan_path_id = scan_path.id, path = %scan_path.path))]
    pub async fn scan_path(&self, scan_path: &ScanPath) -> Result<ScanResult> {
        let mut result = ScanResult::default();
        let base_path = Path::new(&scan_path.path);

        // Check if the scan path exists
        if !base_path.exists() {
            return Err(AppError::FileSystem(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Scan path does not exist: {}", scan_path.path),
            )));
        }

        // Get existing content folder paths for this scan path
        let existing_paths: HashSet<String> =
            ContentRepository::get_folder_paths_by_scan_path(&self.pool, scan_path.id)
                .await?
                .into_iter()
                .collect();

        // Scan for content folders
        let discovered_folders = self.discover_content_folders(base_path)?;
        let discovered_paths: HashSet<String> = discovered_folders
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        // Find removed content (exists in DB but not on disk)
        for existing_path in &existing_paths {
            if !discovered_paths.contains(existing_path) {
                // Content folder was removed from disk
                if let Some(content) = ContentRepository::find_by_folder_path(
                    &self.pool,
                    scan_path.library_id,
                    existing_path,
                )
                .await?
                {
                    ContentRepository::delete(&self.pool, content.id).await?;
                    result.removed.push(content.id);
                }
            }
        }

        // Find new content (exists on disk but not in DB)
        for folder_path in discovered_folders {
            let folder_path_str = folder_path.to_string_lossy().to_string();

            if !existing_paths.contains(&folder_path_str) {
                // New content folder found
                match self.import_content_folder(scan_path, &folder_path).await {
                    Ok((content, scrape_error)) => {
                        if let Some(error_msg) = scrape_error {
                            // Content was imported but metadata scraping failed
                            result.failed_scrape.push((content.clone(), error_msg));
                        }
                        result.added.push(content);
                    }
                    Err(e) => {
                        // Log error but continue scanning
                        error!(folder_path = ?folder_path, error = %e, "Failed to import content");
                    }
                }
            }
        }

        Ok(result)
    }

    /// Discover content folders within a scan path.
    /// Content folders are immediate subdirectories that contain archive files.
    fn discover_content_folders(&self, base_path: &Path) -> Result<Vec<PathBuf>> {
        let mut content_folders = Vec::new();

        let entries = std::fs::read_dir(base_path)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Check if this directory contains any supported archive files
                if self.has_archive_files(&path)? {
                    content_folders.push(path);
                }
            }
        }

        // Sort folders by name using natural sort
        content_folders.sort_by_key(|p| {
            natural_sort_key(&p.file_name().unwrap_or_default().to_string_lossy())
        });

        Ok(content_folders)
    }

    /// Check if a directory contains any supported archive files.
    fn has_archive_files(&self, dir: &Path) -> Result<bool> {
        let entries = std::fs::read_dir(dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file()
                && (ComicArchiveExtractor::is_supported(&path)
                    || NovelArchiveExtractor::is_supported(&path))
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Import a content folder into the database.
    ///
    /// Returns the imported content and an optional error message if metadata scraping failed.
    ///
    /// Requirements: 2.2, 2.3, 2.4, 8.1, 8.2, 8.3
    async fn import_content_folder(
        &self,
        scan_path: &ScanPath,
        folder_path: &Path,
    ) -> Result<(Content, Option<String>)> {
        // Derive title from folder name (Requirement 2.4)
        let title = folder_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| AppError::BadRequest("Invalid folder name".to_string()))?
            .to_string();

        // Detect content type and find chapters
        let (content_type, chapters) = self.detect_content_type_and_chapters(folder_path)?;

        // Auto-scrape metadata from Bangumi if service is available (Requirements: 8.1, 8.2, 8.3)
        let (metadata, scrape_error) = self.auto_scrape_metadata(&title).await;

        // Create the content record
        let new_content = NewContent {
            library_id: scan_path.library_id,
            scan_path_id: scan_path.id,
            content_type,
            title,
            folder_path: folder_path.to_string_lossy().to_string(),
            chapter_count: chapters.len() as i32,
            thumbnail: None,
            metadata,
        };

        let content = ContentRepository::create(&self.pool, new_content).await?;

        // Create chapter records
        let new_chapters: Vec<NewChapter> = chapters
            .into_iter()
            .enumerate()
            .map(|(idx, (chapter_title, file_path))| NewChapter {
                content_id: content.id,
                title: chapter_title,
                file_path,
                sort_order: idx as i32,
            })
            .collect();

        ChapterRepository::create_batch(&self.pool, new_chapters).await?;

        // Generate thumbnail
        let thumbnail = self.generate_thumbnail(&content, folder_path).await;
        if let Ok(Some(thumb_data)) = thumbnail {
            ContentRepository::update_thumbnail(&self.pool, content.id, Some(thumb_data)).await?;
        }

        // Fetch the updated content with thumbnail
        let final_content = ContentRepository::find_by_id(&self.pool, content.id)
            .await?
            .ok_or_else(|| AppError::Internal("Failed to retrieve created content".to_string()))?;

        Ok((final_content, scrape_error))
    }

    /// Auto-scrape metadata from Bangumi for a content title.
    ///
    /// Returns the metadata JSON blob if successful, or None with an error message if failed.
    ///
    /// Requirements: 8.1, 8.2, 8.3
    async fn auto_scrape_metadata(
        &self,
        title: &str,
    ) -> (Option<serde_json::Value>, Option<String>) {
        let Some(ref bangumi_service) = self.bangumi_service else {
            // No Bangumi service configured, skip scraping
            return (None, None);
        };

        match bangumi_service.auto_scrape(title).await {
            Ok(Some(metadata)) => {
                // Successfully scraped metadata (Requirement 8.2)
                (Some(metadata), None)
            }
            Ok(None) => {
                // No results found (Requirement 8.3)
                let error_msg = format!("No Bangumi results found for '{}'", title);
                debug!(title = %title, "No Bangumi results found");
                (None, Some(error_msg))
            }
            Err(e) => {
                // Scraping failed (Requirement 8.3)
                let error_msg = format!("Failed to scrape metadata for '{}': {}", title, e);
                warn!(title = %title, error = %e, "Failed to scrape metadata");
                (None, Some(error_msg))
            }
        }
    }

    /// Detect content type based on archive files and return sorted chapters.
    ///
    /// Requirements: 2.2, 2.3
    fn detect_content_type_and_chapters(
        &self,
        folder_path: &Path,
    ) -> Result<(ContentType, Vec<(String, String)>)> {
        let mut comic_files = Vec::new();
        let mut novel_files = Vec::new();

        let entries = std::fs::read_dir(folder_path)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if ComicArchiveExtractor::is_supported(&path) {
                    comic_files.push(path);
                } else if NovelArchiveExtractor::is_supported(&path) {
                    novel_files.push(path);
                }
            }
        }

        // Determine content type based on which type has more files
        // If equal, prefer comics
        let (content_type, files) =
            if comic_files.len() >= novel_files.len() && !comic_files.is_empty() {
                (ContentType::Comic, comic_files)
            } else if !novel_files.is_empty() {
                (ContentType::Novel, novel_files)
            } else {
                return Err(AppError::BadRequest(
                    "No supported archive files found in folder".to_string(),
                ));
            };

        // Sort files by filename using natural sort
        let mut files = files;
        files.sort_by_key(|p| {
            natural_sort_key(&p.file_name().unwrap_or_default().to_string_lossy())
        });

        // Create chapter entries (title derived from filename without extension)
        let chapters: Vec<(String, String)> = files
            .into_iter()
            .map(|path| {
                let title = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string();
                let file_path = path.to_string_lossy().to_string();
                (title, file_path)
            })
            .collect();

        Ok((content_type, chapters))
    }

    /// Generate a thumbnail for content.
    ///
    /// Requirements: 2.5, 2.6
    async fn generate_thumbnail(
        &self,
        content: &Content,
        folder_path: &Path,
    ) -> Result<Option<Vec<u8>>> {
        match content.content_type {
            ContentType::Comic => self.generate_comic_thumbnail(folder_path),
            ContentType::Novel => self.generate_novel_thumbnail(folder_path),
        }
    }

    /// Generate thumbnail for comics from the first page of the first chapter.
    ///
    /// Requirements: 2.5
    fn generate_comic_thumbnail(&self, folder_path: &Path) -> Result<Option<Vec<u8>>> {
        // Find the first comic archive file
        let entries = std::fs::read_dir(folder_path)?;
        let mut comic_files: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file() && ComicArchiveExtractor::is_supported(p))
            .collect();

        if comic_files.is_empty() {
            return Ok(None);
        }

        // Sort to get the first chapter
        comic_files.sort_by_key(|p| {
            natural_sort_key(&p.file_name().unwrap_or_default().to_string_lossy())
        });

        let first_chapter = &comic_files[0];

        // Extract the first image from the first chapter
        let image_data = ComicArchiveExtractor::extract_first_image(first_chapter)?;

        // Resize and compress the thumbnail
        let thumbnail = self.compress_thumbnail(&image_data)?;

        Ok(Some(thumbnail))
    }

    /// Generate default thumbnail for novels.
    ///
    /// Requirements: 2.6
    fn generate_novel_thumbnail(&self, folder_path: &Path) -> Result<Option<Vec<u8>>> {
        // Check if there's a cover image in the folder
        let cover_names = ["cover.jpg", "cover.jpeg", "cover.png", "cover.webp"];

        for cover_name in cover_names {
            let cover_path = folder_path.join(cover_name);
            if cover_path.exists() {
                let image_data = std::fs::read(&cover_path)?;
                let thumbnail = self.compress_thumbnail(&image_data)?;
                return Ok(Some(thumbnail));
            }
        }

        // Check for EPUB files which might have embedded covers
        let entries = std::fs::read_dir(folder_path)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            let is_epub = path.is_file()
                && path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|ext| ext.to_lowercase() == "epub")
                    .unwrap_or(false);

            if let (true, Ok(Some(cover))) = (is_epub, self.extract_epub_cover(&path)) {
                let thumbnail = self.compress_thumbnail(&cover)?;
                return Ok(Some(thumbnail));
            }
        }

        // No cover found, return None (will use default placeholder in frontend)
        Ok(None)
    }

    /// Extract cover image from an EPUB file.
    fn extract_epub_cover(&self, epub_path: &Path) -> Result<Option<Vec<u8>>> {
        let mut doc = epub::doc::EpubDoc::new(epub_path)
            .map_err(|e| AppError::Archive(format!("Failed to open EPUB: {}", e)))?;

        // Try to get the cover image
        if let Some((cover_data, _mime)) = doc.get_cover() {
            return Ok(Some(cover_data));
        }

        Ok(None)
    }

    /// Compress and resize an image for use as a thumbnail.
    fn compress_thumbnail(&self, image_data: &[u8]) -> Result<Vec<u8>> {
        use image::ImageReader;
        use std::io::Cursor;

        // Load the image
        let img = ImageReader::new(Cursor::new(image_data))
            .with_guessed_format()
            .map_err(|e| AppError::Internal(format!("Failed to read image format: {}", e)))?
            .decode()
            .map_err(|e| AppError::Internal(format!("Failed to decode image: {}", e)))?;

        // Resize to thumbnail size (max 300px width, maintaining aspect ratio)
        let thumbnail = img.thumbnail(300, 450);

        // Encode as JPEG with quality 80
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        thumbnail
            .write_to(&mut cursor, image::ImageFormat::Jpeg)
            .map_err(|e| AppError::Internal(format!("Failed to encode thumbnail: {}", e)))?;

        Ok(buffer)
    }
}
