<h1 align="center">GitNapse Architecture</h1>

<div id="content"></div>
<h2 align="center">Contents</h2>
<ul>
  <li><a href="#high-level">High-Level Design</a></li>
  <li><a href="#execution-flow">Execution Flow</a></li>
  <li><a href="#auth-strategy">Authentication Strategy</a></li>
  <li><a href="#account-strategy">Account Configuration Strategy</a></li>
  <li><a href="#terminal-ux">Terminal UX Strategy</a></li>
  <li><a href="#network-integration">Network and Integration Notes</a></li>
</ul>

<h2 id="high-level" align="center">High-Level Design</h2>
<ul>
  <li><code>src/main.rs</code>: CLI entrypoint, auth commands, and file-download command.</li>
  <li><code>src/app/mod.rs</code>: TUI state machine, event loop, key/mouse dispatch.</li>
  <li><code>src/app/render.rs</code>: layout and widget rendering, pane detection, modal rendering.</li>
  <li><code>src/app/theme.rs</code>: color palette strategy and responsive navigation hints.</li>
  <li><code>src/cache.rs</code>: local preview cache with TTL and disk persistence.</li>
  <li><code>src/github.rs</code>: GitHub API client for search/branches/tree/content/auth-user.</li>
  <li><code>src/auth.rs</code>: token loading, secure storage, token CLI subcommands.</li>
  <li><code>src/oauth.rs</code>: OAuth device-flow login powered by octocrab.</li>
  <li><code>src/oauth_session.rs</code>: secure OAuth session persistence, expiry metadata, and refresh attempt path.</li>
  <li><code>src/config.rs</code>: persisted account preferences (<code>account.json</code>).</li>
  <li><code>src/models.rs</code>: DTO/domain models for GitHub responses and internal tree nodes.</li>
  <li><code>src/syntax.rs</code>: preview syntax-aware formatting.</li>
</ul>

<h2 id="execution-flow" align="center">Execution Flow</h2>
<ol>
  <li>App resolves token from environment or local secure file.</li>
  <li>GitHub client initializes headers and optional bearer auth.</li>
  <li>TUI starts with default query and paginated repository search.</li>
  <li>User opens repository and selects branch when needed.</li>
  <li>Tree is loaded and lazily revealed for large repositories.</li>
  <li>File preview is loaded from cache or API and can be scrolled/focused.</li>
  <li>User can clone repo, download current previewed file, or switch back to search list.</li>
</ol>

<h2 id="auth-strategy" align="center">Authentication Strategy</h2>
<ul>
  <li>Preferred source: <code>GITHUB_TOKEN</code>.</li>
  <li>Fallback source: local stored token under user config directory.</li>
  <li>OAuth device flow is available via <code>gitnapse auth oauth login</code> using octocrab.</li>
  <li>OAuth session metadata is persisted to support token lifecycle handling and optional refresh.</li>
  <li>UNIX permissions are restricted for token file (<code>0600</code>).</li>
  <li>Token can be updated inside TUI via modal and validated against <code>/user</code>.</li>
</ul>

<h2 id="account-strategy" align="center">Account Configuration Strategy</h2>
<ul>
  <li>Persisted file: <code>account.json</code> in project config directory.</li>
  <li>Stored keys include:
    <ul>
      <li><code>preferred_clone_dir</code></li>
      <li><code>last_branch_by_repo</code></li>
    </ul>
  </li>
  <li>Design remains extensible for future account-wide API preferences.</li>
</ul>

<h2 id="terminal-ux" align="center">Terminal UX Strategy</h2>
<ul>
  <li>Keyboard-first navigation with mouse augmentation.</li>
  <li>Responsive split view: horizontal on wide terminals, vertical on narrow terminals.</li>
  <li>Preview pane is independently focusable and scrollable.</li>
  <li>Mouse behaviors:
    <ul>
      <li>single click selects</li>
      <li>double click opens repo/file</li>
      <li>wheel scrolls tree/preview by pointer location</li>
    </ul>
  </li>
  <li>Navigation bar wraps across lines based on terminal width.</li>
  <li>Selection colors use full <code>references.md</code> palette with contrast-safe foreground.</li>
</ul>

<h2 id="network-integration" align="center">Network and Integration Notes</h2>
<ul>
  <li>HTTP layer: <code>reqwest</code> blocking client for deterministic TUI loop behavior.</li>
  <li>OAuth device flow exchange uses <code>octocrab</code> against <code>https://github.com/login/*</code> routes.</li>
  <li>GitHub endpoints:
    <ul>
      <li><code>/search/repositories</code></li>
      <li><code>/repos/{full_name}/branches?per_page=100</code></li>
      <li><code>/repos/{full_name}/git/trees/{branch}?recursive=1</code></li>
      <li><code>/repos/{full_name}/contents/{path}</code></li>
      <li><code>/user</code></li>
    </ul>
  </li>
  <li>Clone integration uses local <code>git</code> executable.</li>
  <li>Single-file download writes content directly to user-selected local path.</li>
</ul>
