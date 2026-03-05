#!/usr/bin/env zsh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=./common.sh
source "${SCRIPT_DIR}/common.sh"

require_cmd docker
require_cmd kubectl
require_cmd helm
require_cmd curl
require_cmd jq

provider="$(resolve_provider)"
ensure_cluster "$provider"

CORE_API_REPOSITORY="${CORE_API_REPOSITORY:-local/core-api}"
CORE_API_TAG="${CORE_API_TAG:-dev}"
OPERATOR_CONSOLE_REPOSITORY="${OPERATOR_CONSOLE_REPOSITORY:-local/operator-console}"
OPERATOR_CONSOLE_TAG="${OPERATOR_CONSOLE_TAG:-dev}"
GROTTO_SERVER_REPOSITORY="${GROTTO_SERVER_REPOSITORY:-local/grotto-server}"
GROTTO_SERVER_TAG="${GROTTO_SERVER_TAG:-dev}"
GROTTO_FLEET_REPOSITORY="${GROTTO_FLEET_REPOSITORY:-local/grotto-fleet-control}"
GROTTO_FLEET_TAG="${GROTTO_FLEET_TAG:-dev}"

CORE_API_IMAGE="${CORE_API_REPOSITORY}:${CORE_API_TAG}"
OPERATOR_CONSOLE_IMAGE="${OPERATOR_CONSOLE_REPOSITORY}:${OPERATOR_CONSOLE_TAG}"
GROTTO_SERVER_IMAGE="${GROTTO_SERVER_REPOSITORY}:${GROTTO_SERVER_TAG}"
GROTTO_FLEET_IMAGE="${GROTTO_FLEET_REPOSITORY}:${GROTTO_FLEET_TAG}"

if [[ "${SKIP_IMAGE_BUILD:-0}" != "1" ]]; then
  echo "Building container images..."
  docker build -t "$CORE_API_IMAGE" -f "${AXIOM_ROOT}/Dockerfile" "${AXIOM_ROOT}"
  docker build -t "$OPERATOR_CONSOLE_IMAGE" -f "${AXIOM_ROOT}/operator-console/Dockerfile" "${AXIOM_ROOT}"
  docker build -t "$GROTTO_SERVER_IMAGE" -f "${GQ_ROOT}/ops/docker/grotto_server.Dockerfile" "${GQ_ROOT}"
  docker build -t "$GROTTO_FLEET_IMAGE" -f "${GQ_ROOT}/ops/docker/grotto_fleet_control.Dockerfile" "${GQ_ROOT}"
else
  echo "Skipping image build because SKIP_IMAGE_BUILD=1"
fi

echo "Importing images into ${provider} cluster ${CLUSTER_NAME}..."
import_image_into_cluster "$provider" "$CORE_API_IMAGE"
import_image_into_cluster "$provider" "$OPERATOR_CONSOLE_IMAGE"
import_image_into_cluster "$provider" "$GROTTO_SERVER_IMAGE"
import_image_into_cluster "$provider" "$GROTTO_FLEET_IMAGE"

kubectl create namespace "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f - >/dev/null

core_api_cluster_dns="${CORE_API_SERVICE}.${NAMESPACE}.svc.cluster.local"
runtime_ws_url="ws://${core_api_cluster_dns}:3000/v2/operator/runtime/ws"
fleet_ws_url="ws://${core_api_cluster_dns}:3000/v2/operator/fleet/ws"

chart_dir="${GQ_ROOT}/ops/helm/multiplayer-stack"

echo "Installing/upgrading Helm release ${RELEASE_NAME} in namespace ${NAMESPACE}..."
helm upgrade --install "$RELEASE_NAME" "$chart_dir" \
  --namespace "$NAMESPACE" \
  --create-namespace \
  --wait \
  --timeout "${HELM_TIMEOUT:-10m}" \
  --set-string namespace="$NAMESPACE" \
  --set-string images.coreApi.repository="$CORE_API_REPOSITORY" \
  --set-string images.coreApi.tag="$CORE_API_TAG" \
  --set-string images.operatorConsole.repository="$OPERATOR_CONSOLE_REPOSITORY" \
  --set-string images.operatorConsole.tag="$OPERATOR_CONSOLE_TAG" \
  --set-string images.grottoServer.repository="$GROTTO_SERVER_REPOSITORY" \
  --set-string images.grottoServer.tag="$GROTTO_SERVER_TAG" \
  --set-string images.grottoFleetControl.repository="$GROTTO_FLEET_REPOSITORY" \
  --set-string images.grottoFleetControl.tag="$GROTTO_FLEET_TAG" \
  --set-string bridge.runtime.wsUrl="$runtime_ws_url" \
  --set-string bridge.fleet.wsUrl="$fleet_ws_url" \
  --set-string grottoServer.serverId="$RUNTIME_SERVER_ID" \
  --set-string grottoServer.serverName="$RUNTIME_SERVER_ID" \
  --set-string fleetControl.allowedServerIds[0]="$RUNTIME_SERVER_ID"

wait_for_deployment_ready "${RELEASE_NAME}-postgres" "180s"
wait_for_deployment_ready "${RELEASE_NAME}-redis" "180s"
wait_for_deployment_ready "${RELEASE_NAME}-core-api" "180s"
wait_for_deployment_ready "${RELEASE_NAME}-operator-console" "180s"
wait_for_deployment_ready "${RELEASE_NAME}-fleet-control" "180s"
if kubectl -n "$NAMESPACE" get deployment "${RELEASE_NAME}-caddy" >/dev/null 2>&1; then
  wait_for_deployment_ready "${RELEASE_NAME}-caddy" "180s"
fi

echo
printf 'Bootstrap complete.\n'
printf 'Namespace: %s\n' "$NAMESPACE"
printf 'Release: %s\n' "$RELEASE_NAME"
printf 'Runtime deployment: %s (replicas=0 until operator start command)\n' "$RUNTIME_DEPLOYMENT"

echo
echo "Next steps:"
echo "1) Bootstrap operator bridge tokens and runtime-node registration"
echo "   scripts/k8s/bootstrap_operator_bridges.sh"
echo "2) Run smoke test"
echo "   scripts/k8s/smoke_operator_flow.sh"
echo "3) Open operator console via same-origin gateway"
echo "   kubectl -n ${NAMESPACE} port-forward svc/${RELEASE_NAME}-caddy 8081:80"
echo "   then open http://127.0.0.1:8081"
