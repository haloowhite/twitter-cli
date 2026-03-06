use std::fmt;

#[derive(Debug)]
pub enum TwitterError {
    Auth(String),
    Api(ApiError),
    Http(String),
    Parse(String),
    Io(String),
    RateLimit { reset_at: Option<i64> },
    AccountBanned,
    AccountSuspended,
}

#[derive(Debug)]
pub struct ApiError {
    pub code: i64,
    pub message: String,
}

impl fmt::Display for TwitterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TwitterError::Auth(msg) => write!(f, "Auth error: {msg}"),
            TwitterError::Api(e) => write!(f, "API error {}: {}", e.code, e.message),
            TwitterError::Http(msg) => write!(f, "HTTP error: {msg}"),
            TwitterError::Parse(msg) => write!(f, "Parse error: {msg}"),
            TwitterError::Io(msg) => write!(f, "IO error: {msg}"),
            TwitterError::RateLimit { reset_at } => {
                if let Some(ts) = reset_at {
                    write!(f, "Rate limited, resets at {ts}")
                } else {
                    write!(f, "Rate limited")
                }
            }
            TwitterError::AccountBanned => write!(f, "Account banned (error 326)"),
            TwitterError::AccountSuspended => write!(f, "Account suspended (error 64)"),
        }
    }
}

impl std::error::Error for TwitterError {}

impl From<std::io::Error> for TwitterError {
    fn from(e: std::io::Error) -> Self {
        TwitterError::Io(e.to_string())
    }
}
