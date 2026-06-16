$ErrorActionPreference = "Stop"

$asset = "gitnapse-$env:RELEASE_TAG-windows-x86_64.exe"
Copy-Item -Path "target/release/gitnapse.exe" -Destination $asset

"ASSET=$asset" | Out-File -FilePath $env:GITHUB_ENV -Encoding utf8 -Append
