<h1 align="center">GitNapse Test Documentation</h1>

<div id="content"></div>
<h2 align="center">Contents</h2>
<ul>
  <li><a href="#overview">Overview</a></li>
  <li><a href="#files">Test Files</a></li>
  <li><a href="#run">How To Run</a></li>
  <li><a href="#related">Related Documents</a></li>
</ul>

<h2 id="overview" align="center">Overview</h2>
<p>
  This section documents automated tests and security-oriented checks implemented for GitNapse.
  Tests are intentionally placed under the repository-level <code>tests/</code> directory to keep them separated from application modules.
</p>

<h2 id="files" align="center">Test Files</h2>
<ul>
  <li><code>tests/github_search_tests.rs</code> - API behavior tests for general search and <code>@me</code> private-repo mode using mocked HTTP endpoints</li>
  <li><code>tests/secure_store_tests.rs</code> - secret storage fallback and file-permission checks</li>
  <li><code>tests/auth_precedence_tests.rs</code> - authentication source precedence checks</li>
</ul>

<h2 id="run" align="center">How To Run</h2>
<pre><code class="language-bash">cargo test
</code></pre>

<h2 id="related" align="center">Related Documents</h2>
<ul>
  <li><a href="./SECURITY_AUDIT.md"><code>SECURITY_AUDIT.md</code></a></li>
  <li><a href="./TEST_COVERAGE.md"><code>TEST_COVERAGE.md</code></a></li>
</ul>
