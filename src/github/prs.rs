use crate::error::GitHubError;
use crate::github::{GitHubClient, with_retry};
use crate::models::{
    CommitInfo, Issue, MergeResponse, PullRequest, PullRequestDetail, PullRequestReview,
    ReviewComment,
};

#[allow(dead_code)]
impl GitHubClient {
    // ── Issues ──────────────────────────────────────────────────────────

    /// Fetch issues for a repository.
    pub fn fetch_issues(
        &self,
        full_name: &str,
        state: &str,
        per_page: u8,
    ) -> Result<Vec<Issue>, GitHubError> {
        let full_name = full_name.to_string();
        let state = state.to_string();
        Self::get_runtime().block_on(self.async_fetch_issues(full_name, state, per_page))
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
            let data: Vec<Issue> = self.send_and_check_json(self.client.get(url)).await?;
            Ok(data)
        })
        .await
    }

    // ── Pull Requests ───────────────────────────────────────────────────

    /// Fetch pull requests for a repository.
    pub fn fetch_pull_requests(
        &self,
        full_name: &str,
        state: &str,
        per_page: u8,
    ) -> Result<Vec<PullRequest>, GitHubError> {
        let full_name = full_name.to_string();
        let state = state.to_string();
        Self::get_runtime().block_on(self.async_fetch_pull_requests(full_name, state, per_page))
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
            let data: Vec<PullRequest> = self.send_and_check_json(self.client.get(url)).await?;
            Ok(data)
        })
        .await
    }

    // ── Pull Request Detail ──────────────────────────────────────────

    /// Fetch details for a single pull request.
    pub fn fetch_pull_request_detail(
        &self,
        full_name: &str,
        number: u64,
    ) -> Result<PullRequestDetail, GitHubError> {
        let full_name = full_name.to_string();
        Self::get_runtime().block_on(self.async_fetch_pull_request_detail(full_name, number))
    }

    async fn async_fetch_pull_request_detail(
        &self,
        full_name: String,
        number: u64,
    ) -> Result<PullRequestDetail, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/pulls/{number}");
            let data: PullRequestDetail = self.send_and_check_json(self.client.get(url)).await?;
            Ok(data)
        })
        .await
    }

    // ── Pull Request Reviews ─────────────────────────────────────────

    /// Fetch reviews for a pull request.
    pub fn fetch_pull_request_reviews(
        &self,
        full_name: &str,
        number: u64,
    ) -> Result<Vec<PullRequestReview>, GitHubError> {
        let full_name = full_name.to_string();
        Self::get_runtime().block_on(self.async_fetch_pull_request_reviews(full_name, number))
    }

    async fn async_fetch_pull_request_reviews(
        &self,
        full_name: String,
        number: u64,
    ) -> Result<Vec<PullRequestReview>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/pulls/{number}/reviews");
            let data: Vec<PullRequestReview> =
                self.send_and_check_json(self.client.get(url)).await?;
            Ok(data)
        })
        .await
    }

    // ── Pull Request Review Comments ─────────────────────────────────

    /// Fetch inline review comments on a pull request.
    pub fn fetch_pull_request_comments(
        &self,
        full_name: &str,
        number: u64,
    ) -> Result<Vec<ReviewComment>, GitHubError> {
        let full_name = full_name.to_string();
        Self::get_runtime().block_on(self.async_fetch_pull_request_comments(full_name, number))
    }

    async fn async_fetch_pull_request_comments(
        &self,
        full_name: String,
        number: u64,
    ) -> Result<Vec<ReviewComment>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/pulls/{number}/comments");
            let data: Vec<ReviewComment> = self.send_and_check_json(self.client.get(url)).await?;
            Ok(data)
        })
        .await
    }

    // ── Pull Request Commits ─────────────────────────────────────────

    /// Fetch commits belonging to a pull request.
    pub fn fetch_pull_request_commits(
        &self,
        full_name: &str,
        number: u64,
    ) -> Result<Vec<CommitInfo>, GitHubError> {
        let full_name = full_name.to_string();
        Self::get_runtime().block_on(self.async_fetch_pull_request_commits(full_name, number))
    }

    async fn async_fetch_pull_request_commits(
        &self,
        full_name: String,
        number: u64,
    ) -> Result<Vec<CommitInfo>, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/pulls/{number}/commits");
            let data: Vec<CommitInfo> = self.send_and_check_json(self.client.get(url)).await?;
            Ok(data)
        })
        .await
    }

    // ── Merge Pull Request ───────────────────────────────────────────

    /// Merge a pull request.
    ///
    /// `commit_title` and `merge_method` are optional. `merge_method` can be
    /// `"merge"`, `"squash"`, or `"rebase"`.
    pub fn merge_pull_request(
        &self,
        full_name: &str,
        number: u64,
        commit_title: Option<&str>,
        merge_method: Option<&str>,
    ) -> Result<MergeResponse, GitHubError> {
        let full_name = full_name.to_string();
        let commit_title = commit_title.map(|s| s.to_string());
        let merge_method = merge_method.map(|s| s.to_string());
        Self::get_runtime().block_on(self.async_merge_pull_request(
            full_name,
            number,
            commit_title,
            merge_method,
        ))
    }

    async fn async_merge_pull_request(
        &self,
        full_name: String,
        number: u64,
        commit_title: Option<String>,
        merge_method: Option<String>,
    ) -> Result<MergeResponse, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/pulls/{number}/merge");

            let mut body = serde_json::json!({});
            if let Some(ref title) = commit_title {
                body["commit_title"] = serde_json::json!(title);
            }
            if let Some(ref method) = merge_method {
                body["merge_method"] = serde_json::json!(method);
            }

            let response = self.client.put(url).json(&body).send().await?;
            self.update_rate_limit_from_response(&response);

            if !response.status().is_success() {
                let status = response.status();
                let body_text = response.text().await.unwrap_or_default();
                return Err(GitHubError::Api {
                    status: status.as_u16(),
                    body: body_text,
                });
            }

            let data: MergeResponse = response.json().await?;
            Ok(data)
        })
        .await
    }

    // ── Create Pull Request Review ───────────────────────────────────

    /// Create a review on a pull request.
    ///
    /// `event` should be `"APPROVE"`, `"REQUEST_CHANGES"`, or `"COMMENT"`.
    pub fn create_pull_request_review(
        &self,
        full_name: &str,
        number: u64,
        body: &str,
        event: &str,
    ) -> Result<PullRequestReview, GitHubError> {
        let full_name = full_name.to_string();
        let body = body.to_string();
        let event = event.to_string();
        Self::get_runtime()
            .block_on(self.async_create_pull_request_review(full_name, number, body, event))
    }

    async fn async_create_pull_request_review(
        &self,
        full_name: String,
        number: u64,
        body: String,
        event: String,
    ) -> Result<PullRequestReview, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/pulls/{number}/reviews");

            let payload = serde_json::json!({
                "body": body,
                "event": event,
            });

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

            let data: PullRequestReview = response.json().await?;
            Ok(data)
        })
        .await
    }

    // ── Update Pull Request ──────────────────────────────────────────

    /// Update a pull request (e.g. close it).
    ///
    /// `state` should be `"open"` or `"closed"`.
    pub fn update_pull_request(
        &self,
        full_name: &str,
        number: u64,
        state: &str,
    ) -> Result<PullRequestDetail, GitHubError> {
        let full_name = full_name.to_string();
        let state = state.to_string();
        Self::get_runtime().block_on(self.async_update_pull_request(full_name, number, state))
    }

    async fn async_update_pull_request(
        &self,
        full_name: String,
        number: u64,
        state: String,
    ) -> Result<PullRequestDetail, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/pulls/{number}");

            let payload = serde_json::json!({
                "state": state,
            });

            let response = self.client.patch(url).json(&payload).send().await?;
            self.update_rate_limit_from_response(&response);

            if !response.status().is_success() {
                let status = response.status();
                let body_text = response.text().await.unwrap_or_default();
                return Err(GitHubError::Api {
                    status: status.as_u16(),
                    body: body_text,
                });
            }

            let data: PullRequestDetail = response.json().await?;
            Ok(data)
        })
        .await
    }

    // ── Create Pull Request Comment ──────────────────────────────────

    /// Create a comment on a pull request (not an inline review comment).
    pub fn create_pull_request_comment(
        &self,
        full_name: &str,
        number: u64,
        body: &str,
    ) -> Result<ReviewComment, GitHubError> {
        let full_name = full_name.to_string();
        let body = body.to_string();
        Self::get_runtime()
            .block_on(self.async_create_pull_request_comment(full_name, number, body))
    }

    async fn async_create_pull_request_comment(
        &self,
        full_name: String,
        number: u64,
        body: String,
    ) -> Result<ReviewComment, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/pulls/{number}/comments");

            let payload = serde_json::json!({
                "body": body,
            });

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

            let data: ReviewComment = response.json().await?;
            Ok(data)
        })
        .await
    }

    // ── Create Pull Request ─────────────────────────────────────────

    /// Create a pull request.
    pub fn create_pull_request(
        &self,
        full_name: &str,
        title: &str,
        head: &str,
        base: &str,
        body: Option<&str>,
    ) -> Result<PullRequestDetail, GitHubError> {
        let full_name = full_name.to_string();
        let title = title.to_string();
        let head = head.to_string();
        let base = base.to_string();
        let body = body.map(|s| s.to_string());
        Self::get_runtime()
            .block_on(self.async_create_pull_request(full_name, title, head, base, body))
    }

    async fn async_create_pull_request(
        &self,
        full_name: String,
        title: String,
        head: String,
        base: String,
        body: Option<String>,
    ) -> Result<PullRequestDetail, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/pulls");

            let mut payload = serde_json::json!({
                "title": title,
                "head": head,
                "base": base,
            });
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

            let data: PullRequestDetail = response.json().await?;
            Ok(data)
        })
        .await
    }

    // ── Create Issue ────────────────────────────────────────────────

    /// Create an issue.
    pub fn create_issue(
        &self,
        full_name: &str,
        title: &str,
        body: Option<&str>,
    ) -> Result<Issue, GitHubError> {
        let full_name = full_name.to_string();
        let title = title.to_string();
        let body = body.map(|s| s.to_string());
        Self::get_runtime().block_on(self.async_create_issue(full_name, title, body))
    }

    async fn async_create_issue(
        &self,
        full_name: String,
        title: String,
        body: Option<String>,
    ) -> Result<Issue, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/issues");

            let mut payload = serde_json::json!({ "title": title });
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

            let data: Issue = response.json().await?;
            Ok(data)
        })
        .await
    }

    // ── Close Issue ──────────────────────────────────────────────────

    /// Close an issue.
    pub fn close_issue(&self, full_name: &str, number: u64) -> Result<Issue, GitHubError> {
        let full_name = full_name.to_string();
        Self::get_runtime().block_on(self.async_close_issue(full_name, number))
    }

    async fn async_close_issue(
        &self,
        full_name: String,
        number: u64,
    ) -> Result<Issue, GitHubError> {
        with_retry(|| async {
            self.check_rate_limit()?;
            let api_base = Self::api_base();
            let url = format!("{api_base}/repos/{full_name}/issues/{number}");

            let payload = serde_json::json!({ "state": "closed" });

            let response = self.client.patch(url).json(&payload).send().await?;
            self.update_rate_limit_from_response(&response);

            if !response.status().is_success() {
                let status = response.status();
                let body_text = response.text().await.unwrap_or_default();
                return Err(GitHubError::Api {
                    status: status.as_u16(),
                    body: body_text,
                });
            }

            let data: Issue = response.json().await?;
            Ok(data)
        })
        .await
    }
}
