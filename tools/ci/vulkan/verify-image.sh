#!/usr/bin/env bash
# Both PR and trusted-release paths run this image validation.
set -euo pipefail
if [[ "$#" -ne 1 || -z "$1" ]]; then
  echo "Usage: $0 <image-reference>" >&2
  exit 64
fi
image="$1"
test -n "${GITHUB_WORKSPACE:-}"
test -d "$GITHUB_WORKSPACE/engine"
docker run --rm --shm-size=256m "$image" \
  /usr/local/bin/runenwerk-image-smoke
# Run an actual Runenwerk native/rendering smoke under Lavapipe and Xvfb.
docker run --rm --shm-size=512m \
  --mount "type=bind,src=$GITHUB_WORKSPACE,dst=/workspace" \
  --workdir /workspace \
  --env RUNENWERK_NATIVE_RENDER_HOST_SMOKE=1 \
  --env RUST_BACKTRACE=1 \
  "$image" bash -lc \
    'set -euo pipefail; git config --global --add safe.directory /workspace; timeout 1500s xvfb-run --auto-servernum --server-args="-screen 0 1280x720x24" cargo +stable test -p engine --test native_host_smoke --locked'
git diff --exit-code
git diff --cached --exit-code
