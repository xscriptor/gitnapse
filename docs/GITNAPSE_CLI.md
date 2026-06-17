<h1 align="center">GitNapse CLI Reference</h1>

<div id="content"></div>
<h2 align="center">Contents</h2>
<ul>
  <li><a href="#overview">Overview</a></li>
  <li><a href="#authentication">Authentication</a></li>
  <li><a href="#git-operations">Git Operations (local git)</a></li>
  <li><a href="#api-operations">GitHub API Operations</a></li>
  <li><a href="#error-handling">Error Handling</a></li>
  <li><a href="#examples">Quick Examples</a></li>
</ul>

<h2 id="overview" align="center">Overview</h2>
<p>
  GitNapse provides a set of CLI commands that operate both via the <strong>GitHub REST API</strong>
  (for queries and PR management) and via <strong>local git</strong> (for clone, commit, push, and
  repository inspection). Authentication is shared across all commands and can use a stored token,
  OAuth session, or the <code>GITHUB_TOKEN</code> environment variable.
</p>

<h2 id="authentication" align="center">Authentication</h2>
<p>
  Commands that hit the GitHub API (<code>clone</code> with <code>owner/repo</code> format,
  <code>pr list/create/merge</code>) resolve credentials in this order:
</p>
<ol>
  <li><code>GITHUB_TOKEN</code> environment variable</li>
  <li>OAuth session (from <code>gitnapse auth oauth login</code>)</li>
  <li>Stored token (from <code>gitnapse auth set</code>)</li>
</ol>
<p>
  Commands that use local git (<code>commit</code>, <code>push</code>, <code>status</code>,
  <code>log</code>, <code>branch</code>) rely on your existing git configuration and credentials.
</p>

<h2 id="git-operations" align="center">Git Operations (local git)</h2>
<p>
  These commands run <code>git</code> locally. They must be executed from inside a git repository
  (except <code>clone</code>).
</p>

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
      <td><code>gitnapse clone</code></td>
      <td>Clone a repository</td>
      <td><code>gitnapse clone xscriptor/gitnapse:develop</code></td>
      <td>
        Accepts <code>owner/repo[:branch]</code> (resolved via API) or a full git URL.
        Creates a subdirectory with the repo name in the current directory.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse commit</code></td>
      <td>Stage all changes and commit</td>
      <td><code>gitnapse commit -m "fix: typo"</code></td>
      <td>Runs <code>git add -A</code> then <code>git commit -m</code>. Requires a non-empty message via <code>-m</code>.</td>
    </tr>
    <tr>
      <td><code>gitnapse push</code></td>
      <td>Push commits to remote</td>
      <td><code>gitnapse push origin main</code></td>
      <td>Optional <code>[remote]</code> and <code>[branch]</code> positional arguments. Defaults to git's default push behavior.</td>
    </tr>
    <tr>
      <td><code>gitnapse status</code></td>
      <td>Show working tree status</td>
      <td><code>gitnapse status</code></td>
      <td>Runs <code>git status --short</code>. Shows <code>(clean)</code> when nothing changed.</td>
    </tr>
    <tr>
      <td><code>gitnapse log</code></td>
      <td>Show commit log</td>
      <td><code>gitnapse log -n 10</code></td>
      <td>Runs <code>git log --oneline -n</code>. Default: 20 entries. Shows <code>(no commits)</code> on empty history.</td>
    </tr>
    <tr>
      <td><code>gitnapse branch</code></td>
      <td>List local and remote branches</td>
      <td><code>gitnapse branch</code></td>
      <td>Runs <code>git branch -a</code>. The current branch is highlighted with <code>*</code>.</td>
    </tr>
  </tbody>
</table>

<h2 id="api-operations" align="center">GitHub API Operations</h2>
<p>
  These commands use the GitHub REST API and require authentication for private repositories
  or for creating/merging PRs.
</p>

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
      <td><code>gitnapse pr list</code></td>
      <td>List pull requests</td>
      <td><code>gitnapse pr list xscriptor/gitnapse -s open</code></td>
      <td>
        Shows PR number, state, additions/deletions, title, and author.
        State flag: <code>open</code> (default), <code>closed</code>, <code>all</code>.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse pr create</code></td>
      <td>Create a pull request</td>
      <td><code>gitnapse pr create xscriptor/gitnapse -t "My PR" -H feature -B main -b "description"</code></td>
      <td>
        Requires <code>--title</code>, <code>--head</code> (source), <code>--base</code> (target).
        Optional <code>--body</code> for description.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse pr merge</code></td>
      <td>Merge a pull request</td>
      <td><code>gitnapse pr merge xscriptor/gitnapse -n 42 -m squash</code></td>
      <td>
        Requires <code>--number</code>. Optional <code>--method</code>: <code>merge</code> (default),
        <code>squash</code>, or <code>rebase</code>.
      </td>
    </tr>
  </tbody>
</table>

<h2 id="error-handling" align="center">Error Handling</h2>
<p>
  All CLI commands provide user-friendly error messages instead of raw Rust traces:
</p>

<table>
  <thead>
    <tr>
      <th>Situation</th>
      <th>Message</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>CWD is not a git repository</td>
      <td><code>not a git repository — run this from inside a git repository</code></td>
    </tr>
    <tr>
      <td>Repository not found on GitHub</td>
      <td><code>repository 'owner/repo' not found on GitHub</code></td>
    </tr>
    <tr>
      <td>Authentication required</td>
      <td><code>authentication required — run 'gitnapse auth set' or 'gitnapse auth oauth login'</code></td>
    </tr>
    <tr>
      <td>Git not installed</td>
      <td><code>git is not installed or not in PATH</code> with install link</td>
    </tr>
    <tr>
      <td>Empty commit message</td>
      <td><code>commit message cannot be empty</code> with usage hint</td>
    </tr>
    <tr>
      <td>Empty repository spec</td>
      <td><code>repository specification is empty</code> with usage hint</td>
    </tr>
    <tr>
      <td>Nothing to commit</td>
      <td><code>nothing to commit (working tree clean)</code></td>
    </tr>
    <tr>
      <td>GitHub API rate limit</td>
      <td><code>GitHub API rate limit exceeded — resets at timestamp ...</code></td>
    </tr>
  </tbody>
</table>

<h2 id="examples" align="center">Quick Examples</h2>

<pre><code># Clone a repo (branch optional)
gitnapse clone xscriptor/gitnapse
gitnapse clone xscriptor/gitnapse:develop
gitnapse clone https://github.com/xscriptor/gitnapse.git

# Inside a cloned repo — commit and push
gitnapse status
gitnapse commit -m "add new feature"
gitnapse push origin main

# Check history and branches
gitnapse log -n 5
gitnapse branch

# Pull request management via API
gitnapse pr list xscriptor/gitnapse
gitnapse pr create xscriptor/gitnapse -t "Feature" -H feat/new -B main
gitnapse pr merge xscriptor/gitnapse -n 7 -m squash
</code></pre>
