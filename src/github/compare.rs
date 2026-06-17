use crate::error::GitHubError;
use crate::github::{GitHubClient, with_retry};
use crate::models::{CommitInfo, CompareResponse};

impl GitHubClient {
    /// Fetch recent commits for a branch.
    pub fn fetch_recent_commits(
        &self,
        full_name: &str,
        branch: &str,
        per_page: u8,
    ) -> Result<Vec<CommitInfo>, GitHubError> {
        let full_name = full_name.to_string();
        let branch = branch.to_string();
        Self::get_runtime().block_on(self.async_fetch_recent_commits(full_name, branch, per_page))
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
            let url =
                format!("{api_base}/repos/{full_name}/commits?sha={branch}&per_page={per_page}");
            let data: Vec<CommitInfo> = self.send_and_check_json(self.client.get(url)).await?;
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
        Self::get_runtime().block_on(self.async_fetch_compare(full_name, base, head))
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
            let data: CompareResponse = self.send_and_check_json(self.client.get(url)).await?;
            Ok(data)
        })
        .await
    }
}
