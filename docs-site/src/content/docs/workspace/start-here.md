---
title: Start Here
description: Scriptless task router for Runenwerk repository work.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-25
related_docs:
  - ./operating-model.md
  - ./authority-model.md
  - ./documentation-structure.md
  - ./routines/README.md
  - ./task-cards/README.md
  - ./planning/README.md
---

# Start Here

Use this page before non-trivial Runenwerk work.

The default workflow must work from GitHub connector, ChatGPT context tooling, Codex patching, manual repo browsing, or a local checkout. Do not require scripts, generated prompts, rendered planning views, Taskfile tasks, or a full repository export to understand what to do.

## Choose the task shape

| Work type | Use this routine | Use this task card |
|---|---|---|
| Investigation | [`routines/investigation-routine.md`](routines/investigation-routine.md) | [`task-cards/github-connector-task.md`](task-cards/github-connector-task.md) |
| Implementation | [`routines/implementation-routine.md`](routines/implementation-routine.md) | [`task-cards/implementation-task.md`](task-cards/implementation-task.md) |
| Architecture review | [`routines/architecture-governance-review-routine.md`](routines/architecture-governance-review-routine.md) | [`task-cards/architecture-review-task.md`](task-cards/architecture-review-task.md) |
| Code refactor | [`routines/code-refactor-routine.md`](routines/code-refactor-routine.md) | [`task-cards/implementation-task.md`](task-cards/implementation-task.md) |
| Documentation cleanup | [`routines/docs-refactor-routine.md`](routines/docs-refactor-routine.md) | [`task-cards/docs-cleanup-task.md`](task-cards/docs-cleanup-task.md) |
| Roadmap or planning update | [`routines/roadmap-update-routine.md`](routines/roadmap-update-routine.md) | [`task-cards/roadmap-update-task.md`](task-cards/roadmap-update-task.md) |
| Phase closeout | [`routines/phase-completion-drift-check-routine.md`](routines/phase-completion-drift-check-routine.md) | [`task-cards/phase-closeout-task.md`](task-cards/phase-closeout-task.md) |
| Pull request review | [`routines/pr-review-routine.md`](routines/pr-review-routine.md) | [`task-cards/github-connector-task.md`](task-cards/github-connector-task.md) |

## Always read these first

For code changes:

```text
AGENTS.md
AI_GUIDE.md
ARCHITECTURE.md
DEPENDENCY_RULES.md
DOMAIN_MAP.md
TESTING.md
```

For documentation changes:

```text
AGENTS.md
AI_GUIDE.md
docs-site/src/content/docs/workspace/documentation-structure.md
docs-site/src/content/docs/workspace/authority-model.md
```

For planning changes:

```text
docs-site/src/content/docs/workspace/planning/README.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/authority-model.md
```

## Default evidence report

Every task should end with:

```text
Files changed:
Exact functions/modules/sections changed:
Authority files inspected:
Manual validation performed:
Local command validation run, or why it was not run:
Remaining risks or blockers:
Next recommended step:
```

## Optional local helpers

Commands and scripts may provide extra evidence when a local checkout is available. They are never required to understand this workflow and they are never the authority for what should be changed.
