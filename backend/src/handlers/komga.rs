use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, header},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    error::{AppError, Result},
    extractors::ComicArchiveExtractor,
    models::{Chapter, Content, ContentType},
    repository::{
        content::{ChapterRepository, ContentRepository},
        library::LibraryRepository,
    },
    state::AppState,
};

// --- DTOs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageWrapperDto<T> {
    pub content: Vec<T>,
    pub empty: bool,
    pub first: bool,
    pub last: bool,
    pub number: usize,
    #[serde(rename = "numberOfElements")]
    pub number_of_elements: usize,
    pub size: usize,
    #[serde(rename = "totalElements")]
    pub total_elements: usize,
    #[serde(rename = "totalPages")]
    pub total_pages: usize,
}

impl<T> PageWrapperDto<T> {
    pub fn new(items: Vec<T>, page: usize, size: usize, total_elements: usize) -> Self {
        let total_pages = if size > 0 {
            total_elements.div_ceil(size)
        } else {
            0
        };
        let number_of_elements = items.len();
        let empty = items.is_empty();
        let first = page == 0;
        let last = page >= total_pages.saturating_sub(1);

        Self {
            content: items,
            empty,
            first,
            last,
            number: page,
            number_of_elements,
            size,
            total_elements,
            total_pages,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesDto {
    pub id: String,
    #[serde(rename = "libraryId")]
    pub library_id: String,
    pub name: String,
    pub created: Option<DateTime<Utc>>,
    #[serde(rename = "lastModified")]
    pub last_modified: Option<DateTime<Utc>>,
    #[serde(rename = "fileLastModified")]
    pub file_last_modified: DateTime<Utc>,
    #[serde(rename = "booksCount")]
    pub books_count: i32,
    pub metadata: SeriesMetadataDto,
    #[serde(rename = "booksMetadata")]
    pub books_metadata: SeriesBooksMetadataDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesMetadataDto {
    pub status: String,
    pub created: DateTime<Utc>,
    #[serde(rename = "lastModified")]
    pub last_modified: DateTime<Utc>,
    pub title: String,
    #[serde(rename = "titleSort")]
    pub title_sort: String,
    pub summary: String,
    #[serde(rename = "summaryLock")]
    pub summary_lock: bool,
    #[serde(rename = "readingDirection")]
    pub reading_direction: String,
    #[serde(rename = "readingDirectionLock")]
    pub reading_direction_lock: bool,
    pub publisher: String,
    #[serde(rename = "publisherLock")]
    pub publisher_lock: bool,
    #[serde(rename = "ageRating")]
    pub age_rating: Option<i32>,
    #[serde(rename = "ageRatingLock")]
    pub age_rating_lock: bool,
    pub language: String,
    #[serde(rename = "languageLock")]
    pub language_lock: bool,
    pub genres: Vec<String>,
    #[serde(rename = "genresLock")]
    pub genres_lock: bool,
    pub tags: Vec<String>,
    #[serde(rename = "tagsLock")]
    pub tags_lock: bool,
    #[serde(rename = "totalBookCount")]
    pub total_book_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesBooksMetadataDto {
    pub authors: Vec<AuthorDto>,
    pub tags: Vec<String>,
    #[serde(rename = "releaseDate")]
    pub release_date: Option<String>,
    pub summary: String,
    #[serde(rename = "summaryNumber")]
    pub summary_number: String,
    pub created: DateTime<Utc>,
    #[serde(rename = "lastModified")]
    pub last_modified: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorDto {
    pub name: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookDto {
    pub id: String,
    #[serde(rename = "seriesId")]
    pub series_id: String,
    #[serde(rename = "seriesTitle")]
    pub series_title: String,
    pub name: String,
    pub number: f32,
    pub created: Option<DateTime<Utc>>,
    #[serde(rename = "lastModified")]
    pub last_modified: Option<DateTime<Utc>>,
    #[serde(rename = "fileLastModified")]
    pub file_last_modified: DateTime<Utc>,
    #[serde(rename = "sizeBytes")]
    pub size_bytes: i64,
    pub size: String,
    pub media: MediaDto,
    pub metadata: BookMetadataDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaDto {
    pub status: String,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    #[serde(rename = "pagesCount")]
    pub pages_count: i32,
    #[serde(rename = "mediaProfile")]
    pub media_profile: String,
    #[serde(rename = "epubDivinaCompatible")]
    pub epub_divina_compatible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookMetadataDto {
    pub title: String,
    #[serde(rename = "titleLock")]
    pub title_lock: bool,
    pub summary: String,
    #[serde(rename = "summaryLock")]
    pub summary_lock: bool,
    pub number: String,
    #[serde(rename = "numberLock")]
    pub number_lock: bool,
    #[serde(rename = "numberSort")]
    pub number_sort: f32,
    #[serde(rename = "numberSortLock")]
    pub number_sort_lock: bool,
    #[serde(rename = "releaseDate")]
    pub release_date: Option<String>,
    #[serde(rename = "releaseDateLock")]
    pub release_date_lock: bool,
    pub authors: Vec<AuthorDto>,
    #[serde(rename = "authorsLock")]
    pub authors_lock: bool,
    pub tags: Vec<String>,
    #[serde(rename = "tagsLock")]
    pub tags_lock: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageDto {
    pub number: i32,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "mediaType")]
    pub media_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDto {
    pub id: String,
    pub name: String,
}

// --- Query Parameters ---

#[derive(Debug, Deserialize)]
pub struct SeriesSearchQuery {
    pub search: Option<String>,
    pub page: Option<usize>,
    pub size: Option<usize>, // Existing API usually has size, Komga might imply it or use pageable defaults
    pub library_id: Option<String>,
    pub status: Option<String>,
    // Add other fields as needed
}

#[derive(Debug, Deserialize)]
pub struct BookSearchQuery {
    pub search: Option<String>,
    pub page: Option<usize>,
    pub unpaged: Option<bool>,
    // Add other fields as needed
}

// --- Handlers ---

// 1. Series

pub async fn get_series_list(
    State(state): State<AppState>,
    Query(query): Query<SeriesSearchQuery>,
) -> Result<Json<PageWrapperDto<SeriesDto>>> {
    let pool = &state.pool;

    // Simplification: We fetch all content and then filter/paginate in memory for now.
    // Ideally, the repository should support pagination.
    // Since existing repo returns Vec<Content>, we will use that.

    let contents = {
        let libs = LibraryRepository::list(pool).await?;
        let mut all_content = Vec::new();
        for lib in libs {
            let mut contents = ContentRepository::list_by_library(pool, lib.id).await?;
            all_content.append(&mut contents);
        }
        all_content
    };

    // Filter by search
    let filtered_contents: Vec<Content> = if let Some(search) = &query.search {
        contents
            .into_iter()
            .filter(|c| c.title.contains(search))
            .collect()
    } else {
        contents
    };

    // Pagination
    let page = query.page.unwrap_or(0);
    let size = query.size.unwrap_or(20);
    let total_elements = filtered_contents.len();

    let start = page * size;
    let end = std::cmp::min(start + size, total_elements);

    let paged_contents = if start < total_elements {
        filtered_contents[start..end].to_vec()
    } else {
        Vec::new()
    };

    let series_dtos: Vec<SeriesDto> = paged_contents
        .into_iter()
        .map(content_to_series_dto)
        .collect();

    Ok(Json(PageWrapperDto::new(
        series_dtos,
        page,
        size,
        total_elements,
    )))
}

pub async fn get_series(
    State(state): State<AppState>,
    Path(series_id): Path<i64>,
) -> Result<Json<SeriesDto>> {
    let pool = &state.pool;
    let content = ContentRepository::find_by_id(pool, series_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Series with id {} not found", series_id)))?;

    Ok(Json(content_to_series_dto(content)))
}

pub async fn get_series_thumbnail(
    State(state): State<AppState>,
    Path(series_id): Path<i64>,
) -> Result<Response> {
    let pool = &state.pool;
    let content = ContentRepository::find_by_id(pool, series_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Series with id {} not found", series_id)))?;

    if let Some(thumb) = content.thumbnail {
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "image/jpeg".parse().unwrap());
        // Simple cache control
        headers.insert(header::CACHE_CONTROL, "max-age=86400".parse().unwrap());
        Ok((headers, thumb).into_response())
    } else {
        Err(AppError::NotFound("Thumbnail not found".to_string()))
    }
}

pub async fn get_books(
    State(state): State<AppState>,
    Path(series_id): Path<i64>,
    Query(query): Query<BookSearchQuery>,
) -> Result<Json<PageWrapperDto<BookDto>>> {
    let pool = &state.pool;

    // Verify series exists
    let content = ContentRepository::find_by_id(pool, series_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Series with id {} not found", series_id)))?;

    let chapters = ChapterRepository::list_by_content(pool, series_id).await?;

    let total_elements = chapters.len();

    // Handling unpaged
    let (page, size, paged_chapters) = if query.unpaged.unwrap_or(false) {
        (0, total_elements, chapters)
    } else {
        let page = query.page.unwrap_or(0);
        let size = 20; // Default size
        let start = page * size;
        let end = std::cmp::min(start + size, total_elements);
        let slice = if start < total_elements {
            chapters[start..end].to_vec()
        } else {
            Vec::new()
        };
        (page, size, slice)
    };

    let book_dtos: Vec<BookDto> = paged_chapters
        .into_iter()
        .map(|c| chapter_to_book_dto(c, &content))
        .collect();

    Ok(Json(PageWrapperDto::new(
        book_dtos,
        page,
        size,
        total_elements,
    )))
}

// 2. Books

pub async fn get_book(
    State(state): State<AppState>,
    Path(book_id): Path<i64>,
) -> Result<Json<BookDto>> {
    let pool = &state.pool;
    let chapter = ChapterRepository::find_by_id(pool, book_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Book with id {} not found", book_id)))?;

    let content = ContentRepository::find_by_id(pool, chapter.content_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Content for book {} not found", book_id)))?;

    Ok(Json(chapter_to_book_dto(chapter, &content)))
}

pub async fn get_book_thumbnail(
    State(state): State<AppState>,
    Path(book_id): Path<i64>,
) -> Result<Response> {
    // For now, redirect to series thumbnail as we might not have chapter thumbnails
    // Or we could try to extract the first page.
    // Given the constraints and existing code, let's fetch the series thumbnail for now
    // or return a placeholder?
    // Komga usually extracts thumbnails from books.
    // Our existing Content model has a thumbnail. Chapter doesn't have a thumbnail field explicitly in DB struct.
    // However, `content::get_thumbnail` serves the content thumbnail.

    // Let's get the chapter, then the content, and serve content thumbnail as fallback.
    let pool = &state.pool;
    let chapter = ChapterRepository::find_by_id(pool, book_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Book with id {} not found", book_id)))?;

    // In a real implementation we would extract page 1.
    // For compatibility, we'll redirect to series thumbnail or similar.
    // But `get_series_thumbnail` expects series_id.

    let content = ContentRepository::find_by_id(pool, chapter.content_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Content not found".into()))?;

    if let Some(thumb) = content.thumbnail {
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "image/jpeg".parse().unwrap());
        headers.insert(header::CACHE_CONTROL, "max-age=86400".parse().unwrap());
        Ok((headers, thumb).into_response())
    } else {
        Err(AppError::NotFound("Thumbnail not found".into()))
    }
}

pub async fn get_page_list(
    State(state): State<AppState>,
    Path(book_id): Path<i64>,
) -> Result<Json<Vec<PageDto>>> {
    let pool = &state.pool;
    let chapter = ChapterRepository::find_by_id(pool, book_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Book with id {} not found", book_id)))?;

    use std::path::Path;
    let archive_path = Path::new(&chapter.file_path);

    // Try to list files, if it fails, fallback to simple counter if page_count > 0
    let mut pages = Vec::new();

    match ComicArchiveExtractor::list_files(archive_path) {
        Ok(images) => {
            for (i, name) in images.iter().enumerate() {
                pages.push(PageDto {
                    number: (i + 1) as i32,
                    file_name: name.clone(),
                    media_type: "image/jpeg".to_string(), // Can guess from extension, but jpeg is safe default for list
                });
            }
        }
        Err(_) => {
            // Fallback
            for i in 1..=chapter.page_count {
                pages.push(PageDto {
                    number: i,
                    file_name: format!("{}.jpg", i),
                    media_type: "image/jpeg".to_string(),
                });
            }
        }
    }

    Ok(Json(pages))
}

pub async fn get_page(
    State(state): State<AppState>,
    Path((book_id, page_number)): Path<(i64, i32)>,
) -> Result<Response> {
    // Reuse existing logic from content handler if possible
    // `crate::handlers::content::get_page` takes `Path((id, chapter_id, page))`.
    // But here we only have `book_id` (which is chapter_id) and `page_number`.
    // We need `content_id` to call existing service logic if strictly following that pattern,
    // but likely the service just needs the file path from chapter.

    // Let's check `services::content::get_page_image`.
    // Since I cannot check it right now without reading another file, I'll rely on `Chapter` having `content_id`.

    // Actually, I can just call the handler `crate::handlers::content::get_page`?
    // No, that handler expects `(content_id, chapter_id, page_number)`.

    // I will use `crate::services::content::get_page_image`.
    // I need to find `content_id` from `book_id`.

    let pool = &state.pool;
    let chapter = ChapterRepository::find_by_id(pool, book_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Book with id {} not found", book_id)))?;

    if page_number < 1 {
        return Err(AppError::BadRequest("Page number must be > 0".into()));
    }
    let page_index = (page_number - 1) as usize;

    use std::path::Path;
    let archive_path = Path::new(&chapter.file_path);

    // We need to list files to get the name at index.
    let images = ComicArchiveExtractor::list_files(archive_path)?;

    if page_index >= images.len() {
        return Err(AppError::NotFound(format!(
            "Page {} not found",
            page_number
        )));
    }

    let image_name = &images[page_index];
    let image_data = ComicArchiveExtractor::extract_file(archive_path, image_name)?;

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "image/jpeg".parse().unwrap());
    headers.insert(header::CACHE_CONTROL, "max-age=86400".parse().unwrap());

    Ok((headers, image_data).into_response())
}

// Libraries

pub async fn get_libraries(State(state): State<AppState>) -> Result<Json<Vec<LibraryDto>>> {
    let pool = &state.pool;
    let libraries = LibraryRepository::list(pool).await?;

    let dtos = libraries
        .into_iter()
        .map(|l| LibraryDto {
            id: l.id.to_string(),
            name: l.name,
        })
        .collect();

    Ok(Json(dtos))
}

// Helpers

fn content_to_series_dto(content: Content) -> SeriesDto {
    // Extract metadata fields from Bangumi JSON if available
    let meta = extract_bangumi_metadata(&content.metadata);

    SeriesDto {
        id: content.id.to_string(),
        library_id: content.library_id.to_string(),
        name: content.title.clone(),
        created: Some(content.created_at),
        last_modified: Some(content.updated_at),
        file_last_modified: content.updated_at, // approximation
        books_count: content.chapter_count,
        metadata: SeriesMetadataDto {
            status: "ONGOING".to_string(), // Default
            created: content.created_at,
            last_modified: content.updated_at,
            title: content.title.clone(),
            title_sort: content.title.clone(),
            summary: meta.summary.clone(),
            summary_lock: false,
            reading_direction: "RIGHT_TO_LEFT".to_string(), // Manga default
            reading_direction_lock: false,
            publisher: meta.publisher.clone(),
            publisher_lock: false,
            age_rating: None,
            age_rating_lock: false,
            language: meta.language.clone(),
            language_lock: false,
            genres: vec![],
            genres_lock: false,
            tags: meta.tags.clone(),
            tags_lock: false,
            total_book_count: Some(content.chapter_count),
        },
        books_metadata: SeriesBooksMetadataDto {
            authors: meta.authors,
            tags: meta.tags,
            release_date: meta.release_date,
            summary: meta.summary,
            summary_number: "".to_string(),
            created: content.created_at,
            last_modified: content.updated_at,
        },
    }
}

/// Extracted metadata from Bangumi API JSON
struct BangumiMetadata {
    summary: String,
    tags: Vec<String>,
    authors: Vec<AuthorDto>,
    publisher: String,
    release_date: Option<String>,
    language: String,
}

impl Default for BangumiMetadata {
    fn default() -> Self {
        Self {
            summary: String::new(),
            tags: vec![],
            authors: vec![],
            publisher: String::new(),
            release_date: None,
            language: "ja".to_string(),
        }
    }
}

/// Extract metadata from Bangumi API JSON blob
fn extract_bangumi_metadata(metadata: &Option<serde_json::Value>) -> BangumiMetadata {
    let Some(meta) = metadata else {
        return BangumiMetadata::default();
    };

    // Extract summary
    let summary = meta
        .get("summary")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Extract tags (top tags by count)
    let tags: Vec<String> = meta
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
                .take(10) // Limit to top 10 tags
                .map(String::from)
                .collect()
        })
        .unwrap_or_default();

    // Extract author from infobox
    let authors: Vec<AuthorDto> = meta
        .get("infobox")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter(|item| {
                    item.get("key")
                        .and_then(|k| k.as_str())
                        .map(|k| k == "作者")
                        .unwrap_or(false)
                })
                .filter_map(|item| item.get("value").and_then(|v| v.as_str()))
                .map(|name| AuthorDto {
                    name: name.to_string(),
                    role: "writer".to_string(),
                })
                .collect()
        })
        .unwrap_or_default();

    // Extract publisher from infobox
    let publisher = meta
        .get("infobox")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            arr.iter()
                .find(|item| {
                    item.get("key")
                        .and_then(|k| k.as_str())
                        .map(|k| k == "出版社")
                        .unwrap_or(false)
                })
                .and_then(|item| item.get("value").and_then(|v| v.as_str()))
        })
        .unwrap_or("")
        .to_string();

    // Extract release date
    let release_date = meta.get("date").and_then(|v| v.as_str()).map(String::from);

    // Default to Japanese for Bangumi content
    let language = "ja".to_string();

    BangumiMetadata {
        summary,
        tags,
        authors,
        publisher,
        release_date,
        language,
    }
}

fn format_size(size: i64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let size = size as f64;

    if size >= GB {
        format!("{:.2} GB", size / GB)
    } else if size >= MB {
        format!("{:.2} MB", size / MB)
    } else if size >= KB {
        format!("{:.2} KB", size / KB)
    } else {
        format!("{} B", size)
    }
}

fn chapter_to_book_dto(chapter: Chapter, content: &Content) -> BookDto {
    BookDto {
        id: chapter.id.to_string(),
        series_id: content.id.to_string(),
        series_title: content.title.clone(),
        name: chapter.title.clone(),
        number: chapter.sort_order as f32, // approximation
        created: None,                     // Chapter doesn't have created_at in struct
        last_modified: None,
        file_last_modified: Utc::now(), // default
        size_bytes: chapter.size,
        size: format_size(chapter.size),
        media: MediaDto {
            status: "READY".to_string(),
            media_type: match content.content_type {
                ContentType::Comic => "application/zip".to_string(), // Common for comics
                ContentType::Novel => "application/epub+zip".to_string(),
            },
            pages_count: chapter.page_count,
            media_profile: match content.content_type {
                ContentType::Comic => "DIVINA".to_string(),
                ContentType::Novel => "EPUB".to_string(),
            },
            epub_divina_compatible: false,
        },
        metadata: BookMetadataDto {
            title: chapter.title.clone(),
            title_lock: false,
            summary: "".to_string(),
            summary_lock: false,
            number: chapter.sort_order.to_string(),
            number_lock: false,
            number_sort: chapter.sort_order as f32,
            number_sort_lock: false,
            release_date: None,
            release_date_lock: false,
            authors: vec![],
            authors_lock: false,
            tags: vec![],
            tags_lock: false,
        },
    }
}
