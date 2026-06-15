# Changelog

## v0.1.1

### Fixed

- **Binary file corruption**: `fetch_file_content` returns `Vec<u8>` instead of using `String::from_utf8_lossy`. Binary files are no longer silently corrupted during download. Preview shows "Binary file" for non-UTF-8 content. (`src/github.rs:308-312`, `src/app/mod.rs:328-342`)

- **Silent keyring deletion error**: `clear_secret` now prints a warning if keyring deletion fails, instead of silently discarding the error. (`src/secure_store.rs:145-150`)

- **Misleading variable name in OAuth**: Renamed `client_secret` to `device_credential` in OAuth device flow — it held the public client_id, not a secret. (`src/oauth.rs:93`)

- **Terminal state recovery on OAuth**: Added `TerminalGuard` with `Drop` to ensure raw mode and alternate screen are restored if OAuth panics after leaving TUI mode. (`src/app/mod.rs:30-37, 688-690`)

- **Fragile test path inclusions**: Replaced `#[path = "../src/..."]` in integration tests with a proper `src/lib.rs` that exposes the public API. Tests now use `gitnapse::*` imports. (`src/lib.rs`, `tests/*.rs`)

- **Cache hash overhead**: Replaced SHA-256 with `DefaultHasher` for cache keys. Faster and sufficient for local filename generation. (`src/cache.rs:63-70`)

- **Antipattern in auth status**: Replaced `.ok().filter().is_some()` with `.is_ok_and()` for conciseness. (`src/auth.rs:84-95`)
