use anyhow::Result;

use crate::auth::browser::{extract_cookies_from_browser, generate_csrf_token};
use crate::auth::login;
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
        extra_cookies: None,
    };
    save_credentials(&creds)?;
    eprintln!("Credentials saved with provided auth_token.");
    Ok(())
}

pub async fn run_auth_login(
    username: Option<&str>,
    password: Option<&str>,
    totp: Option<&str>,
) -> Result<()> {
    let username = match username {
        Some(u) => u.to_string(),
        None => {
            eprint!("Username: ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };

    let password = match password {
        Some(p) => p.to_string(),
        None => {
            eprint!("Password: ");
            // Read password - try to disable echo if possible
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            eprintln!(); // newline after password input
            input.trim().to_string()
        }
    };

    let creds = login::login(&username, &password, totp).await?;
    save_credentials(&creds)?;

    let at = &creds.auth_token;
    eprintln!("Login successful! Credentials saved.");
    eprintln!(
        "auth_token: {}...{}",
        &at[..4.min(at.len())],
        &at[at.len().saturating_sub(4)..]
    );
    Ok(())
}
