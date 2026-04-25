use crate::auth;
use crate::github::GitHubClient;
use crate::oauth_session;
use anyhow::{Context, Result, anyhow};
use http::header::ACCEPT;
use secrecy::{ExposeSecret, SecretString};
use std::process::Command;
use std::time::Duration;

const ENV_OAUTH_CLIENT_ID: &str = "GITNAPSE_GITHUB_OAUTH_CLIENT_ID";
const ENV_GITHUB_CLIENT_ID: &str = "GITHUB_CLIENT_ID";
const DEFAULT_OAUTH_CLIENT_ID: &str = "Iv23liX3yGiGUEYkSlFW";

fn resolve_client_id(client_id: Option<String>) -> Result<String> {
    if let Some(cli_id) = client_id {
        let trimmed = cli_id.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }

    if let Ok(env_id) = std::env::var(ENV_OAUTH_CLIENT_ID) {
        let trimmed = env_id.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }

    if let Ok(env_id) = std::env::var(ENV_GITHUB_CLIENT_ID) {
        let trimmed = env_id.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }

    Ok(DEFAULT_OAUTH_CLIENT_ID.to_string())
}

fn terminal_hyperlink(url: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{url}\x1b]8;;\x1b\\")
}

fn ensure_rustls_crypto_provider() {
    // Some environments cannot auto-select rustls provider at runtime.
    let _ =
        rustls::crypto::CryptoProvider::install_default(rustls::crypto::ring::default_provider());
}

fn try_open_browser(url: &str) -> bool {
    if webbrowser::open(url).is_ok() {
        return true;
    }
    // Fallbacks for terminals/environments where webbrowser backend is unavailable.
    if cfg!(target_os = "linux") {
        if Command::new("xdg-open").arg(url).status().is_ok() {
            return true;
        }
        if Command::new("wslview").arg(url).status().is_ok() {
            return true;
        }
    } else if cfg!(target_os = "macos") {
        if Command::new("open").arg(url).status().is_ok() {
            return true;
        }
    } else if cfg!(target_os = "windows")
        && Command::new("cmd")
            .args(["/C", "start", "", url])
            .status()
            .is_ok()
    {
        return true;
    }
    false
}

pub fn oauth_device_login_cli(
    client_id: Option<String>,
    scopes: Vec<String>,
    timeout_secs: u64,
) -> Result<()> {
    ensure_rustls_crypto_provider();
    let client_id = resolve_client_id(client_id)?;
    let scopes = if scopes.is_empty() {
        vec!["read:user".to_string()]
    } else {
        scopes
            .into_iter()
            .map(|scope| scope.trim().to_string())
            .filter(|scope| !scope.is_empty())
            .collect::<Vec<_>>()
    };

    let client_secret = SecretString::new(client_id.clone().into());
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("Cannot initialize async runtime for OAuth flow")?;

    let (crab, device_codes) = runtime
        .block_on(async {
            let crab = octocrab::Octocrab::builder()
                .base_uri("https://github.com")
                .context("Cannot set OAuth base URI")?
                .add_header(ACCEPT, "application/json".to_string())
                .build()
                .context("Cannot create OAuth client")?;

            let device_codes = crab
                .authenticate_as_device(&client_secret, scopes.iter().map(String::as_str))
                .await
                .context("Unable to request OAuth device codes from GitHub")?;
            Ok::<_, anyhow::Error>((crab, device_codes))
        })
        .context("Unable to request OAuth device codes from GitHub")?;

    println!("OAuth device login started.");
    let opened = try_open_browser(&device_codes.verification_uri);
    if opened {
        println!("1. Browser launch requested automatically.");
        println!("   If no browser appears, open this URL manually.");
    }
    println!(
        "1. Open this URL in your browser: {}",
        device_codes.verification_uri
    );
    println!(
        "   Clickable link (if your terminal supports OSC8): {}",
        terminal_hyperlink(&device_codes.verification_uri)
    );
    println!("2. Enter code: {}", device_codes.user_code);
    println!("3. After authorization, keep this terminal open while token exchange completes.");
    println!("Scopes requested: {}", scopes.join(","));

    let timeout = Duration::from_secs(timeout_secs.max(60));
    let oauth = runtime
        .block_on(async {
            tokio::time::timeout(
                timeout,
                device_codes.poll_until_available(&crab, &client_secret),
            )
            .await
        })
        .map_err(|_| {
            anyhow!(
                "OAuth device flow timed out after {} seconds.",
                timeout.as_secs()
            )
        })?
        .context("OAuth token exchange failed")?;

    let access_token = oauth.access_token.expose_secret().to_string();
    auth::save_token(&access_token).context("Cannot store OAuth access token")?;
    oauth_session::save_from_oauth(&oauth, &client_id)
        .context("Cannot store OAuth session metadata")?;

    let login = GitHubClient::new(Some(&access_token))
        .context("Cannot validate OAuth token with API client")?
        .fetch_authenticated_user()
        .ok()
        .flatten()
        .unwrap_or_else(|| "unknown user".to_string());

    println!("OAuth login completed. Token saved securely for user: {login}");
    Ok(())
}
