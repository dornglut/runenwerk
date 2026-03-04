#!/usr/bin/env zsh
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

export AXIOM_API_BASE_URL="${AXIOM_API_BASE_URL:-http://api.localhost}"
export DEDICATED_SERVER_SHARED_SECRET="${DEDICATED_SERVER_SHARED_SECRET:-}"

echo "Starting Cavern Hunt dedicated server on 127.0.0.1:7000"
echo "Dev certificate will be written to var/dev/server-cert.der"

exec cargo run -p grotto_server "$@"
