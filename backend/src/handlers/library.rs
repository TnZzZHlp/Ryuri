//! Library management handlers.
//!
//! This module provides HTTP handlers for library management endpoints:
//! - GET /api/libraries - List all libraries
//! - POST /api/libraries - Create a new library
//! - GET /api/libraries/{id} - Get a library by ID
//! - PUT /api/libraries/{id} - Update a library
//! - DELETE /api/libraries/{id} - Delete a library
//! - GET /api/libraries/{id}/paths - List scan paths for a library
//! - POST /api/libraries/{id}/paths - Add a scan path to a library
//! - DELETE /api/libraries/{id}/paths/{path_id} - Remove a scan path from a library

use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use tracing::warn;
use rust_i18n::t;

use crate::error::Result;
use crate::models::{
    CreateLibraryRequest, Library, LibraryWithStats, ScanPath, UpdateLibraryRequest,
};
use crate::state::AppState;

/// GET /api/libraries
///
/// Returns a list of all libraries with their statistics.
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<LibraryWithStats>>> {
    let libraries = state.library_service.list().await?;
    Ok(Json(libraries))
}

/// POST /api/libraries
///
/// Creates a new library.
pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateLibraryRequest>,
) -> Result<Json<Library>> {
    let scan_interval = req.scan_interval.unwrap_or(0);
    let watch_mode = req.watch_mode.unwrap_or(false);

    let library = state.library_service.create(req).await?;

    // Start scheduler if scan_interval is set
    if scan_interval > 0
        && let Err(e) = state
            .scheduler_service
            .schedule_scan(library.id, scan_interval)
            .await
    {
        warn!(library_id = library.id, error = %e, "{}", t!("library.schedule_scan_failed"));
    }

    // Start watch service if watch_mode is enabled
    if watch_mode && let Err(e) = state.watch_service.start_watching(library.id).await {
        warn!(library_id = library.id, error = %e, "{}", t!("library.start_watch_failed"));
    }

    Ok(Json(library))
}

/// GET /api/libraries/{id}
///
/// Returns a library by its ID with statistics.
pub async fn get(
    State(state): State<AppState>,
    Path(library_id): Path<i64>,
) -> Result<Json<LibraryWithStats>> {
    let library = state
        .library_service
        .get_with_stats(library_id)
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound(t!("library.not_found", id = library_id).to_string())
        })?;
    Ok(Json(library))
}

/// PUT /api/libraries/{id}
///
/// Updates an existing library.
pub async fn update(
    State(state): State<AppState>,
    Path(library_id): Path<i64>,
    Json(req): Json<UpdateLibraryRequest>,
) -> Result<Json<Library>> {
    let new_scan_interval = req.scan_interval;
    let new_watch_mode = req.watch_mode;

    let library = state.library_service.update(library_id, req).await?;

    // Update scheduler if scan_interval changed
    if let Some(interval) = new_scan_interval {
        if interval > 0 {
            if let Err(e) = state.scheduler_service.schedule_scan(library_id, interval).await {
                warn!(library_id = library_id, error = %e, "{}", t!("library.update_schedule_failed"));
            }
        } else if let Err(e) = state.scheduler_service.cancel_scan(library_id).await {
            warn!(library_id = library_id, error = %e, "{}", t!("library.cancel_scan_failed"));
        }
    }

    // Update watch service if watch_mode changed
    if let Some(watch_mode) = new_watch_mode {
        if watch_mode {
            if let Err(e) = state.watch_service.start_watching(library_id).await {
                warn!(library_id = library_id, error = %e, "{}", t!("library.start_watch_failed"));
            }
        } else if let Err(e) = state.watch_service.stop_watching(library_id).await {
            warn!(library_id = library_id, error = %e, "{}", t!("library.stop_watch_failed"));
        }
    }

    Ok(Json(library))
}

/// DELETE /api/libraries/{id}
///
/// Deletes a library and all associated scan paths and contents.
pub async fn delete(State(state): State<AppState>, Path(library_id): Path<i64>) -> Result<Json<()>> {
    // Stop scheduler before deleting
    if let Err(e) = state.scheduler_service.cancel_scan(library_id).await {
        warn!(library_id, error = %e, "{}", t!("library.cancel_scan_failed"));
    }

    // Stop watch service before deleting
    if let Err(e) = state.watch_service.stop_watching(library_id).await {
        warn!(library_id, error = %e, "{}", t!("library.stop_watch_failed"));
    }

    state.library_service.delete(library_id).await?;
    Ok(Json(()))
}

/// Request to add a scan path.
#[derive(Debug, Clone, Deserialize)]
pub struct AddScanPathRequest {
    /// The file system path to add.
    pub path: String,
}

/// GET /api/libraries/{id}/paths
///
/// Returns all scan paths for a library.
pub async fn list_paths(
    State(state): State<AppState>,
    Path(library_id): Path<i64>,
) -> Result<Json<Vec<ScanPath>>> {
    let paths = state.library_service.list_scan_paths(library_id).await?;
    Ok(Json(paths))
}

/// POST /api/libraries/{id}/paths
///
/// Adds a scan path to a library.
pub async fn add_path(
    State(state): State<AppState>,
    Path(library_id): Path<i64>,
    Json(req): Json<AddScanPathRequest>,
) -> Result<Json<ScanPath>> {
    let scan_path = state
        .library_service
        .add_scan_path(library_id, req.path)
        .await?;

    // Refresh watch service to include new path
    if let Err(e) = state.watch_service.refresh_watching(library_id).await {
        warn!(library_id = library_id, error = %e, "{}", t!("library.refresh_watch_failed"));
    }

    Ok(Json(scan_path))
}

/// Path parameters for scan path operations.
#[derive(Debug, Deserialize)]
pub struct ScanPathParams {
    /// The library ID.
    pub library_id: i64,
    /// The scan path ID.
    pub path_id: i64,
}

/// DELETE /api/libraries/{id}/paths/{path_id}
///
/// Removes a scan path from a library.
pub async fn remove_path(
    State(state): State<AppState>,
    Path(params): Path<ScanPathParams>,
) -> Result<Json<()>> {
    state
        .library_service
        .remove_scan_path(params.library_id, params.path_id)
        .await?;

    // Refresh watch service to remove the path
    if let Err(e) = state.watch_service.refresh_watching(params.library_id).await {
        warn!(library_id = params.library_id, error = %e, "{}", t!("library.refresh_watch_failed"));
    }

    Ok(Json(()))
}
