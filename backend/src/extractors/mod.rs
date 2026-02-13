//! Archive extractors for reading comic, novel, and PDF files.
//!
//! This module contains implementations for extracting content from various archive formats
//! including ZIP, CBZ, CBR, RAR for comics, EPUB for novels, and PDF.

pub mod comic;
pub mod novel;
pub mod pdf;

pub use comic::{ComicArchiveExtractor, NaturalSortPart, natural_sort_key};
pub use novel::NovelArchiveExtractor;
pub use pdf::PdfExtractor;
