---
title: Operating Model
description: Scriptless-by-default operating model for Runenwerk repository work.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-25
related_docs:
  - ./start-here.md
  - ./authority-model.md
  - ./documentation-structure.md
  - ./routines/README.md
  - ./task-cards/README.md
---

# Operating Model

Runenwerk repository work is Markdown-first and file-inspection-first.

The workflow must remain usable from:

```text
GitHub connector
ChatGPT context tooling
Codex patch agents
manual repository browsing
local checkout
```

A local checkout is useful, but it is not assumed.

## Core principle

Every workflow must be completable by reading and editing repository files.

Scripts, Taskfile tasks, renderers, validators, prompt generators, and shell commands are optional evidence helpers. They are not the default workflow, not the planning authority, and not required to understand the next action.

## Default workflow

```text
1. Open start-here.md.
2. Select the matching routine.
3. Read the routine's authority files.
4. Inspect the exact working files.
5. Decide the smallest coherent patch.
6. Apply the patch file-by-file.
7. Run manual validation from the routine.
8. Report command validation as run, skipped, or unavailable.
9. List changed files, exact sections/modules, risks, and next steps.
```

## Authority rules

- Code and tests prove current behavior.
- ADRs, accepted designs, guidelines, and root architecture docs own durable policy.
- Workspace authority docs own repository process.
- Planning Markdown owns active planning state.
- Routines own repeatable human/agent procedure.
- Task cards are reusable prompts; they do not own process.
- Reports and closeouts own historical evidence.
- Scripts are optional local helpers only.

Use [`authority-model.md`](authority-model.md) when these layers conflict.

## Connector mode

When working through a connector or context tool:

- Do not assume a clean worktree.
- Do not assume hidden files were inspected.
- Do not assume generated files are fresh.
- Do not claim command validation.
- Cite or name every authority file used.
- Patch exact files only.
- Report missing evidence explicitly.

## Local checkout mode

When a local checkout is available, commands can add evidence:

```text
cargo fmt --all -- --check
cargo test -p <crate>
cargo test --workspace
python3 tools/docs/validate_docs.py
```

These commands do not replace manual authority review. If command output and authority docs disagree, inspect the source of the disagreement instead of treating the command as policy.

## Generated files

Generated or machine-readable files can be useful mirrors. They must not be required for normal workflow comprehension.

If a generated view is stale or cannot be regenerated, continue from the Markdown planning record and report the generated-view gap.

## Final-report rule

Never finish with only “done.” A final report must include:

```text
Files changed:
Exact functions/modules/sections changed:
Authority files inspected:
Manual validation:
Local command validation:
Known gaps:
Next step:
```
