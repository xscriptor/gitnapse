<h1 align="center">GitNapse</h1>

<div id="content"></div>
<h2 align="center">Contents</h2>
<ul>
  <li><a href="#overview">Overview</a></li>
  <li><a href="#status">Current Status</a></li>
  <li><a href="#quick-start">Quick Start</a></li>
  <li><a href="#docs">Documentation</a></li>
</ul>

<h2 id="overview" align="center">Overview</h2>
<p>
  GitNapse is a Rust-first terminal application for exploring GitHub repositories from the command line.
  It provides repository discovery, branch-aware tree navigation, file previews, syntax-aware highlighting,
  clone workflows, and single-file download capabilities.
</p>

<h2 id="status" align="center">Current Status</h2>
<ul>
  <li>Rust TUI stack based on <code>ratatui</code> + <code>crossterm</code>.</li>
  <li>GitHub API integration for search, branches, tree, file content, and auth-user validation.</li>
  <li>Token authentication through <code>GITHUB_TOKEN</code> or secure local storage.</li>
  <li>Repository tree exploration with lazy loading and branch switching.</li>
  <li>Preview pane with focus support, keyboard/mouse scroll, and syntax-aware display.</li>
  <li>In-app file download modal and CLI file download command.</li>
</ul>

<h2 id="quick-start" align="center">Quick Start</h2>
<pre><code class="language-bash">gitnapse
gitnapse run --query "xscriptor" --page 1 --per-page 30 --cache-ttl-secs 900
gitnapse auth set
</code></pre>

<h2 id="docs" align="center">Documentation</h2>
<ul>
  <li><code>docs/INSTALLATION.md</code> - full install and uninstall by platform</li>
  <li><code>docs/USAGE.md</code> - full command and in-app usage guide</li>
  <li><code>docs/ARCHITECTURE.md</code> - technical architecture details</li>
  <li><code>docs/IMPLEMENTATION_LOG.md</code> - implementation materialization log</li>
</ul>
