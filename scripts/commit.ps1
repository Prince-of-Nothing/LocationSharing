# scripts/commit.ps1 — Gate + commit helper (run after a feature is done).
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File scripts/commit.ps1 -Message "feat(x): ..." -Body "details"
#
# Runs fmt/clippy/build/test, prepends a CHANGELOG entry stub, commits with
# the co-author trailers from AGENTS.md. Does NOT push.

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

$trailers = @(
    "Co-authored-by: Mordret <blacksnake@noreply.codeberg.org>",
    "Co-authored-by: Andrei Cliucovschi <cliucovschiandrei@gmail.com>",
    "Co-authored-by: Pleșu Dinu <dinu.plesu@isa.utm.md>",
    "Co-authored-by: Nedelea Dragoș <63261775+dragos-nedelea@users.noreply.github.com>",
    "Co-authored-by: dana-r19 <200202105+dana-r19@users.noreply.github.com>",
    "Co-authored-by: Tcacenco Nichita <nichita.tcacenco@isa.utm.md>"
)

$gitArgs = @("commit", "-m", $Message)
if ($Body) { $gitArgs += @("-m", $Body) }
foreach ($t in $trailers) { $gitArgs += @("-m", $t) }

git add -A
git commit @gitArgs
Write-Host "Committed: $Message"