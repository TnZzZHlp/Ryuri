//! User repository for database operations.
//!
//! This module provides database access for user-related operations.

use chrono::Utc;
use sqlx::{Pool, Sqlite};

use crate::error::{AppError, Result};
use crate::models::{NewUser, User};

/// Repository for user database operations.
pub struct UserRepository;

impl UserRepository {
    /// Create a new user in the database.
    pub async fn create(pool: &Pool<Sqlite>, new_user: NewUser) -> Result<User> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            r#"
            INSERT INTO users (username, password_hash, bangumi_api_key, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&new_user.username)
        .bind(&new_user.password_hash)
        .bind(&new_user.bangumi_api_key)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await;

        match result {
            Ok(res) => {
                let id = res.last_insert_rowid();
                Self::find_by_id(pool, id).await?.ok_or_else(|| {
                    AppError::Internal("Failed to retrieve created user".to_string())
                })
            }
            Err(e) => {
                // Check for unique constraint violation
                if e.to_string().contains("UNIQUE constraint failed") {
                    Err(AppError::BadRequest(format!(
                        "Username '{}' already exists",
                        new_user.username
                    )))
                } else {
                    Err(AppError::Database(e))
                }
            }
        }
    }

    /// Find a user by ID.
    pub async fn find_by_id(pool: &Pool<Sqlite>, id: i64) -> Result<Option<User>> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, password_hash, bangumi_api_key, created_at, updated_at
            FROM users
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
    }

    /// Find a user by username.
    pub async fn find_by_username(pool: &Pool<Sqlite>, username: &str) -> Result<Option<User>> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, password_hash, bangumi_api_key, created_at, updated_at
            FROM users
            WHERE username = ?
            "#,
        )
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
    }

    /// Update a user's password hash.
    pub async fn update_password(
        pool: &Pool<Sqlite>,
        user_id: i64,
        password_hash: &str,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            r#"
            UPDATE users
            SET password_hash = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(password_hash)
        .bind(&now)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "User with id {} not found",
                user_id
            )));
        }

        Ok(())
    }

    /// Update a user's information (e.g., bangumi_api_key).
    pub async fn update(
        pool: &Pool<Sqlite>,
        user_id: i64,
        bangumi_api_key: Option<String>,
    ) -> Result<User> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            r#"
            UPDATE users
            SET bangumi_api_key = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&bangumi_api_key)
        .bind(&now)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "User with id {} not found",
                user_id
            )));
        }

        Self::find_by_id(pool, user_id)
            .await?
            .ok_or_else(|| AppError::Internal("Failed to retrieve updated user".to_string()))
    }

    /// Check if a username already exists.
    pub async fn username_exists(pool: &Pool<Sqlite>, username: &str) -> Result<bool> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM users WHERE username = ?
            "#,
        )
        .bind(username)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(result.0 > 0)
    }
}
