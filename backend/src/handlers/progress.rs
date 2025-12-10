//! Reading progress handlers.
//!
//! This module provides HTTP handlers for progress management endpoints:
//! - GET /api/contents/{id}/progress - Get overall content progress
//! - GET /api/chapters/{id}/progress - Get chapter progress
//! - PUT /api/chapters/{id}/progress - Update chapter progress

use axum::{
    Json,
    extract::{Path, State},
};

use crate::error::Result;
use crate::models::{ContentProgressResponse, ProgressResponse};
use crate::services::auth::middleware::AuthUser;
use crate::state::AppState;

/// GET /api/contents/{id}/progress
///
/// Returns the overall reading progress for a content.
pub async fn get_content_progress(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(content_id): Path<i64>,
) -> Result<Json<ContentProgressResponse>> {
    let progress = state
        .progress_service
        .get_aggregated_content_progress(auth_user.user_id, content_id)
        .await?;
    Ok(Json(progress))
}

/// GET /api/chapters/{id}/progress
///
/// Returns the reading progress for a specific chapter.
pub async fn get_chapter_progress(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(chapter_id): Path<i64>,
) -> Result<Json<Option<ProgressResponse>>> {
    let progress = state
        .progress_service
        .get_chapter_progress_response(auth_user.user_id, chapter_id)
        .await?;
    Ok(Json(progress))
}

/// Request body for updating chapter progress with optional percentage.
#[derive(Debug, Clone, serde::Deserialize)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
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
