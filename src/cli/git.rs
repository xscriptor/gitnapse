use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::auth;
use crate::error::GitHubError;
use crate::github::GitHubClient;

use super::helpers;

fn parse_repo_spec(spec: &str) -> Result<(String, Option<String>)> {
    if spec.is_empty() {
        return Err(anyhow!(
            "repository specification is empty\n\
             Usage: gitnapse clone <owner/repo>[:branch] [--dir <path>]"
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
                     Usage: gitnapse clone <owner/repo>[:branch] [--dir <path>]"
                ));
            }
            Ok((repo.to_string(), Some(branch.to_string())))
        } else {
            Ok((spec.to_string(), None))
        }
    }
}

// ── Clone ───────────────────────────────────────────────────────────────

pub fn clone_repo(repo_spec: &str, dir: Option<&str>) -> Result<()> {
    helpers::check_git()?;
    let (repo, branch) = parse_repo_spec(repo_spec)?;

    let clone_url = if repo.contains("://") || repo.contains('@') {
        repo.clone()
    } else {
        let token = auth::load_token()?;
        let client = GitHubClient::new(token.as_deref())?;
        let info = client.fetch_repo_by_name(&repo).map_err(|e| {
            let msg = match &e {
                GitHubError::Api { status, body }
                    if *status == 404 || body.contains("Not Found") =>
                {
                    format!("repository '{repo}' not found on GitHub")
                }
                GitHubError::Unauthorized => {
                    "authentication required — run 'gitnapse auth set' or 'gitnapse auth oauth login'".to_string()
                }
                _ => format!("{e}"),
            };
            anyhow!("{msg}")
        })?;
        info.clone_url
    };

    let dest = dir.map(PathBuf::from);

    if let Some(ref p) = dest {
        if p.exists() {
            return Err(anyhow!("destination path '{}' already exists", p.display()));
        }
    }

    let mut cmd = Command::new("git");
    cmd.arg("clone");
    if let Some(ref b) = branch {
        cmd.args(["-b", b]);
    }
    cmd.arg(&clone_url);
    if let Some(ref p) = dest {
        cmd.arg(p);
    }

    let output = cmd.output().context("failed to execute git")?;
    if output.status.success() {
        let dir_name = dest
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| {
                clone_url
                    .rsplit_once('/')
                    .map(|(_, name)| name.trim_end_matches(".git").to_string())
                    .unwrap_or_default()
            });
        println!("✓ Cloned into {dir_name}");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("git clone failed:\n{}", stderr.trim()));
    }
    Ok(())
}

// ── Commit ──────────────────────────────────────────────────────────────

pub fn commit(msg: &str, all: bool) -> Result<()> {
    if msg.trim().is_empty() {
        return Err(anyhow!(
            "commit message cannot be empty\n\
             Usage: gitnapse commit -m \"your message\" [-a]"
        ));
    }

    if all {
        let add = helpers::run_git(&["add", "-A"])?;
        if !add.status.success() {
            let msg = helpers::not_a_repo_or_stderr(&add, "git add failed");
            return Err(anyhow!("{msg}"));
        }
    }

    let commit = helpers::run_git(&["commit", "-m", msg.trim()])?;
    if commit.status.success() {
        let stdout = helpers::stdout_str(&commit);
        println!("✓ {}", stdout.trim());
    } else {
        let stderr = helpers::stderr_msg(&commit);
        if stderr.contains("nothing to commit") {
            println!("nothing to commit (working tree clean)");
        } else if stderr.contains("not a git repository") {
            return Err(anyhow!("{}", helpers::not_a_repo_msg()));
        } else {
            return Err(anyhow!("git commit failed:\n{stderr}"));
        }
    }
    Ok(())
}

// ── Push ────────────────────────────────────────────────────────────────

pub fn push(remote: Option<&str>, branch: Option<&str>, force: bool) -> Result<()> {
    let mut args = vec!["push"];
    if force {
        args.push("--force-with-lease");
    }
    if let Some(r) = remote {
        args.push(r);
    }
    if let Some(b) = branch {
        args.push(b);
    }

    let output = helpers::run_git(&args)?;
    if output.status.success() {
        let stdout = helpers::stdout_str(&output);
        for line in stdout.lines() {
            println!("{line}");
        }
    } else {
        let msg = helpers::not_a_repo_or_stderr(&output, "git push failed");
        return Err(anyhow!("{msg}"));
    }
    Ok(())
}

// ── Pull ────────────────────────────────────────────────────────────────

pub fn pull(remote: Option<&str>, branch: Option<&str>, rebase: bool) -> Result<()> {
    let mut args = vec!["pull"];
    if rebase {
        args.push("--rebase");
    }
    if let Some(r) = remote {
        args.push(r);
    }
    if let Some(b) = branch {
        args.push(b);
    }

    let output = helpers::run_git(&args)?;
    if output.status.success() {
        let stdout = helpers::stdout_str(&output);
        for line in stdout.lines() {
            println!("{line}");
        }
    } else {
        let msg = helpers::not_a_repo_or_stderr(&output, "git pull failed");
        return Err(anyhow!("{msg}"));
    }
    Ok(())
}

// ── Fetch ───────────────────────────────────────────────────────────────

pub fn fetch(prune: bool) -> Result<()> {
    let mut args = vec!["fetch"];
    if prune {
        args.push("--prune");
    }

    let output = helpers::run_git(&args)?;
    if output.status.success() {
        let stdout = helpers::stdout_str(&output);
        for line in stdout.lines() {
            println!("{line}");
        }
    } else {
        let msg = helpers::not_a_repo_or_stderr(&output, "git fetch failed");
        return Err(anyhow!("{msg}"));
    }
    Ok(())
}

// ── Checkout ────────────────────────────────────────────────────────────

pub fn checkout(branch: &str, create: bool) -> Result<()> {
    if branch.trim().is_empty() {
        return Err(anyhow!(
            "branch name cannot be empty\n\
             Usage: gitnapse checkout <branch> [-b]"
        ));
    }

    let mut args = vec!["checkout"];
    if create {
        args.push("-b");
    }
    args.push(branch.trim());

    let output = helpers::run_git(&args)?;
    if output.status.success() {
        println!("✓ Switched to branch '{branch}'");
    } else {
        let msg = helpers::not_a_repo_or_stderr(&output, "git checkout failed");
        return Err(anyhow!("{msg}"));
    }
    Ok(())
}

// ── Diff ────────────────────────────────────────────────────────────────

pub fn diff(staged: bool, path: Option<&str>) -> Result<()> {
    let mut args = vec!["diff"];
    if staged {
        args.push("--cached");
    }
    if let Some(p) = path {
        args.push("--");
        args.push(p);
    }

    let output = helpers::run_git(&args)?;
    if output.status.success() {
        let stdout = helpers::stdout_str(&output);
        if stdout.trim().is_empty() {
            println!("(no changes)");
        } else {
            print!("{stdout}");
        }
    } else {
        let msg = helpers::not_a_repo_or_stderr(&output, "git diff failed");
        return Err(anyhow!("{msg}"));
    }
    Ok(())
}

// ── Stash ───────────────────────────────────────────────────────────────

pub fn stash_push(message: Option<&str>) -> Result<()> {
    let mut args = vec!["stash", "push"];
    if let Some(m) = message {
        args.push("-m");
        args.push(m);
    }

    let output = helpers::run_git(&args)?;
    if output.status.success() {
        let stdout = helpers::stdout_str(&output);
        let trimmed = stdout.trim();
        if trimmed.is_empty() || trimmed.contains("No local changes") {
            println!("no local changes to stash");
        } else {
            println!("{trimmed}");
        }
    } else {
        let msg = helpers::not_a_repo_or_stderr(&output, "git stash failed");
        return Err(anyhow!("{msg}"));
    }
    Ok(())
}

pub fn stash_pop() -> Result<()> {
    let output = helpers::run_git(&["stash", "pop"])?;
    if output.status.success() {
        let stdout = helpers::stdout_str(&output);
        for line in stdout.lines() {
            println!("{line}");
        }
    } else {
        let stderr = helpers::stderr_msg(&output);
        if stderr.contains("No stash entries found") {
            return Err(anyhow!("no stash entries to pop"));
        }
        let msg = helpers::not_a_repo_or_stderr(&output, "git stash pop failed");
        return Err(anyhow!("{msg}"));
    }
    Ok(())
}

pub fn stash_list() -> Result<()> {
    let output = helpers::run_git(&["stash", "list"])?;
    if output.status.success() {
        let stdout = helpers::stdout_str(&output);
        if stdout.trim().is_empty() {
            println!("(no stashes)");
        } else {
            print!("{stdout}");
        }
    } else {
        let msg = helpers::not_a_repo_or_stderr(&output, "git stash list failed");
        return Err(anyhow!("{msg}"));
    }
    Ok(())
}

// ── Tag ─────────────────────────────────────────────────────────────────

pub fn tag_list(pattern: Option<&str>) -> Result<()> {
    let mut args = vec!["tag"];
    args.push("--list");
    if let Some(p) = pattern {
        args.push(p);
    }

    let output = helpers::run_git(&args)?;
    if output.status.success() {
        let stdout = helpers::stdout_str(&output);
        print!("{stdout}");
    } else {
        let msg = helpers::not_a_repo_or_stderr(&output, "git tag failed");
        return Err(anyhow!("{msg}"));
    }
    Ok(())
}

pub fn tag_create(name: &str, message: Option<&str>, target: Option<&str>) -> Result<()> {
    if name.trim().is_empty() {
        return Err(anyhow!("tag name cannot be empty"));
    }

    let mut args = vec!["tag"];
    if let Some(m) = message {
        args.push("-a");
        args.push(name.trim());
        args.push("-m");
        args.push(m);
    } else {
        args.push(name.trim());
    }
    if let Some(t) = target {
        args.push(t);
    }

    let output = helpers::run_git(&args)?;
    if output.status.success() {
        println!("✓ Created tag '{name}'");
    } else {
        let msg = helpers::not_a_repo_or_stderr(&output, "git tag failed");
        return Err(anyhow!("{msg}"));
    }
    Ok(())
}

pub fn tag_delete(name: &str) -> Result<()> {
    if name.trim().is_empty() {
        return Err(anyhow!("tag name cannot be empty"));
    }

    let local = helpers::run_git(&["tag", "-d", name.trim()])?;
    if !local.status.success() {
        let msg = helpers::not_a_repo_or_stderr(&local, "git tag -d failed");
        return Err(anyhow!("{msg}"));
    }

    let remote = helpers::run_git(&["push", "origin", "--delete", name.trim()]);
    if let Ok(out) = remote {
        if out.status.success() {
            println!("✓ Deleted tag '{name}' (local + remote)");
        } else {
            println!("✓ Deleted tag '{name}' locally (remote delete skipped)");
        }
    } else {
        println!("✓ Deleted tag '{name}' locally");
    }
    Ok(())
}

// ── Status ──────────────────────────────────────────────────────────────

pub fn status() -> Result<()> {
    let output = helpers::run_git(&["status", "--short"])?;
    if output.status.success() {
        let stdout = helpers::stdout_str(&output);
        if stdout.trim().is_empty() {
            println!("(clean)");
        } else {
            print!("{stdout}");
        }
    } else {
        let msg = helpers::not_a_repo_or_stderr(&output, "git status failed");
        return Err(anyhow!("{msg}"));
    }
    Ok(())
}

// ── Log ─────────────────────────────────────────────────────────────────

pub fn log_lines(n: usize) -> Result<()> {
    if n == 0 {
        return Err(anyhow!("count must be greater than 0"));
    }
    let output = helpers::run_git(&["log", "--oneline", &format!("-{n}")])?;
    if output.status.success() {
        let stdout = helpers::stdout_str(&output);
        if stdout.trim().is_empty() {
            println!("(no commits)");
        } else {
            print!("{stdout}");
        }
    } else {
        let msg = helpers::not_a_repo_or_stderr(&output, "git log failed");
        return Err(anyhow!("{msg}"));
    }
    Ok(())
}

// ── Branch ──────────────────────────────────────────────────────────────

pub fn branch() -> Result<()> {
    let output = helpers::run_git(&["branch", "-a"])?;
    if output.status.success() {
        let stdout = helpers::stdout_str(&output);
        print!("{stdout}");
    } else {
        let msg = helpers::not_a_repo_or_stderr(&output, "git branch failed");
        return Err(anyhow!("{msg}"));
    }
    Ok(())
}

// ── Reset ───────────────────────────────────────────────────────────────

pub fn reset(target: Option<&str>, hard: bool) -> Result<()> {
    let mut args = vec!["reset"];
    if hard {
        args.push("--hard");
    }
    if let Some(t) = target {
        args.push(t);
    }

    let output = helpers::run_git(&args)?;
    if output.status.success() {
        let mode = if hard { "--hard" } else { "--soft (default)" };
        let tgt = target.unwrap_or("HEAD");
        println!("✓ Reset {mode} to {tgt}");
    } else {
        let msg = helpers::not_a_repo_or_stderr(&output, "git reset failed");
        return Err(anyhow!("{msg}"));
    }
    Ok(())
}

// ── Download File ───────────────────────────────────────────────────────

pub fn download_file(repo: &str, path: &str, r#ref: Option<&str>, out: &PathBuf) -> Result<()> {
    let token = auth::load_token()?;
    let client = GitHubClient::new(token.as_deref())?;

    let bytes = match r#ref {
        Some(branch) if !branch.trim().is_empty() => {
            client.fetch_file_content_by_ref(repo, path, branch)?
        }
        _ => client.fetch_file_content(repo, path)?,
    };

    if let Some(parent) = out.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    fs::write(out, bytes)?;
    println!("Downloaded {}:{} -> {}", repo, path, out.display());
    Ok(())
}
