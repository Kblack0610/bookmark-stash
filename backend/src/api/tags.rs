//! Tags API endpoints

use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};

use stash_shared::{ApiError, Tag};

use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(list_tags))
}

/// GET /api/tags - List all tags with counts
async fn list_tags(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Tag>>, (StatusCode, Json<ApiError>)> {
    let db = state.db.lock().await;

    match db.list_tags() {
        Ok(tags) => Ok(Json(tags)),
        Err(e) => {
            tracing::error!("Failed to list tags: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", e.to_string())),
            ))
        }
    }
}
