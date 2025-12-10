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
use std::sync::Arc;

use crate::error::Result;
use crate::models::{ContentProgressResponse, ProgressResponse};
use crate::services::auth::AuthService;
use crate::services::auth::middleware::{AuthUser, HasAuthService};
use crate::services::progress::ProgressService;

/// Application state for progress handlers.
#[derive(Clone)]
pub struct ProgressState {
    pub progress_service: Arc<ProgressService>,
    pub auth_service: Arc<AuthService>,
}

impl HasAuthService for ProgressState {
    fn auth_service(&self) -> &AuthService {
        &self.auth_service
    }
}

/// GET /api/contents/{id}/progress
///
/// Returns the overall reading progress for a content.
/// Aggregates progress from all chapters to calculate overall percentage.
///
/// # Path Parameters
/// - `id`: The content ID
///
/// # Response
/// ```json
/// {
///     "content_id": 1,
///     "total_chapters": 10,
///     "completed_chapters": 5,
///     "current_chapter_id": 6,
///     "overall_percentage": 55.0
/// }
/// ```
///
/// Requirements: 5.1
pub async fn get_content_progress(
    State(state): State<ProgressState>,
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
///
/// # Path Parameters
/// - `id`: The chapter ID
///
/// # Response
/// ```json
/// {
///     "chapter_id": 1,
///     "position": 5,
///     "percentage": 50.0,
///     "updated_at": "2024-01-01T00:00:00Z"
/// }
/// ```
///
/// Returns null if no progress exists for this chapter.
///
/// Requirements: 5.2
pub async fn get_chapter_progress(
    State(state): State<ProgressState>,
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
///
/// # Path Parameters
/// - `id`: The chapter ID
///
/// # Request Body
/// ```json
/// {
///     "position": 5,
///     "percentage": 50.0
/// }
/// ```
///
/// # Response
/// Returns the updated progress.
///
/// # Errors
/// - 400 Bad Request: Invalid position (negative) or percentage (not in 0-100 range)
/// - 404 Not Found: Chapter does not exist
///
/// Requirements: 5.2, 5.3, 7.4
pub async fn update_chapter_progress(
    State(state): State<ProgressState>,
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
