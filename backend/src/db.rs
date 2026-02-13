//! Database initialization and migration logic.
//!
//! This module handles SQLite database setup, including creating tables
//! and running migrations when the application starts.

use crate::error::{AppError, Result};
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};

/// Database configuration options.
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// Path to the SQLite database file.
    pub database_url: String,
    /// Maximum number of connections in the pool.
    pub max_connections: u32,
}

/// Initialize the database connection pool and run migrations.
///
/// This function creates the database file if it doesn't exist,
/// establishes a connection pool, and creates all required tables.
pub async fn init_db(config: &DbConfig) -> Result<Pool<Sqlite>> {
    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.database_url)
        .await
        .map_err(AppError::Database)?;

    // Run migrations to create tables
    run_migrations(&pool).await?;

    Ok(pool)
}

// Run database migrations to create tables
async fn run_migrations(pool: &Pool<Sqlite>) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| AppError::Database(e.into()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_db_creates_tables() {
        let config = DbConfig {
            database_url: "sqlite::memory:".to_string(),
            max_connections: 1,
        };

        let pool = init_db(&config)
            .await
            .expect("Failed to initialize database");

        // Verify tables exist by querying sqlite_master
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
        )
        .fetch_all(&pool)
        .await
        .expect("Failed to query tables");

        let table_names: Vec<&str> = tables.iter().map(|t| t.0.as_str()).collect();

        assert!(
            table_names.contains(&"libraries"),
            "libraries table should exist"
        );
        assert!(
            table_names.contains(&"scan_paths"),
            "scan_paths table should exist"
        );
        assert!(
            table_names.contains(&"contents"),
            "contents table should exist"
        );
        assert!(
            table_names.contains(&"chapters"),
            "chapters table should exist"
        );
        assert!(table_names.contains(&"users"), "users table should exist");
        assert!(
            table_names.contains(&"reading_progress"),
            "reading_progress table should exist"
        );
    }
}
