# scripts/commit.ps1 — Gate + commit helper (run after a feature is done).
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File scripts/commit.ps1 -Message "feat(x): ..." -Body "details"
#
# Runs fmt/clippy/build/test, prepends a CHANGELOG entry stub, commits.
# Does NOT push.

param(
    [Parameter(Mandatory = $true)][string]$Message,
    [string]$Body = "",
    [switch]$SkipE2E
)

$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path))

function Gate($cmd) { Write-Host ">> $cmd"; Invoke-Expression $cmd; if ($LASTEXITCODE -ne 0) { throw "gate failed: $cmd" } }

Gate "cargo fmt --all -- --check"
Gate "cargo clippy --all-targets -- -D warnings"
Gate "cargo build --locked"
Gate "cargo test --locked"

# Append a one-line stub to CHANGELOG [Unreleased] if not already present.
$cl = "CHANGELOG.md"
if (Test-Path $cl) {
    $text = Get-Content $cl -Raw
    if ($text -notmatch [regex]::Escape($Message)) {
        $stamp = (Get-Date).ToString("yyyy-MM-dd")
        $stub = "- $Message ($stamp)"
        $text = $text -replace "(## \[Unreleased\]\r?\n)", "`$1### Added`n$stub`n"
        Set-Content -Path $cl -Value $text -NoNewline
        Write-Host "CHANGELOG stub added."
    }
}

$gitArgs = @("commit", "-m", $Message)
if ($Body) { $gitArgs += @("-m", $Body) }

git add -A
git commit @gitArgs
Write-Host "Committed: $Message"