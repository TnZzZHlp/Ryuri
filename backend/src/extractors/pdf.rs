//! PDF extractor for rendering PDF pages as images.
//!
//! This module provides functionality to extract page images from PDF files
//! using the MuPDF library. PDFs are treated as comic-type content since
//! they are page-based.

use crate::error::{AppError, Result};
use rust_i18n::t;
use std::path::Path;

/// PDF extractor supporting .pdf files.
pub struct PdfExtractor;

impl PdfExtractor {
    /// Returns the supported extensions for PDF files.
    pub fn supported_extensions() -> &'static [&'static str] {
        &["pdf"]
    }

    /// Checks if a file extension is supported.
    pub fn is_supported(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| Self::supported_extensions().contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
    }

    /// Lists all pages in the PDF as virtual filenames (e.g., "page_001").
    pub fn list_files(pdf_path: &Path) -> Result<Vec<String>> {
        let doc = mupdf::Document::open(pdf_path.to_str().unwrap_or_default())
            .map_err(|e| AppError::Archive(t!("archive.pdf_open_failed", error = e).to_string()))?;

        let count = doc
            .page_count()
            .map_err(|e| AppError::Archive(t!("archive.pdf_open_failed", error = e).to_string()))?;

        let files: Vec<String> = (0..count).map(|i| format!("page_{:03}", i + 1)).collect();

        Ok(files)
    }

    /// Extracts a specific page from the PDF as a PNG image.
    ///
    /// The `file_name` should be in the format "page_NNN" where NNN is the 1-based page number.
    pub fn extract_file(pdf_path: &Path, file_name: &str) -> Result<Vec<u8>> {
        // Parse page number from file_name (e.g., "page_001" -> 0)
        let page_index = Self::parse_page_index(file_name)?;

        Self::render_page(pdf_path, page_index)
    }

    /// Extracts the first page from the PDF (for thumbnail generation).
    pub fn extract_first_image(pdf_path: &Path) -> Result<Vec<u8>> {
        Self::render_page(pdf_path, 0)
    }

    /// Gets the page count of the PDF.
    pub fn page_count(pdf_path: &Path) -> Result<usize> {
        let doc = mupdf::Document::open(pdf_path.to_str().unwrap_or_default())
            .map_err(|e| AppError::Archive(t!("archive.pdf_open_failed", error = e).to_string()))?;

        let count = doc
            .page_count()
            .map_err(|e| AppError::Archive(t!("archive.pdf_open_failed", error = e).to_string()))?;

        Ok(count as usize)
    }

    /// Renders a specific page of the PDF to a PNG image.
    fn render_page(pdf_path: &Path, page_index: i32) -> Result<Vec<u8>> {
        let doc = mupdf::Document::open(pdf_path.to_str().unwrap_or_default())
            .map_err(|e| AppError::Archive(t!("archive.pdf_open_failed", error = e).to_string()))?;

        let page = doc
            .load_page(page_index)
            .map_err(|e| AppError::Archive(t!("archive.pdf_page_failed", error = e).to_string()))?;

        // Render at 2x scale for good quality (144 DPI)
        let matrix = mupdf::Matrix::new_scale(2.0, 2.0);
        let pixmap = page
            .to_pixmap(&matrix, &mupdf::Colorspace::device_rgb(), true, true)
            .map_err(|e| {
                AppError::Archive(t!("archive.pdf_render_failed", error = e).to_string())
            })?;

        // Encode as PNG into a buffer
        let mut buf = Vec::new();
        pixmap
            .write_to(&mut buf, mupdf::pixmap::ImageFormat::PNG)
            .map_err(|e| {
                AppError::Archive(t!("archive.pdf_render_failed", error = e).to_string())
            })?;

        Ok(buf)
    }

    /// Parses a page index from a virtual filename like "page_001".
    fn parse_page_index(file_name: &str) -> Result<i32> {
        let num_str = file_name.strip_prefix("page_").ok_or_else(|| {
            AppError::Archive(t!("archive.invalid_pdf_page", file = file_name).to_string())
        })?;

        let page_num: i32 = num_str.parse().map_err(|_| {
            AppError::Archive(t!("archive.invalid_pdf_page", file = file_name).to_string())
        })?;

        // Convert from 1-based to 0-based
        Ok(page_num - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_extensions() {
        let exts = PdfExtractor::supported_extensions();
        assert!(exts.contains(&"pdf"));
        assert!(!exts.contains(&"zip"));
    }

    #[test]
    fn test_is_supported() {
        assert!(PdfExtractor::is_supported(Path::new("test.pdf")));
        assert!(PdfExtractor::is_supported(Path::new("test.PDF")));
        assert!(!PdfExtractor::is_supported(Path::new("test.zip")));
        assert!(!PdfExtractor::is_supported(Path::new("test.epub")));
    }

    #[test]
    fn test_parse_page_index() {
        assert_eq!(PdfExtractor::parse_page_index("page_001").unwrap(), 0);
        assert_eq!(PdfExtractor::parse_page_index("page_010").unwrap(), 9);
        assert_eq!(PdfExtractor::parse_page_index("page_100").unwrap(), 99);
        assert!(PdfExtractor::parse_page_index("invalid").is_err());
    }
}
