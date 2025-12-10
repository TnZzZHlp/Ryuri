//! Bangumi metadata handlers.
//!
//! This module provides HTTP handlers for Bangumi metadata endpoints:
//! - GET /api/bangumi/search - Search for content on Bangumi.tv

use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::error::Result;
use crate::services::bangumi::{BangumiSearchResult, BangumiService};

/// Application state for Bangumi handlers.
#[derive(Clone)]
pub struct BangumiState {
    /// Optional Bangumi API key for authenticated requests.
    pub api_key: Option<String>,
}

impl BangumiState {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }
}

/// Query parameters for Bangumi search.
#[derive(Debug, Clone, Deserialize)]
pub struct BangumiSearchQuery {
    /// Search query string (content title).
    pub q: String,
}

/// GET /api/bangumi/search
///
/// Searches for content on Bangumi.tv by keyword.
///
/// # Query Parameters
/// - `q`: The search query string (usually the content title)
///
/// # Response
/// ```json
/// [
///     {
///         "id": 12345,
///         "name": "Original Title",
///         "name_cn": "中文标题",
///         "summary": "Description...",
///         "image": "https://example.com/cover.jpg"
///     }
/// ]
/// ```
///
/// Requirements: 8.4
pub async fn search_bangumi(
    State(state): State<BangumiState>,
    Query(query): Query<BangumiSearchQuery>,
) -> Result<Json<Vec<BangumiSearchResult>>> {
    let service = BangumiService::new(state.api_key.clone());
    let results = service.search(&query.q).await?;
    Ok(Json(results))
}
