//! Content repository for database operations.
//!
//! This module provides database access for content and chapter operations.

use chrono::Utc;
use sqlx::{Pool, Sqlite};

use crate::error::{AppError, Result};
use crate::models::{Chapter, Content, ContentType, NewChapter, NewContent};

/// Repository for content database operations.
pub struct ContentRepository;

impl ContentRepository {
    /// Create a new content in the database.
    pub async fn create(pool: &Pool<Sqlite>, new_content: NewContent) -> Result<Content> {
        let now = Utc::now().to_rfc3339();
        let content_type_str = match new_content.content_type {
            ContentType::Comic => "Comic",
            ContentType::Novel => "Novel",
        };

        let result = sqlx::query(
            r#"
            INSERT INTO contents (library_id, scan_path_id, content_type, title, folder_path, chapter_count, thumbnail, metadata, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(new_content.library_id)
        .bind(new_content.scan_path_id)
        .bind(content_type_str)
        .bind(&new_content.title)
        .bind(&new_content.folder_path)
        .bind(new_content.chapter_count)
        .bind(&new_content.thumbnail)
        .bind(new_content.metadata.as_ref().and_then(|m| serde_json::to_vec(m).ok()))
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await;

        match result {
            Ok(res) => {
                let id = res.last_insert_rowid();
                Self::find_by_id(pool, id).await?.ok_or_else(|| {
                    AppError::Internal("Failed to retrieve created content".to_string())
                })
            }
            Err(e) => {
                if e.to_string().contains("UNIQUE constraint failed") {
                    Err(AppError::BadRequest(format!(
                        "Content at path '{}' already exists in this library",
                        new_content.folder_path
                    )))
                } else {
                    Err(AppError::Database(e))
                }
            }
        }
    }

    /// Find a content by ID.
    pub async fn find_by_id(pool: &Pool<Sqlite>, id: i64) -> Result<Option<Content>> {
        sqlx::query_as::<_, Content>(
            r#"
            SELECT id, library_id, scan_path_id, content_type, title, folder_path, chapter_count, thumbnail, metadata, created_at, updated_at
            FROM contents
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
    }

    /// Find a content by folder path within a library.
    pub async fn find_by_folder_path(
        pool: &Pool<Sqlite>,
        library_id: i64,
        folder_path: &str,
    ) -> Result<Option<Content>> {
        sqlx::query_as::<_, Content>(
            r#"
            SELECT id, library_id, scan_path_id, content_type, title, folder_path, chapter_count, thumbnail, metadata, created_at, updated_at
            FROM contents
            WHERE library_id = ? AND folder_path = ?
            "#,
        )
        .bind(library_id)
        .bind(folder_path)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
    }

    /// List all contents for a library.
    pub async fn list_by_library(pool: &Pool<Sqlite>, library_id: i64) -> Result<Vec<Content>> {
        sqlx::query_as::<_, Content>(
            r#"
            SELECT id, library_id, scan_path_id, content_type, title, folder_path, chapter_count, thumbnail, metadata, created_at, updated_at
            FROM contents
            WHERE library_id = ?
            ORDER BY title
            "#,
        )
        .bind(library_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    /// List all contents for a scan path.
    pub async fn list_by_scan_path(pool: &Pool<Sqlite>, scan_path_id: i64) -> Result<Vec<Content>> {
        sqlx::query_as::<_, Content>(
            r#"
            SELECT id, library_id, scan_path_id, content_type, title, folder_path, chapter_count, thumbnail, metadata, created_at, updated_at
            FROM contents
            WHERE scan_path_id = ?
            ORDER BY title
            "#,
        )
        .bind(scan_path_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    /// Search contents by title within a library.
    pub async fn search_by_title(
        pool: &Pool<Sqlite>,
        library_id: i64,
        query: &str,
    ) -> Result<Vec<Content>> {
        let search_pattern = format!("%{}%", query);
        sqlx::query_as::<_, Content>(
            r#"
            SELECT id, library_id, scan_path_id, content_type, title, folder_path, chapter_count, thumbnail, metadata, created_at, updated_at
            FROM contents
            WHERE library_id = ? AND title LIKE ?
            ORDER BY title
            "#,
        )
        .bind(library_id)
        .bind(&search_pattern)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    /// Update content metadata.
    pub async fn update_metadata(
        pool: &Pool<Sqlite>,
        id: i64,
        metadata: Option<serde_json::Value>,
        thumbnail: Option<Vec<u8>>,
    ) -> Result<Content> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE contents
            SET metadata = ?, thumbnail = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(metadata.as_ref().and_then(|m| serde_json::to_vec(m).ok()))
        .bind(&thumbnail)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        Self::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Content with id {} not found", id)))
    }

    /// Update content information (title, metadata, thumbnail).
    /// Fields set to None will not be updated.
    /// To clear metadata or thumbnail, pass Some(None).
    pub async fn update_info(
        pool: &Pool<Sqlite>,
        id: i64,
        title: Option<String>,
        metadata: Option<Option<serde_json::Value>>,
        thumbnail: Option<Option<Vec<u8>>>,
    ) -> Result<Content> {
        use sqlx::Arguments;
        let mut query = "UPDATE contents SET updated_at = ?".to_string();
        let mut args = sqlx::sqlite::SqliteArguments::default();
        let _ = args.add(Utc::now().to_rfc3339());

        if let Some(t) = title {
            query.push_str(", title = ?");
            let _ = args.add(t);
        }

        if let Some(m_opt) = metadata {
            query.push_str(", metadata = ?");
            let _ = args.add(m_opt.and_then(|v| serde_json::to_vec(&v).ok()));
        }

        if let Some(t_opt) = thumbnail {
            query.push_str(", thumbnail = ?");
            let _ = args.add(t_opt);
        }

        query.push_str(" WHERE id = ?");
        let _ = args.add(id);

        sqlx::query_with(&query, args)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        Self::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Content with id {} not found", id)))
    }

    /// Update content thumbnail.
    pub async fn update_thumbnail(
        pool: &Pool<Sqlite>,
        id: i64,
        thumbnail: Option<Vec<u8>>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE contents
            SET thumbnail = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&thumbnail)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    /// Update content chapter count.
    pub async fn update_chapter_count(
        pool: &Pool<Sqlite>,
        id: i64,
        chapter_count: i32,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE contents
            SET chapter_count = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(chapter_count)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    /// Delete a content by ID.
    pub async fn delete(pool: &Pool<Sqlite>, id: i64) -> Result<()> {
        let result = sqlx::query("DELETE FROM contents WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "Content with id {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Delete all contents for a scan path.
    pub async fn delete_by_scan_path(pool: &Pool<Sqlite>, scan_path_id: i64) -> Result<u64> {
        let result = sqlx::query("DELETE FROM contents WHERE scan_path_id = ?")
            .bind(scan_path_id)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(result.rows_affected())
    }

    /// Get all folder paths for a scan path.
    pub async fn get_folder_paths_by_scan_path(
        pool: &Pool<Sqlite>,
        scan_path_id: i64,
    ) -> Result<Vec<String>> {
        let results: Vec<(String,)> =
            sqlx::query_as("SELECT folder_path FROM contents WHERE scan_path_id = ?")
                .bind(scan_path_id)
                .fetch_all(pool)
                .await
                .map_err(AppError::Database)?;

        Ok(results.into_iter().map(|(path,)| path).collect())
    }
}

/// Repository for chapter database operations.
pub struct ChapterRepository;

impl ChapterRepository {
    /// Create a new chapter in the database.
    pub async fn create(pool: &Pool<Sqlite>, new_chapter: NewChapter) -> Result<Chapter> {
        let result = sqlx::query(
            r#"
            INSERT INTO chapters (content_id, title, file_path, sort_order, page_count, size)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(new_chapter.content_id)
        .bind(&new_chapter.title)
        .bind(&new_chapter.file_path)
        .bind(new_chapter.sort_order)
        .bind(new_chapter.page_count)
        .bind(new_chapter.size)
        .execute(pool)
        .await;

        match result {
            Ok(res) => {
                let id = res.last_insert_rowid();
                Self::find_by_id(pool, id).await?.ok_or_else(|| {
                    AppError::Internal("Failed to retrieve created chapter".to_string())
                })
            }
            Err(e) => {
                if e.to_string().contains("UNIQUE constraint failed") {
                    Err(AppError::BadRequest(format!(
                        "Chapter at path '{}' already exists",
                        new_chapter.file_path
                    )))
                } else {
                    Err(AppError::Database(e))
                }
            }
        }
    }

    /// Create multiple chapters in a batch.
    pub async fn create_batch(
        pool: &Pool<Sqlite>,
        chapters: Vec<NewChapter>,
    ) -> Result<Vec<Chapter>> {
        let mut created = Vec::with_capacity(chapters.len());
        for chapter in chapters {
            let c = Self::create(pool, chapter).await?;
            created.push(c);
        }
        Ok(created)
    }

    /// Find a chapter by ID.
    pub async fn find_by_id(pool: &Pool<Sqlite>, id: i64) -> Result<Option<Chapter>> {
        sqlx::query_as::<_, Chapter>(
            r#"
            SELECT id, content_id, title, file_path, sort_order, page_count, size
            FROM chapters
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
    }

    /// List all chapters for a content, ordered by sort_order.
    pub async fn list_by_content(pool: &Pool<Sqlite>, content_id: i64) -> Result<Vec<Chapter>> {
        sqlx::query_as::<_, Chapter>(
            r#"
            SELECT id, content_id, title, file_path, sort_order, page_count, size
            FROM chapters
            WHERE content_id = ?
            ORDER BY sort_order
            "#,
        )
        .bind(content_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    /// Delete all chapters for a content.
    pub async fn delete_by_content(pool: &Pool<Sqlite>, content_id: i64) -> Result<u64> {
        let result = sqlx::query("DELETE FROM chapters WHERE content_id = ?")
            .bind(content_id)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(result.rows_affected())
    }

    /// Get chapter count for a content.
    pub async fn count_by_content(pool: &Pool<Sqlite>, content_id: i64) -> Result<i64> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM chapters WHERE content_id = ?")
            .bind(content_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(result.0)
    }
}
