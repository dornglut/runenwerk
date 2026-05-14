#!/usr/bin/env bash
set -euo pipefail

echo "The ./workflow shim is deprecated. Use Taskfile commands instead:" >&2
echo "  task --list" >&2
echo "  task roadmap:validate" >&2
echo "  task batch:propose -- --goal \"<goal>\" --scope L0" >&2
exit 2
