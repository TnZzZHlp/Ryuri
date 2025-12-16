//! Content management handlers.
//!
//! This module provides HTTP handlers for content management endpoints:
//! - GET /api/libraries/{id}/contents - List all contents in a library
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

use crate::error::Result;
use crate::models::{Chapter, ContentResponse};
use crate::services::content::ContentService;
use crate::state::AppState;

/// GET /api/libraries/{id}/contents
///
/// Returns all contents in a library.
pub async fn list(
    State(state): State<AppState>,
    Path(library_id): Path<i64>,
) -> Result<Json<Vec<ContentResponse>>> {
    let contents = ContentService::list_contents(&state.pool, library_id).await?;
    let responses: Vec<ContentResponse> = contents.into_iter().map(ContentResponse::from).collect();
    Ok(Json(responses))
}

/// Query parameters for search.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct SearchQuery {
    /// Search query string.
    pub q: String,
}

/// GET /api/libraries/{id}/search
///
/// Searches contents by title within a library.
pub async fn search(
    State(state): State<AppState>,
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
pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ContentResponse>> {
    let content = ContentService::get_content(&state.pool, id).await?;
    Ok(Json(ContentResponse::from(content)))
}

/// DELETE /api/contents/{id}
///
/// Deletes a content and all associated chapters.
pub async fn delete(State(state): State<AppState>, Path(id): Path<i64>) -> Result<Json<()>> {
    ContentService::delete_content(&state.pool, id).await?;
    Ok(Json(()))
}

/// GET /api/contents/{id}/chapters
///
/// Returns all chapters for a content.
pub async fn list_chapters(
    State(state): State<AppState>,
    Path(content_id): Path<i64>,
) -> Result<Json<Vec<Chapter>>> {
    let chapters = ContentService::list_chapters(&state.pool, content_id).await?;
    Ok(Json(chapters))
}

/// Path parameters for page requests.
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct PageParams {
    /// The content ID.
    pub content_id: i64,
    /// The chapter index (0-based).
    pub chapter_id: i64,
    /// The page index (0-based).
    pub page: i64,
}

/// GET /api/contents/{id}/chapters/{chapter}/pages/{page}
///
/// Returns a page image from a comic chapter.
pub async fn get_page(
    State(state): State<AppState>,
    Path(params): Path<PageParams>,
) -> Result<impl IntoResponse> {
    let image_data = ContentService::get_page(
        &state.pool,
        params.content_id,
        params.chapter_id,
        params.page,
    )
    .await?;

    // Detect image type from magic bytes
    let content_type = detect_image_type(&image_data);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from(image_data).into_data_stream())?)
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
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct ChapterTextParams {
    /// The content ID.
    pub id: i64,
    /// The chapter index (0-based).
    pub chapter: i32,
}

/// Response for chapter text.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct ChapterTextResponse {
    /// The text content of the chapter.
    pub text: String,
}

/// GET /api/contents/{id}/chapters/{chapter}/text
///
/// Returns the text content of a novel chapter.
pub async fn get_chapter_text(
    State(state): State<AppState>,
    Path(params): Path<ChapterTextParams>,
) -> Result<Json<ChapterTextResponse>> {
    let text = ContentService::get_chapter_text(&state.pool, params.id, params.chapter).await?;
    Ok(Json(ChapterTextResponse { text }))
}

/// Request body for metadata update.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct UpdateMetadataRequest {
    /// The metadata JSON blob to store.
    /// Pass null to clear metadata.
    pub metadata: Option<serde_json::Value>,
}

/// PUT /api/contents/{id}/metadata
///
/// Updates the metadata for a content item.
pub async fn update_metadata(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateMetadataRequest>,
) -> Result<Json<ContentResponse>> {
    let content = ContentService::update_metadata(&state.pool, id, request.metadata).await?;
    Ok(Json(ContentResponse::from(content)))
}

/// GET /api/contents/{id}/thumbnail
///
/// Returns the thumbnail image for a content.
pub async fn get_thumbnail(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse> {
    let thumbnail_data = ContentService::get_thumbnail(&state.pool, id).await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/jpeg")
        .header(header::CACHE_CONTROL, "public, max-age=86400")
        .body(Body::from(thumbnail_data))
        .unwrap())
}
