//! Reading progress repository for database operations.
//!
//! This module provides database access for reading progress operations.

use chrono::Utc;
use sqlx::{Pool, Sqlite};

use crate::error::{AppError, Result};
use crate::models::Content;
use crate::models::{NewReadingProgress, ReadingProgress};

/// Repository for reading progress database operations.
pub struct ProgressRepository;

impl ProgressRepository {
    /// Create or update reading progress for a user on a chapter.
    ///
    /// Uses INSERT OR REPLACE to handle both high-level progress tracking.
    pub async fn upsert(
        pool: &Pool<Sqlite>,
        progress: NewReadingProgress,
    ) -> Result<ReadingProgress> {
        let now = Utc::now().to_rfc3339();

        // Use INSERT OR REPLACE with the unique constraint on (user_id, chapter_id)
        sqlx::query(
            r#"
            INSERT INTO reading_progress (user_id, chapter_id, position, percentage, updated_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(user_id, chapter_id) DO UPDATE SET
                position = excluded.position,
                percentage = excluded.percentage,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(progress.user_id)
        .bind(progress.chapter_id)
        .bind(progress.position)
        .bind(progress.percentage)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        // Fetch the upserted record
        Self::find_by_user_and_chapter(pool, progress.user_id, progress.chapter_id)
            .await?
            .ok_or_else(|| AppError::Internal("Failed to retrieve upserted progress".to_string()))
    }

    /// Find reading progress by user and chapter.
    pub async fn find_by_user_and_chapter(
        pool: &Pool<Sqlite>,
        user_id: i64,
        chapter_id: i64,
    ) -> Result<Option<ReadingProgress>> {
        sqlx::query_as::<_, ReadingProgress>(
            r#"
            SELECT id, user_id, chapter_id, position, percentage, updated_at
            FROM reading_progress
            WHERE user_id = ? AND chapter_id = ?
            "#,
        )
        .bind(user_id)
        .bind(chapter_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
    }

    /// Find all reading progress for a user on a specific content.
    ///
    /// Returns progress for all chapters of the content that the user has read.
    pub async fn find_by_user_and_content(
        pool: &Pool<Sqlite>,
        user_id: i64,
        content_id: i64,
    ) -> Result<Vec<ReadingProgress>> {
        sqlx::query_as::<_, ReadingProgress>(
            r#"
            SELECT rp.id, rp.user_id, rp.chapter_id, rp.position, rp.percentage, rp.updated_at
            FROM reading_progress rp
            INNER JOIN chapters c ON rp.chapter_id = c.id
            WHERE rp.user_id = ? AND c.content_id = ?
            ORDER BY c.sort_order
            "#,
        )
        .bind(user_id)
        .bind(content_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    /// Find the most recently updated progress for a user on a content.
    ///
    /// Returns the chapter progress that was most recently updated.
    pub async fn find_latest_by_user_and_content(
        pool: &Pool<Sqlite>,
        user_id: i64,
        content_id: i64,
    ) -> Result<Option<ReadingProgress>> {
        sqlx::query_as::<_, ReadingProgress>(
            r#"
            SELECT rp.id, rp.user_id, rp.chapter_id, rp.position, rp.percentage, rp.updated_at
            FROM reading_progress rp
            INNER JOIN chapters c ON rp.chapter_id = c.id
            WHERE rp.user_id = ? AND c.content_id = ?
            ORDER BY rp.updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(content_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
    }

    /// Find the most recently read contents by a user.
    ///
    /// Returns the contents that have the most recently updated reading progress.
    pub async fn find_recent_contents_by_user(
        pool: &Pool<Sqlite>,
        user_id: i64,
        limit: i64,
    ) -> Result<Vec<Content>> {
        sqlx::query_as::<_, Content>(
            r#"
            SELECT c.*
            FROM contents c
            JOIN chapters ch ON c.id = ch.content_id
            JOIN reading_progress rp ON ch.id = rp.chapter_id
            WHERE rp.user_id = ?
            GROUP BY c.id
            ORDER BY MAX(rp.updated_at) DESC
            LIMIT ?
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    /// Delete reading progress by ID.
    pub async fn delete(pool: &Pool<Sqlite>, id: i64) -> Result<()> {
        let result = sqlx::query("DELETE FROM reading_progress WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "Reading progress with id {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Delete all reading progress for a user.
    pub async fn delete_by_user(pool: &Pool<Sqlite>, user_id: i64) -> Result<u64> {
        let result = sqlx::query("DELETE FROM reading_progress WHERE user_id = ?")
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(result.rows_affected())
    }

    /// Count chapters with progress for a user on a content.
    pub async fn count_chapters_with_progress(
        pool: &Pool<Sqlite>,
        user_id: i64,
        content_id: i64,
    ) -> Result<i64> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM reading_progress rp
            INNER JOIN chapters c ON rp.chapter_id = c.id
            WHERE rp.user_id = ? AND c.content_id = ?
            "#,
        )
        .bind(user_id)
        .bind(content_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(result.0)
    }

    /// Count completed chapters (percentage >= 100) for a user on a content.
    pub async fn count_completed_chapters(
        pool: &Pool<Sqlite>,
        user_id: i64,
        content_id: i64,
    ) -> Result<i64> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM reading_progress rp
            INNER JOIN chapters c ON rp.chapter_id = c.id
            WHERE rp.user_id = ? AND c.content_id = ? AND rp.percentage >= 100.0
            "#,
        )
        .bind(user_id)
        .bind(content_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(result.0)
    }
}
