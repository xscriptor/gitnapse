use crate::error::GitHubError;
use crate::models::{
    AuthenticatedUser, BranchInfo, CheckRun, CheckRunsResponse, CommitInfo, CompareResponse,
    ContentResponse, Issue, PullRequest, RepoNode, RepoSummary, SearchResponse, TreeResponse,
    WorkflowRun, WorkflowRunsResponse,
};
use base64::Engine;
use reqwest::Client;
use reqwest::Response;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::runtime::Runtime;

const GITHUB_API: &str = "https://api.github.com";

// ── Retry helpers ────────────────────────────────────────────────────

/// Retry a fallible operation up to 3 times when it fails with a network error
/// (for functions that use [`GitHubError`]).
///
/// Non‑network errors are propagated immediately. A short sleep is inserted
/// between retries.
async fn with_retry<F, Fut, T>(f: F) -> Result<T, GitHubError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, GitHubError>>,
{
    let mut last_err = None;
    for attempt in 0..3 {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                if matches!(&e, GitHubError::Network(_)) && attempt < 2 {
                    tokio::time::sleep(Duration::from_millis(500 * (attempt as u64 + 1))).await;
                    last_err = Some(e);
                    continue;
                }
                return Err(e);
            }
        }
    }
    Err(last_err.unwrap_or(GitHubError::Other("Retry exhausted".into())))
}



// ── Client ───────────────────────────────────────────────────────────

pub struct GitHubClient {
    client: Client,
    rate_limit_remaining: Mutex<Option<u32>>,
    rate_limit_reset: Mutex<Option<u64>>,
}

#[derive(Debug, Clone)]
struct MeQuery {
    text_terms: Vec<String>,
    languages: Vec<String>,
}

impl GitHubClient {
    fn api_base() -> String {
        std::env::var("GITNAPSE_GITHUB_API")
            .ok()
            .map(|v| v.trim().trim_end_matches('/').to_string())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| GITHUB_API.to_string())
    }

    /// Parse a `@me` or `me:` query into structured terms and languages.
    ///
    /// Recognised forms:
    ///   - `@me`                         — all authenticated repos
    ///   - `@me   rust`                  — repos matching "rust" (any whitespace after @me)
    ///   - `@me language:rust,go`        — filter by language(s)
    ///   - `me:rust`                     — shorthand me: prefix
    ///
    /// Returns `None` when the query does **not** start with `@me` / `me:`.
    /// `@me,rust` or `@mex` are *not* treated as `@me` queries.
    fn parse_me_query(query: &str) -> Option<MeQuery> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return None;
        }

        let rest = if trimmed.eq_ignore_ascii_case("@me") {
            ""
        } else if trimmed.len() >= 3
            && trimmed[..3].eq_ignore_ascii_case("@me")
            && (trimmed.len() == 3
                || trimmed[3..].starts_with(|c: char| c.is_whitespace()))
        {
            // @me followed by whitespace (or exact @me caught above)
            trimmed[3..].trim()
        } else if let Some(rest) = trimmed.strip_prefix("me:") {
            // me: prefix — rest may be empty (e.g. just "me:")
            rest.trim()
        } else {
            return None;
        };

        let mut text_terms = Vec::new();
        let mut languages = Vec::new();
        for raw in rest.split_whitespace() {
            if let Some(lang_expr) = raw
                .strip_prefix("language:")
                .or_else(|| raw.strip_prefix("lang:"))
            {
                for lang in lang_expr.split(',') {
                    let lang = lang.trim().to_lowercase();
                    if !lang.is_empty() {
                        languages.push(lang);
                    }
                }
            } else {
                let term = raw.trim().to_lowercase();
                if !term.is_empty() {
                    text_terms.push(term);
                }
            }
        }

        Some(MeQuery {
            text_terms,
            languages,
        })
    }

    // ── Rate-limit helpers ──────────────────────────────────────────────

    /// Public read‑only accessor for the last known `x-ratelimit-remaining` value.
    #[allow(dead_code)]
    pub fn rate_limit_remaining(&self) -> Option<u32> {
        *self.rate_limit_remaining.lock().unwrap()
    }

    /// Public read‑only accessor for the last known `x-ratelimit-reset` (Unix timestamp).
    #[allow(dead_code)]
    pub fn rate_limit_reset(&self) -> Option<u64> {
        *self.rate_limit_reset.lock().unwrap()
    }

    /// Extract rate‑limit headers from an HTTP response and cache them on `self`.
    fn update_rate_limit_from_response(&self, response: &Response) {
        if let Some(remaining) = response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
            && let Ok(mut guard) = self.rate_limit_remaining.lock() {
                *guard = Some(remaining);
            }
        if let Some(reset) = response
            .headers()
            .get("x-ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            && let Ok(mut guard) = self.rate_limit_reset.lock() {
                *guard = Some(reset);
            }
    }

    /// Return an error immediately if we already know the rate limit is exhausted.
    fn check_rate_limit(&self) -> Result<(), GitHubError> {
        let remaining = self.rate_limit_remaining.lock().unwrap();
        if let Some(0) = *remaining {
            let reset = self.rate_limit_reset.lock().unwrap();
            if let Some(reset_ts) = *reset {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                if reset_ts > now {
                    return Err(GitHubError::RateLimited { remaining: 0, reset: reset_ts });
                }
            }
            return Err(GitHubError::RateLimited { remaining: 0, reset: 0 });
        }
        Ok(())
    }

    // ── User‑facing methods ─────────────────────────────────────────────

    async fn async_list_authenticated_repositories(
        &self,
        page: u32,
        per_page: u8,
        query: &MeQuery,
    ) -> Result<Vec<RepoSummary>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!(
                "{api_base}/user/repos?visibility=all&affiliation=owner,collaborator,organization_member&sort=updated&direction=desc&per_page={per_page}&page={page}"
            );

            let response = self
                .client
                .get(url)
                .send().await?;
            self.update_rate_limit_from_response(&response);

            if response.status().as_u16() == 401 {
                return Err(GitHubError::Unauthorized);
            }
            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(GitHubError::Api { status: status.as_u16(), body });
            }

            let mut repos: Vec<RepoSummary> = response.json().await?;

            repos.retain(|repo| {
                let language_match = if query.languages.is_empty() {
                    true
                } else {
                    repo.language
                        .as_deref()
                        .map(|lang| lang.to_lowercase())
                        .map(|lang| query.languages.iter().any(|candidate| candidate == &lang))
                        .unwrap_or(false)
                };
                if !language_match {
                    return false;
                }

                if query.text_terms.is_empty() {
                    return true;
                }

                let full_name_lower = repo.full_name.to_lowercase();
                let name_lower = repo.name.to_lowercase();
                let desc_lower = repo.description.as_ref().map(|d| d.to_lowercase());
                query.text_terms.iter().all(|term| {
                    full_name_lower.contains(term)
                        || name_lower.contains(term)
                        || desc_lower.as_deref().is_some_and(|d| d.contains(term))
                })
            });

            Ok(repos)
        }).await
    }

    pub fn new(token: Option<&str>) -> Result<Self, GitHubError> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("gitnapse/0.1"));
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );

        if let Some(token) = token.filter(|t| !t.trim().is_empty()) {
            let value = HeaderValue::from_str(&format!("Bearer {}", token.trim()))
                .map_err(|e| GitHubError::Other(format!("Invalid token value for HTTP header: {e}")))?;
            headers.insert(AUTHORIZATION, value);
        }

        let client = Client::builder().default_headers(headers).build()?;
        Ok(Self {
            client,
            rate_limit_remaining: Mutex::new(None),
            rate_limit_reset: Mutex::new(None),
        })
    }

    pub fn search_repositories_page(
        &self,
        query: &str,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<RepoSummary>, GitHubError> {
        let query = query.to_string();
        Self::get_runtime().block_on(self.async_search_repositories_page(query, page, per_page))
    }

    async fn async_search_repositories_page(
        &self,
        query: String,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<RepoSummary>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let query = query.trim();
            if query.is_empty() {
                return Ok(Vec::new());
            }

            let page = page.max(1);
            let per_page = per_page.clamp(1, 100);
            if let Some(me_query) = Self::parse_me_query(query) {
                return self.async_list_authenticated_repositories(page, per_page, &me_query).await;
            }

            let api_base = Self::api_base();
            let url = format!(
                "{api_base}/search/repositories?q={}&sort=stars&order=desc&per_page={per_page}&page={page}",
                query.replace(' ', "+"),
            );

            let response = self
                .client
                .get(url)
                .send().await?;
            self.update_rate_limit_from_response(&response);

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(GitHubError::Api { status: status.as_u16(), body });
            }

            let data: SearchResponse = response.json().await?;
            Ok(data.items)
        }).await
    }

    /// Fetch all branches for a repository, following pagination automatically.
    ///
    /// The GitHub Branches API returns up to 100 branches per page. This method
    /// iterates over every page until an empty response is received.
    pub fn fetch_branches(&self, full_name: &str) -> Result<Vec<String>, GitHubError> {
        let full_name = full_name.to_string();
        Self::get_runtime().block_on(self.async_fetch_branches(full_name))
    }

    async fn async_fetch_branches(&self, full_name: String) -> Result<Vec<String>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let mut all_branches = Vec::new();
            let mut page: u32 = 1;

            loop {
                let url = format!(
                    "{api_base}/repos/{full_name}/branches?per_page=100&page={page}"
                );
                let response = self
                    .client
                    .get(&url)
                    .send().await?;
                self.update_rate_limit_from_response(&response);

                if !response.status().is_success() {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    return Err(GitHubError::Api { status: status.as_u16(), body });
                }

                let branches: Vec<BranchInfo> = response.json().await?;

                let count = branches.len();
                all_branches.extend(branches.into_iter().map(|b| b.name));

                // If fewer than 100 results were returned, this was the last page.
                if count < 100 {
                    break;
                }
                page += 1;
            }

            Ok(all_branches)
        }).await
    }

    pub fn fetch_repo_tree(&self, full_name: &str, branch: &str) -> Result<Vec<RepoNode>, GitHubError> {
        let full_name = full_name.to_string();
        let branch = branch.to_string();
        Self::get_runtime().block_on(self.async_fetch_repo_tree(full_name, branch))
    }

    async fn async_fetch_repo_tree(&self, full_name: String, branch: String) -> Result<Vec<RepoNode>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let branch_ref: &str = if branch.trim().is_empty() {
                "HEAD"
            } else {
                branch.as_str()
            };
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/git/trees/{branch_ref}?recursive=1");

            let response = self
                .client
                .get(url)
                .send().await?;
            self.update_rate_limit_from_response(&response);

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(GitHubError::Api { status: status.as_u16(), body });
            }

            let data: TreeResponse = response.json().await?;
            let mut nodes = data
                .tree
                .into_iter()
                .map(|entry| {
                    let name = entry
                        .path
                        .rsplit_once('/')
                        .map(|(_, name)| name)
                        .unwrap_or(&entry.path)
                        .to_string();
                    let depth = entry.path.matches('/').count();
                    RepoNode {
                        path: entry.path,
                        name,
                        depth,
                        is_dir: entry.kind == "tree",
                    }
                })
                .collect::<Vec<_>>();

            nodes.sort_by(|a, b| {
                a.path
                    .to_lowercase()
                    .cmp(&b.path.to_lowercase())
                    .then_with(|| b.is_dir.cmp(&a.is_dir))
            });
            Ok(nodes)
        }).await
    }

    pub fn fetch_file_content(&self, full_name: &str, path: &str) -> anyhow::Result<Vec<u8>> {
        self.fetch_file_content_by_ref(full_name, path, "")
    }

    pub fn fetch_file_content_by_ref(
        &self,
        full_name: &str,
        path: &str,
        git_ref: &str,
    ) -> anyhow::Result<Vec<u8>> {
        let full_name = full_name.to_string();
        let path = path.to_string();
        let git_ref = git_ref.to_string();
        Self::get_runtime().block_on(self.async_fetch_file_content_by_ref(full_name, path, git_ref))
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    async fn async_fetch_file_content_by_ref(
        &self,
        full_name: String,
        path: String,
        git_ref: String,
    ) -> Result<Vec<u8>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = if git_ref.trim().is_empty() {
                format!("{api_base}/repos/{full_name}/contents/{path}")
            } else {
                format!(
                    "{api_base}/repos/{full_name}/contents/{path}?ref={}",
                    git_ref.trim()
                )
            };
            let response = self
                .client
                .get(&url)
                .send().await?;
            self.update_rate_limit_from_response(&response);

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();

                // 403 is commonly returned when the Contents API hits the 1 MB size limit.
                // Try the Blob API as a fallback.
                if status.as_u16() == 403
                    && let Ok(bytes) = self.async_fetch_file_content_via_blob_api(&full_name, &path, &git_ref).await {
                        return Ok(bytes);
                    }

                return Err(GitHubError::Api { status: status.as_u16(), body });
            }

            let data: ContentResponse = response.json().await?;
            if data.encoding != "base64" {
                return Err(GitHubError::Other(format!("Unsupported file encoding: {}", data.encoding)));
            }

            let normalized = data.content.replace('\n', "");
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(normalized)?;
            Ok(bytes)
        }).await
    }

    /// Fetch file content via the Git Blobs API, bypassing the 1 MB limit of
    /// the Contents API.
    ///
    /// This method first obtains the file SHA (either through the Contents API
    /// for files ≤ 1 MB, or by scanning the Git tree for larger files) and then
    /// retrieves the full blob contents.
    #[allow(dead_code)]
    pub fn fetch_file_content_via_blob_api(
        &self,
        full_name: &str,
        path: &str,
        branch: &str,
    ) -> Result<Vec<u8>, GitHubError> {
        let full_name = full_name.to_string();
        let path = path.to_string();
        let branch = branch.to_string();
        Self::get_runtime().block_on(self.async_fetch_file_content_via_blob_api(&full_name, &path, &branch))
    }

    async fn async_fetch_file_content_via_blob_api(
        &self,
        full_name: &str,
        path: &str,
        branch: &str,
    ) -> Result<Vec<u8>, GitHubError> {
        self.check_rate_limit()?;
        let api_base = Self::api_base();
        let branch = if branch.trim().is_empty() { "HEAD" } else { branch };

        // ── Step 1: obtain the file SHA ──────────────────────────────
        let sha = self.async_fetch_file_sha(full_name, path, branch).await?;

        // ── Step 2: retrieve the blob ────────────────────────────────
        let blob_url = format!("{api_base}/repos/{full_name}/git/blobs/{sha}");
        let blob_response = self
            .client
            .get(&blob_url)
            .send().await?;
        self.update_rate_limit_from_response(&blob_response);

        if !blob_response.status().is_success() {
            let status = blob_response.status();
            let body = blob_response.text().await.unwrap_or_default();
            return Err(GitHubError::Api { status: status.as_u16(), body });
        }

        let blob: ContentResponse = blob_response.json().await?;
        if blob.encoding != "base64" {
            return Err(GitHubError::Encoding(blob.encoding));
        }

        let normalized = blob.content.replace('\n', "");
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(normalized)?;
        Ok(bytes)
    }

    /// Obtain the Git SHA of a file at the given path and branch.
    ///
    /// Tries the Contents API first (fast path for files ≤ 1 MB). If that
    /// fails with a 403 (size limit), falls back to scanning the recursive
    /// Git tree.
    async fn async_fetch_file_sha(&self, full_name: &str, path: &str, branch: &str) -> Result<String, GitHubError> {
        let api_base = Self::api_base();

        // ── Try the Contents API first ───────────────────────────────
        let contents_url = format!("{api_base}/repos/{full_name}/contents/{path}?ref={branch}");
        let response = self
            .client
            .get(&contents_url)
            .send().await?;
        self.update_rate_limit_from_response(&response);

        if response.status().is_success() {
            let meta: ContentResponse = response.json().await?;
            return Ok(meta.sha);
        }

        // If the Contents API fails with 403 (likely due to file size), fall
        // back to the Tree API.  Any other status is a real error.
        if response.status().as_u16() == 403 {
            return self.async_fetch_file_sha_via_tree(full_name, path, branch).await;
        }

        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(GitHubError::Api { status: status.as_u16(), body })
    }

    /// Scan the recursive Git tree to find the SHA of a file.
    async fn async_fetch_file_sha_via_tree(
        &self,
        full_name: &str,
        path: &str,
        branch: &str,
    ) -> Result<String, GitHubError> {
        let api_base = Self::api_base();
        let url = format!("{api_base}/repos/{full_name}/git/trees/{branch}?recursive=1");
        let response = self
            .client
            .get(&url)
            .send().await?;
        self.update_rate_limit_from_response(&response);

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(GitHubError::Api { status: status.as_u16(), body });
        }

        let tree: TreeResponse = response.json().await?;

        tree.tree
            .into_iter()
            .find(|entry| entry.path == path)
            .map(|entry| entry.sha)
            .ok_or_else(|| GitHubError::NotFound(format!("File '{}' not found in tree for '{}/{}'", path, full_name, branch)))
    }

    pub fn fetch_authenticated_user(&self) -> Result<Option<String>, GitHubError> {
        Self::get_runtime().block_on(self.async_fetch_authenticated_user())
    }

    async fn async_fetch_authenticated_user(&self) -> Result<Option<String>, GitHubError> {
        self.check_rate_limit()?;
        let api_base = Self::api_base();
        let response = self
            .client
            .get(format!("{api_base}/user"))
            .send().await?;
        self.update_rate_limit_from_response(&response);

        if response.status().as_u16() == 401 {
            return Ok(None);
        }
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(GitHubError::Api { status: status.as_u16(), body });
        }
        let user: AuthenticatedUser = response.json().await?;
        Ok(Some(user.login))
    }

    // ── New API methods ──────────────────────────────────────────────────

    /// Fetch recent commits for a branch.
    pub fn fetch_recent_commits(
        &self,
        full_name: &str,
        branch: &str,
        per_page: u8,
    ) -> Result<Vec<CommitInfo>, GitHubError> {
        let full_name = full_name.to_string();
        let branch = branch.to_string();
        Self::get_runtime()
            .block_on(self.async_fetch_recent_commits(full_name, branch, per_page))
    }

    async fn async_fetch_recent_commits(
        &self,
        full_name: String,
        branch: String,
        per_page: u8,
    ) -> Result<Vec<CommitInfo>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!(
                "{api_base}/repos/{full_name}/commits?sha={branch}&per_page={per_page}"
            );
            let response = self.client.get(url).send().await?;
            self.update_rate_limit_from_response(&response);

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(GitHubError::Api {
                    status: status.as_u16(),
                    body,
                });
            }

            let data: Vec<CommitInfo> = response.json().await?;
            Ok(data)
        })
        .await
    }

    /// Compare two commits / branches.
    pub fn fetch_compare(
        &self,
        full_name: &str,
        base: &str,
        head: &str,
    ) -> Result<CompareResponse, GitHubError> {
        let full_name = full_name.to_string();
        let base = base.to_string();
        let head = head.to_string();
        Self::get_runtime()
            .block_on(self.async_fetch_compare(full_name, base, head))
    }

    async fn async_fetch_compare(
        &self,
        full_name: String,
        base: String,
        head: String,
    ) -> Result<CompareResponse, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/compare/{base}...{head}");
            let response = self.client.get(url).send().await?;
            self.update_rate_limit_from_response(&response);

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(GitHubError::Api {
                    status: status.as_u16(),
                    body,
                });
            }

            let data: CompareResponse = response.json().await?;
            Ok(data)
        })
        .await
    }

    /// Fetch issues for a repository.
    pub fn fetch_issues(
        &self,
        full_name: &str,
        state: &str,
        per_page: u8,
    ) -> Result<Vec<Issue>, GitHubError> {
        let full_name = full_name.to_string();
        let state = state.to_string();
        Self::get_runtime()
            .block_on(self.async_fetch_issues(full_name, state, per_page))
    }

    async fn async_fetch_issues(
        &self,
        full_name: String,
        state: String,
        per_page: u8,
    ) -> Result<Vec<Issue>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!(
                "{api_base}/repos/{full_name}/issues?state={state}&per_page={per_page}&sort=updated"
            );
            let response = self.client.get(url).send().await?;
            self.update_rate_limit_from_response(&response);

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(GitHubError::Api {
                    status: status.as_u16(),
                    body,
                });
            }

            let data: Vec<Issue> = response.json().await?;
            Ok(data)
        })
        .await
    }

    /// Fetch pull requests for a repository.
    pub fn fetch_pull_requests(
        &self,
        full_name: &str,
        state: &str,
        per_page: u8,
    ) -> Result<Vec<PullRequest>, GitHubError> {
        let full_name = full_name.to_string();
        let state = state.to_string();
        Self::get_runtime()
            .block_on(self.async_fetch_pull_requests(full_name, state, per_page))
    }

    async fn async_fetch_pull_requests(
        &self,
        full_name: String,
        state: String,
        per_page: u8,
    ) -> Result<Vec<PullRequest>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!(
                "{api_base}/repos/{full_name}/pulls?state={state}&per_page={per_page}&sort=updated"
            );
            let response = self.client.get(url).send().await?;
            self.update_rate_limit_from_response(&response);

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(GitHubError::Api {
                    status: status.as_u16(),
                    body,
                });
            }

            let data: Vec<PullRequest> = response.json().await?;
            Ok(data)
        })
        .await
    }

    /// Fetch check runs for a specific commit ref.
    pub fn fetch_check_runs(
        &self,
        full_name: &str,
        ref_: &str,
    ) -> Result<Vec<CheckRun>, GitHubError> {
        let full_name = full_name.to_string();
        let ref_ = ref_.to_string();
        Self::get_runtime()
            .block_on(self.async_fetch_check_runs(full_name, ref_))
    }

    async fn async_fetch_check_runs(
        &self,
        full_name: String,
        ref_: String,
    ) -> Result<Vec<CheckRun>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/commits/{ref_}/check-runs");
            let response = self.client.get(url).send().await?;
            self.update_rate_limit_from_response(&response);

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(GitHubError::Api {
                    status: status.as_u16(),
                    body,
                });
            }

            let data: CheckRunsResponse = response.json().await?;
            Ok(data.check_runs)
        })
        .await
    }

    /// Fetch workflow runs for a branch.
    #[allow(dead_code)]
    pub fn fetch_workflow_runs(
        &self,
        full_name: &str,
        branch: &str,
        per_page: u8,
    ) -> Result<Vec<WorkflowRun>, GitHubError> {
        let full_name = full_name.to_string();
        let branch = branch.to_string();
        Self::get_runtime()
            .block_on(self.async_fetch_workflow_runs(full_name, branch, per_page))
    }

    async fn async_fetch_workflow_runs(
        &self,
        full_name: String,
        branch: String,
        per_page: u8,
    ) -> Result<Vec<WorkflowRun>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!(
                "{api_base}/repos/{full_name}/actions/runs?branch={branch}&per_page={per_page}"
            );
            let response = self.client.get(url).send().await?;
            self.update_rate_limit_from_response(&response);

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(GitHubError::Api {
                    status: status.as_u16(),
                    body,
                });
            }

            let data: WorkflowRunsResponse = response.json().await?;
            Ok(data.workflow_runs)
        })
        .await
    }

    /// Fetch starred repos for the authenticated user.
    pub fn fetch_starred_repos(
        &self,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<RepoSummary>, GitHubError> {
        Self::get_runtime()
            .block_on(self.async_fetch_starred_repos(page, per_page))
    }

    async fn async_fetch_starred_repos(
        &self,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<RepoSummary>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!(
                "{api_base}/user/starred?per_page={per_page}&page={page}&sort=created&direction=desc"
            );
            let response = self.client.get(url).send().await?;
            self.update_rate_limit_from_response(&response);

            if response.status().as_u16() == 401 {
                return Err(GitHubError::Unauthorized);
            }
            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(GitHubError::Api {
                    status: status.as_u16(),
                    body,
                });
            }

            let data: Vec<RepoSummary> = response.json().await?;
            Ok(data)
        })
        .await
    }

    /// Fetch a single repository by full name (e.g. "owner/repo").
    #[allow(dead_code)]
    pub fn fetch_repo_by_name(&self, full_name: &str) -> Result<RepoSummary, GitHubError> {
        let full_name = full_name.to_string();
        Self::get_runtime()
            .block_on(self.async_fetch_repo_by_name(full_name))
    }

    async fn async_fetch_repo_by_name(
        &self,
        full_name: String,
    ) -> Result<RepoSummary, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}");
            let response = self.client.get(url).send().await?;
            self.update_rate_limit_from_response(&response);

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(GitHubError::Api {
                    status: status.as_u16(),
                    body,
                });
            }

            let data: RepoSummary = response.json().await?;
            Ok(data)
        })
        .await
    }

    pub fn get_runtime() -> &'static Runtime {
        static RUNTIME: OnceLock<Runtime> = OnceLock::new();
        RUNTIME.get_or_init(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Cannot create global tokio runtime for GitHubClient")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_me_query tests ────────────────────────────────────────────

    #[test]
    fn test_parse_me_exact() {
        let q = GitHubClient::parse_me_query("@me");
        assert!(q.is_some());
        let q = q.unwrap();
        assert!(q.text_terms.is_empty());
        assert!(q.languages.is_empty());
    }

    #[test]
    fn test_parse_me_case_insensitive() {
        let q = GitHubClient::parse_me_query("@Me");
        assert!(q.is_some());
        let q = q.unwrap();
        assert!(q.text_terms.is_empty());
    }

    #[test]
    fn test_parse_me_with_terms() {
        let q = GitHubClient::parse_me_query("@me rust");
        assert!(q.is_some());
        let q = q.unwrap();
        assert_eq!(q.text_terms, vec!["rust"]);
        assert!(q.languages.is_empty());
    }

    #[test]
    fn test_parse_me_multiple_spaces() {
        let q = GitHubClient::parse_me_query("@me   rust");
        assert!(q.is_some());
        let q = q.unwrap();
        assert_eq!(q.text_terms, vec!["rust"]);
    }

    #[test]
    fn test_parse_me_with_language() {
        let q = GitHubClient::parse_me_query("@me language:rust");
        assert!(q.is_some());
        let q = q.unwrap();
        assert!(q.text_terms.is_empty());
        assert_eq!(q.languages, vec!["rust"]);
    }

    #[test]
    fn test_parse_me_comma_rejected() {
        assert!(GitHubClient::parse_me_query("@me,rust").is_none());
        assert!(GitHubClient::parse_me_query("@me,").is_none());
    }

    #[test]
    fn test_parse_me_special_chars() {
        let q = GitHubClient::parse_me_query("@me foo/bar");
        assert!(q.is_some());
        let q = q.unwrap();
        assert_eq!(q.text_terms, vec!["foo/bar"]);
    }

    #[test]
    fn test_parse_me_exact_me_colon() {
        let q = GitHubClient::parse_me_query("me:");
        assert!(q.is_some());
        let q = q.unwrap();
        assert!(q.text_terms.is_empty());
        assert!(q.languages.is_empty());
    }

    #[test]
    fn test_parse_me_me_colon_with_terms() {
        let q = GitHubClient::parse_me_query("me:rust");
        assert!(q.is_some());
        let q = q.unwrap();
        assert_eq!(q.text_terms, vec!["rust"]);
    }

    #[test]
    fn test_parse_me_me_colon_multiple_languages() {
        let q = GitHubClient::parse_me_query("me: language:rust,go");
        assert!(q.is_some());
        let q = q.unwrap();
        assert!(q.text_terms.is_empty());
        assert_eq!(q.languages, vec!["rust", "go"]);
    }

    #[test]
    fn test_parse_me_not_triggered() {
        // Not a real @me query
        assert!(GitHubClient::parse_me_query("search term").is_none());
        assert!(GitHubClient::parse_me_query("@mememe").is_none());
        assert!(GitHubClient::parse_me_query("@").is_none());
        assert!(GitHubClient::parse_me_query("").is_none());
    }
}
