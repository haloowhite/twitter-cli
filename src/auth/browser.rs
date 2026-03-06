use anyhow::{Context, Result};

use crate::api::types::Credentials;

/// Extract auth_token and ct0 from browser cookies for x.com / twitter.com.
pub fn extract_cookies_from_browser(browser: &str) -> Result<Credentials> {
    let domains = vec![
        ".x.com".to_string(),
        ".twitter.com".to_string(),
        "x.com".to_string(),
        "twitter.com".to_string(),
    ];

    let cookies = match browser.to_lowercase().as_str() {
        "chrome" => rookie::chrome(Some(domains)),
        "firefox" => rookie::firefox(Some(domains)),
        "edge" => rookie::edge(Some(domains)),
        "safari" => rookie::safari(Some(domains)),
        _ => anyhow::bail!("Unsupported browser: {browser}. Use: chrome, firefox, edge, safari"),
    }
    .map_err(|e| anyhow::anyhow!("Failed to extract cookies from {browser}: {e}"))?;

    let mut auth_token = None;
    let mut ct0 = None;

    for cookie in &cookies {
        match cookie.name.as_str() {
            "auth_token" => auth_token = Some(cookie.value.clone()),
            "ct0" => ct0 = Some(cookie.value.clone()),
            _ => {}
        }
    }

    let auth_token = auth_token.context("auth_token cookie not found. Are you logged in to X?")?;
    let ct0 = ct0.unwrap_or_else(generate_csrf_token);

    Ok(Credentials { auth_token, ct0 })
}

/// Generate a random csrf token (same logic as heimdall).
pub fn generate_csrf_token() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    (0..32)
        .map(|_| {
            let byte: u8 = rng.random();
            format!("{:x}", byte & 0x0f)
        })
        .collect()
}
