//! Import bookmarks from Firefox/Netscape HTML exports.

use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;
use url::Url;

use crate::db::{Database, DbError, ImportOutcome};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedBookmark {
    pub url: String,
    pub title: String,
    pub favicon_url: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSummary {
    pub path: String,
    pub total_discovered: usize,
    pub imported: usize,
    pub skipped: usize,
    pub invalid: usize,
}

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("Failed to read import file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Database(#[from] DbError),
}

pub fn import_firefox_bookmarks(
    db: &Database,
    path: &Path,
    default_tags: &[String],
) -> Result<ImportSummary, ImportError> {
    let resolved_path = expand_home(path);
    let html = fs::read_to_string(&resolved_path)?;
    let bookmarks = parse_firefox_bookmarks(&html, default_tags);

    let mut summary = ImportSummary {
        path: resolved_path.display().to_string(),
        total_discovered: bookmarks.len(),
        imported: 0,
        skipped: 0,
        invalid: 0,
    };

    for bookmark in bookmarks {
        if Url::parse(&bookmark.url).is_err() {
            summary.invalid += 1;
            continue;
        }

        match db.import_bookmark(
            &bookmark.url,
            &bookmark.title,
            bookmark.favicon_url.as_deref(),
            &bookmark.tags,
        )? {
            ImportOutcome::Imported => summary.imported += 1,
            ImportOutcome::SkippedDuplicate => summary.skipped += 1,
        }
    }

    Ok(summary)
}

fn expand_home(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    if let Some(stripped) = path_str.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(stripped);
        }
    }

    path.to_path_buf()
}

fn parse_firefox_bookmarks(html: &str, default_tags: &[String]) -> Vec<ImportedBookmark> {
    let mut bookmarks = Vec::new();
    let mut folder_stack: Vec<String> = Vec::new();
    let mut pending_folder: Option<String> = None;

    for raw_line in html.lines() {
        let line = raw_line.trim();
        let upper = line.to_ascii_uppercase();

        if let Some(folder_name) = extract_tag_text(line, "H3") {
            pending_folder = Some(folder_name);
        }

        if upper.starts_with("<DL") {
            if let Some(folder_name) = pending_folder.take() {
                folder_stack.push(folder_name);
            }
            continue;
        }

        if upper.starts_with("</DL") {
            pending_folder = None;
            folder_stack.pop();
            continue;
        }

        if let Some((url, title, favicon_url)) = extract_anchor(line) {
            let mut tags = default_tags.to_vec();
            tags.push("firefox-import".to_string());
            tags.extend(folder_stack.iter().cloned());
            dedupe_tags(&mut tags);

            bookmarks.push(ImportedBookmark {
                url,
                title,
                favicon_url,
                tags,
            });
        }
    }

    bookmarks
}

fn extract_anchor(line: &str) -> Option<(String, String, Option<String>)> {
    if !line.to_ascii_uppercase().contains("<A ") {
        return None;
    }

    let url = extract_attribute(line, "HREF")?;
    let title = extract_tag_text(line, "A").unwrap_or_else(|| url.clone());
    let favicon_url =
        extract_attribute(line, "ICON_URI").or_else(|| extract_attribute(line, "ICON"));

    Some((url, title, favicon_url))
}

fn extract_attribute(line: &str, attribute: &str) -> Option<String> {
    let upper = line.to_ascii_uppercase();
    let needle = format!(r#"{attribute}=""#);
    let start = upper.find(&needle)? + needle.len();
    let end = line[start..].find('"')?;
    Some(decode_html_entities(&line[start..start + end]))
}

fn extract_tag_text(line: &str, tag: &str) -> Option<String> {
    let upper = line.to_ascii_uppercase();
    let open_tag = format!("<{tag}");
    let close_tag = format!("</{tag}>");
    let open_idx = upper.find(&open_tag)?;
    let content_start = line[open_idx..].find('>')? + open_idx + 1;
    let close_idx = upper[content_start..].find(&close_tag)? + content_start;
    let text = strip_html_tags(&line[content_start..close_idx])
        .trim()
        .to_string();

    if text.is_empty() {
        None
    } else {
        Some(decode_html_entities(&text))
    }
}

fn strip_html_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;

    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }

    out
}

fn decode_html_entities(input: &str) -> String {
    input
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&#x27;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

fn dedupe_tags(tags: &mut Vec<String>) {
    let mut deduped = Vec::with_capacity(tags.len());

    for tag in tags.drain(..) {
        let trimmed = tag.trim();
        if trimmed.is_empty() {
            continue;
        }

        if !deduped.iter().any(|existing| existing == trimmed) {
            deduped.push(trimmed.to_string());
        }
    }

    *tags = deduped;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_firefox_export_and_applies_folder_tags() {
        let html = r#"
<!DOCTYPE NETSCAPE-Bookmark-file-1>
<DL><p>
  <DT><H3 PERSONAL_TOOLBAR_FOLDER="true">Toolbar</H3>
  <DL><p>
    <DT><A HREF="https://example.com" ICON_URI="https://example.com/favicon.ico">Example &amp; Docs</A>
    <DT><H3>Rust</H3>
    <DL><p>
      <DT><A HREF="https://doc.rust-lang.org">Rust Book</A>
    </DL><p>
  </DL><p>
</DL><p>
"#;

        let bookmarks = parse_firefox_bookmarks(html, &[String::from("imported")]);

        assert_eq!(bookmarks.len(), 2);
        assert_eq!(bookmarks[0].title, "Example & Docs");
        assert_eq!(
            bookmarks[0].tags,
            vec!["imported", "firefox-import", "Toolbar"]
        );
        assert_eq!(
            bookmarks[1].tags,
            vec!["imported", "firefox-import", "Toolbar", "Rust"]
        );
    }
}
