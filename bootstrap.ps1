# Bootstrap
$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Write-Host "==> Installing git hooks"
git -C $repoRoot config core.hooksPath scripts/hooks
Write-Host "==> Fetching dependencies"
Push-Location $repoRoot
cargo fetch --locked
Pop-Location
Write-Host "Bootstrap complete."
