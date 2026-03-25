//! Shared types for Stash - browser-first read-it-later system
//!
//! This crate contains all types shared between the backend API and browser extension.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

/// A saved bookmark/article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub excerpt: Option<String>,
    pub content: Option<String>,
    pub site_name: Option<String>,
    pub favicon_url: Option<String>,
    pub status: BookmarkStatus,
    pub is_favorite: bool,
    pub reading_progress: f32,
    pub estimated_read_time: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
    pub tags: Vec<Tag>,
    pub folder_id: Option<i64>,
}

/// Bookmark status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BookmarkStatus {
    #[default]
    Unread,
    Reading,
    Archived,
}

impl BookmarkStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unread => "unread",
            Self::Reading => "reading",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for BookmarkStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for BookmarkStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "unread" => Ok(Self::Unread),
            "reading" => Ok(Self::Reading),
            "archived" => Ok(Self::Archived),
            _ => Err(format!("Invalid status: {}", s)),
        }
    }
}

/// A tag for organizing bookmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
    pub bookmark_count: i32,
}

/// Request to create a new bookmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBookmarkRequest {
    pub url: String,
    pub tags: Option<Vec<String>>,
}

/// Request to update a bookmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBookmarkRequest {
    pub title: Option<String>,
    pub status: Option<BookmarkStatus>,
    pub is_favorite: Option<bool>,
    pub reading_progress: Option<f32>,
    pub tags: Option<Vec<String>>,
    pub folder_id: Option<Option<i64>>,
}

/// Query parameters for listing bookmarks
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListBookmarksQuery {
    pub status: Option<BookmarkStatus>,
    pub is_favorite: Option<bool>,
    pub tag: Option<String>,
    pub folder_id: Option<i64>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// Search query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// Bookmark list response with pagination info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkListResponse {
    pub bookmarks: Vec<Bookmark>,
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
}

/// Search result with highlights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub bookmark: Bookmark,
    pub highlight: Option<String>,
    pub score: f64,
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: i64,
    pub query: String,
}

/// Reading statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingStats {
    pub total_bookmarks: i64,
    pub unread_count: i64,
    pub archived_count: i64,
    pub favorites_count: i64,
    pub total_read_time_minutes: i64,
    pub bookmarks_this_week: i64,
    pub bookmarks_this_month: i64,
}

/// Request to import bookmarks from a Firefox/Netscape HTML export file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirefoxImportRequest {
    pub path: String,
    pub default_tags: Option<Vec<String>>,
}

/// Summary returned after importing a bookmark export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirefoxImportResponse {
    pub path: String,
    pub total_discovered: i32,
    pub imported: i32,
    pub skipped: i32,
    pub invalid: i32,
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ApiError {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

/// Generic API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Success(T),
    Error(ApiError),
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self::Success(data)
    }

    pub fn error(error: ApiError) -> Self {
        Self::Error(error)
    }
}

/// A folder for organizing bookmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub bookmark_count: i32,
    pub subfolder_count: i32,
}

/// Request to create a folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFolderRequest {
    pub name: String,
    pub parent_id: Option<i64>,
}

/// Request to update a folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFolderRequest {
    pub name: Option<String>,
    /// None = don't change, Some(None) = move to root, Some(Some(id)) = reparent
    pub parent_id: Option<Option<i64>>,
    pub position: Option<i32>,
}

/// Validate a URL string
pub fn validate_url(url_str: &str) -> Result<Url, String> {
    Url::parse(url_str).map_err(|e| format!("Invalid URL: {}", e))
}
