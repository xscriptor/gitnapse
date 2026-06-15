use crate::app::{App, Focus, NetworkEvent};
use crate::github::GitHubClient;
use crossterm::event::KeyCode;
use ratatui::text::Line;
use std::sync::Arc;
use std::sync::mpsc;

impl App {
    pub(super) fn scroll_preview_down(&mut self, step: usize, viewport_rows: usize) {
        let max_scroll = self.max_preview_scroll(viewport_rows);
        self.preview_scroll = (self.preview_scroll + step).min(max_scroll);
    }

    pub(super) fn scroll_preview_up(&mut self, step: usize) {
        self.preview_scroll = self.preview_scroll.saturating_sub(step);
    }

    fn max_preview_scroll(&self, viewport_rows: usize) -> usize {
        self.preview_lines
            .len()
            .saturating_sub(viewport_rows.max(1))
    }

    pub(super) fn tree_window(&self, area_height: u16) -> (usize, usize) {
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

    pub(super) fn repo_window(&self, area_height: u16) -> (usize, usize) {
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

    pub fn handle_key(&mut self, code: KeyCode) {
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

    pub(crate) fn handle_key_with_channel(
        &mut self,
        code: KeyCode,
        tx: mpsc::Sender<NetworkEvent>,
        github: Arc<GitHubClient>,
    ) {
        // PR review / creation input mode
        if self.pr_pending_action.is_some() {
            match code {
                KeyCode::Esc => {
                    self.pr_pending_action = None;
                    self.pr_pending_body.clear();
                    self.status = "Action cancelled.".to_string();
                }
                KeyCode::Enter => {
                    let text = self.pr_pending_body.trim().to_string();
                    let action = self.pr_pending_action.take().unwrap_or_default();
                    let Some(repo) = self.current_repo.clone() else {
                        self.status = "No repository loaded.".to_string();
                        return;
                    };
                    let Some(detail) = self.pr_detail.clone() else {
                        // For create_pr, detail is not needed
                        if action.starts_with("create_pr_") {
                            self.handle_pr_creation_step(action, text, tx, github);
                        } else {
                            self.status = "No PR loaded.".to_string();
                        }
                        return;
                    };
                    let full_name = repo.full_name.clone();
                    let number = detail.number;
                    let g = github.clone();

                    match action.as_str() {
                        "approve" => {
                            self.status = "Approving PR...".to_string();
                            std::thread::spawn(move || {
                                let body = if text.is_empty() {
                                    "LGTM, approved."
                                } else {
                                    &text
                                };
                                match g
                                    .create_pull_request_review(&full_name, number, body, "APPROVE")
                                {
                                    Ok(_) => {
                                        let _ = tx.send(NetworkEvent::PrActionResult(
                                            "PR approved.".to_string(),
                                        ));
                                    }
                                    Err(e) => {
                                        let _ = tx.send(NetworkEvent::PrActionResult(format!(
                                            "Approve failed: {e}"
                                        )));
                                    }
                                }
                            });
                        }
                        "request_changes" => {
                            self.status = "Requesting changes...".to_string();
                            std::thread::spawn(move || {
                                let body = if text.is_empty() {
                                    "Please address the requested changes."
                                } else {
                                    &text
                                };
                                match g.create_pull_request_review(
                                    &full_name,
                                    number,
                                    body,
                                    "REQUEST_CHANGES",
                                ) {
                                    Ok(_) => {
                                        let _ = tx.send(NetworkEvent::PrActionResult(
                                            "Changes requested.".to_string(),
                                        ));
                                    }
                                    Err(e) => {
                                        let _ = tx.send(NetworkEvent::PrActionResult(format!(
                                            "Request failed: {e}"
                                        )));
                                    }
                                }
                            });
                        }
                        "comment" => {
                            self.status = "Posting comment...".to_string();
                            std::thread::spawn(move || {
                                let body = if text.is_empty() {
                                    "Reviewed the changes."
                                } else {
                                    &text
                                };
                                match g
                                    .create_pull_request_review(&full_name, number, body, "COMMENT")
                                {
                                    Ok(_) => {
                                        let _ = tx.send(NetworkEvent::PrActionResult(
                                            "Comment posted.".to_string(),
                                        ));
                                    }
                                    Err(e) => {
                                        let _ = tx.send(NetworkEvent::PrActionResult(format!(
                                            "Comment failed: {e}"
                                        )));
                                    }
                                }
                            });
                        }
                        _ => {
                            self.status = format!("Unknown action: {action}");
                        }
                    }
                    self.pr_pending_body.clear();
                }
                KeyCode::Backspace => {
                    self.pr_pending_body.pop();
                }
                KeyCode::Char(ch) => {
                    self.pr_pending_body.push(ch);
                }
                _ => {}
            }
            return;
        }

        // PR number input mode
        if self.focus == Focus::TreeSearch
            && self.pr_detail.is_none()
            && !self.command_palette_visible
        {
            match code {
                KeyCode::Esc => {
                    self.tree_search_input.clear();
                    self.focus = Focus::Tree;
                    self.status = "PR number input cancelled.".to_string();
                }
                KeyCode::Enter => {
                    let input = self.tree_search_input.trim().to_string();
                    if let Ok(number) = input.parse::<u64>() {
                        if let Some(repo) = self.current_repo.clone() {
                            self.status = format!("Loading PR #{number}...");
                            self.focus = Focus::Tree;
                            let g = github.clone();
                            let full_name = repo.full_name.clone();
                            std::thread::spawn(move || {
                                let result = g.fetch_pull_request_detail(&full_name, number);
                                let _ = tx.send(NetworkEvent::PrDetailResult(
                                    result.map_err(|e| e.to_string()),
                                ));
                            });
                        }
                    } else {
                        self.status = format!("Invalid PR number: {input}");
                    }
                }
                KeyCode::Backspace => {
                    self.tree_search_input.pop();
                }
                KeyCode::Char(ch) => {
                    self.tree_search_input.push(ch);
                }
                _ => {}
            }
            return;
        }

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
