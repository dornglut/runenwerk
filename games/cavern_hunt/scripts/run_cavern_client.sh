#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
if git -C "$SCRIPT_DIR" rev-parse --show-toplevel >/dev/null 2>&1; then
  ROOT_DIR="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
else
  ROOT_DIR="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
fi
cd "$ROOT_DIR"

NET_PROFILE="${CAVERN_NET_PROFILE:-local_dev}"
CONFIG_PATH="${CAVERN_CLIENT_CONFIG_PATH:-games/cavern_hunt/assets/networking/client/${NET_PROFILE}.ron}"
if [[ ! -f "$CONFIG_PATH" ]]; then
  echo "Missing client config at $CONFIG_PATH" >&2
  echo "Set CAVERN_CLIENT_CONFIG_PATH or CAVERN_NET_PROFILE." >&2
  exit 1
fi

CERT_PATH_FROM_CONFIG="$(rg -o 'cert_path:\\s*\"[^\"]+\"' "$CONFIG_PATH" 2>/dev/null | head -n1 | sed -E 's/cert_path:\\s*\"([^\"]+)\"/\\1/' || true)"
CERT_PATH="${GROTTO_SERVER_CERT_PATH:-${CERT_PATH_FROM_CONFIG:-var/dev/server-cert.der}}"
WAIT_SECONDS="${CAVERN_WAIT_FOR_CERT_SECONDS:-10}"

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
  echo "Start the server first with games/cavern_hunt/scripts/run_cavern_server.sh" >&2
  exit 1
fi

CARGO_PROFILE_ARGS=()
if [[ "${CAVERN_RELEASE:-1}" == "1" || "${CAVERN_RELEASE:-1}" == "true" || "${CAVERN_RELEASE:-1}" == "TRUE" ]]; then
  CARGO_PROFILE_ARGS+=(--release)
fi

USE_PREBUILT="${CAVERN_USE_PREBUILT:-0}"
if [[ "$USE_PREBUILT" == "1" || "$USE_PREBUILT" == "true" || "$USE_PREBUILT" == "TRUE" ]]; then
  BIN_PROFILE_DIR="debug"
  if [[ "${#CARGO_PROFILE_ARGS[@]}" -gt 0 ]]; then
    BIN_PROFILE_DIR="release"
  fi
  BIN_PATH="target/${BIN_PROFILE_DIR}/grotto_client"
  if [[ ! -x "$BIN_PATH" ]]; then
    echo "Missing prebuilt client binary at $BIN_PATH" >&2
    echo "Build first with: cargo build ${CARGO_PROFILE_ARGS[*]} -p grotto_client" >&2
    exit 1
  fi
  exec "$BIN_PATH" --config "$CONFIG_PATH" "$@"
fi

exec cargo run "${CARGO_PROFILE_ARGS[@]}" -p grotto_client -- --config "$CONFIG_PATH" "$@"
