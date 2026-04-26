# Changelog

All notable changes to this project are documented in this file.

## [0.1.0] - 2026-04-26

### Added
- Rust-first terminal application structure with modular architecture for app state, rendering, auth, API, cache, syntax, and OAuth session management.
- TUI experience powered by `ratatui` + `crossterm`, including responsive panes and contextual navigation hints.
- Repository discovery with pagination, default query behavior, and adaptive list viewport rendering.
- Repository exploration with branch switching, lazy tree loading, tree mode, and file search within tree.
- Preview pane with syntax-aware rendering, keyboard scroll, and mouse wheel scroll support.
- Mouse interactions for pane focus, row selection, and double-click open actions.
- In-app clone workflow with editable destination path.
- In-app single-file download modal and CLI file download command.
- Authentication command set for token set/status/clear and OAuth login/status flows.
- OAuth Device Flow support through `octocrab` with client id resolution precedence (CLI/env/default).
- OAuth session persistence with expiry metadata and optional refresh support when client secret variables are available.
- Secure secret storage abstraction with OS-aware keyring backend and file fallback for environments without keyring support (for example WSL).
- Remote installation/uninstallation scripts:
  - `scripts/install.sh` for Linux/macOS.
  - `scripts/install.ps1` for Windows.
- Release automation workflow to build artifacts for Ubuntu/Fedora/Arch/Windows/macOS and publish GitHub Releases.
- Security and quality workflow for fmt, clippy, tests, and dependency audit.
- Integration and security-focused tests in `tests/` for auth precedence, search behavior (`@me`), and secure storage fallback.

### Changed
- OAuth behavior in TUI was simplified to avoid unreliable in-interface flow and to guide users toward the stable CLI login path.
- Search semantics were extended to support authenticated own-repository queries (`@me`, `me:`) with optional filtering.
- Release workflow was hardened with:
  - Node 24 migration readiness.
  - modern action versions.
  - explicit release repository targeting for `gh`.
  - robust publish token fallback from GitHub App token to workflow `GITHUB_TOKEN` when App integration permissions are insufficient.

### Fixed
- Fixed rustls runtime panic by ensuring crypto provider installation before OAuth flow.
- Fixed missing `.env` loading behavior at startup for auth-related configuration.
- Fixed repository list scroll rendering mismatch (selection moved but viewport content did not).
- Fixed tree and preview focus/scroll edge cases and escape/back navigation behavior.
- Fixed multiple mouse interaction inconsistencies for selection/opening/scrolling.
- Fixed YAML packaging block for Arch metadata generation in release workflow.
- Fixed lockfile consistency issues affecting `--locked` builds in CI.

### Security
- Improved secret handling by preferring keyring storage and limiting plaintext fallback to controlled file mode where needed.
- Added security policy and dependency audit process documentation.
- Added automated vulnerability audit execution in CI.

### Documentation
- Added and expanded documentation set:
  - `docs/USAGE.md`
  - `docs/ARCHITECTURE.md`
  - `docs/OAUTH_AUTHENTICATION.md`
  - `docs/INSTALLATION.md`
  - `docs/REMOTE_INSTALLATION.md`
  - `docs/RELEASE_WORKFLOW.md`
  - `docs/COLLABORATIVE_SECTION.md`
  - `docs/tests/README.md`
  - `docs/tests/SECURITY_AUDIT.md`
  - `docs/tests/TEST_COVERAGE.md`
- Added community and governance files:
  - `SECURITY.md`
  - `CODE_OF_CONDUCT.md`
  - `CONTRIBUTING.md`
