<h1 align="center">GitNapse Collaborative Section</h1>

<div id="content"></div>
<h2 align="center">Contents</h2>
<ul>
  <li><a href="#branch-policy">Branch Policy</a></li>
  <li><a href="#pr-flow">Pull Request Flow</a></li>
  <li><a href="#release-flow">Release Publishing Flow</a></li>
  <li><a href="#maintainer-checklist">Maintainer Checklist</a></li>
</ul>

<h2 id="branch-policy" align="center">Branch Policy</h2>
<ul>
  <li><code>main</code> is protected and must not receive direct pushes.</li>
  <li>All changes must be developed in topic branches and merged through Pull Requests.</li>
  <li>Required checks (CI/build/tests) must pass before merge.</li>
  <li>Use squash merge or rebase merge according to repository settings.</li>
</ul>

<h2 id="pr-flow" align="center">Pull Request Flow</h2>
<ol>
  <li>Create a branch from updated <code>main</code> (example: <code>feat/oauth-improvements</code>).</li>
  <li>Implement the change, run local validation, and update docs when behavior changes.</li>
  <li>Push the branch and open a PR targeting <code>main</code>.</li>
  <li>Request review and address comments with follow-up commits.</li>
  <li>Merge only after required checks and review approvals are complete.</li>
</ol>

<p><strong>Suggested local command sequence:</strong></p>
<pre><code class="language-bash">git checkout main
git pull --ff-only
git checkout -b feat/short-description
# code changes
cargo check
git add .
git commit -m "feat: short description"
git push -u origin feat/short-description
</code></pre>

<h2 id="release-flow" align="center">Release Publishing Flow</h2>
<p>
  Releases are automated by <code>.github/workflows/release.yml</code>. When a version tag is pushed, the workflow compiles binaries for
  Windows, Linux (Ubuntu, Arch, Fedora), and macOS, then uploads assets to GitHub Releases.
</p>
<ol>
  <li>Ensure the target commit is already merged into <code>main</code> through PR.</li>
  <li>Create an annotated semantic version tag (example: <code>v1.2.0</code>).</li>
  <li>Push the tag to origin to trigger the release workflow.</li>
  <li>Wait for Actions jobs to complete and verify uploaded assets in the Release page.</li>
</ol>

<p><strong>Release command sequence:</strong></p>
<pre><code class="language-bash">git checkout main
git pull --ff-only
git tag -a v1.2.0 -m "GitNapse v1.2.0"
git push origin v1.2.0
</code></pre>

<p><strong>Manual rebuild of an existing release tag:</strong></p>
<ul>
  <li>Open <strong>GitHub -&gt; Actions -&gt; Release</strong>.</li>
  <li>Run workflow manually with <code>release_tag</code> set to an existing tag.</li>
  <li>The workflow will upload/update assets for that release tag.</li>
</ul>

<h2 id="maintainer-checklist" align="center">Maintainer Checklist</h2>
<ul>
  <li>Confirm PR merged into <code>main</code> and CI green.</li>
  <li>Confirm docs are aligned with user-visible changes.</li>
  <li>Create semantic version tag from <code>main</code>.</li>
  <li>Validate release assets for all target platforms after workflow completion.</li>
</ul>
