use crate::error::GitHubError;
use crate::github::{GitHubClient, MeQuery, with_retry};
use crate::models::{BranchInfo, RepoNode, RepoSummary, SearchResponse, TreeResponse};

#[allow(dead_code)]
impl GitHubClient {
    pub fn search_repositories_page(
        &self,
        query: &str,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<RepoSummary>, GitHubError> {
        let query = query.to_string();
        Self::get_runtime().block_on(self.async_search_repositories_page(query, page, per_page))
    }

    pub(crate) async fn async_search_repositories_page(
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
                return self
                    .async_list_authenticated_repositories(page, per_page, &me_query)
                    .await;
            }

            let api_base = Self::api_base();
            let url = format!(
                "{api_base}/search/repositories?q={}&sort=stars&order=desc&per_page={per_page}&page={page}",
                query.replace(' ', "+"),
            );

            let data: SearchResponse = self.send_and_check_json(self.client.get(url)).await?;
            Ok(data.items)
        })
        .await
    }

    pub(crate) async fn async_list_authenticated_repositories(
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
        })
        .await
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
                let url = format!("{api_base}/repos/{full_name}/branches?per_page=100&page={page}");
                let branches: Vec<BranchInfo> =
                    self.send_and_check_json(self.client.get(&url)).await?;

                let count = branches.len();
                all_branches.extend(branches.into_iter().map(|b| b.name));

                // If fewer than 100 results were returned, this was the last page.
                if count < 100 {
                    break;
                }
                page += 1;
            }

            Ok(all_branches)
        })
        .await
    }

    pub fn fetch_repo_tree(
        &self,
        full_name: &str,
        branch: &str,
    ) -> Result<Vec<RepoNode>, GitHubError> {
        let full_name = full_name.to_string();
        let branch = branch.to_string();
        Self::get_runtime().block_on(self.async_fetch_repo_tree(full_name, branch))
    }

    async fn async_fetch_repo_tree(
        &self,
        full_name: String,
        branch: String,
    ) -> Result<Vec<RepoNode>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let branch_ref: &str = if branch.trim().is_empty() {
                "HEAD"
            } else {
                branch.as_str()
            };
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/git/trees/{branch_ref}?recursive=1");
            let data: TreeResponse = self.send_and_check_json(self.client.get(url)).await?;
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
        })
        .await
    }

    /// Fetch starred repos for the authenticated user.
    pub fn fetch_starred_repos(
        &self,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<RepoSummary>, GitHubError> {
        Self::get_runtime().block_on(self.async_fetch_starred_repos(page, per_page))
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
        Self::get_runtime().block_on(self.async_fetch_repo_by_name(full_name))
    }

    async fn async_fetch_repo_by_name(
        &self,
        full_name: String,
    ) -> Result<RepoSummary, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}");
            let data: RepoSummary = self.send_and_check_json(self.client.get(url)).await?;
            Ok(data)
        })
        .await
    }
}
