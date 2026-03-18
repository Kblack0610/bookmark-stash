//! Content extraction using scraper (readability-like extraction)

use super::ExtractedContent;
use scraper::{Html, Selector};

/// Average words per minute for reading time calculation
const WORDS_PER_MINUTE: usize = 200;

/// Extract readable content from HTML
pub fn extract_content(html: &str, url: &str) -> ExtractedContent {
    let document = Html::parse_document(html);

    // Extract title
    let title = extract_title(&document).unwrap_or_else(|| url.to_string());

    // Extract meta description as excerpt
    let excerpt = extract_meta_description(&document);

    // Extract main content
    let content = extract_main_content(&document);

    // Extract site name
    let site_name = extract_site_name(&document, url);

    // Extract favicon
    let favicon_url = extract_favicon(&document, url);

    // Calculate reading time
    let estimated_read_time = content.as_ref().map(|c| estimate_read_time(c));

    ExtractedContent {
        title,
        excerpt,
        content,
        site_name,
        favicon_url,
        estimated_read_time,
    }
}

fn extract_title(doc: &Html) -> Option<String> {
    // Try og:title first
    if let Some(og_title) = extract_meta(doc, "og:title") {
        return Some(og_title);
    }

    // Fall back to <title>
    let selector = Selector::parse("title").ok()?;
    doc.select(&selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
}

fn extract_meta_description(doc: &Html) -> Option<String> {
    // Try og:description first
    if let Some(desc) = extract_meta(doc, "og:description") {
        return Some(desc);
    }

    // Fall back to meta description
    let selector = Selector::parse(r#"meta[name="description"]"#).ok()?;
    doc.select(&selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.trim().to_string())
}

fn extract_meta(doc: &Html, property: &str) -> Option<String> {
    let selector = Selector::parse(&format!(r#"meta[property="{}"]"#, property)).ok()?;
    doc.select(&selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.trim().to_string())
}

fn extract_site_name(doc: &Html, url: &str) -> Option<String> {
    // Try og:site_name
    if let Some(name) = extract_meta(doc, "og:site_name") {
        return Some(name);
    }

    // Extract from URL domain
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|s| s.to_string()))
}

fn extract_favicon(doc: &Html, url: &str) -> Option<String> {
    // Try various favicon link tags
    let selectors = [
        r#"link[rel="icon"]"#,
        r#"link[rel="shortcut icon"]"#,
        r#"link[rel="apple-touch-icon"]"#,
    ];

    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(el) = doc.select(&selector).next() {
                if let Some(href) = el.value().attr("href") {
                    // Resolve relative URLs
                    if href.starts_with("http") {
                        return Some(href.to_string());
                    } else if let Ok(base) = url::Url::parse(url) {
                        if let Ok(resolved) = base.join(href) {
                            return Some(resolved.to_string());
                        }
                    }
                }
            }
        }
    }

    // Default to /favicon.ico
    url::Url::parse(url).ok().map(|u| {
        format!(
            "{}://{}/favicon.ico",
            u.scheme(),
            u.host_str().unwrap_or("")
        )
    })
}

fn extract_main_content(doc: &Html) -> Option<String> {
    // Try common content containers in order of preference
    let content_selectors = [
        "article",
        r#"[role="main"]"#,
        "main",
        ".post-content",
        ".article-content",
        ".entry-content",
        ".content",
        "#content",
        ".post",
        ".article",
    ];

    for selector_str in &content_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(el) = doc.select(&selector).next() {
                let text = clean_text(&el.text().collect::<String>());
                if text.len() > 200 {
                    // Get HTML content for reader view
                    return Some(el.inner_html());
                }
            }
        }
    }

    // Fall back to body, but try to remove nav/header/footer
    if let Ok(selector) = Selector::parse("body") {
        if let Some(body) = doc.select(&selector).next() {
            let mut html = body.inner_html();

            // Remove common non-content elements
            let remove_selectors = [
                "nav", "header", "footer", "aside", ".sidebar", ".nav", ".header", ".footer",
            ];
            for rm_sel in &remove_selectors {
                if let Ok(sel) = Selector::parse(rm_sel) {
                    for _ in doc.select(&sel) {
                        // Simple removal by replacing with empty (not perfect but works)
                        html = html.replace(
                            &format!("<{}", rm_sel),
                            &format!("<!-- removed {} -->", rm_sel),
                        );
                    }
                }
            }

            return Some(html);
        }
    }

    None
}

fn clean_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn estimate_read_time(content: &str) -> i32 {
    let text = Html::parse_fragment(content)
        .root_element()
        .text()
        .collect::<String>();

    let word_count = text.split_whitespace().count();
    let minutes = (word_count / WORDS_PER_MINUTE) as i32;

    // Minimum 1 minute
    std::cmp::max(1, minutes)
}
