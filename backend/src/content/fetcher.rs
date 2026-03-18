//! URL fetching with timeout and compression

use reqwest::Client;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Content type not supported: {0}")]
    UnsupportedContentType(String),
}

pub type FetchResult<T> = Result<T, FetchError>;

/// Fetched page data
pub struct FetchedPage {
    pub html: String,
    pub final_url: String,
}

/// HTTP client for fetching URLs
pub struct Fetcher {
    client: Client,
}

impl Default for Fetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Fetcher {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .gzip(true)
            .brotli(true)
            .user_agent("Mozilla/5.0 (compatible; Stash/1.0; +https://github.com/stash)")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Fetch a URL and return the HTML content
    pub async fn fetch(&self, url: &str) -> FetchResult<FetchedPage> {
        let response = self.client.get(url).send().await?;

        // Check content type
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        if let Some(ref ct) = content_type {
            if !ct.contains("text/html") && !ct.contains("application/xhtml") {
                return Err(FetchError::UnsupportedContentType(ct.clone()));
            }
        }

        let final_url = response.url().to_string();
        let html = response.text().await?;

        Ok(FetchedPage { html, final_url })
    }
}
