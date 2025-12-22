//! User repository for database operations.
//!
//! This module provides database access for user-related operations.

use chrono::Utc;
use sqlx::{Pool, Sqlite};
use rust_i18n::t;

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
                    Err(AppError::BadRequest(
                        t!("auth.username_exists_msg", username = new_user.username).to_string(),
                    ))
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

    /// Update a user's information.
    /// Fields set to None will not be updated.
    pub async fn update(
        pool: &Pool<Sqlite>,
        user_id: i64,
        username: Option<String>,
        password_hash: Option<String>,
        bangumi_api_key: Option<Option<String>>,
    ) -> Result<User> {
        use sqlx::Arguments;
        let mut query = "UPDATE users SET updated_at = ?".to_string();
        let mut args = sqlx::sqlite::SqliteArguments::default();
        let _ = args.add(Utc::now().to_rfc3339());

        if let Some(u) = username {
            query.push_str(", username = ?");
            let _ = args.add(u);
        }

        if let Some(p) = password_hash {
            query.push_str(", password_hash = ?");
            let _ = args.add(p);
        }

        if let Some(k_opt) = bangumi_api_key {
            query.push_str(", bangumi_api_key = ?");
            let _ = args.add(k_opt);
        }

        query.push_str(" WHERE id = ?");
        let _ = args.add(user_id);

        let result = sqlx::query_with(&query, args)
            .execute(pool)
            .await
            .map_err(|e| {
                if e.to_string().contains("UNIQUE constraint failed") {
                    AppError::BadRequest(t!("auth.username_exists").to_string())
                } else {
                    AppError::Database(e)
                }
            })?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(
                t!("auth.user_not_found", id = user_id).to_string(),
            ));
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
