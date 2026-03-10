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
CONFIG_PATH="${CAVERN_SERVER_CONFIG_PATH:-games/cavern_hunt/assets/networking/server/${NET_PROFILE}.ron}"
if [[ ! -f "$CONFIG_PATH" ]]; then
  echo "Missing server config at $CONFIG_PATH" >&2
  echo "Set CAVERN_SERVER_CONFIG_PATH or CAVERN_NET_PROFILE." >&2
  exit 1
fi

echo "Starting Cavern Hunt dedicated server with config ${CONFIG_PATH}"

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
  BIN_PATH="target/${BIN_PROFILE_DIR}/grotto_server"
  if [[ ! -x "$BIN_PATH" ]]; then
    echo "Missing prebuilt server binary at $BIN_PATH" >&2
    echo "Build first with: cargo build ${CARGO_PROFILE_ARGS[*]} -p grotto_server" >&2
    exit 1
  fi
  exec "$BIN_PATH" --config "$CONFIG_PATH" "$@"
fi

exec cargo run "${CARGO_PROFILE_ARGS[@]}" -p grotto_server -- --config "$CONFIG_PATH" "$@"
