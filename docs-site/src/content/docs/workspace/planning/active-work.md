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
  - ../../guidelines/programming-principles.md
  - ../../architecture/ui-framework-architecture.md
  - ../../design/active/ui-framework-app-integration-direction-review.md
  - ../../design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
  - ../../design/active/live-uiplugin-runtime-full-cutover-plan.md
  - ../../reports/investigations/live-uiplugin-runtime-current-state-investigation.md
  - ../../reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
  - ./roadmap.md
  - ./production-tracks.md
  - ./decision-register.md
---

# Active Work

This file names the current planning focus for scriptless workflow. It stays short and points to the owning investigation/design records instead of duplicating them.

## Current focus

ID: `PT-UI-RUNTIME-PLATFORM-002`

Title: `Live UiPlugin Runtime Full Platform Cutover Plan`

State: draft docs-only implementation-planning PR in progress.

Lifecycle state: `active-planning` full-platform cutover contract draft. Not `active-implementation`.

Owner: UI runtime/platform planning owns the full cutover contract. Future implementation phases will each name their own engine/UI owner and exact file scope.

Authority files:

```text
docs-site/src/content/docs/design/active/live-uiplugin-runtime-full-cutover-plan.md
docs-site/src/content/docs/design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
docs-site/src/content/docs/reports/investigations/live-uiplugin-runtime-current-state-investigation.md
docs-site/src/content/docs/architecture/ui-framework-architecture.md
docs-site/src/content/docs/design/active/ui-framework-app-integration-direction-review.md
docs-site/src/content/docs/reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
```

Evidence classes: `E2` connector metadata/file inspection, `E3` source/design/planning inspection by path, and `E8` accepted architecture/workflow/planning authority. No `E5` local command validation is available from this connector-only planning session.

Complete investigation gate: inherited complete gate from `PT-UI-RUNTIME-PLATFORM-001`; this full cutover plan adds render/app-engine feature mapping and product acceptance requirements for implementation handoff.

Complete design gate: in progress for the full platform cutover contract. The prior “first runtime slice” framing is corrected: the platform should be planned as a full cutover, then implemented through gated phase PRs.

Implementation contract: not yet authorized. This active work records the full implementation program only. Runtime Rust code remains blocked until this planning PR is reviewed/merged and the next phase PR opens with exact scope.

Allowed files/crates for current focus:

```text
docs-site/src/content/docs/design/active/live-uiplugin-runtime-full-cutover-plan.md
docs-site/src/content/docs/design/active/README.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
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
SurfaceFrame type migration code
scene/debug overlay migration/deletion implementation code
apps/ui_counter_runtime implementation
SDF/world-space/SpatialCanvas implementation
foundation/meta
domain/app_program
generic plugin framework
```

Known blockers: implementation cannot start until this full cutover plan is reviewed/merged and the next phase, `PT-UI-RUNTIME-PLATFORM-003 — UiPlugin Foundation`, opens as a bounded implementation PR.

Next action: review the full cutover-plan PR. If accepted, merge it and open `PT-UI-RUNTIME-PLATFORM-003` for the first implementation phase inside the recorded boundaries.

## Active-work rules

- One current focus is preferred.
- If no current focus exists, say that explicitly.
- Do not promote deferred work without recording the reason.
- Do not mark work completed without evidence.
- If legacy generated views disagree, report them as stale mirrors.
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
