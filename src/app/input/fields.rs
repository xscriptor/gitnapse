use crate::app::{App, Focus};
use anyhow::Context;
use crossterm::event::KeyCode;
use secrecy::{ExposeSecret, SecretString, zeroize::Zeroize};
use std::path::PathBuf;

impl App {
    pub(super) fn handle_search_input(&mut self, code: KeyCode) {
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

    pub(super) fn handle_tree_search_input(&mut self, code: KeyCode) {
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

    pub(super) fn handle_download_path_input(&mut self, code: KeyCode) {
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

    pub(super) fn handle_clone_path_input(&mut self, code: KeyCode) {
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

    pub(super) fn handle_token_input(&mut self, code: KeyCode) {
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

    pub(super) fn handle_oauth_client_id_input(&mut self, code: KeyCode) {
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

    pub(super) fn handle_branch_picker_input(&mut self, code: KeyCode) {
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
}
