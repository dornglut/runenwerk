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
  - ../../reports/closeouts/pt-ui-runtime-platform-003-closeout.md
  - ../../reports/closeouts/pt-ui-runtime-platform-004-closeout.md
  - ../../reports/closeouts/pt-ui-runtime-platform-005-closeout.md
  - ../../reports/closeouts/pt-ui-runtime-platform-006-closeout.md
  - ../../reports/closeouts/pt-ui-runtime-platform-007-closeout.md
  - ../../reports/closeouts/pt-ui-runtime-platform-008-closeout.md
  - ../../reports/closeouts/pt-ui-runtime-platform-009-closeout.md
  - ./roadmap.md
  - ./production-tracks.md
  - ./completed-work.md
  - ./decision-register.md
---

# Active Work

This file names the current planning focus for scriptless workflow. It stays short and points to the owning investigation/design records instead of duplicating them.

## Current focus

ID: `PT-UI-RUNTIME-PLATFORM-010`

Title: `UiPlugin Render Publication`

State: active planning only. No Phase 010 implementation branch is authorized by this record.

Lifecycle state: `active-planning` for Phase 010 only.

Owner: `engine::plugins::ui` owns publication from evaluated UiPlugin runtime frames into the generic surface-frame seam. Render frame/submission contracts own the producer/surface/frame vocabulary. RenderPlugin consumes generic packets without owning `UiScreen`, `IntoUi`, actions, host mutation, or route policy.

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
docs-site/src/content/docs/reports/closeouts/pt-workflow-track-orchestration-001-closeout.md
docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-003-closeout.md
docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-004-closeout.md
docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-005-closeout.md
docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-006-closeout.md
docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-007-closeout.md
docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-008-closeout.md
docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-009-closeout.md
docs-site/src/content/docs/reports/investigations/live-uiplugin-runtime-current-state-investigation.md
docs-site/src/content/docs/reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
docs-site/src/content/docs/design/active/live-uiplugin-runtime-full-cutover-plan.md
docs-site/src/content/docs/architecture/live-uiplugin-runtime-platform-architecture.md
docs-site/src/content/docs/architecture/ui-framework-architecture.md
```

Evidence classes: `E3` source/design/planning inspection by path, `E5` local command validation for completed Phase 009, `E6` PR #97 merge/check metadata, `E8` accepted architecture/workflow/planning authority, and `E9` code/test plus validation plus authority alignment in the Phase 009 closeout report.

Complete investigation gate: complete for opening Phase 010 active planning through the accepted runtime-platform investigation/design authority, Phase 009 closeout evidence, and the current cutover/architecture records. Implementation investigation is pending and must inspect the exact current UiPlugin evaluation resources, render publication target seam, trace/report types, and focused tests before Phase 010 can move to active implementation.

Complete design gate: complete for Phase 010 active planning through the accepted cutover plan, architecture record, and Phase 009 closeout. Complete design gate for Phase 010 implementation remains pending until a separate activation record names exact allowed files, forbidden files, validation commands, evidence expectations, stop conditions, principle compliance, and module decomposition.

Implementation authorization status: `blocked-pending-active-implementation-decision`.

Phase 009 completion truth:

```text
PR #97 merged into main at 50e2dbdf1f9c076f4a76a04543274801d1f1649b.
Closeout report: docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-009-closeout.md.
Remote phase branch was deleted by the merge command.
```

Phase 010 planning contract from accepted cutover authority:

```text
UiPlugin publishes frame submission with producer id and surface identity through the generic seam.
RenderPlugin consumes prepared payload without querying UiScreen, IntoUi, actions, host mutation, or route policy.
render contribution is deterministic for the same runtime frame.
missing UiPlugin frame reports a diagnostic instead of silent success.
frame publication trace records producer, surface, frame revision, dirty cause, and publication result.
```

Candidate Phase 010 scope from accepted cutover authority, pending separate implementation authorization:

```text
engine/src/plugins/ui/render_publish.rs
engine/src/plugins/ui/report.rs
engine/src/plugins/ui/trace.rs
existing generic render submission registry/resource paths only where needed
focused engine/render integration tests
```

Known non-goals for Phase 010 planning:

```text
scene/debug overlay migration or retirement
apps/ui_counter_runtime product packaging
source reload/persistence implementation
SDF or SpatialCanvas implementation
render backend rewrite, graph execution rewrite, or shader changes
source/program/action semantic changes outside publication facts
foundation/meta
domain/app_program
generic plugin framework
phase spec validator implementation
any tools/docs validator or script changes
```

Stop conditions for Phase 010 activation: stop if RenderPlugin becomes the UI runtime owner, pulls from app host state directly, needs a broad backend rewrite, requires scene/debug overlay retirement, requires Counter product packaging, or cannot be bounded to a single publication PR.

Feature support matrix:

```text
UiPlugin install/resource shell: completed by Phase 003.
Public mounting API: completed by Phase 004.
Typed screen/source/action contracts: completed by Phase 005.
Mounted sessions: completed by Phase 006.
Host action dispatch and trace: completed by Phase 007.
Runtime evaluation/invalidation: completed by Phase 008.
SurfaceFrame generic producer boundary: completed by Phase 009.
UiPlugin render publication: active-planning Phase 010.
Scene/debug overlay migration: downstream Phase 011.
Runtime Counter product: downstream Phase 012.
Reload/persistence: downstream Phase 013.
Closeout/adoption lock: downstream Phase 014.
```

Known blockers: Phase 010 implementation is not authorized yet. It needs a separate active-implementation decision after this closeout/planning truth is merged and after current `main` is inspected for exact file scope, focused tests, evidence requirements, validation commands, and stop conditions.

Next action: open a separate `PT-UI-RUNTIME-PLATFORM-010 - UiPlugin Render Publication` active-implementation authorization PR from current `main` if authority and source inspection still agree. Do not open a Phase 010 implementation PR until that authorization is merged.

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
