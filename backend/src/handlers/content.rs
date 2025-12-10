//! Content management handlers.
//!
//! This module provides HTTP handlers for content management endpoints:
//! - GET /api/libraries/{id}/contents - List all contents in a library
//! - POST /api/libraries/{id}/scan - Scan a library for new content
//! - GET /api/libraries/{id}/search - Search contents by title
//! - GET /api/contents/{id} - Get a content by ID
//! - DELETE /api/contents/{id} - Delete a content
//! - PUT /api/contents/{id}/metadata - Update content metadata
//! - GET /api/contents/{id}/chapters - List chapters for a content
//! - GET /api/contents/{id}/chapters/{chapter}/pages/{page} - Get a comic page
//! - GET /api/contents/{id}/chapters/{chapter}/text - Get novel chapter text

use axum::{
    Json,
    body::Body,
    extract::{Path, Query, State},
    http::{Response, StatusCode, header},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

use crate::error::Result;
use crate::models::{Chapter, ContentResponse};
use crate::services::content::ContentService;
use crate::services::scan::{ScanResult, ScanService};

/// Application state for content handlers.
#[derive(Clone)]
pub struct ContentState {
    pub pool: Pool<Sqlite>,
}

impl ContentState {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

/// GET /api/libraries/{id}/contents
///
/// Returns all contents in a library.
///
/// # Path Parameters
/// - `id`: The library ID
///
/// # Response
/// ```json
/// [
///     {
///         "id": 1,
///         "library_id": 1,
///         "content_type": "Comic",
///         "title": "My Manga",
///         "chapter_count": 10,
///         "has_thumbnail": true,
///         "metadata": null,
///         "created_at": "2024-01-01T00:00:00Z"
///     }
/// ]
/// ```
///
/// Requirements: 1.5, 2.8
pub async fn list_contents(
    State(state): State<ContentState>,
    Path(library_id): Path<i64>,
) -> Result<Json<Vec<ContentResponse>>> {
    let contents = ContentService::list_contents(&state.pool, library_id).await?;
    let responses: Vec<ContentResponse> = contents.into_iter().map(ContentResponse::from).collect();
    Ok(Json(responses))
}

/// Response for scan operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResponse {
    /// Number of contents added.
    pub added_count: usize,
    /// Number of contents removed.
    pub removed_count: usize,
    /// Number of contents that failed metadata scraping.
    pub failed_scrape_count: usize,
    /// Added content items.
    pub added: Vec<ContentResponse>,
    /// IDs of removed content items.
    pub removed: Vec<i64>,
}

impl From<ScanResult> for ScanResponse {
    fn from(result: ScanResult) -> Self {
        Self {
            added_count: result.added.len(),
            removed_count: result.removed.len(),
            failed_scrape_count: result.failed_scrape.len(),
            added: result
                .added
                .into_iter()
                .map(ContentResponse::from)
                .collect(),
            removed: result.removed,
        }
    }
}

/// POST /api/libraries/{id}/scan
///
/// Triggers a scan of the library to discover new content.
///
/// # Path Parameters
/// - `id`: The library ID
///
/// # Response
/// ```json
/// {
///     "added_count": 5,
///     "removed_count": 1,
///     "failed_scrape_count": 0,
///     "added": [...],
///     "removed": [123]
/// }
/// ```
///
/// Requirements: 2.1
pub async fn scan_library(
    State(state): State<ContentState>,
    Path(library_id): Path<i64>,
) -> Result<Json<ScanResponse>> {
    let scan_service = ScanService::new(state.pool.clone());
    let result = scan_service.scan_library(library_id).await?;
    Ok(Json(ScanResponse::from(result)))
}

/// Query parameters for search.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchQuery {
    /// Search query string.
    pub q: String,
}

/// GET /api/libraries/{id}/search
///
/// Searches contents by title within a library.
///
/// # Path Parameters
/// - `id`: The library ID
///
/// # Query Parameters
/// - `q`: The search query string
///
/// # Response
/// Returns contents whose titles contain the search keyword (case-insensitive).
///
/// Requirements: 2.10
pub async fn search_contents(
    State(state): State<ContentState>,
    Path(library_id): Path<i64>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<ContentResponse>>> {
    let contents = ContentService::search_contents(&state.pool, library_id, &query.q).await?;
    let responses: Vec<ContentResponse> = contents.into_iter().map(ContentResponse::from).collect();
    Ok(Json(responses))
}

/// GET /api/contents/{id}
///
/// Returns a content by its ID.
///
/// # Path Parameters
/// - `id`: The content ID
///
/// # Response
/// Returns the content, or 404 if not found.
///
/// Requirements: 2.8
pub async fn get_content(
    State(state): State<ContentState>,
    Path(id): Path<i64>,
) -> Result<Json<ContentResponse>> {
    let content = ContentService::get_content(&state.pool, id).await?;
    Ok(Json(ContentResponse::from(content)))
}

/// DELETE /api/contents/{id}
///
/// Deletes a content and all associated chapters.
///
/// # Path Parameters
/// - `id`: The content ID
///
/// # Response
/// Returns 200 OK with empty body on success.
///
/// Requirements: 2.9
pub async fn delete_content(
    State(state): State<ContentState>,
    Path(id): Path<i64>,
) -> Result<Json<()>> {
    ContentService::delete_content(&state.pool, id).await?;
    Ok(Json(()))
}

/// GET /api/contents/{id}/chapters
///
/// Returns all chapters for a content.
///
/// # Path Parameters
/// - `id`: The content ID
///
/// # Response
/// ```json
/// [
///     {
///         "id": 1,
///         "content_id": 1,
///         "title": "Chapter 1",
///         "file_path": "/path/to/chapter1.cbz",
///         "sort_order": 0
///     }
/// ]
/// ```
///
/// Requirements: 3.1, 4.1
pub async fn list_chapters(
    State(state): State<ContentState>,
    Path(content_id): Path<i64>,
) -> Result<Json<Vec<Chapter>>> {
    let chapters = ContentService::list_chapters(&state.pool, content_id).await?;
    Ok(Json(chapters))
}

/// Path parameters for page requests.
#[derive(Debug, Deserialize)]
pub struct PageParams {
    /// The content ID.
    pub id: i64,
    /// The chapter index (0-based).
    pub chapter: i32,
    /// The page index (0-based).
    pub page: i32,
}

/// GET /api/contents/{id}/chapters/{chapter}/pages/{page}
///
/// Returns a page image from a comic chapter.
///
/// # Path Parameters
/// - `id`: The content ID
/// - `chapter`: The chapter index (0-based)
/// - `page`: The page index (0-based)
///
/// # Response
/// Returns the image data with appropriate content-type header.
///
/// Requirements: 3.2, 7.2
pub async fn get_page(
    State(state): State<ContentState>,
    Path(params): Path<PageParams>,
) -> Result<impl IntoResponse> {
    let image_data =
        ContentService::get_page(&state.pool, params.id, params.chapter, params.page).await?;

    // Detect image type from magic bytes
    let content_type = detect_image_type(&image_data);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "public, max-age=86400")
        .body(Body::from(image_data))
        .unwrap())
}

/// Detect image type from magic bytes.
fn detect_image_type(data: &[u8]) -> &'static str {
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        "image/png"
    } else if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
        "image/gif"
    } else if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WEBP" {
        "image/webp"
    } else {
        "application/octet-stream"
    }
}

/// Path parameters for chapter text requests.
#[derive(Debug, Deserialize)]
pub struct ChapterTextParams {
    /// The content ID.
    pub id: i64,
    /// The chapter index (0-based).
    pub chapter: i32,
}

/// Response for chapter text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterTextResponse {
    /// The text content of the chapter.
    pub text: String,
}

/// GET /api/contents/{id}/chapters/{chapter}/text
///
/// Returns the text content of a novel chapter.
///
/// # Path Parameters
/// - `id`: The content ID
/// - `chapter`: The chapter index (0-based)
///
/// # Response
/// ```json
/// {
///     "text": "Chapter content here..."
/// }
/// ```
///
/// Requirements: 4.2, 7.3
pub async fn get_chapter_text(
    State(state): State<ContentState>,
    Path(params): Path<ChapterTextParams>,
) -> Result<Json<ChapterTextResponse>> {
    let text = ContentService::get_chapter_text(&state.pool, params.id, params.chapter).await?;

    Ok(Json(ChapterTextResponse { text }))
}

/// Request body for metadata update.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateMetadataRequest {
    /// The metadata JSON blob to store.
    /// Pass null to clear metadata.
    pub metadata: Option<serde_json::Value>,
}

/// PUT /api/contents/{id}/metadata
///
/// Updates the metadata for a content item.
///
/// # Path Parameters
/// - `id`: The content ID
///
/// # Request Body
/// ```json
/// {
///     "metadata": {
///         "name": "Title",
///         "name_cn": "中文标题",
///         "summary": "Description...",
///         "rating": { "score": 8.5 }
///     }
/// }
/// ```
///
/// # Response
/// Returns the updated content.
///
/// Requirements: 8.5, 8.7
pub async fn update_metadata(
    State(state): State<ContentState>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateMetadataRequest>,
) -> Result<Json<ContentResponse>> {
    let content = ContentService::update_metadata(&state.pool, id, request.metadata).await?;
    Ok(Json(ContentResponse::from(content)))
}
