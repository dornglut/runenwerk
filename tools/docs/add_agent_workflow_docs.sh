#!/usr/bin/env bash
set -euo pipefail

# File: tools/docs/add_agent_workflow_docs.sh
# Purpose:
#   Historical compatibility entrypoint for old agent workflow doc generation.
#
# Workflow docs are now maintained as normal docs-site sources. This script is
# intentionally non-mutating so it cannot overwrite newer prompt templates,
# routines, or planning workflow docs with stale generated content.

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cat <<'MSG'
agent workflow docs are maintained directly under:

  docs-site/src/content/docs/workspace/planning-and-implementation-workflow.md
  docs-site/src/content/docs/workspace/prompt-templates/
  docs-site/src/content/docs/workspace/routines/

Use the kickoff helper for task-specific workflow guidance:

  ./workflow list
  ./workflow implementation --task "<task>" --scope "<scope>"

Validating docs now.
MSG

(cd "$repo_root" && python3 tools/docs/validate_docs.py)
