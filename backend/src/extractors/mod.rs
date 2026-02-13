//! Archive extractors for reading comic, novel, and PDF files.
//!
//! This module contains implementations for extracting content from various file formats
//! including ZIP, CBZ, CBR, RAR for compressed archives, EPUB for novels, and PDF.

pub mod archive;
pub mod epub;
pub mod pdf;

pub use archive::ArchiveExtractor;
pub use epub::EpubExtractor;
pub use pdf::PdfExtractor;

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
}
