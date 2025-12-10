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
use crate::services::bangumi::BangumiSearchResult;
use crate::state::AppState;

/// Query parameters for Bangumi search.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct BangumiSearchQuery {
    /// Search query string (content title).
    pub q: String,
}

/// GET /api/bangumi/search
///
/// Searches for content on Bangumi.tv by keyword.
pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<BangumiSearchQuery>,
) -> Result<Json<Vec<BangumiSearchResult>>> {
    let results = state.bangumi_service.search(&query.q).await?;
    Ok(Json(results))
}
