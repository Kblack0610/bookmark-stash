//! Database module - SQLite with FTS5 full-text search

mod queries;
mod schema;

use rusqlite::Connection;
use thiserror::Error;

pub use queries::ImportOutcome;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

pub type DbResult<T> = Result<T, DbError>;

/// Database wrapper
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create database at the given path
    pub fn open(path: &str) -> DbResult<Self> {
        let conn = Connection::open(path)?;

        // Enable foreign keys and WAL mode
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;",
        )?;

        Ok(Self { conn })
    }

    /// Run migrations
    pub fn migrate(&self) -> DbResult<()> {
        schema::migrate(&self.conn)
    }

    /// Get a reference to the connection
    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}
