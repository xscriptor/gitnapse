<h1 align="center">GitNapse OAuth Authentication</h1>

<div id="content"></div>
<h2 align="center">Contents</h2>
<ul>
  <li><a href="#overview">Overview</a></li>
  <li><a href="#current-auth-modes">Current Login Modes in GitNapse</a></li>
  <li><a href="#oauth-device-flow">OAuth Device Flow with octocrab</a></li>
  <li><a href="#github-setup">GitHub Configuration</a></li>
  <li><a href="#commands">Commands</a></li>
  <li><a href="#security">Security Notes</a></li>
</ul>

<h2 id="overview" align="center">Overview</h2>
<p>
  GitNapse supports multiple authentication paths and now includes OAuth device login implemented
  with <code>octocrab</code>. The implementation is optimized for terminal UX and avoids embedding
  user credentials in CLI history.
</p>

<h2 id="current-auth-modes" align="center">Current Login Modes in GitNapse</h2>
<table>
  <thead>
    <tr>
      <th>Mode</th>
      <th>How it Works</th>
      <th>Best For</th>
      <th>Storage</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>Environment token</td>
      <td>Reads <code>GITHUB_TOKEN</code> at runtime</td>
      <td>CI/CD and ephemeral sessions</td>
      <td>Not persisted by app</td>
    </tr>
    <tr>
      <td>Manual token</td>
      <td><code>gitnapse auth set</code> stores a token</td>
      <td>Local personal workflow</td>
      <td>User config directory, secure file permissions on UNIX</td>
    </tr>
    <tr>
      <td>OAuth device flow</td>
      <td><code>gitnapse auth oauth login ...</code> uses browser authorization and exchanges token via octocrab</td>
      <td>Safer interactive sign-in without pasting long tokens</td>
      <td>Stored in OS keyring when available; secure file fallback otherwise</td>
    </tr>
  </tbody>
</table>

<h2 id="oauth-device-flow" align="center">OAuth Device Flow with octocrab</h2>
<ol>
  <li>GitNapse requests a device code from GitHub using octocrab against <code>https://github.com</code>.</li>
  <li>GitNapse tries to open <code>verification_uri</code> in your default browser automatically.</li>
  <li>If auto-open is unavailable, the terminal shows a clickable OSC8 hyperlink (when terminal supports it) plus the plain URL.</li>
  <li>User authorizes in browser with GitHub account.</li>
  <li>GitNapse polls token endpoint using octocrab's recommended flow logic, respecting <code>interval</code> and <code>slow_down</code> responses.</li>
  <li>The resulting OAuth access token is stored securely and then validated against <code>/user</code>.</li>
</ol>

<h2 id="github-setup" align="center">GitHub Configuration</h2>
<p>
  To use OAuth login, you need an OAuth App in GitHub settings.
</p>
<ul>
  <li>Create an OAuth App in GitHub developer settings.</li>
  <li>Copy the <strong>Client ID</strong> from the app.</li>
  <li>For terminal device flow, no local redirect listener is required.</li>
  <li>Use minimum scopes first (recommended: <code>read:user</code>), then add <code>repo</code> only if private repository access is needed.</li>
</ul>
<p>
  GitNapse accepts either <code>GITNAPSE_GITHUB_OAUTH_CLIENT_ID</code> or compatibility fallback
  <code>GITHUB_CLIENT_ID</code> as Client ID source, and includes a built-in default Client ID for the official GitNapse OAuth app.
</p>
<p>
  If you currently have a GitHub App but not an OAuth App, create the OAuth App as a separate credential set for user login.
</p>

<h2 id="commands" align="center">Commands</h2>
<p><strong>One-time login with explicit Client ID:</strong></p>
<pre><code class="language-bash">gitnapse auth oauth login --client-id YOUR_OAUTH_CLIENT_ID --scope read:user --scope repo
</code></pre>

<p><strong>Use environment for Client ID:</strong></p>
<pre><code class="language-bash">export GITNAPSE_GITHUB_OAUTH_CLIENT_ID=YOUR_OAUTH_CLIENT_ID
gitnapse auth oauth login --scope read:user --scope repo
</code></pre>

<p><strong>Compatibility environment variable:</strong></p>
<pre><code class="language-bash">export GITHUB_CLIENT_ID=YOUR_OAUTH_CLIENT_ID
gitnapse auth oauth login --scope read:user
</code></pre>

<p><strong>TUI shortcut:</strong></p>
<pre><code class="language-bash"># Inside the app, press:
o
</code></pre>
<p>
  The app opens an OAuth Client ID modal and then starts the device login flow.
</p>

<p><strong>Short timeout tuning:</strong></p>
<pre><code class="language-bash">gitnapse auth oauth login --client-id YOUR_OAUTH_CLIENT_ID --timeout-secs 1200
</code></pre>

<h2 id="security" align="center">Security Notes</h2>
<ul>
  <li>Client secret is intentionally not required for this terminal device flow implementation.</li>
  <li>OAuth access token is never printed back to terminal output.</li>
  <li>Primary storage is OS keyring (Credential Manager on Windows, Keychain on macOS, Secret Service/libsecret on Linux when available).</li>
  <li>WSL and no-keyring environments automatically fallback to local file storage with strict UNIX permissions (<code>0600</code>).</li>
  <li>OAuth session metadata is persisted separately (expiry, refresh token, scopes, client id) to support safer session lifecycle handling.</li>
  <li>If <code>GITNAPSE_GITHUB_OAUTH_CLIENT_SECRET</code> or <code>GITHUB_CLIENT_SECRET</code> is present, GitNapse attempts refresh-token exchange when access token is near expiry.</li>
  <li>Prefer least-privilege scopes and rotate/revoke tokens when no longer needed.</li>
  <li>For shared machines, prefer environment-based or ephemeral auth over persistent local token storage.</li>
  <li>On logout (<code>gitnapse auth clear</code>), GitNapse removes credentials from keyring and also deletes fallback file if present.</li>
</ul>

<h3 id="storage-recommendation" align="center">Session Storage Recommendation</h3>
<p>
  Current implementation uses OS keyring when available and falls back to secure local file storage in unsupported environments
  (for example WSL/headless Linux sessions without keyring service).
</p>

<h3 id="oauth-troubleshooting" align="center">OAuth Troubleshooting</h3>
<ul>
  <li>If you previously saw a rustls <code>CryptoProvider</code> panic, update to this build; GitNapse now installs a rustls provider explicitly before OAuth login.</li>
  <li><code>.env</code> is loaded at startup, so <code>GITHUB_CLIENT_ID</code> and related auth vars are available without manual export.</li>
  <li>If browser auto-open does not work in your terminal/session, copy the displayed URL and open it manually.</li>
</ul>
