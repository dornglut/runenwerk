#!/usr/bin/env zsh
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

CLIENT_COUNT="${1:-2}"
if [[ "$CLIENT_COUNT" -lt 1 || "$CLIENT_COUNT" -gt 4 ]]; then
  echo "Usage: scripts/run_cavern_party_local.sh [1-4]" >&2
  exit 1
fi

SERVER_PID=""
cleanup() {
  if [[ -n "$SERVER_PID" ]] && kill -0 "$SERVER_PID" >/dev/null 2>&1; then
    kill "$SERVER_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT INT TERM

scripts/run_cavern_server.sh > /tmp/cavern_hunt_server.log 2>&1 &
SERVER_PID="$!"

for _ in $(seq 1 15); do
  [[ -f "var/dev/server-cert.der" ]] && break
  sleep 1
done

if [[ ! -f "var/dev/server-cert.der" ]]; then
  echo "Server certificate was not generated; check /tmp/cavern_hunt_server.log" >&2
  exit 1
fi

echo "Server started (pid $SERVER_PID). Launching $CLIENT_COUNT client(s)."
echo "Server log: /tmp/cavern_hunt_server.log"

for index in $(seq 1 "$CLIENT_COUNT"); do
  AXIOM_DEVICE_ID="grotto-client-local-$index" \
    scripts/run_cavern_client.sh > "/tmp/cavern_hunt_client_${index}.log" 2>&1 &
done

wait "$SERVER_PID"
