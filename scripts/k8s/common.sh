#!/usr/bin/env zsh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
GQ_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
AXIOM_ROOT="${AXIOM_ROOT:-$(cd "${GQ_ROOT}/../Axiom" && pwd)}"

NAMESPACE="${NAMESPACE:-multiplayer-dev}"
RELEASE_NAME="${RELEASE_NAME:-multiplayer-stack}"
CLUSTER_NAME="${CLUSTER_NAME:-multiplayer-dev}"
K8S_PROVIDER="${K8S_PROVIDER:-auto}"

CORE_API_SERVICE="${RELEASE_NAME}-core-api"
OPERATOR_CONSOLE_SERVICE="${RELEASE_NAME}-operator-console"
FLEET_DEPLOYMENT="${RELEASE_NAME}-fleet-control"
CADDY_SERVICE="${RELEASE_NAME}-caddy"
GROTTO_SERVER_SERVICE="${RELEASE_NAME}-grotto-server"

RUNTIME_SERVER_ID="${RUNTIME_SERVER_ID:-srv-local}"
RUNTIME_DEPLOYMENT="grotto-server-${RUNTIME_SERVER_ID//_/-}"

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "missing required command: $cmd" >&2
    exit 1
  fi
}

resolve_provider() {
  if [[ "$K8S_PROVIDER" == "kind" || "$K8S_PROVIDER" == "k3d" ]]; then
    echo "$K8S_PROVIDER"
    return
  fi

  if command -v kind >/dev/null 2>&1; then
    echo "kind"
    return
  fi
  if command -v k3d >/dev/null 2>&1; then
    echo "k3d"
    return
  fi

  echo "unable to auto-detect Kubernetes provider; install kind or k3d, or set K8S_PROVIDER" >&2
  exit 1
}

use_cluster_context() {
  local provider="$1"
  local context
  if [[ "$provider" == "kind" ]]; then
    context="kind-${CLUSTER_NAME}"
  else
    context="k3d-${CLUSTER_NAME}"
  fi
  kubectl config use-context "$context" >/dev/null
}

ensure_cluster() {
  local provider="$1"

  if [[ "$provider" == "kind" ]]; then
    if kind get clusters | grep -x "$CLUSTER_NAME" >/dev/null 2>&1; then
      echo "kind cluster ${CLUSTER_NAME} already exists"
    else
      kind create cluster --name "$CLUSTER_NAME"
    fi
  else
    if k3d cluster list --no-headers 2>/dev/null | awk '{print $1}' | grep -x "$CLUSTER_NAME" >/dev/null 2>&1; then
      echo "k3d cluster ${CLUSTER_NAME} already exists"
    else
      k3d cluster create "$CLUSTER_NAME"
    fi
  fi

  use_cluster_context "$provider"
}

import_image_into_cluster() {
  local provider="$1"
  local image="$2"

  if [[ "$provider" == "kind" ]]; then
    kind load docker-image "$image" --name "$CLUSTER_NAME"
  else
    k3d image import "$image" -c "$CLUSTER_NAME"
  fi
}

wait_for_deployment_ready() {
  local deployment_name="$1"
  local timeout="${2:-180s}"
  kubectl -n "$NAMESPACE" rollout status "deployment/${deployment_name}" --timeout="$timeout"
}

start_port_forward() {
  local service_name="$1"
  local local_port="$2"
  local remote_port="$3"
  local log_file="$4"

  kubectl -n "$NAMESPACE" port-forward "svc/${service_name}" "${local_port}:${remote_port}" >"$log_file" 2>&1 &
  echo $!
}

wait_for_http_ok() {
  local url="$1"
  local attempts="${2:-60}"
  local delay_seconds="${3:-1}"

  for _ in $(seq 1 "$attempts"); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      return 0
    fi
    sleep "$delay_seconds"
  done

  echo "timed out waiting for ${url}" >&2
  return 1
}
