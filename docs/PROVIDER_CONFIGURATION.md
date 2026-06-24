<h1 align="center">Provider Configuration</h1>

<div id="content"></div>
<h2 align="center">Contents</h2>
<ul>
  <li><a href="#overview">Overview</a></li>
  <li><a href="#default-provider">Default Provider (GitHub)</a></li>
  <li><a href="#provider-kind-table">Provider Kind Table</a></li>
  <li><a href="#auto-detection">Auto-Detection from Remote URL</a></li>
  <li><a href="#adding-a-custom-provider">Adding a Custom Provider</a></li>
  <li><a href="#provider-architecture">Provider Architecture</a></li>
  <li><a href="#future-providers">Future Providers</a></li>
</ul>

<h2 id="overview" align="center">Overview</h2>

GitNapse uses a <strong>provider abstraction</strong> to interact with git hosting
services. The <code>GitProvider</code> trait defines a uniform interface for all
API operations, and concrete implementations translate those calls into the
specific REST API of each service.

The provider system lives in <code>src/provider.rs</code>. The trait covers ~28
methods including repository search, file content retrieval, branch management,
pull request operations, issue tracking, CI status, and release management.

<h2 id="default-provider" align="center">Default Provider (GitHub)</h2>

By default, GitNapse uses the GitHub API. No configuration is needed. The
<code>GitHubClient</code> implements the <code>GitProvider</code> trait and
connects to <code>https://api.github.com</code>.

The API base URL can be overridden via the environment variable:

<pre>
GITNAPSE_GITHUB_API=https://your-enterprise-server.example.com/api/v3
</pre>

<h2 id="provider-kind-table" align="center">Provider Kind Table</h2>

<table>
  <tr>
    <th>ProviderKind</th>
    <th>Display Name</th>
    <th>Detection Pattern (URL contains)</th>
  </tr>
  <tr>
    <td><code>GitHub</code></td>
    <td>GitHub</td>
    <td><code>github</code></td>
  </tr>
  <tr>
    <td><code>AzureDevOps</code></td>
    <td>Azure DevOps</td>
    <td><code>dev.azure</code> or <code>visualstudio.com</code></td>
  </tr>
  <tr>
    <td><code>GitLab</code></td>
    <td>GitLab</td>
    <td><code>gitlab</code></td>
  </tr>
  <tr>
    <td><code>Bitbucket</code></td>
    <td>Bitbucket</td>
    <td><code>bitbucket</code></td>
  </tr>
  <tr>
    <td><code>Other</code></td>
    <td>Other</td>
    <td>Everything else (falls back to GitHub API)</td>
  </tr>
</table>

<h2 id="auto-detection" align="center">Auto-Detection from Remote URL</h2>

GitNapse can automatically detect the provider from a git remote URL. The
<code>detect_provider(remote_url)</code> function performs simple string
matching on the lowercased URL:

<pre>
use gitnapse::provider::{detect_provider, ProviderKind};

let kind = detect_provider("https://github.com/owner/repo.git");
assert_eq!(kind, ProviderKind::GitHub);

let kind = detect_provider("https://dev.azure.com/org/project/_git/repo");
assert_eq!(kind, ProviderKind::AzureDevOps);

let kind = detect_provider("https://gitlab.example.com/group/project.git");
assert_eq!(kind, ProviderKind::GitLab);
</pre>

This detection is not yet wired into the TUI's runtime provider selection. It is
available for CLI usage and as a building block for future automatic provider
switching based on the current repository's remote.

<h2 id="adding-a-custom-provider" align="center">Adding a Custom Provider</h2>

To add support for a new provider (e.g., Azure DevOps), follow these steps:

<h3>1. Create the provider module</h3>

Create a new file, for example <code>src/azure_devops.rs</code>, with a struct
that implements <code>GitProvider</code>:

<pre>
use crate::error::GitHubError;
use crate::models::*;
use crate::provider::GitProvider;
use anyhow::Result;
use reqwest::Client;
use std::sync::Mutex;

pub struct AzureDevOpsProvider {
    client: Client,
    organization: String,
    project: String,
    // rate-limit tracking, etc.
}

impl GitProvider for AzureDevOpsProvider {
    fn search_repositories_page(
        &self,
        query: &str,
        page: u32,
        per_page: u8,
    ) -> Result<Vec<RepoSummary>> {
        // Azure DevOps Git Repositories API call
        // ...
    }

    // ... implement all other trait methods
}
</pre>

<h3>2. Register the provider kind</h3>

Add a variant for your provider to the <code>ProviderKind</code> enum in
<code>src/provider.rs</code> (or reuse <code>Other</code>):

<pre>
pub enum ProviderKind {
    GitHub,
    AzureDevOps,
    GitLab,
    Bitbucket,
    Other,
}
</pre>

Update <code>detect_provider()</code> and <code>ProviderKind::display_name()</code>
accordingly.

<h3>3. Add the factory case</h3>

Update <code>create_provider()</code> in <code>src/provider.rs</code>:

<pre>
pub fn create_provider(
    kind: ProviderKind,
    token: Option<&str>,
) -> Result<Arc<dyn GitProvider>> {
    match kind {
        ProviderKind::GitHub => {
            let client = crate::github::GitHubClient::new(token)?;
            Ok(Arc::new(client))
        }
        ProviderKind::AzureDevOps => {
            let client = AzureDevOpsProvider::new(token)?;
            Ok(Arc::new(client))
        }
        ProviderKind::Other => {
            // fallback
            let client = crate::github::GitHubClient::new(token)?;
            Ok(Arc::new(client))
        }
        _ => {
            // forward-compatible: return a helpful error or fallback
            anyhow::bail!("unsupported provider: {:?}", kind);
        }
    }
}
</pre>

<h3>4. Register the module</h3>

Add <code>pub mod azure_devops;</code> to <code>src/lib.rs</code>.

<h2 id="provider-architecture" align="center">Provider Architecture</h2>

The provider layer is designed for minimal coupling with the rest of the
application:

<pre>
src/
  provider.rs          -- GitProvider trait, ProviderKind enum, factory
  github/
    mod.rs             -- GitHubClient struct definition
    provider_impl.rs   -- impl GitProvider for GitHubClient
    ...
  (future)
  azure_devops.rs      -- AzureDevOpsProvider (example)
  gitlab.rs            -- GitLabProvider (example)
</pre>

The TUI (<code>App</code>) holds <code>Arc&lt;dyn GitProvider&gt;</code> and
calls trait methods directly. The CLI creates a provider via
<code>helpers::make_client()</code> which calls
<code>create_provider(ProviderKind::GitHub, token)</code>.

<h3>Error Handling</h3>

All trait methods return <code>anyhow::Result&lt;T&gt;</code>. Provider-specific
errors (e.g., <code>GitHubError</code>) are wrapped inside <code>anyhow::Error</code>
and can be extracted via <code>e.downcast_ref::&lt;GitHubError&gt;()</code> when
provider-specific error details are needed.

<h3>Rate Limiting</h3>

The trait includes two optional rate-limit methods:
<code>rate_limit_remaining()</code> and <code>rate_limit_reset()</code>.
Providers that do not support rate-limit tracking can simply return
<code>None</code> from both.

<h2 id="future-providers" align="center">Future Providers</h2>

<h3>Azure DevOps</h3>

Azure DevOps uses the Azure REST API with a different URL structure:
<pre>
https://dev.azure.com/{organization}/{project}/_apis/git/repositories
</pre>

Authentication uses either a Personal Access Token (PAT) or Azure AD
credentials. The API version is specified via the <code>api-version</code> query
parameter (e.g., <code>api-version=7.0</code>).

Endpoints that differ from GitHub:
- Pull Requests: <code>/{project}/_apis/git/pullrequests</code>
- Files: <code>/{project}/_apis/git/repositories/{repo}/items</code>
- Branches: <code>/{project}/_apis/git/repositories/{repo}/refs</code>
- CI: Azure Pipelines REST API (separate endpoint)

<h3>GitLab</h3>

GitLab API is largely RESTful and similar to GitHub's in spirit:
<pre>
https://gitlab.com/api/v4/projects
</pre>

<h3>Bitbucket Cloud</h3>

Bitbucket uses the Atlassian REST API:
<pre>
https://api.bitbucket.org/2.0/repositories
</pre>
