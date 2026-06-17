use crate::error::GitHubError;
use crate::github::{GitHubClient, with_retry};
use crate::models::release::Release;

impl GitHubClient {
    pub fn fetch_releases(
        &self,
        full_name: &str,
        per_page: u8,
    ) -> Result<Vec<Release>, GitHubError> {
        let full_name = full_name.to_string();
        Self::get_runtime().block_on(self.async_fetch_releases(full_name, per_page))
    }

    async fn async_fetch_releases(
        &self,
        full_name: String,
        per_page: u8,
    ) -> Result<Vec<Release>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/releases?per_page={per_page}");
            let data: Vec<Release> = self.send_and_check_json(self.client.get(url)).await?;
            Ok(data)
        })
        .await
    }

    pub fn create_release(
        &self,
        full_name: &str,
        tag_name: &str,
        name: Option<&str>,
        body: Option<&str>,
        prerelease: bool,
    ) -> Result<Release, GitHubError> {
        let full_name = full_name.to_string();
        let tag_name = tag_name.to_string();
        let name = name.map(|s| s.to_string());
        let body = body.map(|s| s.to_string());
        Self::get_runtime()
            .block_on(self.async_create_release(full_name, tag_name, name, body, prerelease))
    }

    async fn async_create_release(
        &self,
        full_name: String,
        tag_name: String,
        name: Option<String>,
        body: Option<String>,
        prerelease: bool,
    ) -> Result<Release, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/releases");

            let mut payload = serde_json::json!({
                "tag_name": tag_name,
                "prerelease": prerelease,
            });
            if let Some(ref n) = name {
                payload["name"] = serde_json::json!(n);
            }
            if let Some(ref b) = body {
                payload["body"] = serde_json::json!(b);
            }

            let response = self.client.post(url).json(&payload).send().await?;
            self.update_rate_limit_from_response(&response);
            if !response.status().is_success() {
                let status = response.status();
                let body_text = response.text().await.unwrap_or_default();
                return Err(GitHubError::Api {
                    status: status.as_u16(),
                    body: body_text,
                });
            }
            let data: Release = response.json().await?;
            Ok(data)
        })
        .await
    }

    pub fn create_repo(
        &self,
        name: &str,
        description: Option<&str>,
        private: bool,
    ) -> Result<crate::models::RepoSummary, GitHubError> {
        let name = name.to_string();
        let description = description.map(|s| s.to_string());
        Self::get_runtime().block_on(self.async_create_repo(name, description, private))
    }

    async fn async_create_repo(
        &self,
        name: String,
        description: Option<String>,
        private: bool,
    ) -> Result<crate::models::RepoSummary, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/user/repos");

            let mut payload = serde_json::json!({
                "name": name,
                "private": private,
            });
            if let Some(ref d) = description {
                payload["description"] = serde_json::json!(d);
            }

            let response = self.client.post(url).json(&payload).send().await?;
            self.update_rate_limit_from_response(&response);
            if !response.status().is_success() {
                let status = response.status();
                let body_text = response.text().await.unwrap_or_default();
                return Err(GitHubError::Api {
                    status: status.as_u16(),
                    body: body_text,
                });
            }
            let data: crate::models::RepoSummary = response.json().await?;
            Ok(data)
        })
        .await
    }
}
