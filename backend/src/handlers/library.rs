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
use std::sync::Arc;
use tracing::warn;

use crate::error::Result;
use crate::models::{
    CreateLibraryRequest, Library, LibraryWithStats, ScanPath, UpdateLibraryRequest,
};
use crate::services::library::LibraryService;
use crate::services::scheduler::SchedulerService;
use crate::services::watch::WatchService;

/// Application state containing the library service and optional watch/scheduler services.
#[derive(Clone)]
pub struct LibraryState {
    pub library_service: Arc<LibraryService>,
    pub watch_service: Option<Arc<WatchService>>,
    pub scheduler_service: Option<Arc<SchedulerService>>,
}

impl LibraryState {
    /// Create a new library state with just the library service.
    pub fn new(library_service: Arc<LibraryService>) -> Self {
        Self {
            library_service,
            watch_service: None,
            scheduler_service: None,
        }
    }

    /// Create a new library state with all services.
    pub fn with_services(
        library_service: Arc<LibraryService>,
        watch_service: Arc<WatchService>,
        scheduler_service: Arc<SchedulerService>,
    ) -> Self {
        Self {
            library_service,
            watch_service: Some(watch_service),
            scheduler_service: Some(scheduler_service),
        }
    }
}

/// GET /api/libraries
///
/// Returns a list of all libraries with their statistics.
///
/// # Response
/// ```json
/// [
///     {
///         "id": 1,
///         "name": "My Comics",
///         "scan_interval": 60,
///         "watch_mode": true,
///         "created_at": "2024-01-01T00:00:00Z",
///         "updated_at": "2024-01-01T00:00:00Z",
///         "path_count": 2,
///         "content_count": 100
///     }
/// ]
/// ```
///
/// Requirements: 1.4
pub async fn list_libraries(
    State(state): State<LibraryState>,
) -> Result<Json<Vec<LibraryWithStats>>> {
    let libraries = state.library_service.list().await?;
    Ok(Json(libraries))
}

/// POST /api/libraries
///
/// Creates a new library.
///
/// # Request Body
/// ```json
/// {
///     "name": "My Comics",
///     "scan_interval": 60,
///     "watch_mode": true
/// }
/// ```
///
/// # Response
/// Returns the created library.
///
/// Requirements: 1.1, 1.8, 1.9
pub async fn create_library(
    State(state): State<LibraryState>,
    Json(req): Json<CreateLibraryRequest>,
) -> Result<Json<Library>> {
    let scan_interval = req.scan_interval.unwrap_or(0);
    let watch_mode = req.watch_mode.unwrap_or(false);

    let library = state.library_service.create(req).await?;

    // Start scheduler if scan_interval is set (Requirement 1.8)
    if scan_interval > 0 {
        if let Some(ref scheduler) = state.scheduler_service {
            if let Err(e) = scheduler.schedule_scan(library.id, scan_interval).await {
                warn!(library_id = library.id, error = %e, "Failed to schedule scan for library");
            }
        }
    }

    // Start watch service if watch_mode is enabled (Requirement 1.9)
    if watch_mode {
        if let Some(ref watch) = state.watch_service {
            if let Err(e) = watch.start_watching(library.id).await {
                warn!(library_id = library.id, error = %e, "Failed to start watching library");
            }
        }
    }

    Ok(Json(library))
}

/// GET /api/libraries/{id}
///
/// Returns a library by its ID with statistics.
///
/// # Path Parameters
/// - `id`: The library ID
///
/// # Response
/// Returns the library with statistics, or 404 if not found.
///
/// Requirements: 1.4
pub async fn get_library(
    State(state): State<LibraryState>,
    Path(id): Path<i64>,
) -> Result<Json<LibraryWithStats>> {
    let library = state
        .library_service
        .get_with_stats(id)
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound(format!("Library with id {} not found", id))
        })?;
    Ok(Json(library))
}

/// PUT /api/libraries/{id}
///
/// Updates an existing library.
///
/// # Path Parameters
/// - `id`: The library ID
///
/// # Request Body
/// ```json
/// {
///     "name": "Updated Name",
///     "scan_interval": 120,
///     "watch_mode": false
/// }
/// ```
///
/// # Response
/// Returns the updated library.
///
/// Requirements: 1.7, 1.8, 1.9
pub async fn update_library(
    State(state): State<LibraryState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateLibraryRequest>,
) -> Result<Json<Library>> {
    let new_scan_interval = req.scan_interval;
    let new_watch_mode = req.watch_mode;

    let library = state.library_service.update(id, req).await?;

    // Update scheduler if scan_interval changed (Requirement 1.8)
    if let Some(interval) = new_scan_interval {
        if let Some(ref scheduler) = state.scheduler_service {
            if interval > 0 {
                if let Err(e) = scheduler.schedule_scan(id, interval).await {
                    warn!(library_id = id, error = %e, "Failed to update scan schedule for library");
                }
            } else if let Err(e) = scheduler.cancel_scan(id).await {
                warn!(library_id = id, error = %e, "Failed to cancel scan schedule for library");
            }
        }
    }

    // Update watch service if watch_mode changed (Requirement 1.9)
    if let Some(watch_mode) = new_watch_mode {
        if let Some(ref watch) = state.watch_service {
            if watch_mode {
                if let Err(e) = watch.start_watching(id).await {
                    warn!(library_id = id, error = %e, "Failed to start watching library");
                }
            } else if let Err(e) = watch.stop_watching(id).await {
                warn!(library_id = id, error = %e, "Failed to stop watching library");
            }
        }
    }

    Ok(Json(library))
}

/// DELETE /api/libraries/{id}
///
/// Deletes a library and all associated scan paths and contents.
///
/// # Path Parameters
/// - `id`: The library ID
///
/// # Response
/// Returns 200 OK with empty body on success.
///
/// Requirements: 1.6, 1.8, 1.9
pub async fn delete_library(
    State(state): State<LibraryState>,
    Path(id): Path<i64>,
) -> Result<Json<()>> {
    // Stop scheduler before deleting (Requirement 1.8)
    if let Some(ref scheduler) = state.scheduler_service {
        if let Err(e) = scheduler.cancel_scan(id).await {
            warn!(library_id = id, error = %e, "Failed to cancel scan schedule for library");
        }
    }

    // Stop watch service before deleting (Requirement 1.9)
    if let Some(ref watch) = state.watch_service {
        if let Err(e) = watch.stop_watching(id).await {
            warn!(library_id = id, error = %e, "Failed to stop watching library");
        }
    }

    state.library_service.delete(id).await?;
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
///         "path": "/path/to/comics",
///         "created_at": "2024-01-01T00:00:00Z"
///     }
/// ]
/// ```
///
/// Requirements: 1.2
pub async fn list_scan_paths(
    State(state): State<LibraryState>,
    Path(library_id): Path<i64>,
) -> Result<Json<Vec<ScanPath>>> {
    let paths = state.library_service.list_scan_paths(library_id).await?;
    Ok(Json(paths))
}

/// POST /api/libraries/{id}/paths
///
/// Adds a scan path to a library.
///
/// # Path Parameters
/// - `id`: The library ID
///
/// # Request Body
/// ```json
/// {
///     "path": "/path/to/comics"
/// }
/// ```
///
/// # Response
/// Returns the created scan path.
///
/// Requirements: 1.2, 1.9
pub async fn add_scan_path(
    State(state): State<LibraryState>,
    Path(library_id): Path<i64>,
    Json(req): Json<AddScanPathRequest>,
) -> Result<Json<ScanPath>> {
    let scan_path = state
        .library_service
        .add_scan_path(library_id, req.path)
        .await?;

    // Refresh watch service to include new path (Requirement 1.9)
    if let Some(ref watch) = state.watch_service {
        if let Err(e) = watch.refresh_watching(library_id).await {
            warn!(library_id = library_id, error = %e, "Failed to refresh watching for library");
        }
    }

    Ok(Json(scan_path))
}

/// Path parameters for scan path operations.
#[derive(Debug, Deserialize)]
pub struct ScanPathParams {
    /// The library ID.
    pub id: i64,
    /// The scan path ID.
    pub path_id: i64,
}

/// DELETE /api/libraries/{id}/paths/{path_id}
///
/// Removes a scan path from a library.
/// This will cascade delete all contents imported from this path.
///
/// # Path Parameters
/// - `id`: The library ID
/// - `path_id`: The scan path ID
///
/// # Response
/// Returns 200 OK with empty body on success.
///
/// Requirements: 1.3, 1.9
pub async fn remove_scan_path(
    State(state): State<LibraryState>,
    Path(params): Path<ScanPathParams>,
) -> Result<Json<()>> {
    state
        .library_service
        .remove_scan_path(params.id, params.path_id)
        .await?;

    // Refresh watch service to remove the path (Requirement 1.9)
    if let Some(ref watch) = state.watch_service {
        if let Err(e) = watch.refresh_watching(params.id).await {
            warn!(library_id = params.id, error = %e, "Failed to refresh watching for library");
        }
    }

    Ok(Json(()))
}
