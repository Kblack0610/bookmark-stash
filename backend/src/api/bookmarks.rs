//! Bookmark API endpoints

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, patch, post},
    Router,
};
use serde_json::json;

use stash_shared::{
    ApiError, Bookmark, BookmarkListResponse, CreateBookmarkRequest, ListBookmarksQuery,
    UpdateBookmarkRequest,
};

use crate::{content, AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_bookmark))
        .route("/", get(list_bookmarks))
        .route("/{id}", get(get_bookmark))
        .route("/{id}", patch(update_bookmark))
        .route("/{id}", delete(delete_bookmark))
        .route("/{id}/content", get(get_bookmark_content))
}

/// POST /api/bookmarks - Save a new URL
async fn create_bookmark(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateBookmarkRequest>,
) -> Result<(StatusCode, Json<Bookmark>), (StatusCode, Json<ApiError>)> {
    // Validate URL
    if stash_shared::validate_url(&req.url).is_err() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("invalid_url", "Invalid URL provided")),
        ));
    }

    // Fetch and extract content
    let fetcher = content::Fetcher::new();
    let (title, excerpt, content, site_name, favicon_url, estimated_read_time) =
        match fetcher.fetch(&req.url).await {
            Ok(page) => {
                let extracted = content::extract_content(&page.html, &page.final_url);
                (
                    extracted.title,
                    extracted.excerpt,
                    extracted.content,
                    extracted.site_name,
                    extracted.favicon_url,
                    extracted.estimated_read_time,
                )
            }
            Err(_) => {
                // Use URL as title if fetch fails
                (req.url.clone(), None, None, None, None, None)
            }
        };

    // Save to database
    let db = state.db.lock().await;
    match db.create_bookmark(
        &req,
        &title,
        excerpt.as_deref(),
        content.as_deref(),
        site_name.as_deref(),
        favicon_url.as_deref(),
        estimated_read_time,
    ) {
        Ok(bookmark) => Ok((StatusCode::CREATED, Json(bookmark))),
        Err(e) => {
            tracing::error!("Failed to create bookmark: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", e.to_string())),
            ))
        }
    }
}

/// GET /api/bookmarks - List bookmarks with filters
async fn list_bookmarks(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListBookmarksQuery>,
) -> Result<Json<BookmarkListResponse>, (StatusCode, Json<ApiError>)> {
    let db = state.db.lock().await;
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    match db.list_bookmarks(&query) {
        Ok((bookmarks, total)) => Ok(Json(BookmarkListResponse {
            bookmarks,
            total,
            limit,
            offset,
        })),
        Err(e) => {
            tracing::error!("Failed to list bookmarks: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", e.to_string())),
            ))
        }
    }
}

/// GET /api/bookmarks/:id - Get a single bookmark
async fn get_bookmark(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Bookmark>, (StatusCode, Json<ApiError>)> {
    let db = state.db.lock().await;

    match db.get_bookmark(id) {
        Ok(bookmark) => Ok(Json(bookmark)),
        Err(crate::db::DbError::NotFound(_)) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new(
                "not_found",
                format!("Bookmark {} not found", id),
            )),
        )),
        Err(e) => {
            tracing::error!("Failed to get bookmark: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", e.to_string())),
            ))
        }
    }
}

/// GET /api/bookmarks/:id/content - Get reader-formatted content
async fn get_bookmark_content(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let db = state.db.lock().await;

    match db.get_bookmark_content(id) {
        Ok(content) => Ok(Json(json!({ "content": content }))),
        Err(crate::db::DbError::NotFound(_)) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new(
                "not_found",
                format!("Bookmark {} not found", id),
            )),
        )),
        Err(e) => {
            tracing::error!("Failed to get bookmark content: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", e.to_string())),
            ))
        }
    }
}

/// PATCH /api/bookmarks/:id - Update a bookmark
async fn update_bookmark(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateBookmarkRequest>,
) -> Result<Json<Bookmark>, (StatusCode, Json<ApiError>)> {
    let db = state.db.lock().await;

    match db.update_bookmark(id, &req) {
        Ok(bookmark) => Ok(Json(bookmark)),
        Err(crate::db::DbError::NotFound(_)) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new(
                "not_found",
                format!("Bookmark {} not found", id),
            )),
        )),
        Err(e) => {
            tracing::error!("Failed to update bookmark: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", e.to_string())),
            ))
        }
    }
}

/// DELETE /api/bookmarks/:id - Delete a bookmark
async fn delete_bookmark(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    let db = state.db.lock().await;

    match db.delete_bookmark(id) {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(crate::db::DbError::NotFound(_)) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new(
                "not_found",
                format!("Bookmark {} not found", id),
            )),
        )),
        Err(e) => {
            tracing::error!("Failed to delete bookmark: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", e.to_string())),
            ))
        }
    }
}
