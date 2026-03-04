#!/usr/bin/env zsh
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

CERT_PATH="${GROTTO_SERVER_CERT_PATH:-var/dev/server-cert.der}"
WAIT_SECONDS="${CAVERN_WAIT_FOR_CERT_SECONDS:-10}"

export GROTTO_SERVER_ID="${GROTTO_SERVER_ID:-srv-local}"
export GROTTO_SERVER_NAME="${GROTTO_SERVER_NAME:-localhost}"
export GROTTO_SERVER_ENDPOINT="${GROTTO_SERVER_ENDPOINT:-127.0.0.1:7000}"
export GROTTO_JOIN_TICKET="${GROTTO_JOIN_TICKET:-local-ticket}"
export GROTTO_SERVER_CERT_PATH="$CERT_PATH"
export AXIOM_API_BASE_URL="${AXIOM_API_BASE_URL:-http://api.localhost}"
export AXIOM_DEVICE_ID="${AXIOM_DEVICE_ID:-grotto-client-local}"

if [[ ! -f "$CERT_PATH" ]]; then
  echo "Waiting for server certificate at $CERT_PATH"
  for _ in $(seq 1 "$WAIT_SECONDS"); do
    if [[ -f "$CERT_PATH" ]]; then
      break
    fi
    sleep 1
  done
fi

if [[ ! -f "$CERT_PATH" ]]; then
  echo "Server certificate not found at $CERT_PATH" >&2
  echo "Start the server first with scripts/run_cavern_server.sh" >&2
  exit 1
fi

exec cargo run -p grotto_client "$@"
