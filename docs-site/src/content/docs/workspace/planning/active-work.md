---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-07
related_docs:
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../complete-merge-readiness-gate.md
  - ../evidence-quality-taxonomy.md
  - ../routines/track-orchestration-routine.md
  - ../specs/phase-implementation-spec.md
  - ../../guidelines/programming-principles.md
  - ../../architecture/ui-framework-architecture.md
  - ../../architecture/live-uiplugin-runtime-platform-architecture.md
  - ../../design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
  - ../../design/active/live-uiplugin-runtime-full-cutover-plan.md
  - ../../reports/investigations/live-uiplugin-runtime-current-state-investigation.md
  - ../../reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
  - ./roadmap.md
  - ./production-tracks.md
  - ./completed-work.md
  - ./decision-register.md
---

# Active Work

This file names the current planning focus for scriptless workflow. It stays short and points to the owning investigation/design records instead of duplicating them.

## Current focus

ID: `PT-WORKFLOW-TRACK-ORCHESTRATION-001`

Title: `Track Orchestration and Phase Spec Handoff Workflow`

State: docs-only workflow hardening PR in progress before runtime implementation.

Lifecycle state: `active-planning` workflow-hardening contract draft. Not `active-implementation` for runtime code.

Owner: Workspace workflow owns the orchestration routine, task card, phase-spec handoff docs, and planning-state updates. Runtime UI/platform implementation remains owned by the future `PT-UI-RUNTIME-PLATFORM-003` phase and is not authorized by this work.

Authority files:

```text
AGENTS.md
ARCHITECTURE.md
DOMAIN_MAP.md
TESTING.md
docs-site/src/content/docs/workspace/start-here.md
docs-site/src/content/docs/workspace/operating-model.md
docs-site/src/content/docs/workspace/authority-model.md
docs-site/src/content/docs/workspace/workflow-lifecycle.md
docs-site/src/content/docs/workspace/complete-investigation-gate.md
docs-site/src/content/docs/workspace/complete-design-gate.md
docs-site/src/content/docs/workspace/evidence-quality-taxonomy.md
docs-site/src/content/docs/workspace/complete-merge-readiness-gate.md
docs-site/src/content/docs/workspace/routines/implementation-routine.md
docs-site/src/content/docs/workspace/routines/pr-review-routine.md
docs-site/src/content/docs/workspace/routines/phase-completion-drift-check-routine.md
docs-site/src/content/docs/workspace/routines/roadmap-update-routine.md
docs-site/src/content/docs/workspace/planning/README.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
docs-site/src/content/docs/workspace/planning/completed-work.md
docs-site/src/content/docs/workspace/planning/decision-register.md
docs-site/src/content/docs/design/active/live-uiplugin-runtime-full-cutover-plan.md
docs-site/src/content/docs/architecture/live-uiplugin-runtime-platform-architecture.md
```

Evidence classes: `E2` connector metadata/file inspection, `E3` source/design/planning inspection by path, and `E8` accepted architecture/workflow/planning authority. No `E5` local command validation is available from this connector-only planning session until a local agent runs the docs validation commands.

Complete investigation gate: satisfied by existing workflow authority inspection for a docs-only process hardening PR. The current gap is known: the repo has routines for implementation, PR review, roadmap updates, and phase completion, but no explicit track-manager routine that formalizes one production-track goal executed through multiple bounded phase PRs.

Complete design gate: applies because this changes workflow authority. The design contract is bounded to docs-only workflow hardening: add track orchestration routine, track manager task card, phase implementation spec docs/template, and authority/planning links. No validator, script, runtime implementation, or phase implementation is authorized.

Implementation contract: docs-only workflow hardening. Add or update only workspace routine/task/spec/planning files named below. Runtime Rust implementation remains blocked until this workflow PR merges and `PT-UI-RUNTIME-PLATFORM-003 — UiPlugin Foundation` is separately opened with exact active-implementation authorization.

Allowed files/crates for current focus:

```text
docs-site/src/content/docs/workspace/routines/track-orchestration-routine.md
docs-site/src/content/docs/workspace/task-cards/track-manager-task.md
docs-site/src/content/docs/workspace/specs/README.md
docs-site/src/content/docs/workspace/specs/phase-implementation-spec.md
docs-site/src/content/docs/workspace/specs/templates/phase-implementation-spec.ron
docs-site/src/content/docs/workspace/start-here.md
docs-site/src/content/docs/workspace/routines/README.md
docs-site/src/content/docs/workspace/task-cards/README.md
docs-site/src/content/docs/workspace/authority-model.md
docs-site/src/content/docs/workspace/planning/README.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
docs-site/src/content/docs/workspace/planning/completed-work.md
docs-site/src/content/docs/workspace/planning/decision-register.md
```

Non-owned files/crates for current focus:

```text
runtime Rust implementation
engine UiPlugin code
public AppUiExt code
app.mount_ui implementation
UiScreen / IntoUi implementation
UiActionHandler implementation
render adapter code
SurfaceFrame generic producer boundary implementation code
scene/debug overlay producer migration implementation code
source reload/persistence implementation code
apps/ui_counter_runtime implementation
SDF/world-space/SpatialCanvas implementation
foundation/meta
domain/app_program
generic plugin framework
phase spec validator implementation
any tools/docs validator or script changes
```

Known blockers: `PT-UI-RUNTIME-PLATFORM-003` implementation cannot start until this workflow-hardening PR is reviewed/merged or explicitly deferred, and then Phase 003 is opened separately as active implementation with exact owner files, implementation contract, validation envelope, evidence expectation, and stop conditions.

Next action: review this docs-only workflow-hardening PR. If accepted, merge it and then open `PT-UI-RUNTIME-PLATFORM-003 — UiPlugin Foundation` using the new track orchestration routine and phase spec handoff contract.

## Active-work rules

- One current focus is preferred.
- If no current focus exists, say that explicitly.
- Do not promote deferred work without recording the reason.
- Do not mark work completed without evidence.
- If generated views disagree, report them as stale mirrors.
- Use `../workflow-lifecycle.md` before changing active work state.
- Architecture acceptance does not authorize implementation. Use `active-implementation` only when exact scope, owner, validation, evidence expectation, stop conditions, principle compliance status, and module decomposition status are known.

## Update shape

```text
ID:
Title:
State:
Lifecycle state:
Owner:
Authority files:
Evidence classes:
Complete investigation gate:
Complete design gate:
Implementation contract:
Allowed files/crates:
Non-owned files/crates:
Known blockers:
Next action:
```
