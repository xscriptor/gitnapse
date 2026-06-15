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

### Added

- **Unit tests for syntax.rs**: 9 tests covering keyword highlighting, string/number/comment detection, max_lines, empty content, and unknown extensions. (`src/syntax.rs:134-215`)

- **Unit tests for config.rs**: 3 tests for roundtrip serialization, invalid JSON handling, and missing fields. (`src/config.rs:58-83`)
