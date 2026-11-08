# Pinpoint automatic QA (Windows mirror of scripts/qa.sh).
# Exit 0 = clean tree. Mirrors the map-qa CI job.
Set-StrictMode -Version Latest
Set-Location (Join-Path $PSScriptRoot "..")
$fail = 0
function Ok($m) { Write-Output "OK  $m" }
function Bad($m) { Write-Output "FAIL $m"; $script:fail = 1 }
function Warn($m) { Write-Output "WARN $m" }

Write-Output "== Pinpoint automatic QA (OSM map parity + SDLC) =="

$hits = Get-ChildItem -Path web,mobile,src -Recurse -Include *.js,*.kt,*.kts,*.rs,*.conf -ErrorAction SilentlyContinue |
  Select-String -Pattern 'tile\.openstreetmap\.org/' | Where-Object { $_.Line -notmatch '(?i)never|prohibit|must not|do not|policy|usage' }
if ($hits) { Bad "tile.openstreetmap.org tile URL found in client code" } else { Ok "no tile.openstreetmap.org tile URLs in clients" }

if (Select-String -Path web/public/js/config.js -Pattern 'openstreetmap.org/copyright' -Quiet) { Ok "web attribution" } else { Bad "web attribution missing" }
if (Select-String -Path mobile/THIRD_PARTY_NOTICES.md -Pattern 'OpenStreetMap contributors' -Quiet) { Ok "mobile attribution" } else { Bad "mobile attribution missing" }
if (Select-String -Path src/map.rs -Pattern 'OpenStreetMap contributors' -Quiet) { Ok "rust attribution" } else { Bad "rust attribution missing" }

foreach ($h in @('nominatim.openstreetmap.org','router.project-osrm.org','overpass-api.de','basemaps.cartocdn.com','maps.wikimedia.org')) {
  if (Select-String -Path web/nginx.conf -Pattern ([regex]::Escape($h)) -Quiet) { Ok "CSP has $h" } else { Bad "CSP missing $h" }
}
if (Select-String -Path web/public/sw.js -Pattern 'pinpoint-shell-v3' -Quiet) { Ok "sw.js v3" } else { Bad "sw.js version not v3" }

if (Get-Command node -ErrorAction SilentlyContinue) {
  Get-ChildItem web/public/js/*.js, web/public/sw.js | ForEach-Object {
    $tmp = Join-Path $env:TEMP "qa-check.mjs"
    Copy-Item $_.FullName $tmp -Force
    $null = cmd /c "node --input-type=module --check `"$tmp`" 2>`$null"
    if ($LASTEXITCODE -eq 0) { Ok "node --check $($_.Name)" } else { Bad "syntax $($_.Name)" }
  }
} else { Warn "node not found, skipped syntax checks" }

foreach ($f in @('web/THIRD_PARTY_NOTICES.md','mobile/THIRD_PARTY_NOTICES.md')) {
  foreach ($s in @('Nominatim','OSRM','Overpass')) {
    if (Select-String -Path $f -Pattern $s -Quiet) { Ok "$f mentions $s" } else { Bad "$f missing $s" }
  }
}

if ($fail -eq 0) { Write-Output "QA PASS"; exit 0 } else { Write-Output "QA FAIL"; exit 1 }
