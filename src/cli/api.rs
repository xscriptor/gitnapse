use anyhow::{Result, anyhow};

use super::helpers;

// ── PR Commands ─────────────────────────────────────────────────────────

pub fn pr_list(repo: &str, state: &str) -> Result<()> {
    let full_name = helpers::resolve_full_name(repo)?;
    let client = helpers::make_client()?;
    let prs = client
        .fetch_pull_requests(&full_name, state, 30)
        .map_err(|e| anyhow!(helpers::handle_api_error(&full_name, &e)))?;

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
    repo: &str,
    title: &str,
    head: &str,
    base: &str,
    body: Option<&str>,
) -> Result<()> {
    let full_name = helpers::resolve_full_name(repo)?;
    let client = helpers::make_client()?;
    let pr = client
        .create_pull_request(&full_name, title, head, base, body)
        .map_err(|e| anyhow!(helpers::handle_api_error(&full_name, &e)))?;
    println!("✓ PR #{} created: {}", pr.number, pr.html_url);
    Ok(())
}

pub fn pr_merge(repo: &str, number: u64, method: Option<&str>) -> Result<()> {
    let full_name = helpers::resolve_full_name(repo)?;
    let client = helpers::make_client()?;
    let result = client
        .merge_pull_request(&full_name, number, None, method)
        .map_err(|e| anyhow!(helpers::handle_api_error(&full_name, &e)))?;
    if result.merged {
        println!("✓ PR #{number} merged — SHA: {}", result.sha);
    } else {
        println!("✗ Merge failed: {}", result.message);
    }
    Ok(())
}

// ── Issue Commands ──────────────────────────────────────────────────────

pub fn issue_list(repo: &str, state: &str) -> Result<()> {
    let full_name = helpers::resolve_full_name(repo)?;
    let client = helpers::make_client()?;
    let issues = client
        .fetch_issues(&full_name, state, 30)
        .map_err(|e| anyhow!(helpers::handle_api_error(&full_name, &e)))?;

    if issues.is_empty() {
        println!("No {state} issues for {full_name}");
        return Ok(());
    }

    for issue in &issues {
        let pr_tag = if issue.pull_request.is_some() {
            " PR"
        } else {
            ""
        };
        println!(
            "#{:>4} [{:>7}]{} {} (by {})",
            issue.number, issue.state, pr_tag, issue.title, issue.user.login,
        );
    }
    Ok(())
}

pub fn issue_create(repo: &str, title: &str, body: Option<&str>) -> Result<()> {
    let full_name = helpers::resolve_full_name(repo)?;
    let client = helpers::make_client()?;
    let issue = client
        .create_issue(&full_name, title, body)
        .map_err(|e| anyhow!(helpers::handle_api_error(&full_name, &e)))?;
    println!("✓ Issue #{} created: {}", issue.number, issue.html_url);
    Ok(())
}

pub fn issue_close(repo: &str, number: u64) -> Result<()> {
    let full_name = helpers::resolve_full_name(repo)?;
    let client = helpers::make_client()?;
    let issue = client
        .close_issue(&full_name, number)
        .map_err(|e| anyhow!(helpers::handle_api_error(&full_name, &e)))?;
    println!("✓ Issue #{number} closed ({})", issue.html_url);
    Ok(())
}

// ── CI Commands ─────────────────────────────────────────────────────────

pub fn ci_status(repo: &str, branch: Option<&str>, workflows: bool) -> Result<()> {
    let full_name = helpers::resolve_full_name(repo)?;
    let branch = branch.unwrap_or("main");
    let client = helpers::make_client()?;

    if workflows {
        let runs = client
            .fetch_workflow_runs(&full_name, branch, 30)
            .map_err(|e| anyhow!(helpers::handle_api_error(&full_name, &e)))?;
        if runs.is_empty() {
            println!("No workflow runs for {full_name} on {branch}");
            return Ok(());
        }
        for run in &runs {
            let conclusion = run.conclusion.as_deref().unwrap_or(run.status.as_str());
            println!("  [{:>12}] {} ({})", conclusion, run.name, run.status);
        }
        return Ok(());
    }

    let commits = client
        .fetch_recent_commits(&full_name, branch, 1)
        .map_err(|e| anyhow!(helpers::handle_api_error(&full_name, &e)))?;

    let sha = match commits.first() {
        Some(c) => &c.sha,
        None => return Err(anyhow!("no commits found on branch '{branch}'")),
    };

    let runs = client
        .fetch_check_runs(&full_name, sha)
        .map_err(|e| anyhow!(helpers::handle_api_error(&full_name, &e)))?;

    if runs.is_empty() {
        println!("No CI checks for {full_name} on {branch}");
        return Ok(());
    }

    for run in &runs {
        let conclusion = run.conclusion.as_deref().unwrap_or(run.status.as_str());
        println!("  [{:>12}] {} ({})", conclusion, run.name, run.status);
    }
    Ok(())
}

// ── Compare Commands ────────────────────────────────────────────────────

pub fn compare(repo: &str, base: &str, head: &str) -> Result<()> {
    let full_name = helpers::resolve_full_name(repo)?;
    let client = helpers::make_client()?;
    let cmp = client
        .fetch_compare(&full_name, base, head)
        .map_err(|e| anyhow!(helpers::handle_api_error(&full_name, &e)))?;

    println!(
        "{} — {} ahead, {} behind, {} files changed",
        cmp.status,
        cmp.ahead_by,
        cmp.behind_by,
        cmp.files.len()
    );

    for file in &cmp.files {
        println!(
            "  {:>8} {:>5}+{:<5} {}",
            file.status, file.additions, file.deletions, file.filename
        );
    }
    Ok(())
}

// ── Release Commands ────────────────────────────────────────────────────

pub fn release_list(repo: &str) -> Result<()> {
    let full_name = helpers::resolve_full_name(repo)?;
    let client = helpers::make_client()?;
    let releases = client
        .fetch_releases(&full_name, 30)
        .map_err(|e| anyhow!(helpers::handle_api_error(&full_name, &e)))?;

    if releases.is_empty() {
        println!("No releases for {full_name}");
        return Ok(());
    }

    for r in &releases {
        let pre = if r.prerelease { " (pre-release)" } else { "" };
        let name = r.name.as_deref().unwrap_or(&r.tag_name);
        println!("  {} [{name}]{pre}", r.tag_name);
    }
    Ok(())
}

pub fn release_create(
    repo: &str,
    tag_name: &str,
    name: Option<&str>,
    body: Option<&str>,
    prerelease: bool,
) -> Result<()> {
    let full_name = helpers::resolve_full_name(repo)?;
    let client = helpers::make_client()?;
    let release = client
        .create_release(&full_name, tag_name, name, body, prerelease)
        .map_err(|e| anyhow!(helpers::handle_api_error(&full_name, &e)))?;
    println!(
        "✓ Release {} created: {}",
        release.tag_name, release.html_url
    );
    Ok(())
}

// ── Repo Commands ───────────────────────────────────────────────────────

pub fn repo_create(name: &str, description: Option<&str>, private: bool) -> Result<()> {
    let client = helpers::make_client()?;
    let repo = client
        .create_repo(name, description, private)
        .map_err(|e| anyhow!("{e}"))?;
    println!("✓ Repository created: {}", repo.clone_url);
    Ok(())
}

// ── Search Commands ─────────────────────────────────────────────────────

pub fn search(query: &str) -> Result<()> {
    let client = helpers::make_client()?;
    let repos = client
        .search_repositories_page(query, 1, 30)
        .map_err(|e| anyhow!("{e}"))?;

    if repos.is_empty() {
        println!("No results for '{query}'");
        return Ok(());
    }

    for repo in &repos {
        let lang = repo.language.as_deref().unwrap_or("-");
        println!(
            "  {} ★{} [{}] {}",
            repo.full_name,
            repo.stargazers_count,
            lang,
            repo.description.as_deref().unwrap_or(""),
        );
    }
    Ok(())
}
