# Changelog

## v0.1.1

### Fixed

- **Binary file corruption**: `fetch_file_content` returns `Vec<u8>` instead of using `String::from_utf8_lossy`. Preview shows "Binary file" for non-UTF-8 content. (`src/github.rs:308-312`, `src/app/mod.rs:328-342`)

- **Silent keyring deletion error**: `clear_secret` now prints a warning if keyring deletion fails. (`src/secure_store.rs:145-150`)

- **Misleading variable name in OAuth**: Renamed `client_secret` to `device_credential` — it held the public client_id, not a secret. (`src/oauth.rs:93`)

- **Terminal state recovery on OAuth**: Added `TerminalGuard` with `Drop` to restore raw mode and alternate screen if OAuth panics. (`src/app/mod.rs:30-37, 688-690`)

- **Fragile test path inclusions**: Replaced `#[path = "../src/..."]` with a `src/lib.rs`. Tests now use `gitnapse::*` imports. (`src/lib.rs`, `tests/*.rs`)

- **Cache hash overhead**: Replaced SHA-256 with `DefaultHasher` for cache keys. (`src/cache.rs:63-70`)

- **Redundant String allocation in @me filter**: Replaced `format!`-based `haystack` with direct `Iterator::any` checks per field. (`src/github.rs:122-131`)

- **Frame-by-frame String allocation in tree indent**: Precomputed `INDENTS` array replaces `"  ".repeat(n)` on every frame. (`src/app/render.rs:279`)

- **Unnecessary clones**: Reduced `.clone()` calls in `load_tree_for_current_branch` and `preview_selected_file`. (`src/app/mod.rs:275-360`)

- **Non-idempotent rustls provider**: `ensure_rustls_crypto_provider` now handles the `install_default` error instead of silently discarding it. (`src/oauth.rs:43-47`)

- **rsplit_once clarity**: Replaced `rsplit('/').next()` with `rsplit_once('/')` in tree parsing. (`src/github.rs:247-248`)

- **is_some_and idioms**: Replaced `.ok().filter().is_some()` with `.is_ok_and()` in `auth.rs` and `secure_store.rs`. (`src/auth.rs:84-95`, `src/secure_store.rs:14-17`)

- **TUI event poll rate**: Reduced from 120ms to 16ms for smoother responsiveness (~60 FPS). (`src/app/mod.rs:1069`)

- **Preview scroll viewport**: Replaced hardcoded 30-line viewport with the actual preview pane height from the render layout. (`src/app/mod.rs:882`)

- **PageUp/PageDown step size**: Now uses half the preview viewport instead of a fixed 20 lines. (`src/app/mod.rs:897-904`)

- **Terminal panic recovery**: Installed a panic hook that restores raw mode and leaves the alternate screen, preventing a stuck terminal. (`src/app/mod.rs:1049-1054`)

- **Resize event handling**: Added explicit `Event::Resize` handler that updates the status bar. (`src/app/mod.rs:1090-1092`)

- **Rate limit tracking**: GitHubClient now extracts `x-ratelimit-remaining` and `x-ratelimit-reset` headers from every response and exposes them via public methods. (`src/github.rs:108-131`)

- **Branch pagination**: `fetch_branches` now loops over multiple pages, supporting repos with more than 100 branches. (`src/github.rs`)

- **Blob API fallback**: When the Contents API returns 403 (file >1MB), automatically falls back to the Git Blobs API via SHA lookup. (`src/github.rs`)

- **@me query edge cases**: Improved `parse_me_query` to handle multiple spaces after `@me`, bare `me:`, and reject `@me,` comma forms correctly. (`src/github.rs:46-93`)

- **OAuth runtime reuse**: Created a shared `OnceLock<Runtime>` to avoid allocating a new tokio runtime on every OAuth login. (`src/oauth.rs`)

- **Unused http dependency**: Replaced `use http::header::ACCEPT` with `use reqwest::header::ACCEPT` and removed `http = "1.3"` from Cargo.toml. (`src/oauth.rs:5`, `Cargo.toml`)

- **Unused sha2 dependency**: Removed `sha2 = "0.11"` from Cargo.toml. (`Cargo.toml`)

### Added

- **Unit tests for syntax.rs**: 9 tests covering keyword highlighting, string/number/comment detection, max_lines, empty content, and unknown extensions. (`src/syntax.rs:134-215`)

- **Unit tests for config.rs**: 3 tests for roundtrip serialization, invalid JSON handling, and missing fields. (`src/config.rs:58-83`)

- **Unit tests for github.rs parse_me_query**: 11 tests covering exact match, case insensitivity, multiple spaces, language filters, comma rejection, special characters, and `me:` prefix forms. (`src/github.rs`)

- **Theme externalization**: Color palette can now be customized via `theme.jsonc` in the config directory. Falls back to the built-in 16-color palette if the file is absent. (`src/config.rs:75-130`, `src/app/theme.rs:27-33`, `docs/THEME_CONFIG.md`)

- **12 built-in theme presets**: X, Madrid, Lahabana, Miami, Paris, Tokio, Oslo, Helsinki, Berlin, London, Praha, Bogota. Auto-installed from `themes/` directory on first run. (`themes/*.jsonc`)

- **Keybindings config**: Keybindings can be customized via `keybindings.jsonc` in config directory. Default bindings match the existing hardcoded keys. (`src/config.rs`)

- **Command palette**: Press Ctrl+P to open a VS Code-style command palette with fuzzy search over available actions: search repos, switch branch, find file, clone, download, list issues/PRs, view commits/CI status, compare branches, toggle tree view, and more. Non-blocking with `std::thread::spawn` for network calls. (`src/app/mod.rs`, `src/app/render.rs`)

- **Channel-based async**: Network operations run on background threads via `mpsc` channel, keeping the TUI responsive during API calls. (`src/app/mod.rs`)

- **GitHub API coverage**: Added models and client methods for commits, diffs, issues, pull requests, CI check runs, starred repos, and repository lookup. (`src/models.rs`, `src/github.rs`)

- **Typed error handling**: Introduced `thiserror`-based error enums (`GitHubError`, `AuthError`, `CacheError`, `OAuthError`) across all library modules, replacing raw `anyhow` strings. (`src/error.rs`, `src/github.rs`, `src/cache.rs`, `src/auth.rs`, `src/secure_store.rs`)

- **Retry logic**: Network calls retry up to 3 times with exponential backoff on transient errors. (`src/github.rs`)

- **Async HTTP**: GitHubClient migrated from `reqwest::blocking` to `reqwest::async` with a shared `current_thread` tokio runtime. Public API remains synchronous via `block_on`. (`src/github.rs`, `src/oauth_session.rs`, `Cargo.toml`)

- **Token zeroize**: Token input buffer uses `secrecy::SecretString` and `Zeroize` on escape/save to clear sensitive data from memory. (`src/app/mod.rs`, `src/app/render.rs`)

- **OAuth client_id warning**: Warning printed to stderr if no OAuth client ID environment variable is found. (`src/app/mod.rs:151-159`)

- **10 TUI event tests**: Added tests for key navigation (q, /, Esc, Tab, Up/Down), token input (Esc zeroize, Enter save), and search input. (`src/app/mod.rs`)

- **Preview cache binary support**: Cache now stores raw `Vec<u8>` instead of `String`, supporting binary files and ETag metadata. (`src/cache.rs:12-13, 43-88`)

- **Loading indicators**: Status bar now shows "Loading..." before network operations (search, branch fetch, tree load, file preview). (`src/app/mod.rs:213, 254, 290, 355`)

- **Dynamic tree indent**: Replaced the fixed 9-element `INDENTS` constant with `"  ".repeat(depth.min(20))` for arbitrary-depth directories. (`src/app/render.rs`)

- **CI workflow**: Added `.github/workflows/ci.yml` that runs `cargo fmt --check`, `cargo clippy`, and `cargo test` on every push and PR. (`./github/workflows/ci.yml`)

- **Docstrings**: Added documentation comments (`///`) to all public functions in `config.rs`, `cache.rs`, `syntax.rs`, and `secure_store.rs`.

- **Pull request management**: View PR detail (title, body, stats, branches), approve, request changes, comment, merge, close. Browse reviews, comments, and commits per PR. Enter PR number from tree search. 8 new GitHub API methods. (`src/app/`, `src/github/`, `src/models/`)

- **Custom review comments**: Approve, request changes, and comment actions prompt for custom text before submitting. Esc cancels, Enter submits. (`src/app/commands.rs`)

- **PR creation**: 4-step guided wizard (title, head branch, base branch, optional body) via `Create Pull Request` in command palette. (`src/app/commands.rs`)

- **Three merge methods**: Merge commit, squash, or rebase selectable from PR detail view. (`src/app/commands.rs`)

- **Module refactoring**: Major codebase restructure. `src/github.rs` split into `github/` (6 files), `src/app/mod.rs` split into `app/input/` (4 files), `app/network.rs`, `app/commands.rs`, `app/actions.rs`. `src/config.rs` split into `config/` (4 files). `src/models.rs` split into `models/` (4 files). (`src/github/`, `src/app/`, `src/config/`, `src/models/`)

- **DRY HTTP helpers**: `send_and_check_json()` eliminates ~200 lines of boilerplate across 12 API methods. (`src/github/mod.rs`)

- **Test consolidation**: Moved 3 integration tests into `github/mod.rs`, deleted `tests/github_search_tests.rs`. (`src/github/mod.rs`)

- **Dependency updates**: `keyring` 3.6 -> 4.0 + `keyring-core` 1.0, `octocrab` 0.49 -> 0.53, added `nucleo-matcher` for fuzzy search. (`Cargo.toml`)
