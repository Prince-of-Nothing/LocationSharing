#!/usr/bin/env bash
# Pinpoint — automatic map & compliance QA (mirrors CI job `map-qa`).
# Exit 0 = clean tree. No network calls to OSM services (offline-safe).
# Usage: ./scripts/qa.sh [--fast]
set -u
cd "$(dirname "$0")/.."
fail=0
ok() { printf 'OK  %s\n' "$1"; }
bad() { printf 'FAIL %s\n' "$1"; fail=1; }
warn() { printf 'WARN %s\n' "$1"; }

echo "== Pinpoint automatic QA (OSM map parity + SDLC) =="

# 1. Tile policy: never use tile.openstreetmap.org tile URLs (blocker).
# Bare host mentions in policy docs ("never tile.openstreetmap.org") are OK;
# actual tile URLs look like https://tile.openstreetmap.org/{z}/… — flag those.
if grep -R --include='*.js' --include='*.kt' --include='*.kts' --include='*.rs' --include='*.conf' -n 'tile\.openstreetmap\.org/' web/ mobile/ src/ backend/web/ 2>/dev/null | grep -v -i -E 'never|prohibit|must not|do not|policy|usage' | grep -q .; then
  bad "tile.openstreetmap.org tile URL found in client code (use CARTO/Wikimedia/OpenFreeMap instead)"
else
  ok "no tile.openstreetmap.org tile URLs in clients"
fi

# 2. OSM references present (informational).
for ref in 'wiki.openstreetmap.org/wiki/Main_Page' 'organicmaps.app'; do
  grep -R --include='*.md' --include='*.js' --include='*.kt' --include='*.rs' -q "$ref" docs/sdlc web/ mobile/ src/ 2>/dev/null && ok "references $ref" || warn "missing reference $ref"
done

# 3. Attribution present (blocker).
grep -q 'openstreetmap.org/copyright' web/public/js/config.js && ok "web attribution (config.js)" || bad "web attribution missing (config.js)"
grep -q 'OpenStreetMap contributors' mobile/THIRD_PARTY_NOTICES.md && ok "mobile attribution (notices)" || bad "mobile attribution missing"
grep -q 'OpenStreetMap contributors' src/map.rs && ok "rust attribution (map.rs)" || bad "rust attribution missing (map.rs)"

# 4. CSP sync (nginx vs config).
for host in 'nominatim.openstreetmap.org' 'router.project-osrm.org' 'overpass-api.de'; do
  grep -q "$host" web/nginx.conf && ok "CSP connect-src has $host" || bad "CSP missing $host"
done
grep -q 'basemaps.cartocdn.com' web/nginx.conf && ok "CSP img-src has CARTO" || bad "CSP missing CARTO"
grep -q 'maps.wikimedia.org' web/nginx.conf && ok "CSP img-src has Wikimedia" || bad "CSP missing Wikimedia"

# 5. SW SHELL sync (warn).
for f in web/public/js/*.js; do
  b=$(basename "$f")
  [ "$b" = "config.js" ] || grep -q "$b" web/public/sw.js 2>/dev/null && ok "sw.js lists $b" || warn "sw.js missing $b"
done
grep -q 'pinpoint-shell-v3' web/public/sw.js && ok "sw.js v3" || bad "sw.js version not v3"

# 6. Web syntax (blocker if node present).
if command -v node >/dev/null 2>&1; then
  for f in web/public/js/*.js web/public/sw.js; do
    node --input-type=module --check < "$f" >/dev/null 2>&1 && ok "node --check $f" || bad "syntax error $f"
  done
  node -e "JSON.parse(require('fs').readFileSync('web/public/manifest.webmanifest','utf8'))" 2>/dev/null && ok "manifest JSON" || bad "manifest JSON invalid"
else
  warn "node not found — skipped JS syntax checks"
fi

# 7. Mermaid fences closed (simple lint).
for f in docs/uml/*.md docs/sdlc/*.md; do
  [ -f "$f" ] || continue
  opens=$(grep -c '```mermaid' "$f" 2>/dev/null || true)
  closes=$(grep -c '^```$' "$f" 2>/dev/null || true)
  if [ "$opens" -gt 0 ] && [ "$closes" -lt "$opens" ]; then
    bad "unclosed mermaid fence in $f"
  fi
done
ok "mermaid fence lint done"

# 8. Notices mention OSM services (blocker).
for f in web/THIRD_PARTY_NOTICES.md mobile/THIRD_PARTY_NOTICES.md; do
  for svc in Nominatim OSRM Overpass; do
    grep -q "$svc" "$f" && ok "$f mentions $svc" || bad "$f missing $svc"
  done
done

# 9. SDLC set complete.
for f in docs/sdlc/README.md docs/sdlc/00-overview.md docs/sdlc/01-business-requirements.md docs/sdlc/03-functional-requirements.md docs/sdlc/04-non-functional-requirements.md docs/sdlc/06-technical-requirements.md docs/sdlc/07-openstreetmap-integration.md docs/sdlc/11-qa-and-verification.md; do
  [ -f "$f" ] && ok "exists $f" || bad "missing $f"
done
for f in docs/uml/12-sequence-map-search.md docs/uml/13-sequence-routing-navigation.md docs/uml/14-offline-and-bookmarks.md docs/uml/15-web-slippy-engine.md; do
  [ -f "$f" ] && ok "exists $f" || bad "missing $f"
done

if [ "$fail" -eq 0 ]; then echo "QA PASS"; else echo "QA FAIL"; fi
exit "$fail"
