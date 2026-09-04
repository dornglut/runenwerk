#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
toolchain="nightly-2026-08-25"

cd "$repo_root"
rustup component list --toolchain "$toolchain" | rg -q '^miri-.* \(installed\)$'
cargo +"$toolchain" miri test -p ecs --test miri_c3 --locked -- --nocapture
