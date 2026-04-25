<h1 align="center">Test Coverage Notes</h1>

<h2 align="center">Implemented Coverage</h2>
<ul>
  <li><strong>General repository search</strong>: validates the public search endpoint path and query handling.</li>
  <li><strong>Authenticated private repository mode</strong>: validates <code>@me</code> request path and filtering behavior.</li>
  <li><strong>Unauthorized handling</strong>: validates expected failure when <code>@me</code> runs without valid authentication.</li>
  <li><strong>Secure storage fallback</strong>: validates file backend save/load/clear.</li>
  <li><strong>Unix file permissions</strong>: validates secure permission mode <code>0600</code> when fallback file storage is used.</li>
  <li><strong>Authentication precedence</strong>: validates environment token precedence over other sources.</li>
</ul>

<h2 align="center">Why Integration Tests</h2>
<p>
  Tests are located under repository-level <code>tests/</code> to keep them outside application modules and closer to real user flows.
  Mocked HTTP responses are used to avoid external network dependency and improve deterministic results.
</p>
