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
const READ_CHUNK_SIZE: usize = 64 * 1024;

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
            "epub" => Self::extract_resource_bytes(archive_path, file_name),
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

        let entry_size = entry.size();
        let buf = Self::read_stream_to_vec(&mut entry, Some(entry_size)).map_err(|e| {
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

        let entry_size = entry.size();
        let buffer = Self::read_stream_to_vec(&mut entry, Some(entry_size)).map_err(|e| {
            AppError::Archive(t!("archive.file_read_failed", error = e).to_string())
        })?;

        Ok(buffer)
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
        // Normalize backslashes and strip NUL suffix for robust matching
        let normalized = name.replace('\\', "/");
        let visible = normalized.split('\0').next().unwrap_or(&normalized);
        let lower = visible.to_lowercase();
        IMAGE_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
    }

    /// Reads from a stream incrementally to avoid one-shot full-file reads.
    fn read_stream_to_vec<R: Read>(
        reader: &mut R,
        size_hint: Option<u64>,
    ) -> std::io::Result<Vec<u8>> {
        let mut output = size_hint
            .and_then(|size| usize::try_from(size).ok())
            .map_or_else(Vec::new, Vec::with_capacity);
        let mut chunk = [0u8; READ_CHUNK_SIZE];

        loop {
            let read = reader.read(&mut chunk)?;
            if read == 0 {
                break;
            }
            output.extend_from_slice(&chunk[..read]);
        }

        Ok(output)
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
        assert!(exts.contains(&"epub"));
    }

    #[test]
    fn test_is_epub() {
        assert!(ArchiveExtractor::is_epub(Path::new("book.epub")));
        assert!(ArchiveExtractor::is_epub(Path::new("/path/to/book.EPUB")));
        assert!(!ArchiveExtractor::is_epub(Path::new("comic.cbz")));
        assert!(!ArchiveExtractor::is_epub(Path::new("archive.zip")));
    }

    #[test]
    fn test_read_stream_to_vec() {
        let data = vec![42u8; READ_CHUNK_SIZE * 2 + 123];
        let mut cursor = std::io::Cursor::new(data.clone());
        let output = ArchiveExtractor::read_stream_to_vec(&mut cursor, None).unwrap();

        assert_eq!(output, data);
    }
}
