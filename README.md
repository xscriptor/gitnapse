<h1 align="center">GitNapse</h1>

<div align="center">
<img src="https://raw.githubusercontent.com/xscriptor/xassets/main/github/gitnapse/gitnapse.svg" alt="GitNapse Icon" />
</div>

<div align="right">
  <p><em>Explore <strong>Git</strong> from a new <strong>perspective</strong>, through a <strong>different lens</strong>, in your terminal.</em></p>
</div>

<div align="center">
<img src="https://xscriptor.github.io/badges/social/github.svg" alt="Github badge" /> <img src="https://xscriptor.github.io/badges/languages/rust.svg" alt="Rust badge" /> <img src="https://xscriptor.github.io/badges/os/linux.svg" alt="Linux badge" /> <img src="https://xscriptor.github.io/badges/os/macos.svg" alt="Mac badge" /> <img src="https://xscriptor.github.io/badges/os/windows.svg" alt="Windows badge" />
</div>

<div id="content"></div>
<h2 align="center">Contents</h2>
<ul>
  <li><a href="#previews">Previews</a></li>
  <li><a href="#overview">Overview</a></li>
  <li><a href="#status">Current Status</a></li>
  <li><a href="#quick-start">Quick Start</a></li>
  <li><a href="#remote-install">Remote Install / Uninstall</a></li>
  <li><a href="#release-automation">Release Automation</a></li>
  <li><a href="#docs">Documentation</a></li>
  <li><a href="#about-x">X</a></li>
</ul>

<h2 id="previews" align="center">Previews</h2>

<div align="center">
  <img src="https://raw.githubusercontent.com/xscriptor/xassets/main/github/gitnapse/previews/preview00.png" alt="GitNapse Preview 00" />
</div>

<details>
<summary><b>More Previews...</b></summary>
<br />

<div align="center">
  <img src="https://raw.githubusercontent.com/xscriptor/xassets/main/github/gitnapse/previews/preview01.png" alt="GitNapse Preview 01" />
</div>

<div align="center">
  <img src="https://raw.githubusercontent.com/xscriptor/xassets/main/github/gitnapse/previews/preview02.png" alt="GitNapse Preview 02" />
</div>

<div align="center">
  <img src="https://raw.githubusercontent.com/xscriptor/xassets/main/github/gitnapse/previews/preview03.png" alt="GitNapse Preview 03" />
</div>

<div align="center">
  <img src="https://raw.githubusercontent.com/xscriptor/xassets/main/github/gitnapse/previews/preview04.png" alt="GitNapse Preview 04" />
</div>

</details>

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
gitnapse auth oauth login --client-id YOUR_OAUTH_CLIENT_ID --scope read:user --scope repo
</code></pre>

<h2 id="remote-install" align="center">Remote Install / Uninstall</h2>
<p><strong>Linux / macOS (curl):</strong></p>
<pre><code class="language-bash">curl -fsSL https://raw.githubusercontent.com/xscriptor/gitnapse/main/scripts/install.sh | bash -s -- --action install
curl -fsSL https://raw.githubusercontent.com/xscriptor/gitnapse/main/scripts/install.sh | bash -s -- --action uninstall --cleanup
</code></pre>

<p><strong>Linux / macOS (wget):</strong></p>
<pre><code class="language-bash">wget -qO- https://raw.githubusercontent.com/xscriptor/gitnapse/main/scripts/install.sh | bash -s -- --action install
wget -qO- https://raw.githubusercontent.com/xscriptor/gitnapse/main/scripts/install.sh | bash -s -- --action uninstall --cleanup
</code></pre>

<p><strong>Windows 11 PowerShell:</strong></p>
<pre><code class="language-powershell">irm https://raw.githubusercontent.com/xscriptor/gitnapse/main/scripts/install.ps1 | iex
&amp; ([scriptblock]::Create((irm https://raw.githubusercontent.com/xscriptor/gitnapse/main/scripts/install.ps1))) -Action uninstall -Cleanup
</code></pre>

<h2 id="release-automation" align="center">Release Automation</h2>
<p>
  GitHub Actions release pipeline is available in <code>.github/workflows/release.yml</code>.
  Push a version tag like <code>v1.0.0</code> to build Windows, Linux (Ubuntu/Arch/Fedora), and macOS assets and publish them in GitHub Releases.
</p>

<h2 id="docs" align="center">Documentation</h2>
<ul>
  <li><a href="./docs/INSTALLATION.md"><code>INSTALLATION.md</code></a> - full install and uninstall by platform</li>
  <li><a href="./docs/REMOTE_INSTALLATION.md"><code>REMOTE_INSTALLATION.md</code></a> - remote scripts, parameters, and examples</li>
  <li><a href="./docs/OAUTH_AUTHENTICATION.md"><code>OAUTH_AUTHENTICATION.md</code></a> - OAuth login flows with octocrab and secure setup</li>
  <li><a href="./docs/COLLABORATIVE_SECTION.md"><code>COLLABORATIVE_SECTION.md</code></a> - branch protection, PR workflow, and release publishing collaboration guide</li>
  <li><a href="./docs/RELEASE_WORKFLOW.md"><code>RELEASE_WORKFLOW.md</code></a> - release build/publish workflow and versioning commands</li>
  <li><a href="./docs/USAGE.md"><code>USAGE.md</code></a> - full command and in-app usage guide</li>
  <li><a href="./docs/ARCHITECTURE.md"><code>ARCHITECTURE.md</code></a> - technical architecture details</li>
  <li><a href="./CHANGELOG.md"><code>CHANGELOG.md</code></a> - consolidated implementation history by release</li>
  <li><a href="./docs/IMPLEMENTATION_LOG.md"><code>IMPLEMENTATION_LOG.md</code></a> - implementation materialization log</li>
  <li><a href="./docs/tests/README.md"><code>docs/tests/README.md</code></a> - test and security audit documentation index</li>
  <li><a href="./SECURITY.md"><code>SECURITY.md</code></a> - vulnerability reporting and response policy</li>
  <li><a href="./CODE_OF_CONDUCT.md"><code>CODE_OF_CONDUCT.md</code></a> - expected behavior and community standards</li>
  <li><a href="./CONTRIBUTING.md"><code>CONTRIBUTING.md</code></a> - contribution workflow and pull request guidelines</li>
</ul>


<div id="about-x" align="center">
<h2>X</h2>

<div>
<img src="./assets/gitnapse-icon.png" width="50">
</div>
<a href="https://dev.xscriptor.com">
  <img src="https://xscriptor.github.io/icons/icons/code/product-design/xsvg/verified-filled.svg" width="24" alt="X Web" />
</a>
 & 
<a href="https://github.com/xscriptor">
  <img src="https://xscriptor.github.io/icons/icons/code/product-design/xsvg/github.svg" width="24" alt="X Github Profile" />
</a>
 & 
<a href="https://www.xscriptor.com">
  <img src="https://xscriptor.github.io/icons/icons/code/product-design/xsvg/quotes.svg" width="24" alt="Xscriptor web" />
</a>

</div>
