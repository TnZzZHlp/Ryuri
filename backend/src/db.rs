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

/// Run database migrations to create required tables.
async fn run_migrations(pool: &Pool<Sqlite>) -> Result<()> {
    // Create libraries table
    sqlx::query(SCHEMA_SQL)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

    // Migration: Add page_count column to chapters table if it doesn't exist
    sqlx::query("ALTER TABLE chapters ADD COLUMN page_count INTEGER NOT NULL DEFAULT 0")
        .execute(pool)
        .await
        .ok(); // Ignore error if column already exists

    // Migration: Add name column to api_keys table if it doesn't exist
    sqlx::query("ALTER TABLE api_keys ADD COLUMN name TEXT NOT NULL DEFAULT 'My API Key'")
        .execute(pool)
        .await
        .ok(); // Ignore error if column already exists

    // Migration: Add size column to chapters table if it doesn't exist
    sqlx::query("ALTER TABLE chapters ADD COLUMN size INTEGER NOT NULL DEFAULT 0")
        .execute(pool)
        .await
        .ok(); // Ignore error if column already exists

    Ok(())
}

/// SQL schema for creating all database tables.
const SCHEMA_SQL: &str = r#"
-- Libraries table - content collections
CREATE TABLE IF NOT EXISTS libraries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    scan_interval INTEGER NOT NULL DEFAULT 0,
    watch_mode INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Scan paths table - directories associated with libraries
CREATE TABLE IF NOT EXISTS scan_paths (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id INTEGER NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(library_id, path)
);

-- Contents table - comics and novels
CREATE TABLE IF NOT EXISTS contents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id INTEGER NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    scan_path_id INTEGER NOT NULL REFERENCES scan_paths(id) ON DELETE CASCADE,
    content_type TEXT NOT NULL CHECK (content_type IN ('Comic', 'Novel')),
    title TEXT NOT NULL,
    folder_path TEXT NOT NULL,
    chapter_count INTEGER NOT NULL DEFAULT 0,
    thumbnail BLOB,
    metadata BLOB,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(library_id, folder_path)
);

-- Chapters table - individual chapters within content
CREATE TABLE IF NOT EXISTS chapters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_id INTEGER NOT NULL REFERENCES contents(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    file_path TEXT NOT NULL,
    sort_order INTEGER NOT NULL,
    page_count INTEGER NOT NULL DEFAULT 0,
    size INTEGER NOT NULL DEFAULT 0,
    UNIQUE(content_id, file_path)
);

-- Users table - user accounts
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    bangumi_api_key TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Reading progress table - tracks user reading positions per chapter
CREATE TABLE IF NOT EXISTS reading_progress (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    chapter_id INTEGER NOT NULL REFERENCES chapters(id) ON DELETE CASCADE,
    position INTEGER NOT NULL DEFAULT 0,
    percentage REAL NOT NULL DEFAULT 0.0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id, chapter_id)
);

-- API key tables
CREATE TABLE IF NOT EXISTS api_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL DEFAULT 'My API Key',
    api_key TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);


-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_scan_paths_library ON scan_paths(library_id);
CREATE INDEX IF NOT EXISTS idx_contents_library ON contents(library_id);
CREATE INDEX IF NOT EXISTS idx_contents_scan_path ON contents(scan_path_id);
CREATE INDEX IF NOT EXISTS idx_contents_title ON contents(title);
CREATE INDEX IF NOT EXISTS idx_chapters_content ON chapters(content_id);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_reading_progress_user ON reading_progress(user_id);
CREATE INDEX IF NOT EXISTS idx_reading_progress_chapter ON reading_progress(chapter_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_user ON api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_api_key ON api_keys(api_key);

INSERT INTO users (username, password_hash) VALUES ('admin', '$argon2id$v=19$m=16,t=2,p=1$dGVzdHRlc3Q$e1JfAUgszO1txSZmW/Eu7w') ON CONFLICT DO NOTHING;
"#;

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
