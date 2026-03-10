#!/usr/bin/env zsh
set -euo pipefail

AXIOM_BASE_URL="${AXIOM_BASE_URL:-http://api.localhost}"
AXIOM_WS_BASE_URL="${AXIOM_WS_BASE_URL:-}"
AXIOM_OPERATOR_USERNAME="${AXIOM_OPERATOR_USERNAME:-axiom-operator}"
AXIOM_OPERATOR_PASSWORD="${AXIOM_OPERATOR_PASSWORD:-}"
AXIOM_OPERATOR_TOKEN="${AXIOM_OPERATOR_TOKEN:-}"
RUNTIME_SERVER_ID="${RUNTIME_SERVER_ID:-srv-local}"
FLEET_SERVICE_ID="${FLEET_SERVICE_ID:-fleet-control-default}"

if ! command -v curl >/dev/null 2>&1; then
  echo "curl is required" >&2
  exit 1
fi
if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required" >&2
  exit 1
fi

trimmed_base="${AXIOM_BASE_URL%/}"

to_ws_base_url() {
  local base_url="$1"
  case "$base_url" in
    https://*)
      echo "wss://${base_url#https://}"
      ;;
    http://*)
      echo "ws://${base_url#http://}"
      ;;
    ws://*|wss://*)
      echo "$base_url"
      ;;
    *)
      echo "$base_url"
      ;;
  esac
}

if [[ -z "$AXIOM_WS_BASE_URL" ]]; then
  AXIOM_WS_BASE_URL="$(to_ws_base_url "$trimmed_base")"
fi
AXIOM_WS_BASE_URL="${AXIOM_WS_BASE_URL%/}"

RUNTIME_WS_URL="${RUNTIME_WS_URL:-${AXIOM_WS_BASE_URL}/v2/operator/runtime/ws}"
FLEET_WS_URL="${FLEET_WS_URL:-${AXIOM_WS_BASE_URL}/v2/operator/fleet/ws}"

http_post_json() {
  local url="$1"
  local body="$2"
  local auth_token="${3:-}"
  local response
  if [[ -n "$auth_token" ]]; then
    response="$(
      curl -sS \
        -w '\n%{http_code}' \
        -H "content-type: application/json" \
        -H "authorization: Bearer ${auth_token}" \
        -X POST \
        "$url" \
        -d "$body"
    )"
  else
    response="$(
      curl -sS \
        -w '\n%{http_code}' \
        -H "content-type: application/json" \
        -X POST \
        "$url" \
        -d "$body"
    )"
  fi

  local status_code="${response##*$'\n'}"
  local response_body="${response%$'\n'*}"
  if [[ "$status_code" -lt 200 || "$status_code" -ge 300 ]]; then
    echo "request to ${url} failed with status ${status_code}" >&2
    echo "$response_body" >&2
    exit 1
  fi
  echo "$response_body"
}

if [[ -z "$AXIOM_OPERATOR_TOKEN" ]]; then
  if [[ -z "$AXIOM_OPERATOR_PASSWORD" ]]; then
    echo "Set AXIOM_OPERATOR_PASSWORD (or AXIOM_OPERATOR_TOKEN) before running this script." >&2
    exit 1
  fi
  login_payload="$(
    jq -cn \
      --arg username "$AXIOM_OPERATOR_USERNAME" \
      --arg password "$AXIOM_OPERATOR_PASSWORD" \
      '{ username: $username, password: $password }'
  )"
  login_body="$(http_post_json "${trimmed_base}/v2/auth/operator/login" "$login_payload")"
  AXIOM_OPERATOR_TOKEN="$(echo "$login_body" | jq -r '.access_token // empty')"
  if [[ -z "$AXIOM_OPERATOR_TOKEN" ]]; then
    echo "operator login succeeded but access_token was missing" >&2
    echo "$login_body" >&2
    exit 1
  fi
fi

issue_bridge_token() {
  local kind="$1"
  local bridge_id="$2"
  local payload
  payload="$(
    jq -cn \
      --arg bridge_kind "$kind" \
      --arg bridge_id "$bridge_id" \
      '{ bridge_kind: $bridge_kind, bridge_id: $bridge_id }'
  )"
  local body
  body="$(http_post_json "${trimmed_base}/v2/admin/operator/bridge-tokens" "$payload" "$AXIOM_OPERATOR_TOKEN")"
  local token
  token="$(echo "$body" | jq -r '.access_token // empty')"
  if [[ -z "$token" ]]; then
    echo "bridge token response missing access_token for ${kind}/${bridge_id}" >&2
    echo "$body" >&2
    exit 1
  fi
  echo "$token"
}

runtime_token="$(issue_bridge_token "runtime_bridge" "$RUNTIME_SERVER_ID")"
fleet_token="$(issue_bridge_token "fleet_bridge" "$FLEET_SERVICE_ID")"

print_export() {
  local key="$1"
  local value="$2"
  printf 'export %s=%q\n' "$key" "$value"
}

echo "# Generated Axiom operator bridge exports"
echo "# Runtime server id: ${RUNTIME_SERVER_ID}"
echo "# Fleet service id: ${FLEET_SERVICE_ID}"
print_export CAVERN_AXIOM_OPERATOR_ENABLED "1"
print_export CAVERN_AXIOM_OPERATOR_WS_URL "$RUNTIME_WS_URL"
print_export CAVERN_AXIOM_OPERATOR_RUNTIME_TOKEN "$runtime_token"
print_export GROTTO_FLEET_AXIOM_ENABLED "1"
print_export GROTTO_FLEET_AXIOM_WS_URL "$FLEET_WS_URL"
print_export GROTTO_FLEET_AXIOM_COMMAND_TOKEN "$fleet_token"
print_export GROTTO_FLEET_AXIOM_SERVICE_ID "$FLEET_SERVICE_ID"
echo
echo "# Example usage"
echo "games/cavern_hunt/scripts/run_cavern_server.sh"
echo "cargo run -p grotto_fleet_control -- --config ops/fleet/kubernetes.ron"
