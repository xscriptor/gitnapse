use crate::error::GitHubError;
use crate::github::{GitHubClient, with_retry};
use crate::models::{CheckRun, CheckRunsResponse, WorkflowRun, WorkflowRunsResponse};

impl GitHubClient {
    /// Fetch check runs for a specific commit ref.
    pub fn fetch_check_runs(
        &self,
        full_name: &str,
        ref_: &str,
    ) -> Result<Vec<CheckRun>, GitHubError> {
        let full_name = full_name.to_string();
        let ref_ = ref_.to_string();
        Self::get_runtime().block_on(self.async_fetch_check_runs(full_name, ref_))
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
            let data: CheckRunsResponse = self.send_and_check_json(self.client.get(url)).await?;
            Ok(data.check_runs)
        })
        .await
    }

    /// Fetch workflow runs for a branch.
    pub fn fetch_workflow_runs(
        &self,
        full_name: &str,
        branch: &str,
        per_page: u8,
    ) -> Result<Vec<WorkflowRun>, GitHubError> {
        let full_name = full_name.to_string();
        let branch = branch.to_string();
        Self::get_runtime().block_on(self.async_fetch_workflow_runs(full_name, branch, per_page))
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
            let data: WorkflowRunsResponse = self.send_and_check_json(self.client.get(url)).await?;
            Ok(data.workflow_runs)
        })
        .await
    }
}
