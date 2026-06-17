mod app;
mod auth;
mod cache;
mod cli_commands;
mod config;
mod error;
mod github;
mod models;
mod oauth;
mod oauth_session;
mod secure_store;
mod syntax;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "gitnapse",
    version,
    about = "Terminal GitHub repository explorer"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run interactive terminal UI
    Run(RunArgs),
    /// Download one file from a GitHub repository (curl/wget-like)
    DownloadFile(DownloadFileArgs),
    /// Manage GitHub token authentication
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },
    /// Clone a repository (via API + git)
    Clone(CloneArgs),
    /// Stage all changes and commit with a message
    Commit(CommitArgs),
    /// Push commits to remote
    Push(PushArgs),
    /// Show working tree status
    Status,
    /// Show commit log (default: 20 entries)
    Log(LogArgs),
    /// List local branches
    Branch,
    /// Manage pull requests via GitHub API
    Pr {
        #[command(subcommand)]
        action: PrAction,
    },
}

#[derive(Debug, Clone, Args)]
struct RunArgs {
    /// Initial repository search query
    #[arg(long, default_value = "xscriptor")]
    query: String,
    /// Initial search page
    #[arg(long, default_value_t = 1)]
    page: u32,
    /// Number of repos per page (max 100)
    #[arg(long, default_value_t = 30)]
    per_page: u8,
    /// Preview cache TTL in seconds
    #[arg(long, default_value_t = 900)]
    cache_ttl_secs: u64,
}

#[derive(Debug, Clone, Args)]
struct DownloadFileArgs {
    /// Full repository name, e.g. owner/repo
    #[arg(long)]
    repo: String,
    /// File path in repository, e.g. src/main.rs
    #[arg(long)]
    path: String,
    /// Branch/tag/sha (default: default branch behavior from content API)
    #[arg(long)]
    r#ref: Option<String>,
    /// Output local file path
    #[arg(long)]
    out: PathBuf,
}

#[derive(Debug, Clone, Args)]
struct CloneArgs {
    /// Repository (owner/repo or full git URL), optionally with :branch suffix
    repo: String,
}

#[derive(Debug, Clone, Args)]
struct CommitArgs {
    /// Commit message
    #[arg(short = 'm')]
    message: String,
}

#[derive(Debug, Clone, Args)]
struct PushArgs {
    /// Remote name (default: origin)
    remote: Option<String>,
    /// Branch name
    branch: Option<String>,
}

#[derive(Debug, Clone, Args)]
struct LogArgs {
    /// Number of commits to show
    #[arg(short = 'n', default_value_t = 20)]
    count: usize,
}

#[derive(Debug, Subcommand)]
enum PrAction {
    /// List pull requests for a repository
    List(PrListArgs),
    /// Create a pull request
    Create(PrCreateArgs),
    /// Merge a pull request
    Merge(PrMergeArgs),
}

#[derive(Debug, Clone, Args)]
struct PrListArgs {
    /// Full repository name, e.g. owner/repo
    repo: String,
    /// PR state: open, closed, all
    #[arg(short = 's', long, default_value = "open")]
    state: String,
}

#[derive(Debug, Clone, Args)]
struct PrCreateArgs {
    /// Full repository name, e.g. owner/repo
    repo: String,
    /// PR title
    #[arg(short = 't', long)]
    title: String,
    /// Head (source) branch
    #[arg(short = 'H', long)]
    head: String,
    /// Base (target) branch
    #[arg(short = 'B', long)]
    base: String,
    /// PR body / description
    #[arg(short = 'b', long)]
    body: Option<String>,
}

#[derive(Debug, Clone, Args)]
struct PrMergeArgs {
    /// Full repository name, e.g. owner/repo
    repo: String,
    /// Pull request number
    #[arg(short = 'n', long)]
    number: u64,
    /// Merge method: merge, squash, or rebase
    #[arg(short = 'm', long)]
    method: Option<String>,
}

impl From<RunArgs> for app::RunOptions {
    fn from(value: RunArgs) -> Self {
        Self {
            initial_query: value.query,
            initial_page: value.page.max(1),
            per_page: value.per_page.clamp(1, 100),
            cache_ttl_secs: value.cache_ttl_secs.max(1),
        }
    }
}

#[derive(Debug, Subcommand)]
enum AuthAction {
    /// Set and store a GitHub token securely in user config dir
    Set {
        /// Token value. If omitted, a hidden prompt is used.
        #[arg(long)]
        token: Option<String>,
    },
    /// Delete the stored token
    Clear,
    /// Show token source availability
    Status,
    /// OAuth login using GitHub device flow (octocrab)
    Oauth {
        #[command(subcommand)]
        action: OauthAction,
    },
}

#[derive(Debug, Subcommand)]
enum OauthAction {
    /// Login using OAuth device flow and persist the resulting token
    Login {
        /// GitHub OAuth app Client ID. If omitted, uses GITNAPSE_GITHUB_OAUTH_CLIENT_ID.
        #[arg(long)]
        client_id: Option<String>,
        /// OAuth scopes. Repeat or use comma-separated values.
        #[arg(long = "scope", value_delimiter = ',')]
        scope: Vec<String>,
        /// Poll timeout in seconds while waiting for browser authorization
        #[arg(long, default_value_t = 900)]
        timeout_secs: u64,
    },
    /// Show OAuth login/authentication state
    Status,
}

fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Run(args)) => app::run_with_options(args.into()),
        Some(Command::DownloadFile(args)) => download_file_cli(args),
        Some(Command::Clone(args)) => cli_commands::clone_repo(&args.repo),
        Some(Command::Commit(args)) => cli_commands::commit(&args.message),
        Some(Command::Push(args)) => {
            cli_commands::push(args.remote.as_deref(), args.branch.as_deref())
        }
        Some(Command::Status) => cli_commands::status(),
        Some(Command::Log(args)) => cli_commands::log_lines(args.count),
        Some(Command::Branch) => cli_commands::branch(),
        Some(Command::Pr { action }) => match action {
            PrAction::List(args) => cli_commands::pr_list(&args.repo, &args.state),
            PrAction::Create(args) => {
                cli_commands::pr_create(&args.repo, &args.title, &args.head, &args.base, args.body.as_deref())
            }
            PrAction::Merge(args) => {
                cli_commands::pr_merge(&args.repo, args.number, args.method.as_deref())
            }
        },
        Some(Command::Auth { action }) => match action {
            AuthAction::Set { token } => auth::set_token_cli(token),
            AuthAction::Clear => auth::clear_token_cli(),
            AuthAction::Status => auth::status_cli(),
            AuthAction::Oauth { action } => match action {
                OauthAction::Login {
                    client_id,
                    scope,
                    timeout_secs,
                } => oauth::oauth_device_login_cli(client_id, scope, timeout_secs),
                OauthAction::Status => oauth::oauth_status_cli(),
            },
        },
        None => app::run(),
    }
}

fn download_file_cli(args: DownloadFileArgs) -> Result<()> {
    let token = auth::load_token()?;
    let client = github::GitHubClient::new(token.as_deref())?;

    let bytes = match args.r#ref {
        Some(branch) if !branch.trim().is_empty() => {
            client.fetch_file_content_by_ref(&args.repo, &args.path, &branch)?
        }
        _ => client.fetch_file_content(&args.repo, &args.path)?,
    };

    if let Some(parent) = args.out.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    fs::write(&args.out, bytes)?;
    println!(
        "Downloaded {}:{} -> {}",
        args.repo,
        args.path,
        args.out.display()
    );
    Ok(())
}
