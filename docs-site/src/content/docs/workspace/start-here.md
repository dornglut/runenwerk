---
title: Start Here
description: Single workspace router for scriptless Runenwerk work.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ./operating-model.md
  - ./authority-model.md
  - ./documentation-structure.md
  - ./workflow-lifecycle.md
  - ./complete-investigation-gate.md
  - ./complete-design-gate.md
  - ./evidence-quality-taxonomy.md
  - ./complete-merge-readiness-gate.md
  - ./routines/README.md
  - ./task-cards/README.md
  - ./planning/README.md
---

# Start Here

Use this page for non-trivial Runenwerk work.

The workflow must work from GitHub connector, ChatGPT context tooling, Codex-style patching, manual repo browsing, or a local checkout. Do not require scripts, generated prompts, rendered planning views, Taskfile tasks, or a full repository export to know what to do next.

## Entry points

- Human entrypoint: `README.md`
- AI agent entrypoint: `AGENTS.md`
- Workspace router: this file
- Complete investigation gate: [`complete-investigation-gate.md`](complete-investigation-gate.md)
- Complete design gate: [`complete-design-gate.md`](complete-design-gate.md)
- Evidence quality taxonomy: [`evidence-quality-taxonomy.md`](evidence-quality-taxonomy.md)
- Complete merge readiness gate: [`complete-merge-readiness-gate.md`](complete-merge-readiness-gate.md)

## Choose the task shape

| Work type | Routine | Task card |
|---|---|---|
| Investigation | [`routines/investigation-routine.md`](routines/investigation-routine.md) | [`task-cards/github-connector-task.md`](task-cards/github-connector-task.md) |
| Implementation | [`routines/implementation-routine.md`](routines/implementation-routine.md) | [`task-cards/implementation-task.md`](task-cards/implementation-task.md) |
| Architecture review | [`routines/architecture-governance-review-routine.md`](routines/architecture-governance-review-routine.md) | [`task-cards/review-task.md`](task-cards/review-task.md) |
| Code refactor | [`routines/code-refactor-routine.md`](routines/code-refactor-routine.md) | [`task-cards/implementation-task.md`](task-cards/implementation-task.md) |
| Documentation cleanup | [`routines/docs-refactor-routine.md`](routines/docs-refactor-routine.md) | [`task-cards/docs-cleanup-task.md`](task-cards/docs-cleanup-task.md) |
| Roadmap or planning update | [`routines/roadmap-update-routine.md`](routines/roadmap-update-routine.md) | [`task-cards/implementation-task.md`](task-cards/implementation-task.md) |
| Phase closeout | [`routines/phase-completion-drift-check-routine.md`](routines/phase-completion-drift-check-routine.md) | [`task-cards/review-task.md`](task-cards/review-task.md) |
| Pull request review / merge readiness | [`routines/pr-review-routine.md`](routines/pr-review-routine.md) | [`task-cards/review-task.md`](task-cards/review-task.md) |

## Lifecycle rule

For non-trivial work, classify the current lifecycle state before editing:

```text
idea
investigating
proposed-design
accepted-direction
track-candidate
production-track
active-planning
active-implementation
review
completed
deferred
rejected
superseded
archived
```

Use [`workflow-lifecycle.md`](workflow-lifecycle.md) when a task crosses from design to decision, planning, implementation, review, merge readiness, or closeout.

Use [`complete-investigation-gate.md`](complete-investigation-gate.md) before design/planning/implementation decisions when current reality, ownership, authority, alternatives, evidence, or confidence is not already proven.

Use [`complete-design-gate.md`](complete-design-gate.md) before implementation is authorized for architecture-sensitive, reusable, platform, public API, production-track, workflow, or domain-boundary work.

Use [`evidence-quality-taxonomy.md`](evidence-quality-taxonomy.md) whenever a decision depends on validation, current behavior, authority, confidence, or freshness claims.

Use [`complete-merge-readiness-gate.md`](complete-merge-readiness-gate.md) before recommending a merge, branch cleanup, or phase merge.

## Read first

For code changes:

```text
AGENTS.md
ARCHITECTURE.md
DEPENDENCY_RULES.md
DOMAIN_MAP.md
TESTING.md
```

For documentation changes:

```text
AGENTS.md
docs-site/src/content/docs/workspace/documentation-structure.md
docs-site/src/content/docs/workspace/authority-model.md
docs-site/src/content/docs/workspace/workflow-lifecycle.md
docs-site/src/content/docs/workspace/complete-investigation-gate.md
docs-site/src/content/docs/workspace/complete-design-gate.md
docs-site/src/content/docs/workspace/evidence-quality-taxonomy.md
docs-site/src/content/docs/workspace/complete-merge-readiness-gate.md
```

For planning changes:

```text
docs-site/src/content/docs/workspace/planning/README.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/authority-model.md
docs-site/src/content/docs/workspace/workflow-lifecycle.md
docs-site/src/content/docs/workspace/complete-investigation-gate.md
docs-site/src/content/docs/workspace/complete-design-gate.md
docs-site/src/content/docs/workspace/evidence-quality-taxonomy.md
docs-site/src/content/docs/workspace/complete-merge-readiness-gate.md
```

For all significant changes, use:

```text
docs-site/src/content/docs/guidelines/programming-principles.md
```

## Evidence report

End with:

```text
Files changed:
Exact functions/modules/sections changed:
Authority files inspected:
Evidence classes used:
Complete investigation gate status:
Complete design gate status:
Merge readiness status when relevant:
Manual validation performed:
Command validation run or unavailable:
Remaining risks or blockers:
Next recommended step:
```

## Optional local helpers

Commands and scripts may provide extra evidence when a local checkout is available. They are never required to understand this workflow and they are never the authority for what should be changed.
