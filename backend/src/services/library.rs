//! Library management service.
//!
//! This module provides business logic for library and scan path management.

use sqlx::{Pool, Sqlite};

use crate::error::{AppError, Result};
use crate::models::{
    CreateLibraryRequest, Library, LibraryWithStats, NewLibrary, NewScanPath, ScanPath,
    UpdateLibraryRequest,
};
use crate::repository::library::{LibraryRepository, ScanPathRepository};

/// Service for library management operations.
pub struct LibraryService {
    pool: Pool<Sqlite>,
}

impl LibraryService {
    /// Create a new library service.
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Create a new library.
    ///
    /// Requirements: 1.1
    pub async fn create(&self, req: CreateLibraryRequest) -> Result<Library> {
        // Validate input
        if req.name.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Library name cannot be empty".to_string(),
            ));
        }

        let new_library = NewLibrary {
            name: req.name.trim().to_string(),
            scan_interval: req.scan_interval.unwrap_or(0),
            watch_mode: req.watch_mode.unwrap_or(false),
        };

        LibraryRepository::create(&self.pool, new_library).await
    }

    /// Get a library by ID.
    pub async fn get(&self, id: i64) -> Result<Option<Library>> {
        LibraryRepository::find_by_id(&self.pool, id).await
    }

    /// Get a library by ID, returning an error if not found.
    pub async fn get_or_error(&self, id: i64) -> Result<Library> {
        self.get(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Library with id {} not found", id)))
    }

    /// List all libraries with statistics.
    ///
    /// Requirements: 1.4
    pub async fn list(&self) -> Result<Vec<LibraryWithStats>> {
        LibraryRepository::list_with_stats(&self.pool).await
    }

    /// Update a library.
    ///
    /// Requirements: 1.7
    pub async fn update(&self, id: i64, req: UpdateLibraryRequest) -> Result<Library> {
        // Validate name if provided
        if let Some(ref name) = req.name {
            if name.trim().is_empty() {
                return Err(AppError::BadRequest(
                    "Library name cannot be empty".to_string(),
                ));
            }
        }

        LibraryRepository::update(
            &self.pool,
            id,
            req.name.map(|n| n.trim().to_string()),
            req.scan_interval,
            req.watch_mode,
        )
        .await
    }

    /// Delete a library.
    ///
    /// This will cascade delete all associated scan paths and contents.
    /// Requirements: 1.6
    pub async fn delete(&self, id: i64) -> Result<()> {
        LibraryRepository::delete(&self.pool, id).await
    }

    /// Add a scan path to a library.
    ///
    /// Requirements: 1.2
    pub async fn add_scan_path(&self, library_id: i64, path: String) -> Result<ScanPath> {
        // Verify library exists
        self.get_or_error(library_id).await?;

        // Validate path
        if path.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Scan path cannot be empty".to_string(),
            ));
        }

        let new_scan_path = NewScanPath {
            library_id,
            path: path.trim().to_string(),
        };

        ScanPathRepository::create(&self.pool, new_scan_path).await
    }

    /// Remove a scan path from a library.
    ///
    /// This will cascade delete all contents imported from this path.
    /// Requirements: 1.3
    pub async fn remove_scan_path(&self, library_id: i64, path_id: i64) -> Result<()> {
        ScanPathRepository::delete_by_library_and_id(&self.pool, library_id, path_id).await
    }

    /// List all scan paths for a library.
    pub async fn list_scan_paths(&self, library_id: i64) -> Result<Vec<ScanPath>> {
        // Verify library exists
        self.get_or_error(library_id).await?;

        ScanPathRepository::list_by_library(&self.pool, library_id).await
    }

    /// Get library statistics (path count and content count).
    pub async fn get_stats(&self, library_id: i64) -> Result<(i64, i64)> {
        let path_count = LibraryRepository::count_scan_paths(&self.pool, library_id).await?;
        let content_count = LibraryRepository::count_contents(&self.pool, library_id).await?;
        Ok((path_count, content_count))
    }

    /// Get a library with statistics.
    pub async fn get_with_stats(&self, id: i64) -> Result<Option<LibraryWithStats>> {
        let library = match self.get(id).await? {
            Some(lib) => lib,
            None => return Ok(None),
        };

        let path_count = LibraryRepository::count_scan_paths(&self.pool, id).await?;
        let content_count = LibraryRepository::count_contents(&self.pool, id).await?;

        Ok(Some(LibraryWithStats {
            library,
            path_count,
            content_count,
        }))
    }
}
