use clap::{Args, Subcommand};
use std::path::PathBuf;

use crate::app;

// ── Top-level Args ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Args)]
pub struct RunArgs {
    #[arg(long, default_value = "xscriptor")]
    pub query: String,
    #[arg(long, default_value_t = 1)]
    pub page: u32,
    #[arg(long, default_value_t = 30)]
    pub per_page: u8,
    #[arg(long, default_value_t = 900)]
    pub cache_ttl_secs: u64,
}

#[derive(Debug, Clone, Args)]
pub struct DownloadFileArgs {
    #[arg(long)]
    pub repo: String,
    #[arg(long)]
    pub path: String,
    #[arg(long)]
    pub r#ref: Option<String>,
    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Debug, Clone, Args)]
pub struct CloneArgs {
    pub repo: String,
    #[arg(long)]
    pub dir: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct CommitArgs {
    #[arg(short = 'm')]
    pub message: String,
    #[arg(short = 'a')]
    pub all: bool,
}

#[derive(Debug, Clone, Args)]
pub struct PushArgs {
    pub remote: Option<String>,
    pub branch: Option<String>,
    #[arg(long = "force-with-lease")]
    pub force: bool,
}

#[derive(Debug, Clone, Args)]
pub struct PullArgs {
    pub remote: Option<String>,
    pub branch: Option<String>,
    #[arg(long)]
    pub rebase: bool,
}

#[derive(Debug, Clone, Args)]
pub struct FetchArgs {
    #[arg(long)]
    pub prune: bool,
}

#[derive(Debug, Clone, Args)]
pub struct CheckoutArgs {
    pub branch: String,
    #[arg(short = 'b')]
    pub create: bool,
}

#[derive(Debug, Clone, Args)]
pub struct DiffArgs {
    #[arg(long)]
    pub staged: bool,
    #[arg(long)]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct LogArgs {
    #[arg(short = 'n', default_value_t = 20)]
    pub count: usize,
}

#[derive(Debug, Clone, Args)]
pub struct ResetArgs {
    pub target: Option<String>,
    #[arg(long)]
    pub hard: bool,
}

#[derive(Debug, Clone, Args)]
pub struct CiArgs {
    pub repo: String,
    #[arg(short = 'b', long)]
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct CompareArgs {
    pub repo: String,
    pub base: String,
    pub head: String,
}

// ── PR subcommand args ──────────────────────────────────────────────────

#[derive(Debug, Clone, Args)]
pub struct PrListArgs {
    pub repo: String,
    #[arg(short = 's', long, default_value = "open")]
    pub state: String,
}

#[derive(Debug, Clone, Args)]
pub struct PrCreateArgs {
    pub repo: String,
    #[arg(short = 't', long)]
    pub title: String,
    #[arg(short = 'H', long)]
    pub head: String,
    #[arg(short = 'B', long)]
    pub base: String,
    #[arg(short = 'b', long)]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct PrMergeArgs {
    pub repo: String,
    #[arg(short = 'n', long)]
    pub number: u64,
    #[arg(short = 'm', long)]
    pub method: Option<String>,
}

// ── Issue subcommand args ───────────────────────────────────────────────

#[derive(Debug, Clone, Args)]
pub struct IssueListArgs {
    pub repo: String,
    #[arg(short = 's', long, default_value = "open")]
    pub state: String,
}

#[derive(Debug, Clone, Args)]
pub struct IssueCreateArgs {
    pub repo: String,
    #[arg(short = 't', long)]
    pub title: String,
    #[arg(short = 'b', long)]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct IssueCloseArgs {
    pub repo: String,
    #[arg(short = 'n', long)]
    pub number: u64,
}

// ── Action enums ────────────────────────────────────────────────────────

#[derive(Debug, Subcommand)]
pub enum StashAction {
    Push {
        #[arg(short = 'm')]
        message: Option<String>,
    },
    Pop,
    List,
}

#[derive(Debug, Subcommand)]
pub enum TagAction {
    List {
        pattern: Option<String>,
    },
    Create {
        name: String,
        #[arg(short = 'm')]
        message: Option<String>,
        #[arg(long)]
        target: Option<String>,
    },
    Delete {
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum PrAction {
    List(PrListArgs),
    Create(PrCreateArgs),
    Merge(PrMergeArgs),
}

#[derive(Debug, Subcommand)]
pub enum IssueAction {
    List(IssueListArgs),
    Create(IssueCreateArgs),
    Close(IssueCloseArgs),
}

#[derive(Debug, Subcommand)]
pub enum AuthAction {
    Set {
        #[arg(long)]
        token: Option<String>,
    },
    Clear,
    Status,
    Oauth {
        #[command(subcommand)]
        action: OauthAction,
    },
}

#[derive(Debug, Subcommand)]
pub enum OauthAction {
    Login {
        #[arg(long)]
        client_id: Option<String>,
        #[arg(long = "scope", value_delimiter = ',')]
        scope: Vec<String>,
        #[arg(long, default_value_t = 900)]
        timeout_secs: u64,
    },
    Status,
}

// ── From impls ──────────────────────────────────────────────────────────

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
