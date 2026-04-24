<h1 align="center">GitNapse Installation and Uninstallation</h1>

<div id="content"></div>
<h2 align="center">Contents</h2>
<ul>
  <li><a href="#install-all">Install (All Platforms)</a></li>
  <li><a href="#uninstall-linux">Uninstall Linux</a></li>
  <li><a href="#uninstall-macos">Uninstall macOS</a></li>
  <li><a href="#uninstall-wsl">Uninstall WSL</a></li>
  <li><a href="#uninstall-windows">Uninstall Windows</a></li>
</ul>

<h2 id="install-all" align="center">Install (All Platforms)</h2>
<ul>
  <li>Requirement: Rust toolchain installed (<code>cargo</code>, <code>rustc</code>).</li>
</ul>
<pre><code class="language-bash">cargo install --path .
gitnapse --help
</code></pre>

<h2 id="uninstall-linux" align="center">Uninstall Linux</h2>
<pre><code class="language-bash">cargo uninstall gitnapse
rm -rf ~/.config/GitNapse ~/.cache/GitNapse
</code></pre>

<h2 id="uninstall-macos" align="center">Uninstall macOS</h2>
<pre><code class="language-bash">cargo uninstall gitnapse
rm -rf ~/.config/GitNapse ~/.cache/GitNapse
</code></pre>

<h2 id="uninstall-wsl" align="center">Uninstall WSL</h2>
<p>Run inside your selected WSL distribution.</p>
<pre><code class="language-bash">cargo uninstall gitnapse
rm -rf ~/.config/GitNapse ~/.cache/GitNapse
</code></pre>

<h2 id="uninstall-windows" align="center">Uninstall Windows (PowerShell)</h2>
<pre><code class="language-powershell">cargo uninstall gitnapse
Remove-Item -Recurse -Force "$env:APPDATA\GitNapse" -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\GitNapse" -ErrorAction SilentlyContinue
</code></pre>
