//! Database query functions

use chrono::{DateTime, Utc};
use rusqlite::{params, Row};
use url::Url;

use stash_shared::{
    Bookmark, BookmarkStatus, CreateBookmarkRequest, ListBookmarksQuery, SearchResult, Tag,
    UpdateBookmarkRequest,
};

use super::{Database, DbError, DbResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportOutcome {
    Imported,
    SkippedDuplicate,
}

impl Database {
    /// Create a new bookmark
    pub fn create_bookmark(
        &self,
        req: &CreateBookmarkRequest,
        title: &str,
        excerpt: Option<&str>,
        content: Option<&str>,
        site_name: Option<&str>,
        favicon_url: Option<&str>,
        estimated_read_time: Option<i32>,
    ) -> DbResult<Bookmark> {
        let conn = self.conn();

        conn.execute(
            r#"INSERT INTO bookmarks (url, title, excerpt, content, site_name, favicon_url, estimated_read_time)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            params![req.url, title, excerpt, content, site_name, favicon_url, estimated_read_time],
        )?;

        let id = conn.last_insert_rowid();

        // Add tags if provided
        if let Some(tag_names) = &req.tags {
            for tag_name in tag_names {
                self.add_tag_to_bookmark(id, tag_name)?;
            }
        }

        self.get_bookmark(id)
    }

    /// Get a bookmark by ID
    pub fn get_bookmark(&self, id: i64) -> DbResult<Bookmark> {
        let conn = self.conn();

        let bookmark = conn
            .query_row("SELECT * FROM bookmarks WHERE id = ?1", [id], |row| {
                Self::row_to_bookmark(row)
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    DbError::NotFound(format!("Bookmark {}", id))
                }
                _ => DbError::Sqlite(e),
            })?;

        let tags = self.get_bookmark_tags(id)?;

        Ok(Bookmark { tags, ..bookmark })
    }

    /// Get a bookmark by URL
    pub fn get_bookmark_by_url(&self, url: &str) -> DbResult<Bookmark> {
        let conn = self.conn();

        let bookmark = conn
            .query_row("SELECT * FROM bookmarks WHERE url = ?1", [url], |row| {
                Self::row_to_bookmark(row)
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    DbError::NotFound(format!("Bookmark with URL {}", url))
                }
                _ => DbError::Sqlite(e),
            })?;

        let tags = self.get_bookmark_tags(bookmark.id)?;

        Ok(Bookmark { tags, ..bookmark })
    }

    /// Get bookmark content (for reader view)
    pub fn get_bookmark_content(&self, id: i64) -> DbResult<Option<String>> {
        let conn = self.conn();

        conn.query_row("SELECT content FROM bookmarks WHERE id = ?1", [id], |row| {
            row.get(0)
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DbError::NotFound(format!("Bookmark {}", id)),
            _ => DbError::Sqlite(e),
        })
    }

    /// List bookmarks with filters
    pub fn list_bookmarks(&self, query: &ListBookmarksQuery) -> DbResult<(Vec<Bookmark>, i64)> {
        let conn = self.conn();
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        let mut sql = String::from("SELECT * FROM bookmarks WHERE 1=1");
        let mut count_sql = String::from("SELECT COUNT(*) FROM bookmarks WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(status) = &query.status {
            sql.push_str(" AND status = ?");
            count_sql.push_str(" AND status = ?");
            params_vec.push(Box::new(status.as_str().to_string()));
        }

        if let Some(is_favorite) = query.is_favorite {
            sql.push_str(" AND is_favorite = ?");
            count_sql.push_str(" AND is_favorite = ?");
            params_vec.push(Box::new(is_favorite as i32));
        }

        if let Some(tag) = &query.tag {
            sql.push_str(" AND id IN (SELECT bookmark_id FROM bookmark_tags bt JOIN tags t ON bt.tag_id = t.id WHERE t.name = ?)");
            count_sql.push_str(" AND id IN (SELECT bookmark_id FROM bookmark_tags bt JOIN tags t ON bt.tag_id = t.id WHERE t.name = ?)");
            params_vec.push(Box::new(tag.clone()));
        }

        sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

        // Get total count first
        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();
        let total: i64 = conn.query_row(&count_sql, params_refs.as_slice(), |row| row.get(0))?;

        // Add limit/offset params
        params_vec.push(Box::new(limit));
        params_vec.push(Box::new(offset));

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;
        let bookmark_iter =
            stmt.query_map(params_refs.as_slice(), |row| Self::row_to_bookmark(row))?;

        let mut bookmarks = Vec::new();
        for bookmark in bookmark_iter {
            let mut b = bookmark?;
            b.tags = self.get_bookmark_tags(b.id)?;
            bookmarks.push(b);
        }

        Ok((bookmarks, total))
    }

    /// Update a bookmark
    pub fn update_bookmark(&self, id: i64, req: &UpdateBookmarkRequest) -> DbResult<Bookmark> {
        let conn = self.conn();

        // Build dynamic update query
        let mut updates = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(title) = &req.title {
            updates.push("title = ?");
            params_vec.push(Box::new(title.clone()));
        }

        if let Some(status) = &req.status {
            updates.push("status = ?");
            params_vec.push(Box::new(status.as_str().to_string()));

            if *status == BookmarkStatus::Archived {
                updates.push("read_at = datetime('now')");
            }
        }

        if let Some(is_favorite) = req.is_favorite {
            updates.push("is_favorite = ?");
            params_vec.push(Box::new(is_favorite as i32));
        }

        if let Some(progress) = req.reading_progress {
            updates.push("reading_progress = ?");
            params_vec.push(Box::new(progress as f64));
        }

        if !updates.is_empty() {
            updates.push("updated_at = datetime('now')");

            let sql = format!("UPDATE bookmarks SET {} WHERE id = ?", updates.join(", "));
            params_vec.push(Box::new(id));

            let params_refs: Vec<&dyn rusqlite::ToSql> =
                params_vec.iter().map(|p| p.as_ref()).collect();
            conn.execute(&sql, params_refs.as_slice())?;
        }

        // Update tags if provided
        if let Some(tag_names) = &req.tags {
            // Remove existing tags
            conn.execute("DELETE FROM bookmark_tags WHERE bookmark_id = ?", [id])?;

            // Add new tags
            for tag_name in tag_names {
                self.add_tag_to_bookmark(id, tag_name)?;
            }
        }

        self.get_bookmark(id)
    }

    /// Delete a bookmark
    pub fn delete_bookmark(&self, id: i64) -> DbResult<()> {
        let conn = self.conn();
        let rows = conn.execute("DELETE FROM bookmarks WHERE id = ?", [id])?;

        if rows == 0 {
            return Err(DbError::NotFound(format!("Bookmark {}", id)));
        }

        Ok(())
    }

    /// Import a bookmark without fetching remote content.
    pub fn import_bookmark(
        &self,
        url: &str,
        title: &str,
        favicon_url: Option<&str>,
        tags: &[String],
    ) -> DbResult<ImportOutcome> {
        if let Ok(existing) = self.get_bookmark_by_url(url) {
            for tag in tags {
                self.add_tag_to_bookmark(existing.id, tag)?;
            }
            return Ok(ImportOutcome::SkippedDuplicate);
        }

        let conn = self.conn();
        let site_name = Url::parse(url)
            .ok()
            .and_then(|parsed| parsed.host_str().map(ToString::to_string));

        conn.execute(
            r#"INSERT INTO bookmarks (url, title, site_name, favicon_url)
               VALUES (?1, ?2, ?3, ?4)"#,
            params![url, title, site_name, favicon_url],
        )?;

        let id = conn.last_insert_rowid();
        for tag in tags {
            self.add_tag_to_bookmark(id, tag)?;
        }

        Ok(ImportOutcome::Imported)
    }

    /// Full-text search
    pub fn search(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> DbResult<(Vec<SearchResult>, i64)> {
        let conn = self.conn();

        // Get total count
        let total: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM bookmarks_fts WHERE bookmarks_fts MATCH ?",
                [query],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Get results with ranking
        let mut stmt = conn.prepare(
            r#"SELECT b.*, bm25(bookmarks_fts) as score,
                      snippet(bookmarks_fts, 0, '<mark>', '</mark>', '...', 32) as highlight
               FROM bookmarks_fts
               JOIN bookmarks b ON bookmarks_fts.rowid = b.id
               WHERE bookmarks_fts MATCH ?
               ORDER BY score
               LIMIT ? OFFSET ?"#,
        )?;

        let results = stmt.query_map(params![query, limit, offset], |row| {
            let bookmark = Self::row_to_bookmark(row)?;
            let score: f64 = row.get("score")?;
            let highlight: Option<String> = row.get("highlight")?;
            Ok((bookmark, score, highlight))
        })?;

        let mut search_results = Vec::new();
        for result in results {
            let (mut bookmark, score, highlight) = result?;
            bookmark.tags = self.get_bookmark_tags(bookmark.id)?;
            search_results.push(SearchResult {
                bookmark,
                highlight,
                score: -score, // bm25 returns negative values, lower is better
            });
        }

        Ok((search_results, total))
    }

    /// Get all tags
    pub fn list_tags(&self) -> DbResult<Vec<Tag>> {
        let conn = self.conn();

        let mut stmt = conn.prepare(
            r#"SELECT t.id, t.name, t.color, COUNT(bt.bookmark_id) as bookmark_count
               FROM tags t
               LEFT JOIN bookmark_tags bt ON t.id = bt.tag_id
               GROUP BY t.id
               ORDER BY bookmark_count DESC, t.name"#,
        )?;

        let tags = stmt.query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                bookmark_count: row.get(3)?,
            })
        })?;

        tags.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }

    /// Get reading statistics
    pub fn get_stats(&self) -> DbResult<stash_shared::ReadingStats> {
        let conn = self.conn();

        let total_bookmarks: i64 =
            conn.query_row("SELECT COUNT(*) FROM bookmarks", [], |row| row.get(0))?;

        let unread_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM bookmarks WHERE status = 'unread'",
            [],
            |row| row.get(0),
        )?;

        let archived_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM bookmarks WHERE status = 'archived'",
            [],
            |row| row.get(0),
        )?;

        let favorites_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM bookmarks WHERE is_favorite = 1",
            [],
            |row| row.get(0),
        )?;

        let total_read_time_minutes: i64 = conn.query_row(
            "SELECT COALESCE(SUM(estimated_read_time), 0) FROM bookmarks WHERE status = 'archived'",
            [],
            |row| row.get(0),
        )?;

        let bookmarks_this_week: i64 = conn.query_row(
            "SELECT COUNT(*) FROM bookmarks WHERE created_at > datetime('now', '-7 days')",
            [],
            |row| row.get(0),
        )?;

        let bookmarks_this_month: i64 = conn.query_row(
            "SELECT COUNT(*) FROM bookmarks WHERE created_at > datetime('now', '-30 days')",
            [],
            |row| row.get(0),
        )?;

        Ok(stash_shared::ReadingStats {
            total_bookmarks,
            unread_count,
            archived_count,
            favorites_count,
            total_read_time_minutes,
            bookmarks_this_week,
            bookmarks_this_month,
        })
    }

    // Helper functions

    fn row_to_bookmark(row: &Row) -> rusqlite::Result<Bookmark> {
        let status_str: String = row.get("status")?;
        let status = status_str.parse().unwrap_or_default();

        let created_at_str: String = row.get("created_at")?;
        let updated_at_str: String = row.get("updated_at")?;
        let read_at_str: Option<String> = row.get("read_at")?;

        Ok(Bookmark {
            id: row.get("id")?,
            url: row.get("url")?,
            title: row.get("title")?,
            excerpt: row.get("excerpt")?,
            content: row.get("content")?,
            site_name: row.get("site_name")?,
            favicon_url: row.get("favicon_url")?,
            status,
            is_favorite: row.get::<_, i32>("is_favorite")? != 0,
            reading_progress: row.get("reading_progress")?,
            estimated_read_time: row.get("estimated_read_time")?,
            created_at: DateTime::parse_from_rfc3339(&format!("{}Z", created_at_str))
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&format!("{}Z", updated_at_str))
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            read_at: read_at_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&format!("{}Z", s))
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            }),
            tags: Vec::new(), // Filled in by caller
        })
    }

    fn get_bookmark_tags(&self, bookmark_id: i64) -> DbResult<Vec<Tag>> {
        let conn = self.conn();

        let mut stmt = conn.prepare(
            r#"SELECT t.id, t.name, t.color,
                      (SELECT COUNT(*) FROM bookmark_tags WHERE tag_id = t.id) as bookmark_count
               FROM tags t
               JOIN bookmark_tags bt ON t.id = bt.tag_id
               WHERE bt.bookmark_id = ?"#,
        )?;

        let tags = stmt.query_map([bookmark_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                bookmark_count: row.get(3)?,
            })
        })?;

        tags.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }

    fn add_tag_to_bookmark(&self, bookmark_id: i64, tag_name: &str) -> DbResult<()> {
        let conn = self.conn();

        // Insert tag if it doesn't exist
        conn.execute("INSERT OR IGNORE INTO tags (name) VALUES (?)", [tag_name])?;

        // Get tag ID
        let tag_id: i64 =
            conn.query_row("SELECT id FROM tags WHERE name = ?", [tag_name], |row| {
                row.get(0)
            })?;

        // Link bookmark to tag
        conn.execute(
            "INSERT OR IGNORE INTO bookmark_tags (bookmark_id, tag_id) VALUES (?, ?)",
            params![bookmark_id, tag_id],
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::bookmark_import::import_firefox_bookmarks;

    use super::*;

    fn temp_db_path(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("stash-{name}-{suffix}.db"))
    }

    #[test]
    fn create_list_search_and_import_flow() {
        let db_path = temp_db_path("flow");
        let import_path = db_path.with_extension("html");

        let db = Database::open(db_path.to_str().unwrap()).unwrap();
        db.migrate().unwrap();

        let req = CreateBookmarkRequest {
            url: "https://example.com/articles/alpha".to_string(),
            tags: Some(vec!["rust".to_string()]),
        };

        let created = db
            .create_bookmark(
                &req,
                "Alpha Article",
                Some("Alpha excerpt"),
                Some("<article>Rust alpha</article>"),
                Some("example.com"),
                None,
                Some(5),
            )
            .unwrap();

        let (listed, total) = db.list_bookmarks(&ListBookmarksQuery::default()).unwrap();
        assert_eq!(created.title, "Alpha Article");
        assert_eq!(total, 1);
        assert_eq!(listed.len(), 1);

        let (results, total) = db.search("Rust", 20, 0).unwrap();
        assert_eq!(total, 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].bookmark.url, req.url);

        fs::write(
            &import_path,
            r#"
<!DOCTYPE NETSCAPE-Bookmark-file-1>
<DL><p>
  <DT><H3>Imported</H3>
  <DL><p>
    <DT><A HREF="https://example.com/articles/alpha">Existing Alpha</A>
    <DT><A HREF="https://doc.rust-lang.org/book/">The Rust Book</A>
  </DL><p>
</DL><p>
"#,
        )
        .unwrap();

        let summary =
            import_firefox_bookmarks(&db, &import_path, &[String::from("archive-2026")]).unwrap();

        assert_eq!(summary.total_discovered, 2);
        assert_eq!(summary.imported, 1);
        assert_eq!(summary.skipped, 1);
        assert_eq!(summary.invalid, 0);

        let imported = db
            .get_bookmark_by_url("https://doc.rust-lang.org/book/")
            .unwrap();
        assert_eq!(imported.title, "The Rust Book");
        assert!(imported.tags.iter().any(|tag| tag.name == "archive-2026"));
        assert!(imported.tags.iter().any(|tag| tag.name == "Imported"));
        assert!(imported.tags.iter().any(|tag| tag.name == "firefox-import"));

        let _ = fs::remove_file(db_path);
        let _ = fs::remove_file(import_path);
    }
}
