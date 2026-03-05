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
FLEET_SERVICE_ID="${FLEET_SERVICE_ID:-fleet-control-default}"

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
core_api_cluster_dns="${CORE_API_SERVICE}.${NAMESPACE}.svc.cluster.local"
runtime_ws_url="${RUNTIME_WS_URL:-ws://${core_api_cluster_dns}:3000/v2/operator/runtime/ws}"
fleet_ws_url="${FLEET_WS_URL:-ws://${core_api_cluster_dns}:3000/v2/operator/fleet/ws}"

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

bridge_exports_output="$({
  AXIOM_BASE_URL="$axiom_base_url" \
  AXIOM_OPERATOR_USERNAME="$OPERATOR_USERNAME" \
  AXIOM_OPERATOR_PASSWORD="$OPERATOR_PASSWORD" \
  RUNTIME_SERVER_ID="$RUNTIME_SERVER_ID" \
  FLEET_SERVICE_ID="$FLEET_SERVICE_ID" \
  RUNTIME_WS_URL="$runtime_ws_url" \
  FLEET_WS_URL="$fleet_ws_url" \
  "$GQ_ROOT/scripts/axiom_issue_operator_bridge_tokens.sh"
} )"

while IFS= read -r export_line; do
  eval "$export_line"
done < <(echo "$bridge_exports_output" | grep '^export ')

runtime_secret_name="${RELEASE_NAME}-runtime-bridge-env"
fleet_secret_name="${RELEASE_NAME}-fleet-bridge-env"

kubectl -n "$NAMESPACE" create secret generic "$runtime_secret_name" \
  --from-literal=CAVERN_AXIOM_OPERATOR_ENABLED="${CAVERN_AXIOM_OPERATOR_ENABLED:-1}" \
  --from-literal=CAVERN_AXIOM_OPERATOR_WS_URL="$CAVERN_AXIOM_OPERATOR_WS_URL" \
  --from-literal=CAVERN_AXIOM_OPERATOR_RUNTIME_TOKEN="$CAVERN_AXIOM_OPERATOR_RUNTIME_TOKEN" \
  --from-literal=CAVERN_AXIOM_OPERATOR_HEARTBEAT_SECONDS="${CAVERN_AXIOM_OPERATOR_HEARTBEAT_SECONDS:-10}" \
  --from-literal=CAVERN_AXIOM_OPERATOR_SNAPSHOT_INTERVAL_TICKS="${CAVERN_AXIOM_OPERATOR_SNAPSHOT_INTERVAL_TICKS:-60}" \
  --from-literal=CAVERN_AXIOM_OPERATOR_MAX_BUFFERED_EVENTS="${CAVERN_AXIOM_OPERATOR_MAX_BUFFERED_EVENTS:-256}" \
  --dry-run=client -o yaml | kubectl apply -f - >/dev/null

kubectl -n "$NAMESPACE" create secret generic "$fleet_secret_name" \
  --from-literal=GROTTO_FLEET_AXIOM_ENABLED="${GROTTO_FLEET_AXIOM_ENABLED:-1}" \
  --from-literal=GROTTO_FLEET_AXIOM_WS_URL="$GROTTO_FLEET_AXIOM_WS_URL" \
  --from-literal=GROTTO_FLEET_AXIOM_COMMAND_TOKEN="$GROTTO_FLEET_AXIOM_COMMAND_TOKEN" \
  --from-literal=GROTTO_FLEET_AXIOM_SERVICE_ID="${GROTTO_FLEET_AXIOM_SERVICE_ID:-$FLEET_SERVICE_ID}" \
  --from-literal=GROTTO_FLEET_AXIOM_HEARTBEAT_SECONDS="${GROTTO_FLEET_AXIOM_HEARTBEAT_SECONDS:-10}" \
  --from-literal=GROTTO_FLEET_AXIOM_RECONNECT_BACKOFF_MS="${GROTTO_FLEET_AXIOM_RECONNECT_BACKOFF_MS:-500}" \
  --from-literal=GROTTO_FLEET_AXIOM_MAX_BUFFERED_EVENTS="${GROTTO_FLEET_AXIOM_MAX_BUFFERED_EVENTS:-512}" \
  --dry-run=client -o yaml | kubectl apply -f - >/dev/null

kubectl -n "$NAMESPACE" rollout restart "deployment/${FLEET_DEPLOYMENT}" >/dev/null
if kubectl -n "$NAMESPACE" get deployment "$RUNTIME_DEPLOYMENT" >/dev/null 2>&1; then
  kubectl -n "$NAMESPACE" rollout restart "deployment/${RUNTIME_DEPLOYMENT}" >/dev/null
fi

wait_for_deployment_ready "$FLEET_DEPLOYMENT" "180s"

fleet_bridge_seen=0
for _ in $(seq 1 60); do
  if kubectl -n "$NAMESPACE" logs "deployment/${RELEASE_NAME}-core-api" --since=5m 2>/dev/null | grep -F "operator bridge connected: ${FLEET_SERVICE_ID}" >/dev/null 2>&1; then
    fleet_bridge_seen=1
    break
  fi
  sleep 1
done
if [[ "$fleet_bridge_seen" != "1" ]]; then
  echo "fleet bridge did not appear in core-api logs within timeout" >&2
  exit 1
fi

list_json="$(api_request GET "/v2/admin/resources?kind=runtime_node&limit=100" "" "$operator_token")"
runtime_resource_id="$(echo "$list_json" | jq -r --arg key "$RUNTIME_SERVER_ID" '.resources[]? | select(.resource_key == $key) | .id' | head -n1)"

if [[ -z "$runtime_resource_id" ]]; then
  create_payload="$(jq -cn \
    --arg resource_key "$RUNTIME_SERVER_ID" \
    --arg endpoint "${GROTTO_SERVER_SERVICE}.${NAMESPACE}.svc.cluster.local:7000" \
    '{
      resource_key: $resource_key,
      kind: "runtime_node",
      origin: "admin_managed",
      visibility: "internal",
      region: "local-dev",
      spec: {
        endpoint: $endpoint,
        capacity: { max_players: 4 },
        management_mode: "manual",
        labels: { "grotto.server_id": $resource_key }
      }
    }')"
  create_json="$(api_request POST "/v2/admin/resources" "$create_payload" "$operator_token")"
  runtime_resource_id="$(echo "$create_json" | jq -r '.id // empty')"
fi

if [[ -z "$runtime_resource_id" ]]; then
  echo "failed to resolve runtime node resource id for ${RUNTIME_SERVER_ID}" >&2
  exit 1
fi

availability_payload='{"action":"set_availability","input":{"lifecycle_state":"active"}}'
api_request POST "/v2/admin/resources/${runtime_resource_id}/actions" "$availability_payload" "$operator_token" >/dev/null

echo "Bridge bootstrap complete"
echo "Runtime server id: ${RUNTIME_SERVER_ID}"
echo "Fleet service id: ${FLEET_SERVICE_ID}"
echo "Runtime node resource id: ${runtime_resource_id}"
echo "Fleet and runtime bridge secrets were updated and fleet deployment restarted."
