//! Database schema and migrations

use super::DbResult;
use rusqlite::Connection;

const SCHEMA_VERSION: i32 = 1;

/// Run all pending migrations
pub fn migrate(conn: &Connection) -> DbResult<()> {
    // Check current version
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY)",
        [],
    )?;

    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current_version < SCHEMA_VERSION {
        migrate_v1(conn)?;
    }

    Ok(())
}

fn migrate_v1(conn: &Connection) -> DbResult<()> {
    conn.execute_batch(
        r#"
        -- Main bookmarks table
        CREATE TABLE IF NOT EXISTS bookmarks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL UNIQUE,
            title TEXT NOT NULL,
            excerpt TEXT,
            content TEXT,
            site_name TEXT,
            favicon_url TEXT,
            status TEXT NOT NULL DEFAULT 'unread',
            is_favorite INTEGER NOT NULL DEFAULT 0,
            reading_progress REAL NOT NULL DEFAULT 0.0,
            estimated_read_time INTEGER,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            read_at TEXT
        );

        -- Tags table
        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            color TEXT
        );

        -- Bookmark-tag junction table
        CREATE TABLE IF NOT EXISTS bookmark_tags (
            bookmark_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (bookmark_id, tag_id),
            FOREIGN KEY (bookmark_id) REFERENCES bookmarks(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );

        -- FTS5 virtual table for full-text search
        CREATE VIRTUAL TABLE IF NOT EXISTS bookmarks_fts USING fts5(
            title,
            excerpt,
            content,
            site_name,
            content='bookmarks',
            content_rowid='id'
        );

        -- Triggers to keep FTS index in sync
        CREATE TRIGGER IF NOT EXISTS bookmarks_ai AFTER INSERT ON bookmarks BEGIN
            INSERT INTO bookmarks_fts(rowid, title, excerpt, content, site_name)
            VALUES (new.id, new.title, new.excerpt, new.content, new.site_name);
        END;

        CREATE TRIGGER IF NOT EXISTS bookmarks_ad AFTER DELETE ON bookmarks BEGIN
            INSERT INTO bookmarks_fts(bookmarks_fts, rowid, title, excerpt, content, site_name)
            VALUES ('delete', old.id, old.title, old.excerpt, old.content, old.site_name);
        END;

        CREATE TRIGGER IF NOT EXISTS bookmarks_au AFTER UPDATE ON bookmarks BEGIN
            INSERT INTO bookmarks_fts(bookmarks_fts, rowid, title, excerpt, content, site_name)
            VALUES ('delete', old.id, old.title, old.excerpt, old.content, old.site_name);
            INSERT INTO bookmarks_fts(rowid, title, excerpt, content, site_name)
            VALUES (new.id, new.title, new.excerpt, new.content, new.site_name);
        END;

        -- Indexes for common queries
        CREATE INDEX IF NOT EXISTS idx_bookmarks_status ON bookmarks(status);
        CREATE INDEX IF NOT EXISTS idx_bookmarks_is_favorite ON bookmarks(is_favorite);
        CREATE INDEX IF NOT EXISTS idx_bookmarks_created_at ON bookmarks(created_at);
        CREATE INDEX IF NOT EXISTS idx_bookmark_tags_tag_id ON bookmark_tags(tag_id);

        -- Record migration version
        INSERT INTO schema_version (version) VALUES (1);
        "#,
    )?;

    Ok(())
}
