use anyhow::{Context, Result, anyhow};
use std::process::Command;

use crate::auth;
use crate::error::GitHubError;
use crate::github::GitHubClient;

// ── Git helpers ─────────────────────────────────────────────────────────

pub fn is_git_available() -> bool {
    Command::new("git").arg("--version").output().is_ok()
}

pub fn check_git() -> Result<()> {
    if !is_git_available() {
        return Err(anyhow!(
            "git is not installed or not in PATH\n\
             Install git: https://git-scm.com/downloads"
        ));
    }
    Ok(())
}

pub fn run_git(args: &[&str]) -> Result<std::process::Output> {
    check_git()?;
    Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("failed to execute: git {}", args.join(" ")))
}

pub fn stderr_msg(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_string()
}

pub fn stdout_str(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn not_a_repo_msg() -> String {
    "not a git repository — run this from inside a git repository".to_string()
}

pub fn not_a_repo_or_stderr(output: &std::process::Output, fallback_prefix: &str) -> String {
    let msg = stderr_msg(output);
    if msg.contains("not a git repository") {
        not_a_repo_msg()
    } else {
        format!("{fallback_prefix}:\n{msg}")
    }
}

// ── Repo detection ──────────────────────────────────────────────────────

pub fn detect_repo_from_remote() -> Option<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let url = url.strip_suffix(".git").unwrap_or(&url);

    let after_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .or_else(|| url.strip_prefix("git@"))
        .unwrap_or(&url);

    let path = if let Some(pos) = after_scheme.find(':') {
        &after_scheme[pos + 1..]
    } else if let Some(pos) = after_scheme.find('/') {
        &after_scheme[pos + 1..]
    } else {
        return None;
    };

    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 2 {
        Some(format!("{}/{}", parts[0], parts[1]))
    } else {
        None
    }
}

pub fn resolve_full_name(repo: &str) -> Result<String> {
    if repo.contains('/') {
        Ok(repo.to_string())
    } else if let Some(detected) = detect_repo_from_remote() {
        Ok(detected)
    } else {
        Err(anyhow!(
            "repository name '{repo}' is ambiguous. Use <owner/repo> or run from inside a cloned repo."
        ))
    }
}

// ── API helpers ─────────────────────────────────────────────────────────

pub fn handle_api_error(full_name: &str, e: &GitHubError) -> String {
    match e {
        GitHubError::Api { status, body } if *status == 404 || body.contains("Not Found") => {
            format!("repository '{full_name}' not found on GitHub")
        }
        GitHubError::Unauthorized => {
            "authentication required — run 'gitnapse auth set' or 'gitnapse auth oauth login'"
                .to_string()
        }
        GitHubError::RateLimited { remaining: _, reset } => {
            format!("GitHub API rate limit exceeded — resets at timestamp {reset}")
        }
        _ => format!("{e}"),
    }
}

pub fn make_client() -> Result<GitHubClient> {
    let token = auth::load_token()?;
    GitHubClient::new(token.as_deref()).map_err(|e| anyhow!("failed to create HTTP client: {e}"))
}
