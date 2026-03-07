use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::api::types::Credentials;

fn credentials_path() -> PathBuf {
    let dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".x-cli");
    dir
}

fn credentials_file() -> PathBuf {
    credentials_path().join("credentials.json")
}

pub fn save_credentials(creds: &Credentials) -> Result<()> {
    let dir = credentials_path();
    fs::create_dir_all(&dir).context("Failed to create ~/.x-cli directory")?;

    let json = serde_json::to_string_pretty(creds)?;
    fs::write(credentials_file(), json).context("Failed to write credentials file")?;

    // Set file permissions to owner-only on unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(credentials_file(), perms)?;
    }

    Ok(())
}

pub fn load_credentials() -> Result<Credentials> {
    let path = credentials_file();
    let content = fs::read_to_string(&path)
        .with_context(|| format!("No credentials found at {}. Run 'x auth' first.", path.display()))?;
    let creds: Credentials = serde_json::from_str(&content).context("Invalid credentials file")?;
    Ok(creds)
}
