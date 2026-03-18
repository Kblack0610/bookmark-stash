//! Search API endpoints

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};

use stash_shared::{ApiError, SearchQuery, SearchResponse};

use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(search))
}

/// GET /api/search?q=... - Full-text search
async fn search(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<ApiError>)> {
    if query.q.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new(
                "invalid_query",
                "Search query cannot be empty",
            )),
        ));
    }

    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    let db = state.db.lock().await;

    match db.search(&query.q, limit, offset) {
        Ok((results, total)) => Ok(Json(SearchResponse {
            results,
            total,
            query: query.q,
        })),
        Err(e) => {
            tracing::error!("Search failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("search_error", e.to_string())),
            ))
        }
    }
}
