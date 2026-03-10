#!/usr/bin/env zsh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=./common.sh
source "${SCRIPT_DIR}/common.sh"

require_cmd kubectl
require_cmd curl
require_cmd jq

provider="$(resolve_provider)"
use_cluster_context "$provider"

API_LOCAL_PORT="${API_LOCAL_PORT:-18080}"
OPERATOR_USERNAME="${OPERATOR_USERNAME:-axiom-operator}"
OPERATOR_PASSWORD="${OPERATOR_PASSWORD:-operator-password}"

api_pf_log="$(mktemp -t axiom-api-port-forward.XXXXXX.log)"
api_pf_pid="$(start_port_forward "$CORE_API_SERVICE" "$API_LOCAL_PORT" "3000" "$api_pf_log")"
cleanup() {
  if kill -0 "$api_pf_pid" >/dev/null 2>&1; then
    kill "$api_pf_pid" >/dev/null 2>&1 || true
  fi
  rm -f "$api_pf_log"
}
trap cleanup EXIT INT TERM

wait_for_http_ok "http://127.0.0.1:${API_LOCAL_PORT}/health" 60 1
axiom_base_url="http://127.0.0.1:${API_LOCAL_PORT}"

api_request() {
  local method="$1"
  local path="$2"
  local body="${3:-}"
  local token="${4:-}"

  local tmp_body
  tmp_body="$(mktemp -t axiom-api-body.XXXXXX.json)"

  local -a curl_args
  curl_args=(-sS -o "$tmp_body" -w "%{http_code}" -X "$method" "${axiom_base_url}${path}")
  if [[ -n "$token" ]]; then
    curl_args+=(-H "authorization: Bearer ${token}")
  fi
  curl_args+=(-H "content-type: application/json")
  if [[ -n "$body" ]]; then
    curl_args+=(-d "$body")
  fi

  local status
  status="$(curl "${curl_args[@]}")"

  if [[ "$status" -lt 200 || "$status" -ge 300 ]]; then
    echo "API request failed: ${method} ${path} (HTTP ${status})" >&2
    cat "$tmp_body" >&2
    rm -f "$tmp_body"
    exit 1
  fi

  cat "$tmp_body"
  rm -f "$tmp_body"
}

login_payload="$(jq -cn --arg username "$OPERATOR_USERNAME" --arg password "$OPERATOR_PASSWORD" '{ username: $username, password: $password }')"
login_json="$(api_request POST "/v2/auth/operator/login" "$login_payload")"
operator_token="$(echo "$login_json" | jq -r '.access_token // empty')"
if [[ -z "$operator_token" ]]; then
  echo "failed to acquire operator access token" >&2
  exit 1
fi

list_json="$(api_request GET "/v2/admin/resources?kind=runtime_node&limit=100" "" "$operator_token")"
runtime_resource_id="$(echo "$list_json" | jq -r --arg key "$RUNTIME_SERVER_ID" '.resources[]? | select(.resource_key == $key) | .id' | head -n1)"
if [[ -z "$runtime_resource_id" ]]; then
  echo "runtime node ${RUNTIME_SERVER_ID} not found; run games/cavern_hunt/scripts/k8s/bootstrap_operator_bridges.sh first" >&2
  exit 1
fi

start_payload='{"op":"start_server","profile":null}'
start_json="$(api_request POST "/v2/admin/runtime-nodes/${runtime_resource_id}/operator-commands" "$start_payload" "$operator_token")"
start_status="$(echo "$start_json" | jq -r '.status // empty')"
if [[ "$start_status" != "accepted" ]]; then
  echo "start_server command was not accepted" >&2
  echo "$start_json" >&2
  exit 1
fi

for _ in $(seq 1 120); do
  replicas="$(kubectl -n "$NAMESPACE" get deployment "$RUNTIME_DEPLOYMENT" -o jsonpath='{.spec.replicas}' 2>/dev/null || echo 0)"
  available="$(kubectl -n "$NAMESPACE" get deployment "$RUNTIME_DEPLOYMENT" -o jsonpath='{.status.availableReplicas}' 2>/dev/null || echo 0)"
  if [[ "$replicas" == "1" && "$available" == "1" ]]; then
    break
  fi
  sleep 1
done

replicas="$(kubectl -n "$NAMESPACE" get deployment "$RUNTIME_DEPLOYMENT" -o jsonpath='{.spec.replicas}')"
available="$(kubectl -n "$NAMESPACE" get deployment "$RUNTIME_DEPLOYMENT" -o jsonpath='{.status.availableReplicas}')"
if [[ "$replicas" != "1" || "$available" != "1" ]]; then
  echo "runtime deployment did not scale to 1 available replica" >&2
  exit 1
fi

inspect_payload='{"op":"inspect_logs","query":{"limit":50,"cursor":null,"from_ts_ms":null,"to_ts_ms":null,"level":null}}'
inspect_json="$(api_request POST "/v2/admin/runtime-nodes/${runtime_resource_id}/operator-commands" "$inspect_payload" "$operator_token")"
inspect_status="$(echo "$inspect_json" | jq -r '.status // empty')"
if [[ "$inspect_status" != "accepted" ]]; then
  echo "inspect_logs command was not accepted" >&2
  echo "$inspect_json" >&2
  exit 1
fi
if ! echo "$inspect_json" | jq -e '.payload.page.lines | arrays' >/dev/null; then
  echo "inspect_logs payload.page.lines is missing or not an array" >&2
  echo "$inspect_json" >&2
  exit 1
fi

stop_payload='{"op":"stop_server","graceful_timeout_ms":8000}'
stop_json="$(api_request POST "/v2/admin/runtime-nodes/${runtime_resource_id}/operator-commands" "$stop_payload" "$operator_token")"
stop_status="$(echo "$stop_json" | jq -r '.status // empty')"
if [[ "$stop_status" != "accepted" ]]; then
  echo "stop_server command was not accepted" >&2
  echo "$stop_json" >&2
  exit 1
fi

for _ in $(seq 1 120); do
  replicas="$(kubectl -n "$NAMESPACE" get deployment "$RUNTIME_DEPLOYMENT" -o jsonpath='{.spec.replicas}' 2>/dev/null || echo 0)"
  available="$(kubectl -n "$NAMESPACE" get deployment "$RUNTIME_DEPLOYMENT" -o jsonpath='{.status.availableReplicas}' 2>/dev/null || echo 0)"
  if [[ "$replicas" == "0" && ( -z "$available" || "$available" == "0" ) ]]; then
    break
  fi
  sleep 1
done

replicas="$(kubectl -n "$NAMESPACE" get deployment "$RUNTIME_DEPLOYMENT" -o jsonpath='{.spec.replicas}')"
available="$(kubectl -n "$NAMESPACE" get deployment "$RUNTIME_DEPLOYMENT" -o jsonpath='{.status.availableReplicas}' || true)"
if [[ "$replicas" != "0" ]]; then
  echo "runtime deployment did not scale down to zero" >&2
  exit 1
fi

if ! kubectl -n "$NAMESPACE" logs "deployment/${RELEASE_NAME}-core-api" --since=10m | grep -F "operator bridge connected: ${RUNTIME_SERVER_ID}" >/dev/null 2>&1; then
  echo "runtime bridge connection log entry not found in core-api logs" >&2
  exit 1
fi

if ! kubectl -n "$NAMESPACE" logs "deployment/${RELEASE_NAME}-core-api" --since=10m | grep -F "operator bridge connected: ${FLEET_SERVICE_ID:-fleet-control-default}" >/dev/null 2>&1; then
  echo "fleet bridge connection log entry not found in core-api logs" >&2
  exit 1
fi

echo "Smoke test passed: start_server, inspect_logs, stop_server verified for ${RUNTIME_SERVER_ID}."
