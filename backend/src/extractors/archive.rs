//! Archive extractor for ZIP, CBZ, CBR, RAR, and EPUB formats.
//!
//! This module provides functionality to extract content from compressed archive files.
//! Supported formats:
//! - ZIP/CBZ: Standard ZIP archives (CBZ is just ZIP with a different extension)
//! - CBR/RAR: RAR archives
//! - EPUB: Electronic publication format (ZIP with specific structure)

use crate::error::{AppError, Result};
use rust_i18n::t;
use serde::Serialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::natural_sort_key;

/// Supported image extensions for comics.
const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "bmp"];

/// A single item in the EPUB spine (reading order).
#[derive(Debug, Clone, Serialize)]
pub struct SpineEntry {
    /// Path of the resource within the EPUB ZIP archive.
    pub path: String,
    /// MIME type of the resource.
    pub mime_type: String,
}

/// Archive extractor supporting ZIP, CBZ, CBR, RAR, and EPUB formats.
pub struct ArchiveExtractor;

impl ArchiveExtractor {
    /// Returns the supported archive extensions.
    pub fn supported_extensions() -> &'static [&'static str] {
        &["zip", "cbz", "cbr", "rar", "epub"]
    }

    /// Checks if a file extension is supported.
    pub fn is_supported(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| Self::supported_extensions().contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
    }

    /// Checks if a file is an EPUB.
    pub fn is_epub(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase() == "epub")
            .unwrap_or(false)
    }

    /// Lists all image files in the archive, sorted by filename.
    /// For EPUB files, lists spine items (chapter idrefs) in reading order.
    pub fn list_files(archive_path: &Path) -> Result<Vec<String>> {
        let ext = archive_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            "zip" | "cbz" => Self::list_zip_files(archive_path),
            "cbr" | "rar" => Self::list_rar_files(archive_path),
            "epub" => Self::list_epub_files(archive_path),
            _ => Err(AppError::Archive(
                t!("archive.unsupported_comic_format", extension = ext).to_string(),
            )),
        }
    }

    /// Extracts a specific file from the archive.
    /// For EPUB, extracts text content (stripped of HTML) from a spine item.
    pub fn extract_file(archive_path: &Path, file_name: &str) -> Result<Vec<u8>> {
        let ext = archive_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            "zip" | "cbz" => Self::extract_zip_file(archive_path, file_name),
            "cbr" | "rar" => Self::extract_rar_file(archive_path, file_name),
            _ => Err(AppError::Archive(
                t!("archive.unsupported_comic_format", extension = ext).to_string(),
            )),
        }
    }

    /// Extracts the first image from the archive (for thumbnail generation).
    pub fn extract_first_image(archive_path: &Path) -> Result<Vec<u8>> {
        let files = Self::list_files(archive_path)?;
        let first_image = files
            .first()
            .ok_or_else(|| AppError::Archive(t!("archive.no_images_found").to_string()))?;
        Self::extract_file(archive_path, first_image)
    }

    /// Gets the page count (number of images) in the archive.
    /// For EPUB, returns the number of spine items.
    pub fn page_count(archive_path: &Path) -> Result<usize> {
        let files = Self::list_files(archive_path)?;
        Ok(files.len())
    }

    // ── EPUB-specific methods ─────────────────────────────────────────────

    /// Returns the EPUB spine as a list of resolved file paths and MIME types.
    ///
    /// Maps spine idrefs through `doc.resources` to get actual ZIP paths.
    pub fn get_epub_spine(archive_path: &Path) -> Result<Vec<SpineEntry>> {
        let doc = epub::doc::EpubDoc::new(archive_path).map_err(|e| {
            AppError::Archive(t!("archive.epub_open_failed", error = e).to_string())
        })?;

        let mut entries = Vec::new();
        for item in &doc.spine {
            if let Some(resource) = doc.resources.get(&item.idref) {
                entries.push(SpineEntry {
                    path: resource.path.to_string_lossy().to_string(),
                    mime_type: resource.mime.clone(),
                });
            }
        }

        Ok(entries)
    }

    /// Extracts raw bytes of a resource from within the EPUB ZIP archive.
    ///
    /// This is used for serving individual EPUB resources (XHTML, images, CSS,
    /// fonts, etc.) to the frontend for on-demand rendering.
    pub fn extract_resource_bytes(archive_path: &Path, resource_path: &str) -> Result<Vec<u8>> {
        let file = File::open(archive_path).map_err(|e| {
            AppError::Archive(t!("archive.epub_open_failed", error = e).to_string())
        })?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| {
            AppError::Archive(t!("archive.epub_open_failed", error = e).to_string())
        })?;

        let mut entry = archive.by_name(resource_path).map_err(|_| {
            AppError::NotFound(
                t!("archive.chapter_not_found_in_epub", file = resource_path).to_string(),
            )
        })?;

        let mut buf = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut buf).map_err(|e| {
            AppError::Archive(format!(
                "Failed to read EPUB resource '{}': {}",
                resource_path, e
            ))
        })?;

        Ok(buf)
    }

    // ── ZIP/CBZ implementation ────────────────────────────────────────────

    fn list_zip_files(archive_path: &Path) -> Result<Vec<String>> {
        let file = File::open(archive_path)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| AppError::Archive(t!("archive.zip_open_failed", error = e).to_string()))?;

        let mut files: Vec<String> = Vec::new();
        for i in 0..archive.len() {
            let entry = archive.by_index(i).map_err(|e| {
                AppError::Archive(t!("archive.zip_read_entry_failed", error = e).to_string())
            })?;
            let name = entry.name().to_string();
            if Self::is_image_file(&name) {
                files.push(name);
            }
        }

        // Sort files using natural sort order
        files.sort_by_key(|a| natural_sort_key(a));
        Ok(files)
    }

    fn extract_zip_file(archive_path: &Path, file_name: &str) -> Result<Vec<u8>> {
        let file = File::open(archive_path)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| AppError::Archive(t!("archive.zip_open_failed", error = e).to_string()))?;

        let mut entry = archive.by_name(file_name).map_err(|_| {
            AppError::Archive(t!("archive.file_not_found", file = file_name).to_string())
        })?;

        let mut buffer = Vec::new();
        entry.read_to_end(&mut buffer).map_err(|e| {
            AppError::Archive(t!("archive.file_read_failed", error = e).to_string())
        })?;

        Ok(buffer)
    }

    // ── RAR/CBR implementation ────────────────────────────────────────────

    fn list_rar_files(archive_path: &Path) -> Result<Vec<String>> {
        let archive = unrar::Archive::new(archive_path)
            .open_for_listing()
            .map_err(|e| AppError::Archive(t!("archive.rar_open_failed", error = e).to_string()))?;

        let mut files: Vec<String> = Vec::new();
        let entries = archive
            .into_iter()
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| {
                AppError::Archive(t!("archive.rar_read_entries_failed", error = e).to_string())
            })?;

        for entry in entries {
            let name = entry.filename.to_string_lossy().to_string();
            if Self::is_image_file(&name) {
                files.push(name);
            }
        }

        // Sort files using natural sort order
        files.sort_by_key(|a| natural_sort_key(a));
        Ok(files)
    }

    fn extract_rar_file(archive_path: &Path, file_name: &str) -> Result<Vec<u8>> {
        // Create a temporary directory for extraction
        let temp_dir = std::env::temp_dir().join(format!("comic_extract_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;

        let archive = unrar::Archive::new(archive_path)
            .open_for_processing()
            .map_err(|e| AppError::Archive(t!("archive.rar_open_failed", error = e).to_string()))?;

        // Process entries to find and extract the target file
        let mut current = archive;
        loop {
            match current.read_header() {
                Ok(Some(header)) => {
                    let name = header.entry().filename.to_string_lossy().to_string();
                    if name == file_name {
                        // Extract this file to temp directory
                        let _next = header.extract_to(&temp_dir).map_err(|e| {
                            AppError::Archive(
                                t!("archive.rar_extract_failed", error = e).to_string(),
                            )
                        })?;

                        // Read the extracted file
                        let extracted_path = temp_dir.join(&name);
                        let content = std::fs::read(&extracted_path).map_err(|e| {
                            AppError::Archive(t!("archive.file_read_failed", error = e).to_string())
                        })?;

                        // Clean up temp directory
                        let _ = std::fs::remove_dir_all(&temp_dir);

                        return Ok(content);
                    } else {
                        // Skip this entry
                        current = header.skip().map_err(|e| {
                            AppError::Archive(t!("archive.rar_skip_failed", error = e).to_string())
                        })?;
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    let _ = std::fs::remove_dir_all(&temp_dir);
                    return Err(AppError::Archive(
                        t!("archive.rar_read_entries_failed", error = e).to_string(),
                    ));
                }
            }
        }

        // Clean up temp directory
        let _ = std::fs::remove_dir_all(&temp_dir);

        Err(AppError::Archive(
            t!("archive.file_not_found", file = file_name).to_string(),
        ))
    }

    // ── EPUB implementation ───────────────────────────────────────────────

    fn list_epub_files(archive_path: &Path) -> Result<Vec<String>> {
        let doc = epub::doc::EpubDoc::new(archive_path).map_err(|e| {
            AppError::Archive(t!("archive.epub_open_failed", error = e).to_string())
        })?;

        // Get the spine (reading order) from the EPUB
        // SpineItem has an idref field that we use as the identifier
        let files: Vec<String> = doc.spine.iter().map(|item| item.idref.clone()).collect();
        Ok(files)
    }

    // ── Helper methods ────────────────────────────────────────────────────

    /// Checks if a filename is an image file based on extension.
    fn is_image_file(name: &str) -> bool {
        let lower = name.to_lowercase();
        IMAGE_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_image_file() {
        assert!(ArchiveExtractor::is_image_file("test.jpg"));
        assert!(ArchiveExtractor::is_image_file("test.PNG"));
        assert!(ArchiveExtractor::is_image_file("folder/test.jpeg"));
        assert!(!ArchiveExtractor::is_image_file("test.txt"));
        assert!(!ArchiveExtractor::is_image_file("test.xml"));
    }

    #[test]
    fn test_supported_extensions() {
        let exts = ArchiveExtractor::supported_extensions();
        assert!(exts.contains(&"zip"));
        assert!(exts.contains(&"cbz"));
        assert!(exts.contains(&"cbr"));
        assert!(exts.contains(&"rar"));
        assert!(exts.contains(&"epub"));
    }

    #[test]
    fn test_is_epub() {
        assert!(ArchiveExtractor::is_epub(Path::new("book.epub")));
        assert!(ArchiveExtractor::is_epub(Path::new("/path/to/book.EPUB")));
        assert!(!ArchiveExtractor::is_epub(Path::new("comic.cbz")));
        assert!(!ArchiveExtractor::is_epub(Path::new("archive.zip")));
    }
}
