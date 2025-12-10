//! Library repository for database operations.
//!
//! This module provides database access for library and scan path operations.

use chrono::Utc;
use sqlx::{Pool, Sqlite};

use crate::error::{AppError, Result};
use crate::models::{Library, LibraryWithStats, NewLibrary, NewScanPath, ScanPath};

/// Repository for library database operations.
pub struct LibraryRepository;

impl LibraryRepository {
    /// Create a new library in the database.
    pub async fn create(pool: &Pool<Sqlite>, new_library: NewLibrary) -> Result<Library> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            r#"
            INSERT INTO libraries (name, scan_interval, watch_mode, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&new_library.name)
        .bind(new_library.scan_interval)
        .bind(new_library.watch_mode)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        let id = result.last_insert_rowid();
        Self::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::Internal("Failed to retrieve created library".to_string()))
    }

    /// Find a library by ID.
    pub async fn find_by_id(pool: &Pool<Sqlite>, id: i64) -> Result<Option<Library>> {
        sqlx::query_as::<_, Library>(
            r#"
            SELECT id, name, scan_interval, watch_mode, created_at, updated_at
            FROM libraries
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
    }

    /// List all libraries.
    pub async fn list(pool: &Pool<Sqlite>) -> Result<Vec<Library>> {
        sqlx::query_as::<_, Library>(
            r#"
            SELECT id, name, scan_interval, watch_mode, created_at, updated_at
            FROM libraries
            ORDER BY name
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    /// List all libraries with statistics (path count and content count).
    pub async fn list_with_stats(pool: &Pool<Sqlite>) -> Result<Vec<LibraryWithStats>> {
        let libraries = Self::list(pool).await?;
        let mut result = Vec::with_capacity(libraries.len());

        for library in libraries {
            let path_count = Self::count_scan_paths(pool, library.id).await?;
            let content_count = Self::count_contents(pool, library.id).await?;
            result.push(LibraryWithStats {
                library,
                path_count,
                content_count,
            });
        }

        Ok(result)
    }

    /// Update a library.
    pub async fn update(
        pool: &Pool<Sqlite>,
        id: i64,
        name: Option<String>,
        scan_interval: Option<i32>,
        watch_mode: Option<bool>,
    ) -> Result<Library> {
        let existing = Self::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Library with id {} not found", id)))?;

        let now = Utc::now().to_rfc3339();
        let new_name = name.unwrap_or(existing.name);
        let new_scan_interval = scan_interval.unwrap_or(existing.scan_interval);
        let new_watch_mode = watch_mode.unwrap_or(existing.watch_mode);

        sqlx::query(
            r#"
            UPDATE libraries
            SET name = ?, scan_interval = ?, watch_mode = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&new_name)
        .bind(new_scan_interval)
        .bind(new_watch_mode)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        Self::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::Internal("Failed to retrieve updated library".to_string()))
    }

    /// Delete a library by ID.
    /// This will cascade delete all associated scan paths and contents.
    pub async fn delete(pool: &Pool<Sqlite>, id: i64) -> Result<()> {
        let result = sqlx::query("DELETE FROM libraries WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "Library with id {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Count scan paths for a library.
    pub async fn count_scan_paths(pool: &Pool<Sqlite>, library_id: i64) -> Result<i64> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM scan_paths WHERE library_id = ?")
            .bind(library_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(result.0)
    }

    /// Count contents for a library.
    pub async fn count_contents(pool: &Pool<Sqlite>, library_id: i64) -> Result<i64> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM contents WHERE library_id = ?")
            .bind(library_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(result.0)
    }
}

/// Repository for scan path database operations.
pub struct ScanPathRepository;

impl ScanPathRepository {
    /// Create a new scan path.
    pub async fn create(pool: &Pool<Sqlite>, new_scan_path: NewScanPath) -> Result<ScanPath> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            r#"
            INSERT INTO scan_paths (library_id, path, created_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(new_scan_path.library_id)
        .bind(&new_scan_path.path)
        .bind(&now)
        .execute(pool)
        .await;

        match result {
            Ok(res) => {
                let id = res.last_insert_rowid();
                Self::find_by_id(pool, id).await?.ok_or_else(|| {
                    AppError::Internal("Failed to retrieve created scan path".to_string())
                })
            }
            Err(e) => {
                if e.to_string().contains("UNIQUE constraint failed") {
                    Err(AppError::BadRequest(format!(
                        "Scan path '{}' already exists in this library",
                        new_scan_path.path
                    )))
                } else {
                    Err(AppError::Database(e))
                }
            }
        }
    }

    /// Find a scan path by ID.
    pub async fn find_by_id(pool: &Pool<Sqlite>, id: i64) -> Result<Option<ScanPath>> {
        sqlx::query_as::<_, ScanPath>(
            r#"
            SELECT id, library_id, path, created_at
            FROM scan_paths
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
    }

    /// List all scan paths for a library.
    pub async fn list_by_library(pool: &Pool<Sqlite>, library_id: i64) -> Result<Vec<ScanPath>> {
        sqlx::query_as::<_, ScanPath>(
            r#"
            SELECT id, library_id, path, created_at
            FROM scan_paths
            WHERE library_id = ?
            ORDER BY path
            "#,
        )
        .bind(library_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    /// Delete a scan path by ID.
    /// This will cascade delete all contents imported from this path.
    pub async fn delete(pool: &Pool<Sqlite>, id: i64) -> Result<()> {
        let result = sqlx::query("DELETE FROM scan_paths WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "Scan path with id {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Delete a scan path by library ID and path ID.
    pub async fn delete_by_library_and_id(
        pool: &Pool<Sqlite>,
        library_id: i64,
        path_id: i64,
    ) -> Result<()> {
        let result = sqlx::query("DELETE FROM scan_paths WHERE id = ? AND library_id = ?")
            .bind(path_id)
            .bind(library_id)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "Scan path with id {} not found in library {}",
                path_id, library_id
            )));
        }

        Ok(())
    }

    /// Check if a path exists in a library.
    pub async fn path_exists(pool: &Pool<Sqlite>, library_id: i64, path: &str) -> Result<bool> {
        let result: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM scan_paths WHERE library_id = ? AND path = ?")
                .bind(library_id)
                .bind(path)
                .fetch_one(pool)
                .await
                .map_err(AppError::Database)?;

        Ok(result.0 > 0)
    }
}
