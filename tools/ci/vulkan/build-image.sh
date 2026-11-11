#!/usr/bin/env bash
# Bind each build to an actual upstream digest; record it for review.
set -euo pipefail
if [[ "$#" -ne 1 || -z "$1" ]]; then
  echo "Usage: $0 <local-image-tag>" >&2
  exit 64
fi
image_tag="$1"
docker pull --platform linux/amd64 ubuntu:24.04
base_ref="$(docker image inspect --format '{{index .RepoDigests 0}}' ubuntu:24.04)"
if ! [[ "$base_ref" =~ ^ubuntu@sha256:[0-9a-f]{64}$ ]]; then
  echo "::error::Cannot resolve exact upstream Ubuntu image digest: $base_ref" >&2
  exit 1
fi
echo "Immutable Ubuntu base: $base_ref"
docker build --pull --platform linux/amd64 \
  --build-arg "UBUNTU_BASE=$base_ref" \
  --tag "$image_tag" \
  --file tools/ci/vulkan/Dockerfile tools/ci/vulkan
docker image inspect "$image_tag" --format '{{.Id}}'
