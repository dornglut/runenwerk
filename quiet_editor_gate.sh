#!/usr/bin/env bash
set -euo pipefail

# File: quiet_editor_gate.sh
# Purpose:
#   Run the fast editor/ECS validation slice used during active editor work.
#
# This gate is additive. Milestone and broad closeout validation still uses
# ./quiet_full_gate.sh.

if [[ ! -f "Cargo.toml" ]]; then
  echo "error: run this script from the Runenwerk repository root" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

run_quiet() {
  local name="$1"
  shift

  local stdout_file="$tmp_dir/${name}.stdout"
  local stderr_file="$tmp_dir/${name}.stderr"

  echo "==> $name"

  if "$@" >"$stdout_file" 2>"$stderr_file"; then
    if grep -E "^(warning|error)(\\[|:)" "$stderr_file" >/dev/null 2>&1; then
      grep -E "^(warning|error)(\\[|:)" "$stderr_file"
    else
      echo "ok"
    fi
    return 0
  fi

  echo "FAILED: $*" >&2
  echo >&2

  if [[ -s "$stderr_file" ]]; then
    echo "--- stderr ---" >&2
    cat "$stderr_file" >&2
  fi

  if [[ -s "$stdout_file" ]]; then
    echo "--- stdout ---" >&2
    cat "$stdout_file" >&2
  fi

  return 1
}

packages=(
  ecs
  ecs_macros
  editor_inspector
  editor_scene
  editor_shell
  runenwerk_editor
)

package_args=()
for package in "${packages[@]}"; do
  package_args+=("-p" "$package")
done

run_quiet "fmt" cargo fmt --all -- --check
run_quiet "docs" python3 tools/docs/validate_docs.py
run_quiet "clippy" cargo clippy \
  "${package_args[@]}" \
  --all-targets \
  --all-features \
  --message-format=short \
  -- \
  -D warnings

if cargo nextest --version >/dev/null 2>&1; then
  run_quiet "test" cargo nextest run \
    "${package_args[@]}" \
    --all-features
else
  run_quiet "test" cargo test \
    "${package_args[@]}" \
    --all-features \
    --quiet
fi

echo
echo "quiet editor gate passed"
