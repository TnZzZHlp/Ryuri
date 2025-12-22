//! Reading progress handlers.
//!
//! This module provides HTTP handlers for progress management endpoints:
//! - GET /api/contents/{id}/progress - Get overall content progress
//! - GET /api/chapters/{id}/progress - Get chapter progress
//! - PUT /api/chapters/{id}/progress - Update chapter progress

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;

use crate::error::Result;
use crate::middlewares::auth::AuthUser;
use crate::models::{ContentResponse, ProgressResponse};
use crate::state::AppState;

/// Query parameters for recent progress.
#[derive(Debug, Deserialize)]
pub struct RecentProgressQuery {
    #[serde(default = "default_recent_limit")]
    pub limit: i64,
}

fn default_recent_limit() -> i64 {
    10
}

/// GET /api/contents/{id}/progress
///
/// Returns the reading progress for all chapters of a content.
pub async fn get_content_progress(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(content_id): Path<i64>,
) -> Result<Json<Vec<ProgressResponse>>> {
    let progress = state
        .progress_service
        .get_content_progress(auth_user.user_id, content_id)
        .await?;
    Ok(Json(progress.into_iter().map(ProgressResponse::from).collect()))
}

/// GET /api/progress/recent
///
/// Returns the most recently read contents.
pub async fn get_recent_progress(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(query): Query<RecentProgressQuery>,
) -> Result<Json<Vec<ContentResponse>>> {
    let contents = state
        .progress_service
        .get_recent_contents(auth_user.user_id, query.limit)
        .await?;
    Ok(Json(contents))
}

/// GET /api/chapters/{id}/progress
///
/// Returns the reading progress for all chapters of the content that the specified chapter belongs to.
pub async fn get_chapter_progress(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(chapter_id): Path<i64>,
) -> Result<Json<Vec<ProgressResponse>>> {
    let progresses = state
        .progress_service
        .get_chapter_siblings_progress(auth_user.user_id, chapter_id)
        .await?;
    Ok(Json(progresses))
}

/// Request body for updating chapter progress with optional percentage.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct UpdateProgressWithPercentageRequest {
    /// Current position within the chapter (page number or character offset).
    pub position: i32,
    /// Optional percentage (0.0 to 100.0). If not provided, defaults to 0.0.
    #[serde(default)]
    pub percentage: Option<f32>,
}

/// PUT /api/chapters/{id}/progress
///
/// Updates the reading progress for a specific chapter.
pub async fn update_chapter_progress(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(chapter_id): Path<i64>,
    Json(req): Json<UpdateProgressWithPercentageRequest>,
) -> Result<Json<ProgressResponse>> {
    let progress = if let Some(percentage) = req.percentage {
        state
            .progress_service
            .update_progress_with_percentage(
                auth_user.user_id,
                chapter_id,
                req.position,
                percentage,
            )
            .await?
    } else {
        state
            .progress_service
            .update_progress(auth_user.user_id, chapter_id, req.position)
            .await?
    };
    Ok(Json(ProgressResponse::from(progress)))
}
