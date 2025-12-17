//! Novel archive extractor for ZIP, EPUB, TXT formats.
//!
//! This module provides functionality to extract text content from novel archive files.
//! Supported formats:
//! - ZIP: Standard ZIP archives containing text files
//! - EPUB: Electronic publication format
//! - TXT: Plain text files (not actually archives, but treated as single-chapter novels)

use crate::error::{AppError, Result};
use std::path::Path;

/// Novel archive extractor supporting EPUB format.
pub struct NovelArchiveExtractor;

impl NovelArchiveExtractor {
    /// Returns the supported archive extensions for novels.
    pub fn supported_extensions() -> &'static [&'static str] {
        &["epub"]
    }

    /// Checks if a file extension is supported.
    pub fn is_supported(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| Self::supported_extensions().contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
    }

    /// Lists all text files in the archive, sorted by filename.
    pub fn list_files(archive_path: &Path) -> Result<Vec<String>> {
        let ext = archive_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            "epub" => Self::list_epub_files(archive_path),
            _ => Err(AppError::Archive(format!(
                "Unsupported archive format: {}",
                ext
            ))),
        }
    }

    /// Extracts text content from a specific file in the archive.
    pub fn extract_file(archive_path: &Path, file_name: &str) -> Result<String> {
        let ext = archive_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            "epub" => Self::extract_epub_file(archive_path, file_name),
            _ => Err(AppError::Archive(format!(
                "Unsupported archive format: {}",
                ext
            ))),
        }
    }

    /// Extracts all text content from the archive as a single string.
    pub fn extract_all_text(archive_path: &Path) -> Result<String> {
        let files = Self::list_files(archive_path)?;
        let mut all_text = String::new();

        for file in files {
            let text = Self::extract_file(archive_path, &file)?;
            if !all_text.is_empty() {
                all_text.push_str("\n\n");
            }
            all_text.push_str(&text);
        }

        Ok(all_text)
    }

    /// Gets the chapter count (number of text files) in the archive.
    pub fn chapter_count(archive_path: &Path) -> Result<usize> {
        let files = Self::list_files(archive_path)?;
        Ok(files.len())
    }

    // EPUB implementation
    // EPUB files are essentially ZIP files with a specific structure

    fn list_epub_files(archive_path: &Path) -> Result<Vec<String>> {
        let doc = epub::doc::EpubDoc::new(archive_path)
            .map_err(|e| AppError::Archive(format!("Failed to open EPUB: {}", e)))?;

        // Get the spine (reading order) from the EPUB
        // SpineItem has an idref field that we use as the identifier
        let files: Vec<String> = doc.spine.iter().map(|item| item.idref.clone()).collect();
        Ok(files)
    }

    fn extract_epub_file(archive_path: &Path, file_name: &str) -> Result<String> {
        let mut doc = epub::doc::EpubDoc::new(archive_path)
            .map_err(|e| AppError::Archive(format!("Failed to open EPUB: {}", e)))?;

        // Get the resource by ID (spine item idref)
        // get_resource_str returns Option<(String, String)> where first is content, second is mime type
        let (content, _mime_type) = doc.get_resource_str(file_name).ok_or_else(|| {
            AppError::Archive(format!("Chapter not found in EPUB: {}", file_name))
        })?;

        // Strip HTML tags and return plain text
        Ok(Self::strip_html(&content))
    }

    /// Cleans text content by normalizing whitespace.
    fn clean_text(text: &str) -> String {
        // Normalize line endings and trim
        text.replace("\r\n", "\n")
            .replace('\r', "\n")
            .trim()
            .to_string()
    }

    /// Strips HTML tags from content, returning plain text.
    fn strip_html(html: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;
        let mut in_script = false;
        let mut in_style = false;

        let chars: Vec<char> = html.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            if c == '<' {
                in_tag = true;
                // Check for script/style tags
                let remaining: String = chars[i..].iter().take(10).collect();
                let lower = remaining.to_lowercase();
                if lower.starts_with("<script") {
                    in_script = true;
                } else if lower.starts_with("<style") {
                    in_style = true;
                } else if lower.starts_with("</script") {
                    in_script = false;
                } else if lower.starts_with("</style") {
                    in_style = false;
                }
            } else if c == '>' {
                in_tag = false;
            } else if !in_tag && !in_script && !in_style {
                result.push(c);
            }

            i += 1;
        }

        // Decode common HTML entities
        let result = result
            .replace("&nbsp;", " ")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&apos;", "'");

        Self::clean_text(&result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_extensions() {
        let exts = NovelArchiveExtractor::supported_extensions();
        assert!(exts.contains(&"epub"));
        assert!(!exts.contains(&"zip"));
        assert!(!exts.contains(&"txt"));
    }

    #[test]
    fn test_strip_html_simple() {
        let html = "<p>Hello, <b>world</b>!</p>";
        let text = NovelArchiveExtractor::strip_html(html);
        assert_eq!(text, "Hello, world!");
    }

    #[test]
    fn test_strip_html_with_entities() {
        let html = "<p>Hello &amp; goodbye &lt;world&gt;</p>";
        let text = NovelArchiveExtractor::strip_html(html);
        assert_eq!(text, "Hello & goodbye <world>");
    }

    #[test]
    fn test_strip_html_with_script() {
        let html = "<p>Before</p><script>alert('hi');</script><p>After</p>";
        let text = NovelArchiveExtractor::strip_html(html);
        assert_eq!(text, "BeforeAfter");
    }

    #[test]
    fn test_clean_text() {
        let text = "  Hello\r\nWorld\r  ";
        let cleaned = NovelArchiveExtractor::clean_text(text);
        assert_eq!(cleaned, "Hello\nWorld");
    }
}
