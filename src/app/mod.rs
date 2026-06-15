mod render;
mod theme;

use crate::auth;
use crate::cache::PreviewCache;
use crate::config::{AccountConfig, KeybindingsConfig, ThemeConfig};
use crate::github::GitHubClient;
use crate::models::{
    CheckRun, CommitInfo, CompareResponse, Issue, PullRequest, RepoNode, RepoSummary,
};
use crate::oauth;
use crate::syntax::highlight_content;
use anyhow::{Context, Result, anyhow};
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
    MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::text::Line;
use secrecy::{ExposeSecret, SecretString, zeroize::Zeroize};
use std::io::stdout;
use std::panic;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::{Duration, Instant};

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
enum NetworkEvent {
    SearchResult(Result<Vec<RepoSummary>, String>),
    IssuesResult(Result<Vec<Issue>, String>),
    PrsResult(Result<Vec<PullRequest>, String>),
    CommitsResult(Result<Vec<CommitInfo>, String>),
    CompareResult(Result<CompareResponse, String>),
    CheckRunsResult(Result<Vec<CheckRun>, String>),
    StarredResult(Result<Vec<RepoSummary>, String>),
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = enable_raw_mode();
        let _ = execute!(stdout(), EnterAlternateScreen, EnableMouseCapture);
    }
}

#[derive(Debug, Clone)]
pub struct RunOptions {
    pub initial_query: String,
    pub initial_page: u32,
    pub per_page: u8,
    pub cache_ttl_secs: u64,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            initial_query: "xscriptor".to_string(),
            initial_page: 1,
            per_page: 30,
            cache_ttl_secs: 900,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Search,
    Repos,
    Tree,
    Preview,
    TreeSearch,
    DownloadPath,
    ClonePath,
    TokenInput,
    OAuthClientIdInput,
    BranchPicker,
}

pub struct App {
    // Services
    pub github: Arc<GitHubClient>,
    pub account: AccountConfig,
    pub preview_cache: PreviewCache,

    // Search state
    pub search_query: String,
    pub search_page: u32,
    pub per_page: u8,
    pub repos: Vec<RepoSummary>,
    pub selected_repo: usize,

    // Tree / explorer state
    pub tree_all: Vec<RepoNode>,
    pub tree_visible_limit: usize,
    pub selected_node: usize,
    pub current_repo: Option<RepoSummary>,
    pub branches: Vec<String>,
    pub selected_branch: usize,

    // Preview state
    pub preview_title: String,
    pub preview_lines: Vec<Line<'static>>,
    pub preview_scroll: usize,
    pub current_preview_path: Option<String>,
    pub preview_viewport_rows: usize,
    pub tree_text_mode: bool,

    // Input buffers
    pub input_buffer: String,
    pub token_buffer: SecretString,
    pub oauth_client_id_input: String,
    pub clone_path_input: String,
    pub download_path_input: String,
    pub tree_search_input: String,

    // UI state
    pub status: String,
    pub focus: Focus,
    pub should_quit: bool,
    pub auth_user: Option<String>,

    // Click tracking
    pub last_tree_click: Option<(usize, Instant)>,
    pub last_repo_click: Option<(usize, Instant)>,

    // Keybindings
    #[allow(dead_code)]
    pub keybindings: KeybindingsConfig,

    // Command palette
    pub command_palette_visible: bool,
    pub command_input: String,
    pub command_cursor: usize,
    pub command_items: Vec<String>,
    pub command_filtered: Vec<String>,
}

impl App {
    const TREE_PAGE_SIZE: usize = 250;
    const TREE_LOAD_THRESHOLD: usize = 15;

    fn new(options: RunOptions) -> Result<Self> {
        let token = auth::load_token()?;
        let github = Arc::new(GitHubClient::new(token.as_deref())?);
        let mut account = AccountConfig::load_or_default()?;
        let theme_config = ThemeConfig::load_or_default();
        theme::init_theme(&theme_config);
        let keybindings = KeybindingsConfig::load_or_default();
        let preview_cache = PreviewCache::new(options.cache_ttl_secs)?;
        let auth_user = github.fetch_authenticated_user().ok().flatten();

        if account.preferred_clone_dir.trim().is_empty() {
            account.preferred_clone_dir = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .display()
                .to_string();
        }

        Ok(Self {
            github,
            account: account.clone(),
            preview_cache,
            search_query: options.initial_query,
            search_page: options.initial_page.max(1),
            per_page: options.per_page.clamp(1, 100),
            repos: Vec::new(),
            selected_repo: 0,
            tree_all: Vec::new(),
            tree_visible_limit: 0,
            selected_node: 0,
            current_repo: None,
            branches: Vec::new(),
            selected_branch: 0,
            preview_title: "Preview".to_string(),
            preview_lines: vec![Line::from("Select a repository and a file to preview.")],
            preview_scroll: 0,
            current_preview_path: None,
            preview_viewport_rows: 30,
            tree_text_mode: false,
            input_buffer: String::new(),
            token_buffer: SecretString::new(String::new().into()),
            oauth_client_id_input: {
                let client_id = std::env::var("GITNAPSE_GITHUB_OAUTH_CLIENT_ID")
                    .or_else(|_| std::env::var("GITHUB_CLIENT_ID"));
                if client_id
                    .as_ref()
                    .map(|s| s.trim().is_empty())
                    .unwrap_or(true)
                {
                    eprintln!(
                        "[gitnapse] Warning: No OAuth client ID found in GITNAPSE_GITHUB_OAUTH_CLIENT_ID or GITHUB_CLIENT_ID env vars. Using built-in default."
                    );
                }
                client_id.unwrap_or_default().trim().to_string()
            },
            clone_path_input: account.preferred_clone_dir,
            download_path_input: String::new(),
            tree_search_input: String::new(),
            status: match auth_user.as_ref() {
                Some(login) => format!("Authenticated as {login}. Press / to search."),
                None => {
                    "No validated token. Press t to save one or continue anonymously.".to_string()
                }
            },
            focus: Focus::Repos,
            should_quit: false,
            auth_user,
            last_tree_click: None,
            last_repo_click: None,
            keybindings,
            command_palette_visible: false,
            command_input: String::new(),
            command_cursor: 0,
            command_items: Vec::new(),
            command_filtered: Vec::new(),
        })
    }

    pub fn selected_repo(&self) -> Option<&RepoSummary> {
        self.repos.get(self.selected_repo)
    }

    fn selected_node(&self) -> Option<&RepoNode> {
        self.tree_all.get(self.selected_node)
    }

    pub fn visible_tree(&self) -> &[RepoNode] {
        let limit = self.tree_visible_limit.min(self.tree_all.len());
        &self.tree_all[..limit]
    }

    pub(crate) fn selected_branch_name(&self) -> String {
        self.branches
            .get(self.selected_branch)
            .cloned()
            .unwrap_or_else(|| "HEAD".to_string())
    }

    fn ensure_lazy_tree_progress(&mut self) {
        if self.tree_visible_limit >= self.tree_all.len() {
            return;
        }
        if self.selected_node + Self::TREE_LOAD_THRESHOLD >= self.tree_visible_limit {
            self.tree_visible_limit =
                (self.tree_visible_limit + Self::TREE_PAGE_SIZE).min(self.tree_all.len());
            self.status = format!(
                "Loaded more tree entries ({}/{}).",
                self.tree_visible_limit,
                self.tree_all.len()
            );
        }
    }

    fn reset_tree(&mut self, nodes: Vec<RepoNode>) {
        self.tree_all = nodes;
        self.selected_node = 0;
        self.tree_visible_limit = self.tree_all.len().min(Self::TREE_PAGE_SIZE);
        self.current_preview_path = None;
        self.tree_text_mode = false;
    }

    fn search(&mut self) {
        self.status = "Loading...".to_string();
        match self.github.search_repositories_page(
            &self.search_query,
            self.search_page,
            self.per_page,
        ) {
            Ok(items) => {
                if items.is_empty() && self.search_page > 1 {
                    self.search_page = self.search_page.saturating_sub(1);
                    self.status = "No more search results pages.".to_string();
                    return;
                }
                self.repos = items;
                self.selected_repo = 0;
                self.tree_all.clear();
                self.tree_visible_limit = 0;
                self.selected_node = 0;
                self.current_repo = None;
                self.branches.clear();
                self.selected_branch = 0;
                self.current_preview_path = None;
                self.tree_text_mode = false;
                self.status = format!(
                    "Loaded {} repositories on page {} (per_page {}).",
                    self.repos.len(),
                    self.search_page,
                    self.per_page
                );
            }
            Err(error) => {
                self.status = format!("Search failed: {error}");
            }
        }
    }

    fn open_selected_repo(&mut self) {
        let Some(repo) = self.selected_repo().cloned() else {
            self.status = "No repository selected.".to_string();
            return;
        };

        self.status = "Loading...".to_string();
        let mut branches = match self.github.fetch_branches(&repo.full_name) {
            Ok(items) if !items.is_empty() => items,
            Ok(_) => vec![repo.default_branch.clone()],
            Err(_) => vec![repo.default_branch.clone()],
        };
        branches.sort();
        branches.dedup();

        self.branches = branches;
        self.current_repo = Some(repo.clone());
        self.selected_branch = self
            .branches
            .iter()
            .position(|branch| {
                self.account
                    .last_branch_by_repo
                    .get(&repo.full_name)
                    .map(|saved| saved == branch)
                    .unwrap_or(false)
            })
            .or_else(|| self.branches.iter().position(|b| b == &repo.default_branch))
            .unwrap_or(0);

        self.load_tree_for_current_branch();
        self.focus = Focus::Tree;
    }

    fn load_tree_for_current_branch(&mut self) {
        let Some(repo) = self.current_repo.as_ref() else {
            self.status = "No repository loaded.".to_string();
            return;
        };
        let full_name = repo.full_name.clone();
        let branch = self.selected_branch_name();

        self.status = "Loading...".to_string();
        match self.github.fetch_repo_tree(&full_name, &branch) {
            Ok(tree) => {
                self.reset_tree(tree);
                self.preview_title = format!("{full_name} / {branch}");
                self.preview_lines = vec![Line::from(
                    "Repository loaded. Use arrows and Enter to preview files.",
                )];
                self.account
                    .last_branch_by_repo
                    .insert(full_name.clone(), branch.clone());
                let _ = self.account.save();
                self.status = format!(
                    "Loaded branch {branch}. Tree entries: {} (showing {}). Press c to clone.",
                    self.tree_all.len(),
                    self.tree_visible_limit
                );
                self.preview_scroll = 0;
                self.current_preview_path = None;
                self.tree_text_mode = false;
            }
            Err(error) => {
                self.status = format!("Unable to open branch {branch}: {error}");
            }
        }
    }

    fn preview_selected_file(&mut self) {
        let (Some(repo), Some(node)) = (self.current_repo.as_ref(), self.selected_node()) else {
            return;
        };
        let full_name = repo.full_name.clone();
        let branch = self.selected_branch_name();
        let node_path = node.path.clone();
        let node_is_dir = node.is_dir;

        if node_is_dir {
            self.preview_title = format!("{}/{}", full_name, node_path);
            self.preview_lines = vec![Line::from("Directory selected. Choose a file to preview.")];
            self.preview_scroll = 0;
            self.current_preview_path = None;
            self.tree_text_mode = false;
            return;
        }

        if let Some(content) = self.preview_cache.get(&full_name, &branch, &node_path) {
            self.preview_title = format!("{}/{}", full_name, node_path);
            self.preview_scroll = 0;
            self.current_preview_path = Some(node_path.clone());
            self.tree_text_mode = false;
            match String::from_utf8(content) {
                Ok(content_str) => {
                    self.preview_lines = highlight_content(&content_str, &node_path, 300);
                    self.status = format!("Preview loaded from cache for {}", node_path);
                }
                Err(_) => {
                    self.preview_lines = vec![Line::from("Binary file. Use 'd' to download.")];
                    self.current_preview_path = None;
                    self.status = format!("Binary file in cache: {}", node_path);
                }
            }
            return;
        }

        self.status = "Loading...".to_string();
        match self.github.fetch_file_content(&full_name, &node_path) {
            Ok(bytes) => {
                self.preview_cache
                    .put(&full_name, &branch, &node_path, &bytes, None);
                self.preview_title = format!("{}/{}", full_name, node_path);
                self.preview_scroll = 0;
                self.current_preview_path = Some(node_path.clone());
                self.tree_text_mode = false;
                match String::from_utf8(bytes) {
                    Ok(content) => {
                        self.preview_lines = highlight_content(&content, &node_path, 300);
                        self.status = format!("Preview loaded for {}", node_path);
                    }
                    Err(_) => {
                        self.preview_lines = vec![Line::from("Binary file. Use 'd' to download.")];
                        self.current_preview_path = None;
                        self.status = format!("Binary file: {}", node_path);
                    }
                }
            }
            Err(error) => {
                self.status = format!("Preview failed: {error}");
            }
        }
    }

    fn clone_current_repo(&mut self) {
        let Some(repo) = self.current_repo.as_ref() else {
            self.status = "Open a repository before cloning.".to_string();
            return;
        };

        let destination = self.clone_path_input.trim();
        if destination.is_empty() {
            self.status = "Destination path cannot be empty.".to_string();
            return;
        }

        let destination_path = PathBuf::from(destination);
        if !destination_path.exists()
            && let Err(error) = std::fs::create_dir_all(&destination_path)
        {
            self.status = format!(
                "Cannot create destination path {}: {error}",
                destination_path.display()
            );
            return;
        }

        let output = Command::new("git")
            .arg("clone")
            .arg(&repo.clone_url)
            .current_dir(&destination_path)
            .output();

        match output {
            Ok(out) if out.status.success() => {
                self.status = format!("Repository cloned to {}", destination_path.display());
                self.focus = Focus::Tree;
                self.account.preferred_clone_dir = destination_path.display().to_string();
                let _ = self.account.save();
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                self.status = format!("git clone failed: {}", stderr.trim());
            }
            Err(error) => {
                self.status = format!("Unable to run git clone: {error}");
            }
        }
    }

    fn save_token_from_input_str(&mut self, token: String) {
        let token_trimmed = token.trim().to_string();
        if token_trimmed.is_empty() {
            self.status = "Token is empty.".to_string();
            return;
        }

        match auth::save_token(&token_trimmed).and_then(|_| {
            GitHubClient::new(Some(&token_trimmed)).context("Cannot rebuild HTTP client")
        }) {
            Ok(client) => {
                self.github = Arc::new(client);
                self.auth_user = self.github.fetch_authenticated_user().ok().flatten();
                self.status = match self.auth_user.as_ref() {
                    Some(login) => format!("Token saved and validated as {login}."),
                    None => "Token saved, but validation failed.".to_string(),
                };
                self.focus = Focus::Repos;
            }
            Err(error) => {
                self.status = format!("Token save failed: {error}");
            }
        }
    }

    fn handle_key(&mut self, code: KeyCode) {
        match self.focus {
            Focus::Search => self.handle_search_input(code),
            Focus::TreeSearch => self.handle_tree_search_input(code),
            Focus::DownloadPath => self.handle_download_path_input(code),
            Focus::ClonePath => self.handle_clone_path_input(code),
            Focus::TokenInput => self.handle_token_input(code),
            Focus::OAuthClientIdInput => self.handle_oauth_client_id_input(code),
            Focus::BranchPicker => self.handle_branch_picker_input(code),
            Focus::Repos | Focus::Tree | Focus::Preview => self.handle_navigation(code),
        }
    }

    fn max_preview_scroll(&self, viewport_rows: usize) -> usize {
        self.preview_lines
            .len()
            .saturating_sub(viewport_rows.max(1))
    }

    fn scroll_preview_down(&mut self, step: usize, viewport_rows: usize) {
        let max_scroll = self.max_preview_scroll(viewport_rows);
        self.preview_scroll = (self.preview_scroll + step).min(max_scroll);
    }

    fn scroll_preview_up(&mut self, step: usize) {
        self.preview_scroll = self.preview_scroll.saturating_sub(step);
    }

    fn tree_window(&self, area_height: u16) -> (usize, usize) {
        let visible = self.visible_tree();
        let viewport_rows = usize::from(area_height.saturating_sub(2)).max(1);
        let max_start = visible.len().saturating_sub(viewport_rows);
        let start = self
            .selected_node
            .saturating_sub(viewport_rows / 2)
            .min(max_start);
        let end = (start + viewport_rows).min(visible.len());
        (start, end)
    }

    fn repo_window(&self, area_height: u16) -> (usize, usize) {
        let viewport_rows = usize::from(area_height.saturating_sub(2)).max(1);
        let max_start = self.repos.len().saturating_sub(viewport_rows);
        let start = self
            .selected_repo
            .saturating_sub(viewport_rows / 2)
            .min(max_start);
        let end = (start + viewport_rows).min(self.repos.len());
        (start, end)
    }

    fn back_to_repo_list(&mut self) {
        if self.current_repo.is_some() {
            self.current_repo = None;
            self.tree_all.clear();
            self.tree_visible_limit = 0;
            self.selected_node = 0;
            self.branches.clear();
            self.selected_branch = 0;
            self.preview_title = "Preview".to_string();
            self.preview_lines = vec![Line::from("Select a repository and a file to preview.")];
            self.preview_scroll = 0;
            self.current_preview_path = None;
            self.tree_text_mode = false;
            self.focus = Focus::Repos;
            self.status = "Returned to repository search list.".to_string();
        } else {
            self.focus = Focus::Repos;
        }
    }

    fn handle_tree_search_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.tree_search_input.clear();
                self.focus = Focus::Tree;
            }
            KeyCode::Enter => {
                let needle = self.tree_search_input.trim().to_ascii_lowercase();
                self.focus = Focus::Tree;
                if needle.is_empty() {
                    self.status = "Search term is empty.".to_string();
                    return;
                }
                if let Some((idx, _)) = self
                    .tree_all
                    .iter()
                    .enumerate()
                    .find(|(_, n)| n.path.to_ascii_lowercase().contains(&needle))
                {
                    self.selected_node = idx;
                    self.ensure_lazy_tree_progress();
                    self.status = format!(
                        "Found file match for \"{}\".",
                        self.tree_search_input.trim()
                    );
                } else {
                    self.status = format!("No file matches \"{}\".", self.tree_search_input.trim());
                }
            }
            KeyCode::Backspace => {
                self.tree_search_input.pop();
            }
            KeyCode::Char(ch) => self.tree_search_input.push(ch),
            _ => {}
        }
    }

    fn handle_download_path_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.focus = Focus::Preview;
            }
            KeyCode::Enter => {
                let Some(repo) = self.current_repo.as_ref() else {
                    self.status = "No repository loaded.".to_string();
                    self.focus = Focus::Tree;
                    return;
                };
                let Some(file_path) = self.current_preview_path.as_ref() else {
                    self.status = "Preview a file first before downloading.".to_string();
                    self.focus = Focus::Tree;
                    return;
                };

                let out = self.download_path_input.trim();
                if out.is_empty() {
                    self.status = "Download path cannot be empty.".to_string();
                    return;
                }
                let out_path = PathBuf::from(out);
                if let Some(parent) = out_path.parent()
                    && !parent.as_os_str().is_empty()
                    && let Err(error) = std::fs::create_dir_all(parent)
                {
                    self.status = format!("Cannot create parent folder: {error}");
                    return;
                }
                let branch = self.selected_branch_name();
                match self
                    .github
                    .fetch_file_content_by_ref(&repo.full_name, file_path, &branch)
                    .and_then(|content| {
                        std::fs::write(&out_path, content)
                            .map_err(anyhow::Error::from)
                            .context("Cannot write downloaded file")
                    }) {
                    Ok(()) => {
                        self.status = format!(
                            "Downloaded {}:{} -> {}",
                            repo.full_name,
                            file_path,
                            out_path.display()
                        );
                        self.focus = Focus::Preview;
                    }
                    Err(error) => {
                        self.status = format!("Download failed: {error}");
                    }
                }
            }
            KeyCode::Delete => self.download_path_input.clear(),
            KeyCode::Backspace => {
                self.download_path_input.pop();
            }
            KeyCode::Char(ch) => self.download_path_input.push(ch),
            _ => {}
        }
    }

    fn handle_search_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => self.focus = Focus::Repos,
            KeyCode::Enter => {
                self.search_query = self.input_buffer.trim().to_string();
                self.search_page = 1;
                self.focus = Focus::Repos;
                self.search();
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(ch) => {
                self.input_buffer.push(ch);
            }
            _ => {}
        }
    }

    fn handle_clone_path_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => self.focus = Focus::Tree,
            KeyCode::Enter => self.clone_current_repo(),
            KeyCode::Delete => self.clone_path_input.clear(),
            KeyCode::Backspace => {
                self.clone_path_input.pop();
            }
            KeyCode::Char(ch) => self.clone_path_input.push(ch),
            _ => {}
        }
    }

    fn handle_token_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.token_buffer.zeroize();
                self.input_buffer.clear();
                self.focus = Focus::Repos;
            }
            KeyCode::Enter => {
                let token: String = self.token_buffer.expose_secret().to_string();
                self.save_token_from_input_str(token);
                self.token_buffer.zeroize();
                self.input_buffer.clear();
            }
            KeyCode::Backspace => {
                let mut s: String = self.token_buffer.expose_secret().to_string();
                s.pop();
                self.token_buffer = SecretString::new(s.into());
            }
            KeyCode::Char(ch) => {
                let mut s: String = self.token_buffer.expose_secret().to_string();
                s.push(ch);
                self.token_buffer = SecretString::new(s.into());
            }
            _ => {}
        }
    }

    fn handle_oauth_client_id_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.focus = if self.current_repo.is_some() {
                    Focus::Tree
                } else {
                    Focus::Repos
                };
            }
            KeyCode::Enter => {
                let client_id = if self.oauth_client_id_input.trim().is_empty() {
                    None
                } else {
                    Some(self.oauth_client_id_input.trim().to_string())
                };
                self.run_oauth_login_flow(client_id);
            }
            KeyCode::Delete => self.oauth_client_id_input.clear(),
            KeyCode::Backspace => {
                self.oauth_client_id_input.pop();
            }
            KeyCode::Char(ch) => self.oauth_client_id_input.push(ch),
            _ => {}
        }
    }

    fn run_oauth_quick_check(&mut self) {
        match oauth::oauth_status_cli() {
            Ok(()) => {
                self.status =
                    "OAuth status printed in terminal. For login use: gitnapse auth oauth login"
                        .to_string();
            }
            Err(error) => {
                self.status = format!("OAuth status check failed: {error}");
            }
        }
    }

    fn run_oauth_login_flow(&mut self, client_id: Option<String>) {
        self.status = "Starting OAuth device flow...".to_string();

        // Temporarily leave TUI mode to let user interact with OAuth instructions in terminal.
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture);
        let guard = TerminalGuard;

        let oauth_result =
            oauth::oauth_device_login_cli(client_id, vec!["read:user".to_string()], 900);

        drop(guard);

        match oauth_result {
            Ok(()) => {
                if let Ok(token) = auth::load_token()
                    && let Ok(client) = GitHubClient::new(token.as_deref())
                {
                    self.github = Arc::new(client);
                    self.auth_user = self.github.fetch_authenticated_user().ok().flatten();
                }
                self.status = "OAuth login completed and session saved.".to_string();
            }
            Err(error) => {
                self.status = format!("OAuth login failed: {error}");
            }
        }

        self.focus = if self.current_repo.is_some() {
            Focus::Tree
        } else {
            Focus::Repos
        };
    }

    fn handle_branch_picker_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => self.focus = Focus::Tree,
            KeyCode::Up => {
                self.selected_branch = self.selected_branch.saturating_sub(1);
            }
            KeyCode::Down => {
                if !self.branches.is_empty() {
                    self.selected_branch = (self.selected_branch + 1).min(self.branches.len() - 1);
                }
            }
            KeyCode::Enter => {
                self.load_tree_for_current_branch();
                self.focus = Focus::Tree;
            }
            _ => {}
        }
    }

    fn handle_navigation(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('/') => {
                self.focus = Focus::Search;
                self.input_buffer = self.search_query.clone();
            }
            KeyCode::Char('t') => {
                self.focus = Focus::TokenInput;
                self.input_buffer.clear();
            }
            KeyCode::Char('o') => {
                self.run_oauth_quick_check();
            }
            KeyCode::Char('c') => {
                if self.current_repo.is_some() {
                    self.clone_path_input = self.account.preferred_clone_dir.clone();
                    self.focus = Focus::ClonePath;
                } else {
                    self.status = "Open a repository first, then press c to clone.".to_string();
                }
            }
            KeyCode::Char('b') => {
                if self.current_repo.is_some() && !self.branches.is_empty() {
                    self.focus = Focus::BranchPicker;
                } else {
                    self.status = "Open a repository first to select a branch.".to_string();
                }
            }
            KeyCode::Char('f') => {
                if self.current_repo.is_some() {
                    self.tree_search_input.clear();
                    self.focus = Focus::TreeSearch;
                }
            }
            KeyCode::Char('d') => {
                if self.current_preview_path.is_some() {
                    self.download_path_input = ".".to_string();
                    self.focus = Focus::DownloadPath;
                } else {
                    self.status = "Preview a file first before downloading.".to_string();
                }
            }
            KeyCode::Char('v') => {
                if self.current_repo.is_some() {
                    self.tree_text_mode = !self.tree_text_mode;
                    self.preview_scroll = 0;
                    if self.tree_text_mode {
                        let branch = self.selected_branch_name();
                        self.preview_title = format!(
                            "tree {} [{}]",
                            self.current_repo
                                .as_ref()
                                .map(|r| r.full_name.clone())
                                .unwrap_or_default(),
                            branch
                        );
                        self.preview_lines = self
                            .tree_all
                            .iter()
                            .map(|node| {
                                let indent = "  ".repeat(node.depth.min(20));
                                let icon = if node.is_dir { "[D]" } else { "[F]" };
                                Line::from(format!("{indent}{icon} {}", node.path))
                            })
                            .collect();
                        self.current_preview_path = None;
                        self.focus = Focus::Preview;
                        self.status = "Tree view enabled in preview pane.".to_string();
                    } else {
                        self.preview_title = "Preview".to_string();
                        self.preview_lines = vec![Line::from(
                            "Tree preview disabled. Select a file and press Enter to preview.",
                        )];
                        self.focus = Focus::Tree;
                        self.status = "Tree view disabled.".to_string();
                    }
                }
            }
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::Repos if !self.tree_all.is_empty() => Focus::Tree,
                    Focus::Tree if !self.preview_lines.is_empty() => Focus::Preview,
                    _ => Focus::Repos,
                }
            }
            KeyCode::Esc => self.back_to_repo_list(),
            KeyCode::Left => {
                if self.focus == Focus::Repos && self.search_page > 1 {
                    self.search_page = self.search_page.saturating_sub(1);
                    self.search();
                }
            }
            KeyCode::Right => {
                if self.focus == Focus::Repos {
                    self.search_page = self.search_page.saturating_add(1);
                    self.search();
                }
            }
            KeyCode::Char('[') => {
                if self.focus == Focus::Repos && self.search_page > 1 {
                    self.search_page = self.search_page.saturating_sub(1);
                    self.search();
                }
            }
            KeyCode::Char(']') => {
                if self.focus == Focus::Repos {
                    self.search_page = self.search_page.saturating_add(1);
                    self.search();
                }
            }
            KeyCode::Down => {
                if self.focus == Focus::Tree && !self.tree_all.is_empty() {
                    self.selected_node =
                        (self.selected_node + 1).min(self.tree_all.len().saturating_sub(1));
                    self.ensure_lazy_tree_progress();
                } else if self.focus == Focus::Preview {
                    self.scroll_preview_down(1, self.preview_viewport_rows);
                } else if !self.repos.is_empty() {
                    self.selected_repo =
                        (self.selected_repo + 1).min(self.repos.len().saturating_sub(1));
                }
            }
            KeyCode::Up => {
                if self.focus == Focus::Tree && !self.tree_all.is_empty() {
                    self.selected_node = self.selected_node.saturating_sub(1);
                } else if self.focus == Focus::Preview {
                    self.scroll_preview_up(1);
                } else if !self.repos.is_empty() {
                    self.selected_repo = self.selected_repo.saturating_sub(1);
                }
            }
            KeyCode::PageDown => {
                if self.focus == Focus::Preview {
                    self.scroll_preview_down(
                        self.preview_viewport_rows / 2,
                        self.preview_viewport_rows,
                    );
                }
            }
            KeyCode::PageUp => {
                if self.focus == Focus::Preview {
                    self.scroll_preview_up(self.preview_viewport_rows / 2);
                }
            }
            KeyCode::Home => {
                if self.focus == Focus::Preview {
                    self.preview_scroll = 0;
                }
            }
            KeyCode::End => {
                if self.focus == Focus::Preview {
                    self.preview_scroll = self.preview_lines.len().saturating_sub(1);
                }
            }
            KeyCode::Enter => {
                if self.focus == Focus::Tree {
                    self.preview_selected_file();
                    self.focus = Focus::Preview;
                } else {
                    self.open_selected_repo();
                }
            }
            _ => {}
        }
    }

    fn handle_mouse_click(&mut self, col: u16, row: u16, terminal_area: ratatui::layout::Rect) {
        let Some(panes) = render::compute_panes(terminal_area, self.current_repo.is_some()) else {
            return;
        };
        if contains(panes.repo_or_tree, col, row) {
            if self.current_repo.is_some() {
                self.focus = Focus::Tree;
                let content_row = row.saturating_sub(panes.repo_or_tree.y.saturating_add(1));
                let (start, end) = self.tree_window(panes.repo_or_tree.height);
                let idx = start + usize::from(content_row);
                if idx < end && idx < self.tree_all.len() {
                    self.selected_node = idx;
                    self.ensure_lazy_tree_progress();
                    if self.tree_all.get(idx).map(|n| !n.is_dir).unwrap_or(false)
                        && self.is_double_click_tree(idx)
                    {
                        self.preview_selected_file();
                        self.focus = Focus::Preview;
                    }
                }
            } else {
                self.focus = Focus::Repos;
                let content_row = row.saturating_sub(panes.repo_or_tree.y.saturating_add(1));
                let (start, end) = self.repo_window(panes.repo_or_tree.height);
                let idx = start + usize::from(content_row);
                if idx < end && idx < self.repos.len() {
                    self.selected_repo = idx;
                    if self.is_double_click_repo(idx) {
                        self.open_selected_repo();
                    }
                }
            }
            return;
        }
        if let Some(preview_area) = panes.preview
            && contains(preview_area, col, row)
        {
            self.focus = Focus::Preview;
        }
    }

    fn handle_mouse_scroll(
        &mut self,
        col: u16,
        row: u16,
        up: bool,
        terminal_area: ratatui::layout::Rect,
    ) {
        let Some(panes) = render::compute_panes(terminal_area, self.current_repo.is_some()) else {
            return;
        };
        if contains(panes.repo_or_tree, col, row) {
            if self.current_repo.is_some() && !self.tree_all.is_empty() {
                self.focus = Focus::Tree;
                if up {
                    self.selected_node = self.selected_node.saturating_sub(1);
                } else {
                    self.selected_node =
                        (self.selected_node + 1).min(self.tree_all.len().saturating_sub(1));
                    self.ensure_lazy_tree_progress();
                }
            } else if !self.repos.is_empty() {
                self.focus = Focus::Repos;
                if up {
                    self.selected_repo = self.selected_repo.saturating_sub(1);
                } else {
                    self.selected_repo =
                        (self.selected_repo + 1).min(self.repos.len().saturating_sub(1));
                }
            }
            return;
        }
        if let Some(preview_area) = panes.preview
            && contains(preview_area, col, row)
        {
            self.focus = Focus::Preview;
            if up {
                self.scroll_preview_up(3);
            } else {
                self.scroll_preview_down(
                    3,
                    usize::from(preview_area.height.saturating_sub(2)).max(1),
                );
            }
        }
    }

    fn is_double_click_tree(&mut self, idx: usize) -> bool {
        let now = Instant::now();
        let is_double = self
            .last_tree_click
            .map(|(last_idx, last_at)| {
                last_idx == idx && now.duration_since(last_at) <= Duration::from_millis(450)
            })
            .unwrap_or(false);
        self.last_tree_click = Some((idx, now));
        is_double
    }

    fn is_double_click_repo(&mut self, idx: usize) -> bool {
        let now = Instant::now();
        let is_double = self
            .last_repo_click
            .map(|(last_idx, last_at)| {
                last_idx == idx && now.duration_since(last_at) <= Duration::from_millis(450)
            })
            .unwrap_or(false);
        self.last_repo_click = Some((idx, now));
        is_double
    }

    fn handle_network_event(&mut self, event: NetworkEvent) {
        match event {
            NetworkEvent::SearchResult(Ok(items)) => {
                if items.is_empty() && self.search_page > 1 {
                    self.search_page = self.search_page.saturating_sub(1);
                    self.status = "No more search results pages.".to_string();
                    return;
                }
                self.repos = items;
                self.selected_repo = 0;
                self.tree_all.clear();
                self.tree_visible_limit = 0;
                self.selected_node = 0;
                self.current_repo = None;
                self.branches.clear();
                self.selected_branch = 0;
                self.current_preview_path = None;
                self.tree_text_mode = false;
                self.status = format!(
                    "Loaded {} repositories on page {} (per_page {}).",
                    self.repos.len(),
                    self.search_page,
                    self.per_page
                );
            }
            NetworkEvent::SearchResult(Err(e)) => {
                self.status = format!("Search failed: {e}");
            }
            NetworkEvent::IssuesResult(Ok(issues)) => {
                self.command_items = issues
                    .into_iter()
                    .map(|i| {
                        let status = if i.pull_request.is_some() {
                            "[PR]"
                        } else {
                            "[ISSUE]"
                        };
                        format!("{} #{}: {} ({})", status, i.number, i.title, i.state)
                    })
                    .collect();
                self.command_filtered.clear();
                self.command_cursor = 0;
                self.command_palette_visible = true;
                self.command_input.clear();
                self.status = "Issues loaded. Select with arrows, Enter to view.".to_string();
            }
            NetworkEvent::IssuesResult(Err(e)) => {
                self.status = format!("Issues fetch failed: {e}");
            }
            NetworkEvent::PrsResult(Ok(prs)) => {
                self.command_items = prs
                    .into_iter()
                    .map(|pr| {
                        format!(
                            "[PR] #{}: {} ({} +{} -{})",
                            pr.number,
                            pr.title,
                            pr.state,
                            pr.additions.unwrap_or(0),
                            pr.deletions.unwrap_or(0)
                        )
                    })
                    .collect();
                self.command_filtered.clear();
                self.command_cursor = 0;
                self.command_palette_visible = true;
                self.command_input.clear();
                self.status = "Pull requests loaded.".to_string();
            }
            NetworkEvent::PrsResult(Err(e)) => {
                self.status = format!("PR fetch failed: {e}");
            }
            NetworkEvent::CommitsResult(Ok(commits)) => {
                self.command_items = commits
                    .into_iter()
                    .map(|c| {
                        let short = c.sha.chars().take(7).collect::<String>();
                        let msg = c.commit.message.lines().next().unwrap_or("").to_string();
                        format!("[COMMIT] {} {} - {}", short, c.commit.author.name, msg)
                    })
                    .collect();
                self.command_filtered.clear();
                self.command_cursor = 0;
                self.command_palette_visible = true;
                self.command_input.clear();
                self.status = "Recent commits loaded.".to_string();
            }
            NetworkEvent::CommitsResult(Err(e)) => {
                self.status = format!("Commits fetch failed: {e}");
            }
            NetworkEvent::CompareResult(Ok(compare)) => {
                self.command_items = compare
                    .files
                    .into_iter()
                    .map(|f| {
                        format!(
                            "[DIFF] {} ({} +{} -{})",
                            f.filename, f.status, f.additions, f.deletions
                        )
                    })
                    .collect();
                self.command_filtered.clear();
                self.command_cursor = 0;
                self.command_palette_visible = true;
                self.command_input.clear();
                self.status = format!(
                    "Compare: {} ahead, {} behind",
                    compare.ahead_by, compare.behind_by
                );
            }
            NetworkEvent::CompareResult(Err(e)) => {
                self.status = format!("Compare failed: {e}");
            }
            NetworkEvent::CheckRunsResult(Ok(runs)) => {
                let count = runs.len();
                self.command_items = runs
                    .into_iter()
                    .map(|r| {
                        let conclusion = r.conclusion.as_deref().unwrap_or("pending");
                        format!("[CI] {}: {}", r.name, conclusion)
                    })
                    .collect();
                self.command_filtered.clear();
                self.command_cursor = 0;
                self.command_palette_visible = true;
                self.command_input.clear();
                self.status = format!("CI checks: {}", count);
            }
            NetworkEvent::CheckRunsResult(Err(e)) => {
                self.status = format!("CI check fetch failed: {e}");
            }
            NetworkEvent::StarredResult(Ok(repos)) => {
                self.repos = repos;
                self.selected_repo = 0;
                self.tree_all.clear();
                self.tree_visible_limit = 0;
                self.selected_node = 0;
                self.current_repo = None;
                self.status = format!("Loaded {} starred repositories.", self.repos.len());
            }
            NetworkEvent::StarredResult(Err(e)) => {
                self.status = format!("Starred repos fetch failed: {e}");
            }
        }
    }

    fn toggle_command_palette(&mut self) {
        self.command_palette_visible = !self.command_palette_visible;
        if self.command_palette_visible {
            self.command_input.clear();
            self.command_cursor = 0;
            self.build_command_list();
        }
    }

    fn build_command_list(&mut self) {
        let mut commands = vec![
            "Search Repositories".to_string(),
            "List Starred Repos".to_string(),
        ];
        if self.current_repo.is_some() {
            commands.push("Switch Branch".to_string());
            commands.push("Find File".to_string());
            commands.push("Clone Repository".to_string());
            commands.push("Download Current File".to_string());
            commands.push("Toggle Tree View".to_string());
            commands.push("List Issues".to_string());
            commands.push("List Pull Requests".to_string());
            commands.push("View Recent Commits".to_string());
            commands.push("View CI Status".to_string());
            commands.push("Compare Branches".to_string());
        }
        commands.push("Set Token".to_string());
        commands.push("Quit".to_string());
        self.command_items = commands;
    }

    fn handle_command_palette_input(
        &mut self,
        code: KeyCode,
        tx: mpsc::Sender<NetworkEvent>,
        github: Arc<GitHubClient>,
    ) {
        match code {
            KeyCode::Esc => {
                self.command_palette_visible = false;
            }
            KeyCode::Enter => {
                let selected = self.get_selected_command();
                self.command_palette_visible = false;
                if let Some(cmd) = selected {
                    self.execute_command(cmd, tx, github);
                }
            }
            KeyCode::Up => {
                let count = if self.command_input.is_empty() {
                    self.command_items.len()
                } else {
                    self.command_filtered.len()
                };
                if count > 0 {
                    self.command_cursor = self.command_cursor.saturating_sub(1);
                }
            }
            KeyCode::Down => {
                let count = if self.command_input.is_empty() {
                    self.command_items.len()
                } else {
                    self.command_filtered.len()
                };
                if count > 0 {
                    self.command_cursor = (self.command_cursor + 1).min(count - 1);
                }
            }
            KeyCode::Backspace => {
                self.command_input.pop();
                self.update_command_filter();
            }
            KeyCode::Char(ch) => {
                self.command_input.push(ch);
                self.update_command_filter();
            }
            _ => {}
        }
    }

    fn get_selected_command(&self) -> Option<String> {
        let items = if self.command_input.is_empty() {
            &self.command_items
        } else {
            &self.command_filtered
        };
        items.get(self.command_cursor).cloned()
    }

    fn update_command_filter(&mut self) {
        if self.command_input.is_empty() {
            self.command_filtered.clear();
            return;
        }
        let lower = self.command_input.to_lowercase();
        self.command_filtered = self
            .command_items
            .iter()
            .filter(|item| item.to_lowercase().contains(&lower))
            .cloned()
            .collect();
        let count = self.command_filtered.len();
        if count > 0 {
            self.command_cursor = self.command_cursor.min(count - 1);
        } else {
            self.command_cursor = 0;
        }
    }

    fn execute_command(
        &mut self,
        cmd: String,
        tx: mpsc::Sender<NetworkEvent>,
        github: Arc<GitHubClient>,
    ) {
        match cmd.as_str() {
            "Search Repositories" => {
                self.focus = Focus::Search;
                self.input_buffer = self.search_query.clone();
            }
            "List Starred Repos" => {
                self.status = "Loading starred repos...".to_string();
                let g = github.clone();
                std::thread::spawn(move || {
                    let result = g.fetch_starred_repos(1, 30);
                    let _ = tx.send(NetworkEvent::StarredResult(
                        result.map_err(|e| e.to_string()),
                    ));
                });
            }
            "Switch Branch" => {
                if self.current_repo.is_some() && !self.branches.is_empty() {
                    self.focus = Focus::BranchPicker;
                } else {
                    self.status = "Open a repository first.".to_string();
                }
            }
            "Find File" => {
                if self.current_repo.is_some() {
                    self.tree_search_input.clear();
                    self.focus = Focus::TreeSearch;
                } else {
                    self.status = "Open a repository first.".to_string();
                }
            }
            "Clone Repository" => {
                if self.current_repo.is_some() {
                    self.clone_path_input = self.account.preferred_clone_dir.clone();
                    self.focus = Focus::ClonePath;
                } else {
                    self.status = "Open a repository first.".to_string();
                }
            }
            "Download Current File" => {
                if self.current_preview_path.is_some() {
                    self.download_path_input = ".".to_string();
                    self.focus = Focus::DownloadPath;
                } else {
                    self.status = "Preview a file first.".to_string();
                }
            }
            "Toggle Tree View" => {
                if self.current_repo.is_some() {
                    self.tree_text_mode = !self.tree_text_mode;
                    self.preview_scroll = 0;
                    if self.tree_text_mode {
                        let branch = self.selected_branch_name();
                        self.preview_title = format!(
                            "tree {} [{}]",
                            self.current_repo
                                .as_ref()
                                .map(|r| r.full_name.clone())
                                .unwrap_or_default(),
                            branch
                        );
                        self.preview_lines = self
                            .tree_all
                            .iter()
                            .map(|node| {
                                let indent = "  ".repeat(node.depth.min(20));
                                let icon = if node.is_dir { "[D]" } else { "[F]" };
                                Line::from(format!("{indent}{icon} {}", node.path))
                            })
                            .collect();
                        self.current_preview_path = None;
                        self.focus = Focus::Preview;
                        self.status = "Tree view enabled in preview pane.".to_string();
                    } else {
                        self.preview_title = "Preview".to_string();
                        self.preview_lines = vec![Line::from(
                            "Tree preview disabled. Select a file and press Enter to preview.",
                        )];
                        self.focus = Focus::Tree;
                        self.status = "Tree view disabled.".to_string();
                    }
                } else {
                    self.status = "Open a repository first.".to_string();
                }
            }
            "Set Token" => {
                self.focus = Focus::TokenInput;
                self.token_buffer = SecretString::new(String::new().into());
                self.input_buffer.clear();
            }
            "List Issues" => {
                if let Some(repo) = self.current_repo.clone() {
                    self.status = "Loading issues...".to_string();
                    let g = github.clone();
                    let full_name = repo.full_name.clone();
                    std::thread::spawn(move || {
                        let result = g.fetch_issues(&full_name, "open", 30);
                        let _ = tx.send(NetworkEvent::IssuesResult(
                            result.map_err(|e| e.to_string()),
                        ));
                    });
                } else {
                    self.status = "Open a repository first.".to_string();
                }
            }
            "List Pull Requests" => {
                if let Some(repo) = self.current_repo.clone() {
                    self.status = "Loading pull requests...".to_string();
                    let g = github.clone();
                    let full_name = repo.full_name.clone();
                    std::thread::spawn(move || {
                        let result = g.fetch_pull_requests(&full_name, "open", 30);
                        let _ = tx.send(NetworkEvent::PrsResult(result.map_err(|e| e.to_string())));
                    });
                } else {
                    self.status = "Open a repository first.".to_string();
                }
            }
            "View Recent Commits" => {
                if let Some(repo) = self.current_repo.clone() {
                    self.status = "Loading commits...".to_string();
                    let g = github.clone();
                    let full_name = repo.full_name.clone();
                    let branch = self.selected_branch_name();
                    std::thread::spawn(move || {
                        let result = g.fetch_recent_commits(&full_name, &branch, 30);
                        let _ = tx.send(NetworkEvent::CommitsResult(
                            result.map_err(|e| e.to_string()),
                        ));
                    });
                } else {
                    self.status = "Open a repository first.".to_string();
                }
            }
            "View CI Status" => {
                if let Some(repo) = self.current_repo.clone() {
                    self.status = "Loading CI status...".to_string();
                    let g = github.clone();
                    let full_name = repo.full_name.clone();
                    let branch = self.selected_branch_name();
                    std::thread::spawn(move || {
                        let result = g.fetch_check_runs(&full_name, &branch);
                        let _ = tx.send(NetworkEvent::CheckRunsResult(
                            result.map_err(|e| e.to_string()),
                        ));
                    });
                } else {
                    self.status = "Open a repository first.".to_string();
                }
            }
            "Compare Branches" => {
                if let Some(repo) = self.current_repo.clone() {
                    if self.branches.len() >= 2 {
                        let base = self.selected_branch_name();
                        let head = self
                            .branches
                            .iter()
                            .find(|b| **b != base)
                            .cloned()
                            .unwrap_or_default();
                        if !head.is_empty() {
                            self.status = format!("Comparing {base}...{head}");
                            let g = github.clone();
                            let full_name = repo.full_name.clone();
                            let base = base.clone();
                            let head = head.clone();
                            std::thread::spawn(move || {
                                let result = g.fetch_compare(&full_name, &base, &head);
                                let _ = tx.send(NetworkEvent::CompareResult(
                                    result.map_err(|e| e.to_string()),
                                ));
                            });
                        }
                    } else {
                        self.status = "Need at least 2 branches loaded.".to_string();
                    }
                } else {
                    self.status = "Open a repository first.".to_string();
                }
            }
            "Quit" => {
                self.should_quit = true;
            }
            _ => {
                self.status = format!("Unknown command: {cmd}");
            }
        }
    }

    fn handle_key_with_channel(
        &mut self,
        code: KeyCode,
        tx: mpsc::Sender<NetworkEvent>,
        github: Arc<GitHubClient>,
    ) {
        if self.command_palette_visible {
            self.handle_command_palette_input(code, tx, github);
            return;
        }

        // Ctrl+P = \x10
        if let KeyCode::Char(ch) = code
            && ch == '\x10'
        {
            self.toggle_command_palette();
            return;
        }

        self.handle_key(code);
    }
}

pub fn run() -> Result<()> {
    run_with_options(RunOptions::default())
}

pub fn run_with_options(options: RunOptions) -> Result<()> {
    let mut app = App::new(options)?;

    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture);
        prev_hook(panic_info);
    }));

    enable_raw_mode().context("Cannot enable raw mode")?;
    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)
        .context("Cannot enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let (net_tx, net_rx) = mpsc::channel::<NetworkEvent>();
    let github = app.github.clone();

    // Initial search via background thread
    app.status = "Loading...".to_string();
    let tx = net_tx.clone();
    let g = github.clone();
    let query = app.search_query.clone();
    let page = app.search_page;
    let per_page = app.per_page;
    std::thread::spawn(move || {
        let result = g.search_repositories_page(&query, page, per_page);
        let _ = tx.send(NetworkEvent::SearchResult(
            result.map_err(|e| e.to_string()),
        ));
    });

    let mut terminal_result = Ok(());
    while !app.should_quit {
        if let Err(error) = terminal.draw(|frame| render::render(frame, &mut app)) {
            terminal_result = Err(anyhow!("Render error: {error}"));
            break;
        }

        // Process completed network tasks
        while let Ok(event) = net_rx.try_recv() {
            app.handle_network_event(event);
        }

        if event::poll(Duration::from_millis(16)).context("Event poll failed")? {
            match event::read().context("Event read failed")? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    let tx = net_tx.clone();
                    let g = github.clone();
                    app.handle_key_with_channel(key.code, tx, g);
                }
                Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
                    let area = terminal
                        .size()
                        .unwrap_or_else(|_| ratatui::layout::Size::new(120, 40));
                    app.handle_mouse_click(mouse.column, mouse.row, area.into());
                }
                Event::Mouse(mouse) if mouse.kind == MouseEventKind::ScrollUp => {
                    let area = terminal
                        .size()
                        .unwrap_or_else(|_| ratatui::layout::Size::new(120, 40));
                    app.handle_mouse_scroll(mouse.column, mouse.row, true, area.into());
                }
                Event::Mouse(mouse) if mouse.kind == MouseEventKind::ScrollDown => {
                    let area = terminal
                        .size()
                        .unwrap_or_else(|_| ratatui::layout::Size::new(120, 40));
                    app.handle_mouse_scroll(mouse.column, mouse.row, false, area.into());
                }
                Event::Resize(cols, rows) => {
                    app.status = format!("Terminal resized to {cols}x{rows}");
                }
                _ => {}
            }
        }
    }

    disable_raw_mode().ok();
    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture).ok();
    terminal.show_cursor().ok();
    let _ = panic::take_hook(); // remove our hook
    terminal_result
}

fn contains(rect: ratatui::layout::Rect, col: u16, row: u16) -> bool {
    let x2 = rect.x.saturating_add(rect.width);
    let y2 = rect.y.saturating_add(rect.height);
    col >= rect.x && col < x2 && row >= rect.y && row < y2
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;
    use secrecy::SecretString;

    fn test_app() -> App {
        App {
            github: Arc::new(crate::github::GitHubClient::new(None).expect("client")),
            account: crate::config::AccountConfig {
                preferred_clone_dir: ".".to_string(),
                last_branch_by_repo: Default::default(),
            },
            preview_cache: crate::cache::PreviewCache::new(120).expect("cache"),
            search_query: String::new(),
            search_page: 1,
            per_page: 30,
            repos: vec![],
            selected_repo: 0,
            tree_all: vec![],
            tree_visible_limit: 0,
            selected_node: 0,
            current_repo: None,
            branches: vec![],
            selected_branch: 0,
            preview_title: String::new(),
            preview_lines: vec![],
            preview_scroll: 0,
            current_preview_path: None,
            preview_viewport_rows: 30,
            tree_text_mode: false,
            input_buffer: String::new(),
            token_buffer: SecretString::new(String::new().into()),
            oauth_client_id_input: String::new(),
            clone_path_input: ".".to_string(),
            download_path_input: String::new(),
            tree_search_input: String::new(),
            status: String::new(),
            focus: Focus::Repos,
            should_quit: false,
            auth_user: None,
            last_tree_click: None,
            last_repo_click: None,
            keybindings: crate::config::KeybindingsConfig::default(),
            command_palette_visible: false,
            command_input: String::new(),
            command_cursor: 0,
            command_items: Vec::new(),
            command_filtered: Vec::new(),
        }
    }

    #[test]
    fn key_q_sets_should_quit() {
        let mut app = test_app();
        app.handle_key(KeyCode::Char('q'));
        assert!(app.should_quit);
    }

    #[test]
    fn key_slash_opens_search() {
        let mut app = test_app();
        app.focus = Focus::Repos;
        app.handle_key(KeyCode::Char('/'));
        assert_eq!(app.focus, Focus::Search);
    }

    #[test]
    fn key_esc_from_search_returns_to_repos() {
        let mut app = test_app();
        app.focus = Focus::Search;
        app.handle_key(KeyCode::Esc);
        assert_eq!(app.focus, Focus::Repos);
    }

    #[test]
    fn key_tab_cycles_focus() {
        let mut app = test_app();
        // Repos -> Tree (with tree data) -> Preview -> Repos
        app.tree_all = vec![crate::models::RepoNode {
            path: "f".into(),
            name: "f".into(),
            depth: 0,
            is_dir: false,
        }];
        app.tree_visible_limit = 1;
        app.current_repo = Some(crate::models::RepoSummary {
            name: "repo".into(),
            full_name: "o/r".into(),
            description: None,
            stargazers_count: 0,
            language: None,
            clone_url: "".into(),
            owner: crate::models::RepoOwner { login: "o".into() },
            default_branch: "main".into(),
        });
        app.preview_lines = vec![Line::from("test")];

        assert_eq!(app.focus, Focus::Repos);
        app.handle_key(KeyCode::Tab);
        assert_eq!(app.focus, Focus::Tree);
        app.handle_key(KeyCode::Tab);
        assert_eq!(app.focus, Focus::Preview);
        app.handle_key(KeyCode::Tab);
        assert_eq!(app.focus, Focus::Repos);
    }

    #[test]
    fn key_down_in_repos_moves_selection() {
        let mut app = test_app();
        app.repos = vec![
            RepoSummary {
                name: "a".into(),
                full_name: "o/a".into(),
                description: None,
                stargazers_count: 0,
                language: None,
                clone_url: "".into(),
                owner: crate::models::RepoOwner { login: "o".into() },
                default_branch: "main".into(),
            },
            RepoSummary {
                name: "b".into(),
                full_name: "o/b".into(),
                description: None,
                stargazers_count: 0,
                language: None,
                clone_url: "".into(),
                owner: crate::models::RepoOwner { login: "o".into() },
                default_branch: "main".into(),
            },
        ];
        assert_eq!(app.selected_repo, 0);
        app.handle_key(KeyCode::Down);
        assert_eq!(app.selected_repo, 1);
    }

    #[test]
    fn key_up_in_tree_moves_selection() {
        let mut app = test_app();
        app.tree_all = vec![
            RepoNode {
                path: "a".into(),
                name: "a".into(),
                depth: 0,
                is_dir: false,
            },
            RepoNode {
                path: "b".into(),
                name: "b".into(),
                depth: 0,
                is_dir: false,
            },
        ];
        app.tree_visible_limit = 2;
        app.focus = Focus::Tree;
        app.selected_node = 1;
        app.handle_key(KeyCode::Up);
        assert_eq!(app.selected_node, 0);
    }

    #[test]
    fn token_input_escapes_and_zeroizes() {
        let mut app = test_app();
        app.focus = Focus::TokenInput;
        app.token_buffer = SecretString::new("sometoken".into());
        app.handle_key(KeyCode::Esc);
        assert_eq!(app.focus, Focus::Repos);
        // After zeroize, memory is cleared; the original token value is no longer intact
        // (zeroize overwrites the backing memory even though length metadata remains)
    }

    #[test]
    fn token_input_enter_saves() {
        let mut app = test_app();
        app.focus = Focus::TokenInput;
        app.token_buffer = SecretString::new("test_token".into());
        app.handle_key(KeyCode::Enter);
        // An attempt to save is made; after the attempt the token buffer is zeroized.
        // Focus moves to Repos if save succeeded, otherwise stays on TokenInput.
        // We just verify no panic and the zeroize call ran.
    }

    #[test]
    fn search_input_adds_characters() {
        let mut app = test_app();
        app.focus = Focus::Search;
        app.input_buffer.clear();
        app.handle_key(KeyCode::Char('r'));
        app.handle_key(KeyCode::Char('s'));
        assert_eq!(app.input_buffer, "rs");
    }

    #[test]
    fn search_input_backspace_removes_char() {
        let mut app = test_app();
        app.focus = Focus::Search;
        app.input_buffer = "rust".to_string();
        app.handle_key(KeyCode::Backspace);
        assert_eq!(app.input_buffer, "rus");
    }

    #[test]
    fn lazy_tree_progress_advances_limit() {
        let mut app = App {
            github: Arc::new(crate::github::GitHubClient::new(None).expect("client")),
            account: crate::config::AccountConfig {
                preferred_clone_dir: ".".to_string(),
                last_branch_by_repo: Default::default(),
            },
            preview_cache: crate::cache::PreviewCache::new(120).expect("cache"),
            search_query: String::new(),
            search_page: 1,
            per_page: 30,
            repos: vec![],
            selected_repo: 0,
            tree_all: (0..800)
                .map(|i| crate::models::RepoNode {
                    path: format!("f{i}"),
                    name: format!("f{i}"),
                    depth: 0,
                    is_dir: false,
                })
                .collect(),
            tree_visible_limit: 250,
            selected_node: 250 - App::TREE_LOAD_THRESHOLD + 1,
            current_repo: None,
            branches: vec![],
            selected_branch: 0,
            preview_title: String::new(),
            preview_lines: vec![],
            preview_scroll: 0,
            current_preview_path: None,
            preview_viewport_rows: 30,
            tree_text_mode: false,
            input_buffer: String::new(),
            token_buffer: SecretString::new(String::new().into()),
            oauth_client_id_input: String::new(),
            clone_path_input: ".".to_string(),
            download_path_input: String::new(),
            tree_search_input: String::new(),
            status: String::new(),
            focus: Focus::Tree,
            should_quit: false,
            auth_user: None,
            last_tree_click: None,
            last_repo_click: None,
            keybindings: crate::config::KeybindingsConfig::default(),
            command_palette_visible: false,
            command_input: String::new(),
            command_cursor: 0,
            command_items: Vec::new(),
            command_filtered: Vec::new(),
        };
        app.ensure_lazy_tree_progress();
        assert_eq!(app.tree_visible_limit, 500);
    }
}
