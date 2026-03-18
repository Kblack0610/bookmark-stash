//! REST API endpoints

mod bookmarks;
mod import;
mod search;
mod tags;

use axum::{routing::get, Router};
use std::sync::Arc;

use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health))
        .nest("/bookmarks", bookmarks::router())
        .nest("/import", import::router())
        .nest("/search", search::router())
        .nest("/tags", tags::router())
        .route("/stats", get(stats))
}

async fn health() -> &'static str {
    "OK"
}

async fn stats(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> axum::response::Json<stash_shared::ReadingStats> {
    let db = state.db.lock().await;
    let stats = db
        .get_stats()
        .unwrap_or_else(|_| stash_shared::ReadingStats {
            total_bookmarks: 0,
            unread_count: 0,
            archived_count: 0,
            favorites_count: 0,
            total_read_time_minutes: 0,
            bookmarks_this_week: 0,
            bookmarks_this_month: 0,
        });
    axum::response::Json(stats)
}
