//! Archive extractor for ZIP, CBZ, CBR, RAR, 7Z, and EPUB formats.
//!
//! This module provides functionality to extract content from compressed archive files.
//! Uses `unarc-rs` for unified archive handling with streaming support.
//!
//! Supported formats:
//! - ZIP/CBZ: Standard ZIP archives
//! - CBR/RAR: RAR archives (RAR5 only)
//! - 7Z/CB7: 7-Zip archives
//! - EPUB: Electronic publication format (ZIP with specific structure)

use crate::error::{AppError, Result};
use rust_i18n::t;
use serde::Serialize;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

use super::natural_sort_key;

use unarc_rs::unified::{ArchiveFormat, UnifiedArchive};

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

/// Archive extractor supporting ZIP, CBZ, CBR, RAR, 7Z, and EPUB formats.
///
/// All extractions use streaming reads to minimize memory usage.
pub struct ArchiveExtractor;

impl ArchiveExtractor {
    /// Returns the supported archive extensions.
    pub fn supported_extensions() -> &'static [&'static str] {
        &["zip", "cbz", "cbr", "rar", "7z", "cb7", "epub"]
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

    /// Detects the archive format from file extension.
    fn detect_format(archive_path: &Path) -> Result<ArchiveFormat> {
        let ext = archive_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            "zip" | "cbz" => Ok(ArchiveFormat::Zip),
            "rar" | "cbr" => Ok(ArchiveFormat::Rar),
            "7z" | "cb7" => Ok(ArchiveFormat::SevenZ),
            "epub" => Ok(ArchiveFormat::Zip), // EPUB is a ZIP file
            _ => Err(AppError::Archive(
                t!("archive.unsupported_comic_format", extension = ext).to_string(),
            )),
        }
    }

    /// Lists all image files in the archive, sorted by filename.
    /// For EPUB files, lists spine items (chapter idrefs) in reading order.
    pub fn list_files(archive_path: &Path) -> Result<Vec<String>> {
        // EPUB uses special handling via epub crate
        if Self::is_epub(archive_path) {
            return Self::list_epub_files(archive_path);
        }

        let format = Self::detect_format(archive_path)?;
        let file = File::open(archive_path)
            .map_err(|e| AppError::Archive(t!("archive.open_failed", error = e).to_string()))?;
        let reader = BufReader::new(file);

        let mut archive = UnifiedArchive::open_with_format(reader, format)
            .map_err(|e| AppError::Archive(t!("archive.open_failed", error = e).to_string()))?;

        let mut files: Vec<String> = Vec::new();

        while let Some(entry) = archive.next_entry().map_err(|e| {
            AppError::Archive(t!("archive.read_entry_failed", error = e).to_string())
        })? {
            let name = entry.name().to_string();
            if Self::is_image_file(&name) {
                files.push(name);
            }
        }

        // Sort files using natural sort order
        files.sort_by_key(|a| natural_sort_key(a));
        Ok(files)
    }

    /// Extracts a specific file from the archive using streaming.
    ///
    /// This method streams data in chunks to minimize memory usage.
    pub fn extract_file(archive_path: &Path, file_name: &str) -> Result<Vec<u8>> {
        // EPUB uses special handling
        if Self::is_epub(archive_path) {
            return Self::extract_resource_bytes(archive_path, file_name);
        }

        let format = Self::detect_format(archive_path)?;
        let file = File::open(archive_path)
            .map_err(|e| AppError::Archive(t!("archive.open_failed", error = e).to_string()))?;
        let reader = BufReader::new(file);

        let mut archive = UnifiedArchive::open_with_format(reader, format)
            .map_err(|e| AppError::Archive(t!("archive.open_failed", error = e).to_string()))?;

        // Find the requested file
        while let Some(entry) = archive.next_entry().map_err(|e| {
            AppError::Archive(t!("archive.read_entry_failed", error = e).to_string())
        })? {
            if entry.name() == file_name {
                // Stream the data in chunks
                let mut output = Vec::with_capacity(entry.original_size() as usize);
                archive.read_to(&entry, &mut output).map_err(|e| {
                    AppError::Archive(t!("archive.file_read_failed", error = e).to_string())
                })?;
                return Ok(output);
            }
        }

        Err(AppError::Archive(
            t!("archive.file_not_found", file = file_name).to_string(),
        ))
    }

    /// Extracts a file and streams it directly to a writer.
    ///
    /// This is the most memory-efficient method for large files.
    pub fn extract_file_to<W: Write>(
        archive_path: &Path,
        file_name: &str,
        writer: &mut W,
    ) -> Result<u64> {
        // EPUB uses special handling
        if Self::is_epub(archive_path) {
            return Self::extract_resource_to(archive_path, file_name, writer);
        }

        let format = Self::detect_format(archive_path)?;
        let file = File::open(archive_path)
            .map_err(|e| AppError::Archive(t!("archive.open_failed", error = e).to_string()))?;
        let reader = BufReader::new(file);

        let mut archive = UnifiedArchive::open_with_format(reader, format)
            .map_err(|e| AppError::Archive(t!("archive.open_failed", error = e).to_string()))?;

        // Find the requested file
        while let Some(entry) = archive.next_entry().map_err(|e| {
            AppError::Archive(t!("archive.read_entry_failed", error = e).to_string())
        })? {
            if entry.name() == file_name {
                return archive.read_to(&entry, writer).map_err(|e| {
                    AppError::Archive(t!("archive.file_read_failed", error = e).to_string())
                });
            }
        }

        Err(AppError::Archive(
            t!("archive.file_not_found", file = file_name).to_string(),
        ))
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
        let mut output = Vec::new();
        Self::extract_resource_to(archive_path, resource_path, &mut output)?;
        Ok(output)
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

    /// Extracts an EPUB resource and streams it directly to a writer.
    fn extract_resource_to<W: Write>(
        archive_path: &Path,
        resource_path: &str,
        writer: &mut W,
    ) -> Result<u64> {
        let file = File::open(archive_path).map_err(|e| {
            AppError::Archive(t!("archive.epub_open_failed", error = e).to_string())
        })?;
        let reader = BufReader::new(file);

        let mut archive =
            UnifiedArchive::open_with_format(reader, ArchiveFormat::Zip).map_err(|e| {
                AppError::Archive(t!("archive.epub_open_failed", error = e).to_string())
            })?;

        while let Some(entry) = archive.next_entry().map_err(|e| {
            AppError::Archive(t!("archive.read_entry_failed", error = e).to_string())
        })? {
            if entry.name() == resource_path {
                return archive.read_to(&entry, writer).map_err(|e| {
                    AppError::Archive(t!("archive.file_read_failed", error = e).to_string())
                });
            }
        }

        Err(AppError::NotFound(
            t!("archive.chapter_not_found_in_epub", file = resource_path).to_string(),
        ))
    }

    // ── Helper methods ────────────────────────────────────────────────────

    /// Checks if a filename is an image file based on extension.
    fn is_image_file(name: &str) -> bool {
        // Normalize backslashes and strip NUL suffix for robust matching
        let normalized = name.replace('\\', "/");
        let visible = normalized.split('\0').next().unwrap_or(&normalized);
        let lower = visible.to_lowercase();
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
        assert!(ArchiveExtractor::is_image_file("folder\\test.jpeg"));
        assert!(ArchiveExtractor::is_image_file(
            "folder\\test.jpg\0ignored_binary_suffix"
        ));
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
        assert!(exts.contains(&"7z"));
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
