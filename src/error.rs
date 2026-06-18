use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitHubError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("GitHub API responded with {status}: {body}")]
    Api { status: u16, body: String },

    #[error("Authentication required")]
    Unauthorized,

    #[error("Rate limit exhausted, resets at unix time {reset}")]
    RateLimited { remaining: u32, reset: u64 },

    #[error("File too large for Contents API: {0}")]
    FileTooLarge(String),

    #[error("Unsupported encoding: {0}")]
    Encoding(String),

    #[error("Cannot parse GitHub response: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("Base64 decode error: {0}")]
    Decode(#[from] base64::DecodeError),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("No authentication token available")]
    NoToken,

    #[error("Invalid token format")]
    InvalidToken,

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Keyring error: {0}")]
    Keyring(String),

    #[error("Cannot resolve config directory")]
    NoConfigDir,

    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Cannot resolve cache directory")]
    NoCacheDir,

    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("OAuth device flow error: {0}")]
    DeviceFlow(String),

    #[error("OAuth timed out after {0}s")]
    Timeout(u64),

    #[error("Token exchange failed: {0}")]
    TokenExchange(String),

    #[error("Cannot refresh token: {0}")]
    RefreshFailed(String),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("{0}")]
    Other(String),
}
