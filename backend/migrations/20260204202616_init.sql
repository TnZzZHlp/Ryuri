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

-- Contents table
CREATE TABLE IF NOT EXISTS contents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id INTEGER NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    scan_path_id INTEGER NOT NULL REFERENCES scan_paths(id) ON DELETE CASCADE,
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
    file_type TEXT NOT NULL DEFAULT '',
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
