#!/usr/bin/env bash
set -euo pipefail

# File: tools/validation/quiet_full_gate.sh
# Purpose:
#   Run the long-term workspace gate without successful-test spam.
#
# Output policy:
#   - clippy: show warnings/errors in short form
#   - tests: hide successful test binaries and successful test names
#   - failures: print the full failing command output
#
# Run from the Runenwerk repository root.

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
    # Show warnings/errors if any command emitted them despite success.
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

# Fast format check; normally silent.
run_quiet "fmt" cargo fmt --all -- --check

# Short compiler diagnostics, no long cargo chatter.
run_quiet "clippy" cargo clippy \
  --workspace \
  --all-targets \
  --all-features \
  --message-format=short \
  -- \
  -D warnings

# Quiet test harness. On failure, the captured full output is printed.
run_quiet "test" cargo test \
  --workspace \
  --all-features \
  --quiet

echo
echo "quiet full gate passed"
