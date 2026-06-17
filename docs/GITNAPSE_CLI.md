<h1 align="center">GitNapse CLI Reference</h1>

<div id="content"></div>
<h2 align="center">Contents</h2>
<ul>
  <li><a href="#overview">Overview</a></li>
  <li><a href="#authentication">Authentication</a></li>
  <li><a href="#git-operations">Git Operations (local git)</a></li>
  <li><a href="#api-operations">GitHub API Operations</a></li>
  <li><a href="#auto-detect">Auto-detect Repository</a></li>
  <li><a href="#error-handling">Error Handling</a></li>
  <li><a href="#examples">Quick Examples</a></li>
  <li><a href="#full-command-list">Full Command List</a></li>
</ul>

<h2 id="overview" align="center">Overview</h2>
<p>
  GitNapse provides a set of CLI commands that operate both via the <strong>GitHub REST API</strong>
  (for queries, PR management, issues, CI checks, comparisons) and via <strong>local git</strong>
  (for clone, commit, push, pull, fetch, checkout, diff, stash, tag, reset, status, log, branch).
  Authentication is shared across all commands.
</p>

<h2 id="authentication" align="center">Authentication</h2>
<p>
  Commands that hit the GitHub API (<code>clone</code> with <code>owner/repo</code> format,
  <code>pr</code>, <code>issue</code>, <code>ci</code>, <code>compare</code>) resolve
  credentials in this order:
</p>
<ol>
  <li><code>GITHUB_TOKEN</code> environment variable</li>
  <li>OAuth session (from <code>gitnapse auth oauth login</code>)</li>
  <li>Stored token (from <code>gitnapse auth set</code>)</li>
</ol>

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
      <td><code>gitnapse clone xscriptor/gitnapse:develop --dir ./myclone</code></td>
      <td>
        Accepts <code>owner/repo[:branch]</code> (resolved via API) or a full git URL.
        Optional <code>--dir</code> for destination. Checks if target already exists.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse commit</code></td>
      <td>Commit changes</td>
      <td><code>gitnapse commit -m "fix: typo" -a</code></td>
      <td>
        <code>-m</code> is required. <code>-a</code> stages all changes first
        (<code>git add -A</code>). Without <code>-a</code>, commits only staged changes.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse push</code></td>
      <td>Push commits to remote</td>
      <td><code>gitnapse push origin main --force-with-lease</code></td>
      <td>
        Optional <code>[remote]</code> and <code>[branch]</code>.
        <code>--force-with-lease</code> for safe force push.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse pull</code></td>
      <td>Pull changes from remote</td>
      <td><code>gitnapse pull --rebase</code></td>
      <td>
        Optional <code>[remote]</code> and <code>[branch]</code>.
        <code>--rebase</code> to rebase instead of merge.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse fetch</code></td>
      <td>Fetch from remote</td>
      <td><code>gitnapse fetch --prune</code></td>
      <td><code>--prune</code> removes stale remote-tracking branches.</td>
    </tr>
    <tr>
      <td><code>gitnapse checkout</code></td>
      <td>Switch branches</td>
      <td><code>gitnapse checkout -b feature/new</code></td>
      <td><code>-b</code> creates a new branch before switching.</td>
    </tr>
    <tr>
      <td><code>gitnapse diff</code></td>
      <td>Show working tree diff</td>
      <td><code>gitnapse diff --staged --path src/main.rs</code></td>
      <td><code>--staged</code> shows staged changes. <code>--path</code> filters by file.</td>
    </tr>
    <tr>
      <td><code>gitnapse stash push</code></td>
      <td>Stash changes</td>
      <td><code>gitnapse stash push -m "WIP"</code></td>
      <td><code>-m</code> adds a description to the stash.</td>
    </tr>
    <tr>
      <td><code>gitnapse stash pop</code></td>
      <td>Restore topmost stash</td>
      <td><code>gitnapse stash pop</code></td>
      <td>Restores and removes the latest stash.</td>
    </tr>
    <tr>
      <td><code>gitnapse stash list</code></td>
      <td>List stashes</td>
      <td><code>gitnapse stash list</code></td>
      <td>Shows all stashed entries.</td>
    </tr>
    <tr>
      <td><code>gitnapse tag list</code></td>
      <td>List tags</td>
      <td><code>gitnapse tag list "v*"</code></td>
      <td>Optional glob pattern to filter.</td>
    </tr>
    <tr>
      <td><code>gitnapse tag create</code></td>
      <td>Create a tag</td>
      <td><code>gitnapse tag create v1.0 -m "Release 1.0"</code></td>
      <td><code>-m</code> creates an annotated tag.</td>
    </tr>
    <tr>
      <td><code>gitnapse tag delete</code></td>
      <td>Delete a tag (local + remote)</td>
      <td><code>gitnapse tag delete v1.0</code></td>
      <td>Deletes locally and pushes the deletion to remote.</td>
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
      <td>Runs <code>git log --oneline -n</code>. Default: 20 entries.</td>
    </tr>
    <tr>
      <td><code>gitnapse branch</code></td>
      <td>List branches</td>
      <td><code>gitnapse branch</code></td>
      <td>Runs <code>git branch -a</code>.</td>
    </tr>
    <tr>
      <td><code>gitnapse reset</code></td>
      <td>Reset current HEAD</td>
      <td><code>gitnapse reset HEAD~1 --hard</code></td>
      <td>Target defaults to <code>HEAD</code>. <code>--hard</code> discards working tree changes.</td>
    </tr>
  </tbody>
</table>

<h2 id="api-operations" align="center">GitHub API Operations</h2>
<p>
  These commands use the GitHub REST API and require authentication for private repositories
  or for creating/merging PRs and issues.
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
        State: <code>open</code> (default), <code>closed</code>, <code>all</code>.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse pr create</code></td>
      <td>Create a pull request</td>
      <td><code>gitnapse pr create xscriptor/gitnapse -t "Feature" -H feat/new -B main</code></td>
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
        <code>--number</code> required. Method: <code>merge</code> (default),
        <code>squash</code>, or <code>rebase</code>.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse issue list</code></td>
      <td>List issues</td>
      <td><code>gitnapse issue list xscriptor/gitnapse -s open</code></td>
      <td>
        Shows issue number, state, title, and author. PRs show a <code>PR</code> marker.
        State: <code>open</code> (default), <code>closed</code>, <code>all</code>.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse issue create</code></td>
      <td>Create an issue</td>
      <td><code>gitnapse issue create xscriptor/gitnapse -t "Bug" -b "description"</code></td>
      <td><code>--title</code> required. Optional <code>--body</code>.</td>
    </tr>
    <tr>
      <td><code>gitnapse issue close</code></td>
      <td>Close an issue</td>
      <td><code>gitnapse issue close xscriptor/gitnapse -n 42</code></td>
      <td>Requires <code>--number</code>.</td>
    </tr>
    <tr>
      <td><code>gitnapse ci</code></td>
      <td>Show CI status</td>
      <td><code>gitnapse ci xscriptor/gitnapse -b main</code></td>
      <td>
        Shows check runs for the latest commit on the branch.
        Branch defaults to <code>main</code>.
      </td>
    </tr>
    <tr>
      <td><code>gitnapse compare</code></td>
      <td>Compare two branches</td>
      <td><code>gitnapse compare xscriptor/gitnapse main develop</code></td>
      <td>Shows ahead/behind count, files changed with additions and deletions per file.</td>
    </tr>
  </tbody>
</table>

<h2 id="auto-detect" align="center">Auto-detect Repository</h2>
<p>
  When using API commands (<code>pr</code>, <code>issue</code>, <code>ci</code>,
  <code>compare</code>), you can omit <code>owner/</code> if you are inside a cloned
  repository — GitNapse will parse the <code>origin</code> remote to extract
  <code>owner/repo</code> automatically.
</p>

<pre><code># Inside /home/user/projects/gitnapse (cloned from xscriptor/gitnapse)
gitnapse pr list            # works — detects xscriptor/gitnapse
gitnapse issue list         # works
gitnapse ci                 # works

# Outside a cloned repo, you must specify:
gitnapse pr list xscriptor/gitnapse
</code></pre>

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
      <td>Destination path exists</td>
      <td><code>destination path '...' already exists</code></td>
    </tr>
    <tr>
      <td>Nothing to commit</td>
      <td><code>nothing to commit (working tree clean)</code></td>
    </tr>
    <tr>
      <td>No stash entries</td>
      <td><code>no stash entries to pop</code></td>
    </tr>
    <tr>
      <td>GitHub API rate limit</td>
      <td><code>GitHub API rate limit exceeded — resets at timestamp ...</code></td>
    </tr>
  </tbody>
</table>

<h2 id="examples" align="center">Quick Examples</h2>

<pre><code># Clone a repo
gitnapse clone xscriptor/gitnapse
gitnapse clone xscriptor/gitnapse:develop --dir ./myclone
gitnapse clone https://github.com/xscriptor/gitnapse.git

# Daily workflow
gitnapse pull --rebase
gitnapse status
gitnapse diff --staged
gitnapse commit -m "feat: add login" -a
gitnapse push origin main

# Branches and history
gitnapse checkout -b feature/new
gitnapse branch
gitnapse log -n 10

# Stashing
gitnapse stash push -m "WIP before refactor"
gitnapse stash list
gitnapse stash pop

# Tags
gitnapse tag list "v*"
gitnapse tag create v1.0 -m "Release 1.0"
gitnapse tag delete v0.9

# Reset
gitnapse reset HEAD~1
gitnapse reset HEAD~2 --hard

# Pull request management via API
gitnapse pr list
gitnapse pr create -t "Feature" -H feat/new -B main
gitnapse pr merge -n 7 -m squash

# Issues via API
gitnapse issue list -s open
gitnapse issue create -t "Bug found" -b "details here"
gitnapse issue close -n 42

# CI and comparison
gitnapse ci xscriptor/gitnapse -b main
gitnapse compare xscriptor/gitnapse main develop
</code></pre>

<h2 id="full-command-list" align="center">Full Command List</h2>

<pre><code>gitnapse
├── run              TUI with options
├── download-file    Download a file from GitHub
├── auth             Token and OAuth management
├── clone            Clone a repository (API + git)
├── commit           Commit changes (with -a for all)
├── push             Push to remote (--force-with-lease)
├── pull             Pull from remote (--rebase)
├── fetch            Fetch from remote (--prune)
├── checkout         Switch branches (-b to create)
├── diff             Show diff (--staged, --path)
├── stash
│   ├── push         Stash changes
│   ├── pop          Restore topmost stash
│   └── list         List stashes
├── tag
│   ├── list         List tags
│   ├── create       Create a tag
│   └── delete       Delete a tag (local + remote)
├── status           Working tree status
├── log              Commit log (-n)
├── branch           List branches
├── reset            Reset HEAD (--hard)
├── pr
│   ├── list         List PRs
│   ├── create       Create a PR
│   └── merge        Merge a PR
├── issue
│   ├── list         List issues
│   ├── create       Create an issue
│   └── close        Close an issue
├── ci               CI status for a branch
└── compare          Compare two branches
</code></pre>
