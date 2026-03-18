//! Content fetching and extraction module

mod extractor;
mod fetcher;

pub use extractor::*;
pub use fetcher::*;

/// Extracted content from a URL
#[derive(Debug, Clone)]
pub struct ExtractedContent {
    pub title: String,
    pub excerpt: Option<String>,
    pub content: Option<String>,
    pub site_name: Option<String>,
    pub favicon_url: Option<String>,
    pub estimated_read_time: Option<i32>,
}
