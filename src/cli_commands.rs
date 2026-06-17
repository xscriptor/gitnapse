use anyhow::{Context, Result, anyhow};
use std::process::Command;

use crate::auth;
use crate::github::GitHubClient;

fn is_git_available() -> bool {
    Command::new("git").arg("--version").output().is_ok()
}

fn check_git() -> Result<()> {
    if !is_git_available() {
        return Err(anyhow!(
            "git is not installed or not in PATH\n\
             Install git: https://git-scm.com/downloads"
        ));
    }
    Ok(())
}

fn run_git(args: &[&str]) -> Result<std::process::Output> {
    check_git()?;
    Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("failed to execute: git {}", args.join(" ")))
}

fn parse_repo_spec(spec: &str) -> Result<(String, Option<String>)> {
    if spec.is_empty() {
        return Err(anyhow!(
            "repository specification is empty\n\
             Usage: gitnapse clone <owner/repo>[:branch]"
        ));
    }
    if spec.contains("://") || spec.contains('@') {
        if let Some(pos) = spec.rfind(':') {
            let url_part = &spec[..pos];
            let branch_part = &spec[pos + 1..];
            if !branch_part.is_empty()
                && !branch_part.contains('/')
                && !branch_part.contains('.')
            {
                return Ok((url_part.to_string(), Some(branch_part.to_string())));
            }
        }
        Ok((spec.to_string(), None))
    } else {
        if let Some((repo, branch)) = spec.split_once(':') {
            if repo.is_empty() {
                return Err(anyhow!(
                    "invalid repository specification '{spec}'\n\
                     Usage: gitnapse clone <owner/repo>[:branch]"
                ));
            }
            Ok((repo.to_string(), Some(branch.to_string())))
        } else {
            Ok((spec.to_string(), None))
        }
    }
}

pub fn clone_repo(repo_spec: &str) -> Result<()> {
    check_git()?;
    let (repo, branch) = parse_repo_spec(repo_spec)?;

    let clone_url = if repo.contains("://") || repo.contains('@') {
        repo.clone()
    } else {
        let token = auth::load_token()?;
        let client = GitHubClient::new(token.as_deref())?;
        let info = client.fetch_repo_by_name(&repo).map_err(|e| {
            let msg = match &e {
                crate::error::GitHubError::Api { status, body }
                    if *status == 404 || body.contains("Not Found") =>
                {
                    format!("repository '{repo}' not found on GitHub")
                }
                crate::error::GitHubError::Unauthorized => {
                    "authentication required — run 'gitnapse auth set' or 'gitnapse auth oauth login'".to_string()
                }
                _ => format!("{e}"),
            };
            anyhow!("{msg}")
        })?;
        info.clone_url
    };

    let mut cmd = Command::new("git");
    cmd.arg("clone");
    if let Some(ref b) = branch {
        cmd.args(["-b", b]);
    }
    cmd.arg(&clone_url);

    let output = cmd.output().context("failed to execute git")?;
    if output.status.success() {
        println!("✓ Cloned {} successfully", repo_spec);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("git clone failed:\n{}", stderr.trim()));
    }
    Ok(())
}

pub fn commit(msg: &str) -> Result<()> {
    if msg.trim().is_empty() {
        return Err(anyhow!(
            "commit message cannot be empty\n\
             Usage: gitnapse commit -m \"your message\""
        ));
    }

    let add = run_git(&["add", "-A"])?;
    if !add.status.success() {
        let stderr = String::from_utf8_lossy(&add.stderr);
        let msg = if stderr.contains("not a git repository") {
            "not a git repository — run this from inside a git repository".to_string()
        } else {
            format!("git add failed:\n{}", stderr.trim())
        };
        return Err(anyhow!("{msg}"));
    }

    let commit = run_git(&["commit", "-m", msg.trim()])?;
    if commit.status.success() {
        let stdout = String::from_utf8_lossy(&commit.stdout);
        println!("✓ {}", stdout.trim());
    } else {
        let stderr = String::from_utf8_lossy(&commit.stderr);
        if stderr.contains("nothing to commit") {
            println!("nothing to commit (working tree clean)");
        } else {
            return Err(anyhow!("git commit failed:\n{}", stderr.trim()));
        }
    }
    Ok(())
}

pub fn push(remote: Option<&str>, branch: Option<&str>) -> Result<()> {
    let mut args = vec!["push"];
    if let Some(r) = remote.as_deref() {
        args.push(r);
    }
    if let Some(b) = branch.as_deref() {
        args.push(b);
    }

    let output = run_git(&args)?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            println!("{line}");
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = if stderr.contains("not a git repository") {
            "not a git repository — run this from inside a git repository".to_string()
        } else {
            format!("git push failed:\n{}", stderr.trim())
        };
        return Err(anyhow!("{msg}"));
    }
    Ok(())
}

pub fn status() -> Result<()> {
    let output = run_git(&["status", "--short"])?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            println!("(clean)");
        } else {
            print!("{stdout}");
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = if stderr.contains("not a git repository") {
            "not a git repository — run this from inside a git repository".to_string()
        } else {
            format!("git status failed:\n{}", stderr.trim())
        };
        return Err(anyhow!("{msg}"));
    }
    Ok(())
}

pub fn log_lines(n: usize) -> Result<()> {
    if n == 0 {
        return Err(anyhow!("count must be greater than 0"));
    }
    let output = run_git(&["log", "--oneline", &format!("-{n}")])?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            println!("(no commits)");
        } else {
            print!("{stdout}");
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = if stderr.contains("not a git repository") {
            "not a git repository — run this from inside a git repository".to_string()
        } else {
            format!("git log failed:\n{}", stderr.trim())
        };
        return Err(anyhow!("{msg}"));
    }
    Ok(())
}

pub fn branch() -> Result<()> {
    let output = run_git(&["branch", "-a"])?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        print!("{stdout}");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = if stderr.contains("not a git repository") {
            "not a git repository — run this from inside a git repository".to_string()
        } else {
            format!("git branch failed:\n{}", stderr.trim())
        };
        return Err(anyhow!("{msg}"));
    }
    Ok(())
}

fn make_client() -> Result<GitHubClient> {
    let token = auth::load_token()?;
    GitHubClient::new(token.as_deref()).map_err(|e| anyhow!("failed to create HTTP client: {e}"))
}

fn handle_api_error(full_name: &str, e: &crate::error::GitHubError) -> String {
    match e {
        crate::error::GitHubError::Api { status, body }
            if *status == 404 || body.contains("Not Found") =>
        {
            format!("repository '{full_name}' not found on GitHub")
        }
        crate::error::GitHubError::Unauthorized => {
            "authentication required — run 'gitnapse auth set' or 'gitnapse auth oauth login'"
                .to_string()
        }
        crate::error::GitHubError::RateLimited { remaining: _, reset } => {
            format!("GitHub API rate limit exceeded — resets at timestamp {reset}")
        }
        _ => format!("{e}"),
    }
}

pub fn pr_list(full_name: &str, state: &str) -> Result<()> {
    let client = make_client()?;
    let prs = client.fetch_pull_requests(full_name, state, 30).map_err(|e| {
        anyhow!(handle_api_error(full_name, &e))
    })?;

    if prs.is_empty() {
        println!("No {state} pull requests for {full_name}");
        return Ok(());
    }

    for pr in &prs {
        println!(
            "#{:>4} [{:>7}] {:>5}+ {:<4}- {} (by {})",
            pr.number,
            pr.state,
            pr.additions.unwrap_or(0),
            pr.deletions.unwrap_or(0),
            pr.title,
            pr.user.login,
        );
    }
    Ok(())
}

pub fn pr_create(
    full_name: &str,
    title: &str,
    head: &str,
    base: &str,
    body: Option<&str>,
) -> Result<()> {
    let client = make_client()?;
    let pr = client
        .create_pull_request(full_name, title, head, base, body)
        .map_err(|e| anyhow!(handle_api_error(full_name, &e)))?;
    println!("✓ PR #{} created: {}", pr.number, pr.html_url);
    Ok(())
}

pub fn pr_merge(full_name: &str, number: u64, method: Option<&str>) -> Result<()> {
    let client = make_client()?;
    let result = client
        .merge_pull_request(full_name, number, None, method)
        .map_err(|e| anyhow!(handle_api_error(full_name, &e)))?;
    if result.merged {
        println!("✓ PR #{number} merged — SHA: {}", result.sha);
    } else {
        println!("✗ Merge failed: {}", result.message);
    }
    Ok(())
}
