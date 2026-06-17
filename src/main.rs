use anyhow::Result;
use clap::{Parser, Subcommand};

use gitnapse::{app, auth, cli, oauth};

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
    Run(cli::RunArgs),
    /// Download one file from a GitHub repository (curl/wget-like)
    DownloadFile(cli::DownloadFileArgs),
    /// Manage GitHub token authentication
    Auth {
        #[command(subcommand)]
        action: cli::AuthAction,
    },
    /// Clone a repository (via API + git)
    Clone(cli::CloneArgs),
    /// Stage (with -a) and commit changes
    Commit(cli::CommitArgs),
    /// Push commits to remote
    Push(cli::PushArgs),
    /// Pull changes from remote (with --rebase)
    Pull(cli::PullArgs),
    /// Fetch from remote (with --prune)
    Fetch(cli::FetchArgs),
    /// Switch branches or restore files
    Checkout(cli::CheckoutArgs),
    /// Show working tree diff
    Diff(cli::DiffArgs),
    /// Stash changes
    Stash {
        #[command(subcommand)]
        action: cli::StashAction,
    },
    /// Manage tags
    Tag {
        #[command(subcommand)]
        action: cli::TagAction,
    },
    /// Show working tree status
    Status,
    /// Show commit log (default: 20 entries)
    Log(cli::LogArgs),
    /// List branches
    Branch,
    /// Reset current HEAD
    Reset(cli::ResetArgs),
    /// Manage pull requests via GitHub API
    Pr {
        #[command(subcommand)]
        action: cli::PrAction,
    },
    /// Manage issues via GitHub API
    Issue {
        #[command(subcommand)]
        action: cli::IssueAction,
    },
    /// Show CI status for a repository
    Ci(cli::CiArgs),
    /// Compare two branches
    Compare(cli::CompareArgs),
}

fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Run(args)) => app::run_with_options(args.into()),
        Some(Command::DownloadFile(args)) => {
            cli::download_file(&args.repo, &args.path, args.r#ref.as_deref(), &args.out)
        }
        Some(Command::Clone(args)) => cli::clone_repo(&args.repo, args.dir.as_deref()),
        Some(Command::Commit(args)) => cli::commit(&args.message, args.all),
        Some(Command::Push(args)) => cli::push(args.remote.as_deref(), args.branch.as_deref(), args.force),
        Some(Command::Pull(args)) => cli::pull(args.remote.as_deref(), args.branch.as_deref(), args.rebase),
        Some(Command::Fetch(args)) => cli::fetch(args.prune),
        Some(Command::Checkout(args)) => cli::checkout(&args.branch, args.create),
        Some(Command::Diff(args)) => cli::diff(args.staged, args.path.as_deref()),
        Some(Command::Stash { action }) => match action {
            cli::StashAction::Push { message } => cli::stash_push(message.as_deref()),
            cli::StashAction::Pop => cli::stash_pop(),
            cli::StashAction::List => cli::stash_list(),
        },
        Some(Command::Tag { action }) => match action {
            cli::TagAction::List { pattern } => cli::tag_list(pattern.as_deref()),
            cli::TagAction::Create { name, message, target } => {
                cli::tag_create(&name, message.as_deref(), target.as_deref())
            }
            cli::TagAction::Delete { name } => cli::tag_delete(&name),
        },
        Some(Command::Status) => cli::status(),
        Some(Command::Log(args)) => cli::log_lines(args.count),
        Some(Command::Branch) => cli::branch(),
        Some(Command::Reset(args)) => cli::reset(args.target.as_deref(), args.hard),
        Some(Command::Pr { action }) => match action {
            cli::PrAction::List(a) => cli::pr_list(&a.repo, &a.state),
            cli::PrAction::Create(a) => cli::pr_create(&a.repo, &a.title, &a.head, &a.base, a.body.as_deref()),
            cli::PrAction::Merge(a) => cli::pr_merge(&a.repo, a.number, a.method.as_deref()),
        },
        Some(Command::Issue { action }) => match action {
            cli::IssueAction::List(a) => cli::issue_list(&a.repo, &a.state),
            cli::IssueAction::Create(a) => cli::issue_create(&a.repo, &a.title, a.body.as_deref()),
            cli::IssueAction::Close(a) => cli::issue_close(&a.repo, a.number),
        },
        Some(Command::Ci(args)) => cli::ci_status(&args.repo, args.branch.as_deref()),
        Some(Command::Compare(args)) => cli::compare(&args.repo, &args.base, &args.head),
        Some(Command::Auth { action }) => match action {
            cli::AuthAction::Set { token } => auth::set_token_cli(token),
            cli::AuthAction::Clear => auth::clear_token_cli(),
            cli::AuthAction::Status => auth::status_cli(),
            cli::AuthAction::Oauth { action } => match action {
                cli::OauthAction::Login { client_id, scope, timeout_secs } => {
                    oauth::oauth_device_login_cli(client_id, scope, timeout_secs)
                }
                cli::OauthAction::Status => oauth::oauth_status_cli(),
            },
        },
        None => app::run(),
    }
}
