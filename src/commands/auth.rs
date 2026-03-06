use anyhow::Result;

use crate::auth::browser::{extract_cookies_from_browser, generate_csrf_token};
use crate::auth::storage::save_credentials;
use crate::api::types::Credentials;

pub fn run_auth_browser(browser: &str) -> Result<()> {
    let creds = extract_cookies_from_browser(browser)?;
    save_credentials(&creds)?;
    eprintln!("Credentials saved from {browser}.");
    eprintln!("auth_token: {}...{}", &creds.auth_token[..4.min(creds.auth_token.len())], &creds.auth_token[creds.auth_token.len().saturating_sub(4)..]);
    Ok(())
}

pub fn run_auth_token(token: &str) -> Result<()> {
    let creds = Credentials {
        auth_token: token.to_string(),
        ct0: generate_csrf_token(),
    };
    save_credentials(&creds)?;
    eprintln!("Credentials saved with provided auth_token.");
    Ok(())
}
