//! API key repository for database operations.
//!
//! This module provides database access for API key operations.

use chrono::Utc;
use sqlx::{Pool, Sqlite};
use rust_i18n::t;

use crate::error::{AppError, Result};
use crate::models::{ApiKey, NewApiKey};

/// Repository for API key database operations.
pub struct ApiKeyRepository;

impl ApiKeyRepository {
    /// Create a new API key in the database.
    pub async fn create(pool: &Pool<Sqlite>, new_key: NewApiKey) -> Result<ApiKey> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            r#"
            INSERT INTO api_keys (user_id, name, api_key)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(new_key.user_id)
        .bind(&new_key.name)
        .bind(&new_key.api_key)
        .execute(pool)
        .await;

        match result {
            Ok(res) => {
                let id = res.last_insert_rowid();
                Ok(ApiKey {
                    id,
                    user_id: new_key.user_id,
                    name: new_key.name,
                    api_key: new_key.api_key,
                    created_at: now.parse().unwrap(),
                })
            }
            Err(e) => {
                if e.to_string().contains("UNIQUE constraint failed") {
                    Err(AppError::BadRequest(
                        t!("auth.api_key_exists", key = new_key.api_key).to_string(),
                    ))
                } else {
                    Err(AppError::Database(e))
                }
            }
        }
    }

    /// Retrieve an API key by its key string.
    /// Returns `Ok(Some(ApiKey))` if found, `Ok(None)` if not found.
    /// Returns `Err` on database errors.
    pub async fn get_by_key(pool: &Pool<Sqlite>, key: &str) -> Result<Option<ApiKey>> {
        sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, user_id, name, api_key, created_at
            FROM api_keys
            WHERE api_key = ?
            "#,
        )
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
    }

    /// Delete an API key by its ID.
    pub async fn delete(pool: &Pool<Sqlite>, id: i64) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM api_keys
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    /// List all API keys by user ID.
    /// Returns a vector of ApiKey.
    /// Returns `Err` on database errors.
    pub async fn list_by_user(pool: &Pool<Sqlite>, user_id: i64) -> Result<Vec<ApiKey>> {
        sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, user_id, name, api_key, created_at
            FROM api_keys
            WHERE user_id = ?
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }
}
