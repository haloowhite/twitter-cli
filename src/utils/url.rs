use regex::Regex;
use std::sync::LazyLock;

static TCO_URL_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https?://t\.co/[A-Za-z0-9]+").unwrap());

/// Expand t.co short URLs in text using Twitter API entities data (no network request).
pub fn expand_urls_from_entities(text: &str, entities: &serde_json::Value) -> String {
    let urls = match entities.get("urls").and_then(|u| u.as_array()) {
        Some(arr) => arr,
        None => return text.to_string(),
    };

    let mut result = text.to_string();
    for url_entity in urls {
        let short = url_entity
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let expanded = url_entity
            .get("expanded_url")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if !short.is_empty() && !expanded.is_empty() {
            result = result.replace(short, expanded);
        }
    }
    result
}

/// Find all t.co URLs in text.
pub fn find_tco_urls(text: &str) -> Vec<String> {
    TCO_URL_PATTERN
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Check if a URL is a t.co short URL.
pub fn is_tco_url(url: &str) -> bool {
    url.starts_with("https://t.co/") || url.starts_with("http://t.co/")
}
