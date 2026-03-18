//! Import endpoints for local bookmark archives.

use std::path::PathBuf;
use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::Json, routing::post, Router};

use stash_shared::{ApiError, FirefoxImportRequest, FirefoxImportResponse};

use crate::{bookmark_import, AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/firefox", post(import_firefox))
}

async fn import_firefox(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FirefoxImportRequest>,
) -> Result<Json<FirefoxImportResponse>, (StatusCode, Json<ApiError>)> {
    let db = state.db.lock().await;
    let default_tags = req.default_tags.unwrap_or_default();

    let summary =
        bookmark_import::import_firefox_bookmarks(&db, &PathBuf::from(&req.path), &default_tags)
            .map_err(|error| {
                tracing::error!("Firefox import failed: {}", error);
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError::new("import_failed", error.to_string())),
                )
            })?;

    Ok(Json(FirefoxImportResponse {
        path: summary.path,
        total_discovered: summary.total_discovered as i32,
        imported: summary.imported as i32,
        skipped: summary.skipped as i32,
        invalid: summary.invalid as i32,
    }))
}
