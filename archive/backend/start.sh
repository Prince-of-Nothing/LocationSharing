#!/usr/bin/env bash
# Pinpoint — Start the full stack (backend + web client)
#
# Usage: ./start.sh [port]
#   PORT defaults to 8000
#
# Prerequisites:
#   - Go 1.23+ installed and on PATH
#   - This script must be run from the backend/ directory

set -euo pipefail

cd "$(dirname "$0")"

PORT="${PORT:-8000}"
export PORT
export DB_DRIVER="${DB_DRIVER:-memory}"
export SEED_DEMO_DATA="${SEED_DEMO_DATA:-true}"
export WEB_ROOT="${WEB_ROOT:-../web/public}"
export COOKIE_INSECURE="${COOKIE_INSECURE:-true}"

echo "=============================================="
echo "  Pinpoint — Starting Backend + Web Client"
echo "=============================================="
echo ""
echo "  Backend:  http://localhost:${PORT}/"
echo "  Web App:  http://localhost:${PORT}/"
echo "  API:      http://localhost:${PORT}/api/v1/"
echo ""
echo "  Demo accounts:"
echo "    alice@example.com  / DemoPass123!"
echo "    bob@example.com    / DemoPass456!"
echo "    admin@example.com  / ChangeMe123! (must change password)"
echo ""
echo "  Press Ctrl+C to stop"
echo "=============================================="

go run ./cmd/server