//! Archive extractors for reading comic and novel files.
//!
//! This module contains implementations for extracting content from various archive formats
//! including ZIP, CBZ, CBR, RAR for comics and ZIP, EPUB, TXT for novels.

pub mod comic;
pub mod novel;

pub use comic::{ComicArchiveExtractor, NaturalSortPart, natural_sort_key};
pub use novel::NovelArchiveExtractor;
