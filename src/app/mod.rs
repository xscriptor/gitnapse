mod render;
mod theme;

use crate::auth;
use crate::cache::PreviewCache;
use crate::config::AccountConfig;
use crate::github::GitHubClient;
use crate::models::{RepoNode, RepoSummary};
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
use ratatui::backend::CrosstermBackend;
use ratatui::text::Line;
use ratatui::Terminal;
use std::io::stdout;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

const TREE_PAGE_SIZE: usize = 250;
const TREE_LOAD_THRESHOLD: usize = 15;

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
    BranchPicker,
}

pub struct App {
    pub github: GitHubClient,
    pub account: AccountConfig,
    pub search_query: String,
    pub repos: Vec<RepoSummary>,
    pub selected_repo: usize,
    pub tree_all: Vec<RepoNode>,
    pub tree_visible_limit: usize,
    pub selected_node: usize,
    pub branches: Vec<String>,
    pub selected_branch: usize,
    pub preview_title: String,
    pub preview_lines: Vec<Line<'static>>,
    pub preview_scroll: usize,
    pub current_preview_path: Option<String>,
    pub tree_search_input: String,
    pub download_path_input: String,
    pub tree_text_mode: bool,
    pub status: String,
    pub focus: Focus,
    pub input_buffer: String,
    pub clone_path_input: String,
    pub should_quit: bool,
    pub current_repo: Option<RepoSummary>,
    pub auth_user: Option<String>,
    pub search_page: u32,
    pub per_page: u8,
    pub preview_cache: PreviewCache,
    pub last_tree_click: Option<(usize, Instant)>,
    pub last_repo_click: Option<(usize, Instant)>,
}

impl App {
    fn new(options: RunOptions) -> Result<Self> {
        let token = auth::load_token()?;
        let github = GitHubClient::new(token.as_deref())?;
        let mut account = AccountConfig::load_or_default()?;
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
            search_query: options.initial_query,
            repos: Vec::new(),
            selected_repo: 0,
            tree_all: Vec::new(),
            tree_visible_limit: 0,
            selected_node: 0,
            branches: Vec::new(),
            selected_branch: 0,
            preview_title: "Preview".to_string(),
            preview_lines: vec![Line::from("Select a repository and a file to preview.")],
            preview_scroll: 0,
            current_preview_path: None,
            tree_search_input: String::new(),
            download_path_input: String::new(),
            tree_text_mode: false,
            status: match auth_user.as_ref() {
                Some(login) => format!("Authenticated as {login}. Press / to search."),
                None => "No validated token. Press t to save one or continue anonymously."
                    .to_string(),
            },
            focus: Focus::Repos,
            input_buffer: String::new(),
            clone_path_input: account.preferred_clone_dir,
            should_quit: false,
            current_repo: None,
            auth_user,
            search_page: options.initial_page.max(1),
            per_page: options.per_page.clamp(1, 100),
            preview_cache,
            last_tree_click: None,
            last_repo_click: None,
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
        if self.selected_node + TREE_LOAD_THRESHOLD >= self.tree_visible_limit {
            self.tree_visible_limit = (self.tree_visible_limit + TREE_PAGE_SIZE).min(self.tree_all.len());
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
        self.tree_visible_limit = self.tree_all.len().min(TREE_PAGE_SIZE);
        self.current_preview_path = None;
        self.tree_text_mode = false;
    }

    fn search(&mut self) {
        match self
            .github
            .search_repositories_page(&self.search_query, self.search_page, self.per_page)
        {
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
            self.preview_lines = highlight_content(&content, &node_path, 300);
            self.preview_scroll = 0;
            self.current_preview_path = Some(node_path.clone());
            self.tree_text_mode = false;
            self.status = format!("Preview loaded from cache for {}", node_path);
            return;
        }

        match self.github.fetch_file_content(&full_name, &node_path) {
            Ok(content) => {
                self.preview_cache.put(&full_name, &branch, &node_path, &content);
                self.preview_title = format!("{}/{}", full_name, node_path);
                self.preview_lines = highlight_content(&content, &node_path, 300);
                self.preview_scroll = 0;
                self.current_preview_path = Some(node_path.clone());
                self.tree_text_mode = false;
                self.status = format!("Preview loaded for {}", node_path);
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
        if !destination_path.exists() {
            if let Err(error) = std::fs::create_dir_all(&destination_path) {
                self.status = format!(
                    "Cannot create destination path {}: {error}",
                    destination_path.display()
                );
                return;
            }
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

    fn save_token_from_input(&mut self) {
        let token_owned = self.input_buffer.trim().to_string();
        if token_owned.is_empty() {
            self.status = "Token is empty.".to_string();
            return;
        }

        match auth::save_token(&token_owned)
            .and_then(|_| GitHubClient::new(Some(&token_owned)).context("Cannot rebuild HTTP client"))
        {
            Ok(client) => {
                self.github = client;
                self.auth_user = self.github.fetch_authenticated_user().ok().flatten();
                self.status = match self.auth_user.as_ref() {
                    Some(login) => format!("Token saved and validated as {login}."),
                    None => "Token saved, but validation failed.".to_string(),
                };
                self.input_buffer.clear();
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
            Focus::BranchPicker => self.handle_branch_picker_input(code),
            Focus::Repos | Focus::Tree | Focus::Preview => self.handle_navigation(code),
        }
    }

    fn max_preview_scroll(&self, viewport_rows: usize) -> usize {
        self.preview_lines.len().saturating_sub(viewport_rows.max(1))
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
        let start = self.selected_node.saturating_sub(viewport_rows / 2).min(max_start);
        let end = (start + viewport_rows).min(visible.len());
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
                    self.status = format!("Found file match for \"{}\".", self.tree_search_input.trim());
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
                self.input_buffer.clear();
                self.focus = Focus::Repos;
            }
            KeyCode::Enter => self.save_token_from_input(),
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(ch) => self.input_buffer.push(ch),
            _ => {}
        }
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
                                let indent = "  ".repeat(node.depth.min(10));
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
                    self.selected_node = (self.selected_node + 1).min(self.tree_all.len().saturating_sub(1));
                    self.ensure_lazy_tree_progress();
                } else if self.focus == Focus::Preview {
                    self.scroll_preview_down(1, 30);
                } else if !self.repos.is_empty() {
                    self.selected_repo = (self.selected_repo + 1).min(self.repos.len().saturating_sub(1));
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
                    self.scroll_preview_down(20, 30);
                }
            }
            KeyCode::PageUp => {
                if self.focus == Focus::Preview {
                    self.scroll_preview_up(20);
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
                    if self
                        .tree_all
                        .get(idx)
                        .map(|n| !n.is_dir)
                        .unwrap_or(false)
                        && self.is_double_click_tree(idx)
                    {
                        self.preview_selected_file();
                        self.focus = Focus::Preview;
                    }
                }
            } else {
                self.focus = Focus::Repos;
                let content_row = row.saturating_sub(panes.repo_or_tree.y.saturating_add(1));
                let idx = usize::from(content_row);
                if idx < self.repos.len() {
                    self.selected_repo = idx;
                    if self.is_double_click_repo(idx) {
                        self.open_selected_repo();
                    }
                }
            }
            return;
        }
        if let Some(preview_area) = panes.preview && contains(preview_area, col, row) {
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
                    self.selected_node = (self.selected_node + 1).min(self.tree_all.len().saturating_sub(1));
                    self.ensure_lazy_tree_progress();
                }
            } else if !self.repos.is_empty() {
                self.focus = Focus::Repos;
                if up {
                    self.selected_repo = self.selected_repo.saturating_sub(1);
                } else {
                    self.selected_repo = (self.selected_repo + 1).min(self.repos.len().saturating_sub(1));
                }
            }
            return;
        }
        if let Some(preview_area) = panes.preview && contains(preview_area, col, row) {
            self.focus = Focus::Preview;
            if up {
                self.scroll_preview_up(3);
            } else {
                self.scroll_preview_down(3, usize::from(preview_area.height.saturating_sub(2)).max(1));
            }
        }
    }

    fn is_double_click_tree(&mut self, idx: usize) -> bool {
        let now = Instant::now();
        let is_double = self
            .last_tree_click
            .map(|(last_idx, last_at)| last_idx == idx && now.duration_since(last_at) <= Duration::from_millis(450))
            .unwrap_or(false);
        self.last_tree_click = Some((idx, now));
        is_double
    }

    fn is_double_click_repo(&mut self, idx: usize) -> bool {
        let now = Instant::now();
        let is_double = self
            .last_repo_click
            .map(|(last_idx, last_at)| last_idx == idx && now.duration_since(last_at) <= Duration::from_millis(450))
            .unwrap_or(false);
        self.last_repo_click = Some((idx, now));
        is_double
    }
}

pub fn run() -> Result<()> {
    run_with_options(RunOptions::default())
}

pub fn run_with_options(options: RunOptions) -> Result<()> {
    let mut app = App::new(options)?;
    app.search();

    enable_raw_mode().context("Cannot enable raw mode")?;
    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)
        .context("Cannot enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut terminal_result = Ok(());
    while !app.should_quit {
        if let Err(error) = terminal.draw(|frame| render::render(frame, &app)) {
            terminal_result = Err(anyhow!("Render error: {error}"));
            break;
        }

        if event::poll(Duration::from_millis(120)).context("Event poll failed")? {
            match event::read().context("Event read failed")? {
                Event::Key(key) if key.kind == KeyEventKind::Press => app.handle_key(key.code),
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
                _ => {}
            }
        }
    }

    disable_raw_mode().ok();
    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture).ok();
    terminal.show_cursor().ok();
    terminal_result
}

fn contains(rect: ratatui::layout::Rect, col: u16, row: u16) -> bool {
    let x2 = rect.x.saturating_add(rect.width);
    let y2 = rect.y.saturating_add(rect.height);
    col >= rect.x && col < x2 && row >= rect.y && row < y2
}

#[cfg(test)]
mod tests {
    use super::{TREE_LOAD_THRESHOLD, App};

    #[test]
    fn lazy_tree_progress_advances_limit() {
        let mut app = App {
            github: crate::github::GitHubClient::new(None).expect("client"),
            account: crate::config::AccountConfig {
                preferred_clone_dir: ".".to_string(),
                last_branch_by_repo: Default::default(),
            },
            search_query: String::new(),
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
            selected_node: 250 - TREE_LOAD_THRESHOLD + 1,
            branches: vec![],
            selected_branch: 0,
            preview_title: String::new(),
            preview_lines: vec![],
            preview_scroll: 0,
            current_preview_path: None,
            tree_search_input: String::new(),
            download_path_input: String::new(),
            tree_text_mode: false,
            status: String::new(),
            focus: super::Focus::Tree,
            input_buffer: String::new(),
            clone_path_input: ".".to_string(),
            should_quit: false,
            current_repo: None,
            auth_user: None,
            search_page: 1,
            per_page: 30,
            preview_cache: crate::cache::PreviewCache::new(120).expect("cache"),
            last_tree_click: None,
            last_repo_click: None,
        };
        app.ensure_lazy_tree_progress();
        assert_eq!(app.tree_visible_limit, 500);
    }
}
