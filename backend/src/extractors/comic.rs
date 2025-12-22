//! Comic archive extractor for ZIP, CBZ, CBR, RAR formats.
//!
//! This module provides functionality to extract images from comic archive files.
//! Supported formats:
//! - ZIP/CBZ: Standard ZIP archives (CBZ is just ZIP with a different extension)
//! - CBR/RAR: RAR archives

use crate::error::{AppError, Result};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use rust_i18n::t;

/// Supported image extensions for comics.
const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "bmp"];

/// Comic archive extractor supporting ZIP, CBZ, CBR, and RAR formats.
pub struct ComicArchiveExtractor;

impl ComicArchiveExtractor {
    /// Returns the supported archive extensions for comics.
    pub fn supported_extensions() -> &'static [&'static str] {
        &["zip", "cbz", "cbr", "rar"]
    }

    /// Checks if a file extension is supported.
    pub fn is_supported(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| Self::supported_extensions().contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
    }

    /// Lists all image files in the archive, sorted by filename.
    pub fn list_files(archive_path: &Path) -> Result<Vec<String>> {
        let ext = archive_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            "zip" | "cbz" => Self::list_zip_files(archive_path),
            "cbr" | "rar" => Self::list_rar_files(archive_path),
            _ => Err(AppError::Archive(
                t!("archive.unsupported_comic_format", extension = ext).to_string(),
            )),
        }
    }

    /// Extracts a specific file from the archive.
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
    pub fn page_count(archive_path: &Path) -> Result<usize> {
        let files = Self::list_files(archive_path)?;
        Ok(files.len())
    }

    // ZIP/CBZ implementation
    fn list_zip_files(archive_path: &Path) -> Result<Vec<String>> {
        let file = File::open(archive_path)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| AppError::Archive(t!("archive.zip_open_failed", error = e).to_string()))?;

        let mut files: Vec<String> = Vec::new();
        for i in 0..archive.len() {
            let entry = archive
                .by_index(i)
                .map_err(|e| AppError::Archive(t!("archive.zip_read_entry_failed", error = e).to_string()))?;
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

        let mut entry = archive
            .by_name(file_name)
            .map_err(|_| AppError::Archive(t!("archive.file_not_found", file = file_name).to_string()))?;

        let mut buffer = Vec::new();
        entry
            .read_to_end(&mut buffer)
            .map_err(|e| AppError::Archive(t!("archive.file_read_failed", error = e).to_string()))?;

        Ok(buffer)
    }

    // RAR/CBR implementation

    fn list_rar_files(archive_path: &Path) -> Result<Vec<String>> {
        let archive = unrar::Archive::new(archive_path)
            .open_for_listing()
            .map_err(|e| AppError::Archive(t!("archive.rar_open_failed", error = e).to_string()))?;

        let mut files: Vec<String> = Vec::new();
        let entries = archive
            .into_iter()
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| AppError::Archive(t!("archive.rar_read_entries_failed", error = e).to_string()))?;

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
                            AppError::Archive(t!("archive.rar_extract_failed", error = e).to_string())
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
                    return Err(AppError::Archive(t!("archive.rar_read_entries_failed", error = e).to_string()));
                }
            }
        }

        // Clean up temp directory
        let _ = std::fs::remove_dir_all(&temp_dir);

        Err(AppError::Archive(t!("archive.file_not_found", file = file_name).to_string()))
    }

    /// Checks if a filename is an image file based on extension.
    fn is_image_file(name: &str) -> bool {
        let lower = name.to_lowercase();
        IMAGE_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
    }
}

/// Generates a natural sort key for a string.
/// This handles numeric portions correctly (e.g., "page2" < "page10").
pub fn natural_sort_key(s: &str) -> Vec<NaturalSortPart> {
    let mut parts = Vec::new();
    let mut current_num = String::new();
    let mut current_str = String::new();

    for c in s.chars() {
        if c.is_ascii_digit() {
            if !current_str.is_empty() {
                parts.push(NaturalSortPart::Text(current_str.to_lowercase()));
                current_str.clear();
            }
            current_num.push(c);
        } else {
            if !current_num.is_empty()
                && let Ok(num) = current_num.parse::<u64>()
            {
                parts.push(NaturalSortPart::Number(num));
                current_num.clear();
            } else if !current_num.is_empty() {
                current_num.clear();
            }
            current_str.push(c);
        }
    }

    // Handle remaining parts
    if !current_num.is_empty()
        && let Ok(num) = current_num.parse::<u64>()
    {
        parts.push(NaturalSortPart::Number(num));
    }
    if !current_str.is_empty() {
        parts.push(NaturalSortPart::Text(current_str.to_lowercase()));
    }

    parts
}

/// A part of a natural sort key - either text or a number.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NaturalSortPart {
    /// Numeric portion (compared numerically)
    Number(u64),
    /// Text portion (compared lexicographically, case-insensitive)
    Text(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_sort_key_simple() {
        let key1 = natural_sort_key("page1.jpg");
        let key2 = natural_sort_key("page2.jpg");
        let key10 = natural_sort_key("page10.jpg");

        assert!(key1 < key2);
        assert!(key2 < key10);
    }

    #[test]
    fn test_natural_sort_key_mixed() {
        let key1 = natural_sort_key("chapter1_page01.jpg");
        let key2 = natural_sort_key("chapter1_page02.jpg");
        let key3 = natural_sort_key("chapter2_page01.jpg");

        assert!(key1 < key2);
        assert!(key2 < key3);
    }

    #[test]
    fn test_is_image_file() {
        assert!(ComicArchiveExtractor::is_image_file("test.jpg"));
        assert!(ComicArchiveExtractor::is_image_file("test.PNG"));
        assert!(ComicArchiveExtractor::is_image_file("folder/test.jpeg"));
        assert!(!ComicArchiveExtractor::is_image_file("test.txt"));
        assert!(!ComicArchiveExtractor::is_image_file("test.xml"));
    }

    #[test]
    fn test_supported_extensions() {
        let exts = ComicArchiveExtractor::supported_extensions();
        assert!(exts.contains(&"zip"));
        assert!(exts.contains(&"cbz"));
        assert!(exts.contains(&"cbr"));
        assert!(exts.contains(&"rar"));
    }
}
