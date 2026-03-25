//! API client for communicating with the Stash backend
#![allow(dead_code)]

use gloo_net::http::{Request, RequestBuilder};
use stash_shared::{
    Bookmark, BookmarkListResponse, CreateBookmarkRequest, ReadingStats, SearchResponse, Tag,
    UpdateBookmarkRequest,
};

const DEFAULT_API_URL: &str = "http://localhost:3030/api";

/// Get the API base URL from localStorage or use default
fn get_api_url() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(url)) = storage.get_item("stash_api_url") {
                    return url;
                }
            }
        }
    }
    DEFAULT_API_URL.to_string()
}

/// Get the API token from localStorage
fn get_api_token() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(token) = storage.get_item("stash_api_token") {
                    return token;
                }
            }
        }
    }
    None
}

/// Build a request with optional auth header
fn build_request(method: &str, path: &str) -> RequestBuilder {
    let url = format!("{}{}", get_api_url(), path);
    let builder = match method {
        "GET" => Request::get(&url),
        "POST" => Request::post(&url),
        "PATCH" => Request::patch(&url),
        "DELETE" => Request::delete(&url),
        _ => Request::get(&url),
    };

    if let Some(token) = get_api_token() {
        builder.header("Authorization", &format!("Bearer {}", token))
    } else {
        builder
    }
}

/// Build a request with JSON content type
fn build_json_request(method: &str, path: &str) -> RequestBuilder {
    let url = format!("{}{}", get_api_url(), path);
    let mut builder = match method {
        "GET" => Request::get(&url),
        "POST" => Request::post(&url),
        "PATCH" => Request::patch(&url),
        "DELETE" => Request::delete(&url),
        _ => Request::get(&url),
    };

    builder = builder.header("Content-Type", "application/json");

    if let Some(token) = get_api_token() {
        builder.header("Authorization", &format!("Bearer {}", token))
    } else {
        builder
    }
}

/// API client error type
#[derive(Debug, Clone)]
pub struct ApiError {
    pub message: String,
}

impl From<gloo_net::Error> for ApiError {
    fn from(e: gloo_net::Error) -> Self {
        Self {
            message: e.to_string(),
        }
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

/// Create a new bookmark
pub async fn create_bookmark(url: &str, tags: Option<Vec<String>>) -> ApiResult<Bookmark> {
    let req = CreateBookmarkRequest {
        url: url.to_string(),
        tags,
    };

    let response = build_json_request("POST", "/bookmarks")
        .json(&req)?
        .send()
        .await?;

    if response.ok() {
        Ok(response.json().await?)
    } else {
        Err(ApiError {
            message: format!("Failed to create bookmark: {}", response.status()),
        })
    }
}

/// List bookmarks with optional filters
pub async fn list_bookmarks(
    status: Option<&str>,
    is_favorite: Option<bool>,
    tag: Option<&str>,
    limit: Option<i32>,
    offset: Option<i32>,
) -> ApiResult<BookmarkListResponse> {
    let mut path = "/bookmarks?".to_string();
    let mut params = Vec::new();

    if let Some(s) = status {
        params.push(format!("status={}", s));
    }
    if let Some(fav) = is_favorite {
        params.push(format!("is_favorite={}", fav));
    }
    if let Some(t) = tag {
        params.push(format!("tag={}", t));
    }
    if let Some(l) = limit {
        params.push(format!("limit={}", l));
    }
    if let Some(o) = offset {
        params.push(format!("offset={}", o));
    }

    path.push_str(&params.join("&"));

    let response = build_request("GET", &path).send().await?;

    if response.ok() {
        Ok(response.json().await?)
    } else {
        Err(ApiError {
            message: format!("Failed to list bookmarks: {}", response.status()),
        })
    }
}

/// Get a single bookmark
pub async fn get_bookmark(id: i64) -> ApiResult<Bookmark> {
    let response = build_request("GET", &format!("/bookmarks/{}", id))
        .send()
        .await?;

    if response.ok() {
        Ok(response.json().await?)
    } else {
        Err(ApiError {
            message: format!("Bookmark not found: {}", id),
        })
    }
}

/// Get bookmark content for reader view
pub async fn get_bookmark_content(id: i64) -> ApiResult<Option<String>> {
    let response = build_request("GET", &format!("/bookmarks/{}/content", id))
        .send()
        .await?;

    if response.ok() {
        let json: serde_json::Value = response.json().await?;
        Ok(json
            .get("content")
            .and_then(|c| c.as_str())
            .map(|s| s.to_string()))
    } else {
        Err(ApiError {
            message: format!("Failed to get content: {}", id),
        })
    }
}

/// Update a bookmark
pub async fn update_bookmark(id: i64, req: &UpdateBookmarkRequest) -> ApiResult<Bookmark> {
    let response = build_json_request("PATCH", &format!("/bookmarks/{}", id))
        .json(req)?
        .send()
        .await?;

    if response.ok() {
        Ok(response.json().await?)
    } else {
        Err(ApiError {
            message: format!("Failed to update bookmark: {}", response.status()),
        })
    }
}

/// Delete a bookmark
pub async fn delete_bookmark(id: i64) -> ApiResult<()> {
    let response = build_request("DELETE", &format!("/bookmarks/{}", id))
        .send()
        .await?;

    if response.ok() {
        Ok(())
    } else {
        Err(ApiError {
            message: format!("Failed to delete bookmark: {}", id),
        })
    }
}

/// Search bookmarks
pub async fn search(
    query: &str,
    limit: Option<i32>,
    offset: Option<i32>,
) -> ApiResult<SearchResponse> {
    let mut path = format!("/search?q={}", urlencoding::encode(query));

    if let Some(l) = limit {
        path.push_str(&format!("&limit={}", l));
    }
    if let Some(o) = offset {
        path.push_str(&format!("&offset={}", o));
    }

    let response = build_request("GET", &path).send().await?;

    if response.ok() {
        Ok(response.json().await?)
    } else {
        Err(ApiError {
            message: format!("Search failed: {}", response.status()),
        })
    }
}

/// Get all tags
pub async fn get_tags() -> ApiResult<Vec<Tag>> {
    let response = build_request("GET", "/tags").send().await?;

    if response.ok() {
        Ok(response.json().await?)
    } else {
        Err(ApiError {
            message: format!("Failed to get tags: {}", response.status()),
        })
    }
}

/// Get reading stats
pub async fn get_stats() -> ApiResult<ReadingStats> {
    let response = build_request("GET", "/stats").send().await?;

    if response.ok() {
        Ok(response.json().await?)
    } else {
        Err(ApiError {
            message: format!("Failed to get stats: {}", response.status()),
        })
    }
}
