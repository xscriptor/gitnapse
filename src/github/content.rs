use crate::error::GitHubError;
use crate::github::{GitHubClient, with_retry};
use crate::models::{AuthenticatedUser, ContentResponse, TreeResponse};
use base64::Engine;

#[allow(dead_code)]
impl GitHubClient {
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
        Self::get_runtime()
            .block_on(self.async_fetch_file_content_by_ref(full_name, path, git_ref))
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    pub(crate) async fn async_fetch_file_content_by_ref(
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
            let response = self.client.get(&url).send().await?;
            self.update_rate_limit_from_response(&response);

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();

                // 403 is commonly returned when the Contents API hits the 1 MB size limit.
                // Try the Blob API as a fallback.
                if status.as_u16() == 403
                    && let Ok(bytes) = self
                        .async_fetch_file_content_via_blob_api(&full_name, &path, &git_ref)
                        .await
                {
                    return Ok(bytes);
                }

                return Err(GitHubError::Api {
                    status: status.as_u16(),
                    body,
                });
            }

            let data: ContentResponse = response.json().await?;
            if data.encoding != "base64" {
                return Err(GitHubError::Other(format!(
                    "Unsupported file encoding: {}",
                    data.encoding
                )));
            }

            let normalized = data.content.replace('\n', "");
            let bytes = base64::engine::general_purpose::STANDARD.decode(normalized)?;
            Ok(bytes)
        })
        .await
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
        Self::get_runtime()
            .block_on(self.async_fetch_file_content_via_blob_api(&full_name, &path, &branch))
    }

    pub(crate) async fn async_fetch_file_content_via_blob_api(
        &self,
        full_name: &str,
        path: &str,
        branch: &str,
    ) -> Result<Vec<u8>, GitHubError> {
        self.check_rate_limit()?;
        let api_base = Self::api_base();
        let branch = if branch.trim().is_empty() {
            "HEAD"
        } else {
            branch
        };

        // ── Step 1: obtain the file SHA ──────────────────────────────
        let sha = self.async_fetch_file_sha(full_name, path, branch).await?;

        // ── Step 2: retrieve the blob ────────────────────────────────
        let blob_url = format!("{api_base}/repos/{full_name}/git/blobs/{sha}");
        let blob_response = self.client.get(&blob_url).send().await?;
        self.update_rate_limit_from_response(&blob_response);

        if !blob_response.status().is_success() {
            let status = blob_response.status();
            let body = blob_response.text().await.unwrap_or_default();
            return Err(GitHubError::Api {
                status: status.as_u16(),
                body,
            });
        }

        let blob: ContentResponse = blob_response.json().await?;
        if blob.encoding != "base64" {
            return Err(GitHubError::Encoding(blob.encoding));
        }

        let normalized = blob.content.replace('\n', "");
        let bytes = base64::engine::general_purpose::STANDARD.decode(normalized)?;
        Ok(bytes)
    }

    /// Obtain the Git SHA of a file at the given path and branch.
    ///
    /// Tries the Contents API first (fast path for files ≤ 1 MB). If that
    /// fails with a 403 (size limit), falls back to scanning the recursive
    /// Git tree.
    pub(crate) async fn async_fetch_file_sha(
        &self,
        full_name: &str,
        path: &str,
        branch: &str,
    ) -> Result<String, GitHubError> {
        let api_base = Self::api_base();

        // ── Try the Contents API first ───────────────────────────────
        let contents_url = format!("{api_base}/repos/{full_name}/contents/{path}?ref={branch}");
        let response = self.client.get(&contents_url).send().await?;
        self.update_rate_limit_from_response(&response);

        if response.status().is_success() {
            let meta: ContentResponse = response.json().await?;
            return Ok(meta.sha);
        }

        // If the Contents API fails with 403 (likely due to file size), fall
        // back to the Tree API.  Any other status is a real error.
        if response.status().as_u16() == 403 {
            return self
                .async_fetch_file_sha_via_tree(full_name, path, branch)
                .await;
        }

        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(GitHubError::Api {
            status: status.as_u16(),
            body,
        })
    }

    /// Scan the recursive Git tree to find the SHA of a file.
    pub(crate) async fn async_fetch_file_sha_via_tree(
        &self,
        full_name: &str,
        path: &str,
        branch: &str,
    ) -> Result<String, GitHubError> {
        let api_base = Self::api_base();
        let url = format!("{api_base}/repos/{full_name}/git/trees/{branch}?recursive=1");
        let tree: TreeResponse = self.send_and_check_json(self.client.get(&url)).await?;

        tree.tree
            .into_iter()
            .find(|entry| entry.path == path)
            .map(|entry| entry.sha)
            .ok_or_else(|| {
                GitHubError::NotFound(format!(
                    "File '{}' not found in tree for '{}/{}'",
                    path, full_name, branch
                ))
            })
    }

    pub fn fetch_authenticated_user(&self) -> Result<Option<String>, GitHubError> {
        Self::get_runtime().block_on(self.async_fetch_authenticated_user())
    }

    async fn async_fetch_authenticated_user(&self) -> Result<Option<String>, GitHubError> {
        self.check_rate_limit()?;
        let api_base = Self::api_base();
        let response = self.client.get(format!("{api_base}/user")).send().await?;
        self.update_rate_limit_from_response(&response);

        if response.status().as_u16() == 401 {
            return Ok(None);
        }
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(GitHubError::Api {
                status: status.as_u16(),
                body,
            });
        }
        let user: AuthenticatedUser = response.json().await?;
        Ok(Some(user.login))
    }
}
