use crate::models::{
    AuthenticatedUser, BranchInfo, ContentResponse, RepoNode, RepoSummary, SearchResponse, TreeResponse,
};
use anyhow::{Context, Result, anyhow};
use base64::Engine;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};

const GITHUB_API: &str = "https://api.github.com";

pub struct GitHubClient {
    client: Client,
}

impl GitHubClient {
    pub fn new(token: Option<&str>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("gitnapse/0.1"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.github+json"));

        if let Some(token) = token.filter(|t| !t.trim().is_empty()) {
            let value = HeaderValue::from_str(&format!("Bearer {}", token.trim()))
                .context("Invalid token value for HTTP header")?;
            headers.insert(AUTHORIZATION, value);
        }

        let client = Client::builder().default_headers(headers).build()?;
        Ok(Self { client })
    }

    pub fn search_repositories_page(&self, query: &str, page: u32, per_page: u8) -> Result<Vec<RepoSummary>> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(Vec::new());
        }

        let page = page.max(1);
        let per_page = per_page.clamp(1, 100);
        let url = format!(
            "{GITHUB_API}/search/repositories?q={}&sort=stars&order=desc&per_page={per_page}&page={page}",
            query.replace(' ', "+"),
        );

        let response = self
            .client
            .get(url)
            .send()
            .context("Network error while searching repositories")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(anyhow!("GitHub search failed ({status}): {body}"));
        }

        let data: SearchResponse = response.json().context("Invalid search response from GitHub")?;
        Ok(data.items)
    }

    pub fn fetch_branches(&self, full_name: &str) -> Result<Vec<String>> {
        let url = format!("{GITHUB_API}/repos/{full_name}/branches?per_page=100");
        let response = self
            .client
            .get(url)
            .send()
            .context("Network error while fetching branches")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(anyhow!("GitHub branch fetch failed ({status}): {body}"));
        }

        let branches: Vec<BranchInfo> = response.json().context("Invalid branch response from GitHub")?;
        Ok(branches.into_iter().map(|b| b.name).collect())
    }

    pub fn fetch_repo_tree(&self, full_name: &str, branch: &str) -> Result<Vec<RepoNode>> {
        let branch = if branch.trim().is_empty() { "HEAD" } else { branch };
        let url = format!("{GITHUB_API}/repos/{full_name}/git/trees/{branch}?recursive=1");

        let response = self
            .client
            .get(url)
            .send()
            .context("Network error while fetching repository tree")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(anyhow!("GitHub tree fetch failed ({status}): {body}"));
        }

        let data: TreeResponse = response.json().context("Invalid tree response from GitHub")?;
        let mut nodes = data
            .tree
            .into_iter()
            .map(|entry| {
                let name = entry
                    .path
                    .rsplit('/')
                    .next()
                    .unwrap_or(entry.path.as_str())
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
    }

    pub fn fetch_file_content(&self, full_name: &str, path: &str) -> Result<String> {
        self.fetch_file_content_by_ref(full_name, path, "")
    }

    pub fn fetch_file_content_by_ref(&self, full_name: &str, path: &str, git_ref: &str) -> Result<String> {
        let url = if git_ref.trim().is_empty() {
            format!("{GITHUB_API}/repos/{full_name}/contents/{path}")
        } else {
            format!(
                "{GITHUB_API}/repos/{full_name}/contents/{path}?ref={}",
                git_ref.trim()
            )
        };
        let response = self
            .client
            .get(url)
            .send()
            .context("Network error while fetching file content")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(anyhow!("GitHub content fetch failed ({status}): {body}"));
        }

        let data: ContentResponse = response.json().context("Invalid content response from GitHub")?;
        if data.encoding != "base64" {
            return Err(anyhow!("Unsupported file encoding: {}", data.encoding));
        }

        let normalized = data.content.replace('\n', "");
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(normalized)
            .context("Cannot decode base64 file content")?;
        let text = String::from_utf8_lossy(&bytes).to_string();
        Ok(text)
    }

    pub fn fetch_authenticated_user(&self) -> Result<Option<String>> {
        let response = self
            .client
            .get(format!("{GITHUB_API}/user"))
            .send()
            .context("Network error while validating token")?;

        if response.status().as_u16() == 401 {
            return Ok(None);
        }
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(anyhow!("GitHub user lookup failed ({status}): {body}"));
        }
        let user: AuthenticatedUser = response.json().context("Invalid user response from GitHub")?;
        Ok(Some(user.login))
    }
}
