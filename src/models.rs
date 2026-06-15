#![allow(dead_code)]
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RepoSummary {
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub stargazers_count: u64,
    pub language: Option<String>,
    pub clone_url: String,
    pub owner: RepoOwner,
    pub default_branch: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepoOwner {
    pub login: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub items: Vec<RepoSummary>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BranchInfo {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct TreeResponse {
    pub tree: Vec<TreeEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TreeEntry {
    pub path: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub sha: String,
}

#[derive(Debug, Clone)]
pub struct RepoNode {
    pub path: String,
    pub name: String,
    pub depth: usize,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContentResponse {
    pub sha: String,
    pub content: String,
    pub encoding: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthenticatedUser {
    pub login: String,
}

/// A commit in a repository
#[derive(Debug, Clone, Deserialize)]
pub struct CommitInfo {
    pub sha: String,
    pub commit: CommitDetails,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommitDetails {
    pub message: String,
    pub author: CommitAuthor,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommitAuthor {
    pub name: String,
    pub date: String,
}

/// A comparison between two commits (for diffs)
#[derive(Debug, Deserialize)]
pub struct CompareResponse {
    pub status: String,
    pub ahead_by: u32,
    pub behind_by: u32,
    pub total_commits: u32,
    pub files: Vec<DiffFile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiffFile {
    pub filename: String,
    pub status: String,
    pub additions: u32,
    pub deletions: u32,
    pub changes: u32,
    pub patch: Option<String>,
}

/// A GitHub Issue
#[derive(Debug, Clone, Deserialize)]
pub struct Issue {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub html_url: String,
    pub user: IssueUser,
    pub labels: Vec<IssueLabel>,
    pub created_at: String,
    pub updated_at: String,
    pub body: Option<String>,
    pub pull_request: Option<PrInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IssueUser {
    pub login: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IssueLabel {
    pub name: String,
    pub color: String,
}

/// Minimal PR info (present in issue if it's a PR)
#[derive(Debug, Clone, Deserialize)]
pub struct PrInfo {
    pub url: Option<String>,
    pub html_url: Option<String>,
}

/// A Pull Request
#[derive(Debug, Clone, Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub html_url: String,
    pub user: IssueUser,
    pub body: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub additions: Option<u32>,
    pub deletions: Option<u32>,
    pub changed_files: Option<u32>,
}

/// CI check run
#[derive(Debug, Deserialize)]
pub struct CheckRunsResponse {
    pub check_runs: Vec<CheckRun>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CheckRun {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub html_url: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// Workflow run
#[derive(Debug, Deserialize)]
pub struct WorkflowRunsResponse {
    pub workflow_runs: Vec<WorkflowRun>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowRun {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
}
