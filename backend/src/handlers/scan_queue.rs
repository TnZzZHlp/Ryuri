//! Scan queue handlers.
//!
//! This module provides HTTP handlers for scan queue management endpoints:
//! - POST /api/libraries/{id}/scan - Submit a scan task (High priority)
//! - GET /api/scan-tasks/{id} - Get task status
//! - GET /api/scan-tasks - List all tasks (pending + recent history)
//! - DELETE /api/scan-tasks/{id} - Cancel a task

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_i18n::t;

use crate::error::{AppError, Result};
use crate::models::{ScanTask, TaskPriority};
use crate::state::AppState;

/// Response for submitting a scan task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct SubmitScanResponse {
    /// The task identifier.
    pub task_id: Uuid,
    /// The task details.
    pub task: ScanTask,
}

/// POST /api/libraries/{id}/scan
///
/// Submits a scan task for a library with High priority.
/// If a task already exists for the library, returns the existing task.
///
/// Requirements: 1.1, 4.1, 4.2, 5.1
pub async fn submit_scan(
    State(state): State<AppState>,
    Path(library_id): Path<i64>,
) -> Result<Json<SubmitScanResponse>> {
    // Verify library exists
    let library = state.library_service.get(library_id).await?;
    if library.is_none() {
        return Err(AppError::NotFound(t!("library.id_not_found", id = library_id).to_string()));
    }

    // Submit task with High priority (manual scan)
    let task_id = state
        .scan_queue_service
        .submit_task(library_id, TaskPriority::High)
        .await;

    // Get the task details
    let task = state
        .scan_queue_service
        .get_task(task_id)
        .await
        .ok_or_else(|| AppError::Internal("Failed to retrieve submitted task".to_string()))?;

    Ok(Json(SubmitScanResponse { task_id, task }))
}

/// GET /api/scan-tasks/{id}
///
/// Returns the status and details of a scan task.
///
/// Requirements: 2.1
pub async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<ScanTask>> {
    let task = state
        .scan_queue_service
        .get_task(task_id)
        .await
        .ok_or_else(|| AppError::NotFound(t!("scan_queue.task_not_found", id = task_id).to_string()))?;

    Ok(Json(task))
}

/// Query parameters for listing tasks.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct ListTasksQuery {
    /// Maximum number of history tasks to return (default: 50).
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

/// Response for listing tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct ListTasksResponse {
    /// Tasks currently pending in the queue.
    pub pending: Vec<ScanTask>,
    /// Tasks currently being processed.
    pub processing: Vec<ScanTask>,
    /// Recently completed/failed/cancelled tasks.
    pub history: Vec<ScanTask>,
}

/// GET /api/scan-tasks
///
/// Returns all pending tasks and recent task history.
///
/// Requirements: 2.3
pub async fn list_tasks(
    State(state): State<AppState>,
    Query(query): Query<ListTasksQuery>,
) -> Result<Json<ListTasksResponse>> {
    let processing = state.scan_queue_service.list_processing().await;
    let pending = state.scan_queue_service.list_pending().await;
    let history = state.scan_queue_service.list_history(query.limit).await;

    Ok(Json(ListTasksResponse {
        pending,
        processing,
        history,
    }))
}

/// DELETE /api/scan-tasks/{id}
///
/// Cancels a pending or running scan task.
///
/// Requirements: 3.1
pub async fn cancel_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<ScanTask>> {
    // Cancel the task
    state.scan_queue_service.cancel_task(task_id).await?;

    // Return the updated task
    let task = state
        .scan_queue_service
        .get_task(task_id)
        .await
        .ok_or_else(|| AppError::NotFound(t!("scan_queue.task_not_found", id = task_id).to_string()))?;

    Ok(Json(task))
}
