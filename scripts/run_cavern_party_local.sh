#!/usr/bin/env zsh
set -euo pipefail
# Some environments disallow lowering process priority for background jobs.
# Keep local multiplayer launch portable by disabling zsh background niceness.
setopt no_bg_nice

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

CLIENT_COUNT="${1:-2}"
if [[ "$CLIENT_COUNT" -lt 1 || "$CLIENT_COUNT" -gt 4 ]]; then
  echo "Usage: scripts/run_cavern_party_local.sh [1-4]" >&2
  exit 1
fi
if [[ "$CLIENT_COUNT" -ge 3 ]]; then
  NET_PROFILE_DEFAULT="four_local_conservative"
else
  NET_PROFILE_DEFAULT="two_local_balanced"
fi
NET_PROFILE="${CAVERN_NET_PROFILE:-$NET_PROFILE_DEFAULT}"
CLIENT_CONFIG_PATH="${CAVERN_CLIENT_CONFIG_PATH:-game/assets/networking/client/${NET_PROFILE}.ron}"
SERVER_CONFIG_PATH="${CAVERN_SERVER_CONFIG_PATH:-game/assets/networking/server/${NET_PROFILE}.ron}"

SERVER_PID=""
CERT_PATH_FROM_CONFIG="$(rg -o 'cert_output_path:\\s*\"[^\"]+\"' "$SERVER_CONFIG_PATH" 2>/dev/null | head -n1 | sed -E 's/cert_output_path:\\s*\"([^\"]+)\"/\\1/' || true)"
CERT_PATH="${CERT_PATH_FROM_CONFIG:-var/dev/server-cert.der}"
CLIENT_START_STAGGER_SECONDS="${CAVERN_CLIENT_START_STAGGER_SECONDS:-0.15}"

if [[ -z "${CAVERN_RELEASE:-}" ]]; then
  export CAVERN_RELEASE=1
fi

BUILD_PROFILE_ARGS=()
if [[ "${CAVERN_RELEASE}" == "1" || "${CAVERN_RELEASE}" == "true" || "${CAVERN_RELEASE}" == "TRUE" ]]; then
  BUILD_PROFILE_ARGS+=(--release)
fi

echo "Building binaries once for local party (${BUILD_PROFILE_ARGS:+release})..."
cargo build "${BUILD_PROFILE_ARGS[@]}" -p grotto_server -p grotto_client
cleanup() {
  if [[ -n "$SERVER_PID" ]] && kill -0 "$SERVER_PID" >/dev/null 2>&1; then
    kill "$SERVER_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT INT TERM

rm -f "$CERT_PATH"
CAVERN_USE_PREBUILT=1 \
  CAVERN_SERVER_CONFIG_PATH="${SERVER_CONFIG_PATH}" \
  scripts/run_cavern_server.sh > /tmp/cavern_hunt_server.log 2>&1 &
SERVER_PID="$!"

for _ in $(seq 1 15); do
  if ! kill -0 "$SERVER_PID" >/dev/null 2>&1; then
    echo "Server process exited early; check /tmp/cavern_hunt_server.log" >&2
    tail -n 40 /tmp/cavern_hunt_server.log >&2 || true
    exit 1
  fi
  [[ -f "$CERT_PATH" ]] && break
  sleep 1
done

if [[ ! -f "$CERT_PATH" ]]; then
  echo "Server certificate was not generated; check /tmp/cavern_hunt_server.log" >&2
  tail -n 40 /tmp/cavern_hunt_server.log >&2 || true
  exit 1
fi

if ! kill -0 "$SERVER_PID" >/dev/null 2>&1; then
  echo "Server process is not running after startup; check /tmp/cavern_hunt_server.log" >&2
  tail -n 40 /tmp/cavern_hunt_server.log >&2 || true
  exit 1
fi

echo "Server started (pid $SERVER_PID). Launching $CLIENT_COUNT client(s)."
echo "Server log: /tmp/cavern_hunt_server.log"

CLIENT_MATERIAL_PROFILE="${CAVERN_MATERIAL_PROFILE:-performance}"
echo "Client material profile: ${CLIENT_MATERIAL_PROFILE}"
echo "Network profile: ${NET_PROFILE}"
echo "Client config: ${CLIENT_CONFIG_PATH}"
echo "Server config: ${SERVER_CONFIG_PATH}"

for index in $(seq 1 "$CLIENT_COUNT"); do
  CAVERN_USE_PREBUILT=1 \
    CAVERN_CLIENT_CONFIG_PATH="${CLIENT_CONFIG_PATH}" \
    CAVERN_MATERIAL_PROFILE="$CLIENT_MATERIAL_PROFILE" \
    scripts/run_cavern_client.sh > "/tmp/cavern_hunt_client_${index}.log" 2>&1 &
  if [[ "$index" -lt "$CLIENT_COUNT" ]]; then
    sleep "$CLIENT_START_STAGGER_SECONDS"
  fi
done

wait "$SERVER_PID"
