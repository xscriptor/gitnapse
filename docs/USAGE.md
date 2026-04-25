<h1 align="center">GitNapse Usage Guide</h1>

<div id="content"></div>
<h2 align="center">Contents</h2>
<ul>
  <li><a href="#requirements">Requirements</a></li>
  <li><a href="#cli-table">CLI Command Table</a></li>
  <li><a href="#in-app-controls">In-App Control Table</a></li>
  <li><a href="#my-private-repos">My Private Repositories</a></li>
  <li><a href="#workflows">Core Workflows</a></li>
  <li><a href="#troubleshooting">Troubleshooting</a></li>
</ul>

<h2 id="requirements" align="center">Requirements</h2>
<ul>
  <li>Rust toolchain: <code>cargo</code>, <code>rustc</code></li>
  <li>Internet connection for GitHub API requests</li>
  <li>Local <code>git</code> available in <code>PATH</code></li>
</ul>

<h2 id="cli-table" align="center">CLI Command Table</h2>
<table>
  <thead>
    <tr>
      <th>Command</th>
      <th>Purpose</th>
      <th>Example</th>
      <th>Notes</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>gitnapse</code></td>
      <td>Run TUI with default options</td>
      <td><code>gitnapse</code></td>
      <td>Defaults to query <code>xscriptor</code></td>
    </tr>
    <tr>
      <td><code>gitnapse run ...</code></td>
      <td>Run TUI with explicit parameters</td>
      <td><code>gitnapse run --query "xscriptor" --page 1 --per-page 30 --cache-ttl-secs 900</code></td>
      <td>Controls search bootstrap and preview cache TTL</td>
    </tr>
    <tr>
      <td><code>gitnapse run --query "@me"</code></td>
      <td>List authenticated repositories (including private)</td>
      <td><code>gitnapse run --query "@me"</code></td>
      <td>Requires valid login/token; supports optional filter: <code>@me keyword</code></td>
    </tr>
    <tr>
      <td><code>gitnapse auth set</code></td>
      <td>Store GitHub token interactively</td>
      <td><code>gitnapse auth set</code></td>
      <td>Hidden prompt for secure input</td>
    </tr>
    <tr>
      <td><code>gitnapse auth set --token ...</code></td>
      <td>Store token from argument</td>
      <td><code>gitnapse auth set --token YOUR_GITHUB_TOKEN</code></td>
      <td>Useful for scripted environments</td>
    </tr>
    <tr>
      <td><code>gitnapse auth status</code></td>
      <td>Display token source availability</td>
      <td><code>gitnapse auth status</code></td>
      <td>Shows env-token and stored-token state</td>
    </tr>
    <tr>
      <td><code>gitnapse auth clear</code></td>
      <td>Remove stored token</td>
      <td><code>gitnapse auth clear</code></td>
      <td>Does not modify <code>GITHUB_TOKEN</code> env variable</td>
    </tr>
    <tr>
      <td><code>gitnapse auth oauth login ...</code></td>
      <td>OAuth login (device flow via octocrab)</td>
      <td><code>gitnapse auth oauth login --client-id YOUR_OAUTH_CLIENT_ID --scope read:user --scope repo</code></td>
      <td>Starts browser-based device authorization and stores access token securely</td>
    </tr>
    <tr>
      <td><code>gitnapse download-file ...</code></td>
      <td>Download one file (curl/wget-like)</td>
      <td><code>gitnapse download-file --repo owner/repo --path src/main.rs --out ./main.rs</code></td>
      <td>Supports <code>--ref</code> for branch/tag/sha</td>
    </tr>
  </tbody>
</table>

<h2 id="in-app-controls" align="center">In-App Control Table</h2>
<table>
  <thead>
    <tr>
      <th>Key / Input</th>
      <th>Area</th>
      <th>Action</th>
      <th>Details</th>
    </tr>
  </thead>
  <tbody>
    <tr><td><code>/</code></td><td>Global</td><td>Open search input</td><td>Edit repository search query</td></tr>
    <tr><td><code>Enter</code></td><td>Contextual</td><td>Execute/open/preview</td><td>Search, open repo, or preview selected file</td></tr>
    <tr><td><code>Tab</code></td><td>Global</td><td>Cycle focus</td><td><code>Repos -&gt; Tree -&gt; Preview</code></td></tr>
    <tr><td><code>Esc</code></td><td>Global</td><td>Back navigation</td><td>Close modal or return from repo view to result list</td></tr>
    <tr><td><code>↑ / ↓</code></td><td>Tree / Preview</td><td>Navigate / Scroll</td><td>Moves selection in tree or scrolls preview when focused</td></tr>
    <tr><td><code>PgUp / PgDn</code></td><td>Preview</td><td>Fast scroll</td><td>Page-sized preview movement</td></tr>
    <tr><td><code>Home / End</code></td><td>Preview</td><td>Jump bounds</td><td>Go to top / bottom of preview</td></tr>
    <tr><td><code>← / [</code></td><td>Repos list</td><td>Previous page</td><td>Move to previous GitHub search page</td></tr>
    <tr><td><code>→ / ]</code></td><td>Repos list</td><td>Next page</td><td>Move to next GitHub search page</td></tr>
    <tr><td><code>b</code></td><td>Repo view</td><td>Branch picker</td><td>Open branch selector modal</td></tr>
    <tr><td><code>f</code></td><td>Repo view</td><td>File search</td><td>Find file by name/path substring in loaded tree</td></tr>
    <tr><td><code>v</code></td><td>Repo view</td><td>Toggle tree text view</td><td>Show whole repository tree in preview pane</td></tr>
    <tr><td><code>c</code></td><td>Repo view</td><td>Clone modal</td><td>Prompt destination path and run clone</td></tr>
    <tr><td><code>d</code></td><td>Preview</td><td>Download modal</td><td>Save current previewed file to local path</td></tr>
    <tr><td><code>Del</code></td><td>Path modals</td><td>Clear path input</td><td>Works in clone/download path inputs</td></tr>
    <tr><td><code>t</code></td><td>Global</td><td>Token modal</td><td>Save token from inside the TUI</td></tr>
    <tr><td><code>o</code></td><td>Global</td><td>OAuth modal</td><td>Client ID is optional; press <code>Enter</code> empty to use default and start device-flow login</td></tr>
    <tr><td><code>q</code></td><td>Global</td><td>Quit</td><td>Exit application</td></tr>
    <tr><td>Mouse left click</td><td>Tree / Preview / Repos</td><td>Focus & select</td><td>Single click selects, double click opens (repo/file)</td></tr>
    <tr><td>Mouse wheel</td><td>Tree / Preview</td><td>Scroll</td><td>Scroll behavior depends on pointer position</td></tr>
  </tbody>
</table>

<h2 id="my-private-repos" align="center">My Private Repositories</h2>
<p>
  GitHub search endpoint does not guarantee full private-repository discovery by username query.
  To list your own repositories (including private ones), use the authenticated query mode:
</p>
<ul>
  <li>Inside TUI search input (<code>/</code>): <code>@me</code></li>
  <li>Optional filter: <code>@me rust</code> or <code>me:rust</code></li>
  <li>CLI start: <code>gitnapse run --query "@me"</code></li>
</ul>
<p>
  This mode requires a valid authenticated session/token and uses your account repository listing API scope.
</p>

<h2 id="workflows" align="center">Core Workflows</h2>

<h3 id="workflow-repo" align="center">Open and Explore a Repository</h3>
<ol>
  <li>Search repositories with <code>/</code> then <code>Enter</code>.</li>
  <li>Select one result and open it with <code>Enter</code> or double click.</li>
  <li>Browse files in tree pane; press <code>Enter</code> or double click file for preview.</li>
  <li>Use <code>Tab</code> to focus preview and scroll long content.</li>
</ol>

<h3 id="workflow-branch" align="center">Switch Branch</h3>
<ol>
  <li>Press <code>b</code> in repo view.</li>
  <li>Select branch with arrows.</li>
  <li>Press <code>Enter</code> to reload tree/preview context on that branch.</li>
</ol>

<h3 id="workflow-clone" align="center">Clone Repository</h3>
<ol>
  <li>Open repository view.</li>
  <li>Press <code>c</code> and set destination path.</li>
  <li>Press <code>Enter</code> to clone.</li>
</ol>

<h3 id="workflow-download" align="center">Download Current Previewed File In-App</h3>
<ol>
  <li>Open preview for a file.</li>
  <li>Press <code>d</code>.</li>
  <li>Provide output path.</li>
  <li>Press <code>Enter</code> to save.</li>
</ol>

<h2 id="troubleshooting" align="center">Troubleshooting</h2>
<ul>
  <li>If API limits are hit, set a token with <code>gitnapse auth set</code> or export <code>GITHUB_TOKEN</code>.</li>
  <li>For OAuth device flow, you can provide <code>--client-id</code>; if omitted, GitNapse uses env variables and then built-in default OAuth Client ID.</li>
  <li>You can also use <code>GITHUB_CLIENT_ID</code> as compatibility fallback for OAuth client ID.</li>
  <li>If OAuth URL is not clickable in your terminal, GitNapse still tries to auto-open browser; otherwise copy/open the displayed URL manually.</li>
  <li>To inspect your private repositories from TUI search, use <code>@me</code> (or <code>@me keyword</code> to filter).</li>
  <li>If token is saved but requests fail, run <code>gitnapse auth status</code> and validate token permissions.</li>
  <li>If clone/download fails, verify destination path permissions and filesystem access.</li>
  <li>If no repos appear, refine query terms (owner/org/repo keywords).</li>
</ul>
