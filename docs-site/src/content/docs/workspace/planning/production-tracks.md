---
title: Production Tracks
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-07
related_docs:
  - ../workflow-lifecycle.md
  - ../routines/track-orchestration-routine.md
  - ../specs/phase-implementation-spec.md
  - ../../architecture/ui-framework-architecture.md
  - ../../architecture/live-uiplugin-runtime-platform-architecture.md
  - ../../design/active/ui-framework-app-integration-direction-review.md
  - ../../design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
  - ../../design/active/live-uiplugin-runtime-full-cutover-plan.md
  - ../../reports/investigations/live-uiplugin-runtime-current-state-investigation.md
  - ../../design/active/ui-component-platform-overlay-popup-layering-design.md
  - ../../design/active/ui-component-platform-text-editing-design.md
  - ../../design/active/ui-component-platform-generic-text-design.md
  - ../../design/active/ui-component-platform-surface2d-design.md
---

# Production Tracks

Use this file for strategic production-track planning in Markdown.

## PT-UI-COMPONENT-PLATFORM

Track ID: `PT-UI-COMPONENT-PLATFORM`

Title: UI Component Platform

Track type: product / architecture proof

State: active

Lifecycle state: `production-track`

Goal:

```text
Make reusable Runenwerk UI controls and component surfaces story-proven, descriptor-backed, inspectable, executable, and consumable by higher product tracks without moving reusable control semantics into those consumers.
```

Milestones:

```text
001 ControlPackage / ControlKernel contract — completed by user report
002 Authoring Kit — completed by user validation report
003 Story proof envelope consumption — completed by user validation report
004 Catalog / discovery / inspection — completed by user validation report
005 Input / gesture / device — completed by user validation report
006 State binding / Host Intent — completed by user validation report
007 Theme / State / Style — completed by user validation report
008 Accessibility / Focus / Inspection — completed by user validation report
009 Layout / Container / Virtualization — completed by user validation report through 009A/009B/009C
010 Render Surface / Output — completed through PR #34 and user validation report
011 Base Control Packages — completed through PR #37 and user validation report
012 Generic Interaction — completed through PR #43 and user validation report
012A Executable Interaction Story — completed through PR #43 and user validation report
013 Overlay / Popup / Layering — completed through PR #44 and local validation report
014 Text Editing / Editable Text Behavior — completed through PR #46 and local validation report
015 Generic Text — completed through PR #48 baseline and PR #49 hardening
016 Surface2D — completed through PR #61 after PR #62 workflow hardening
017 SpatialCanvas — downstream, blocked behind runtime platform cutover
018 NodeCanvas — downstream
019 PortGraphCanvas — downstream
020 ProgressionTreeView — downstream; may be reframed as a generic tree/hierarchical graph package before implementation
021 TrackSurface / Timeline — downstream
022 Transitions / effects — downstream
023 Adoption gates — downstream final hardening, not the first adoption proof
024 Runtime-proven closeout — downstream
```

Current blocker:

```text
No Phase 16 product blocker remains. The bounded ECS-backed app-integration proof is completed through PR #72 and closeout report `../../reports/closeouts/pt-ui-framework-app-integration-002-closeout.md`.

The full `PT-UI-RUNTIME-PLATFORM-002` cutover plan is completed through PR #76. The `PT-WORKFLOW-TRACK-ORCHESTRATION-001` workflow gate is completed through PR #77 and closeout truth. `PT-UI-RUNTIME-PLATFORM-003 — UiPlugin Foundation` is completed through PR #79 and closeout truth. `PT-UI-RUNTIME-PLATFORM-004 — App Mounting API` is completed through PR #82 and closeout truth. `PT-UI-RUNTIME-PLATFORM-005 — Typed Screen / Source / Action Contracts` is completed through PR #85 and closeout truth. `PT-UI-RUNTIME-PLATFORM-006 — Mounted Surface Session Runtime` is completed through PR #88 and closeout truth. `PT-UI-RUNTIME-PLATFORM-007 — Host Action Dispatch and Runtime Trace` is completed through PR #91 and closeout truth. `PT-UI-RUNTIME-PLATFORM-008 — Runtime Evaluation, State Snapshot, and Invalidation` is completed through PR #94 and closeout truth. `PT-UI-RUNTIME-PLATFORM-009 — SurfaceFrame Generic Producer Boundary` is completed through PR #97 and closeout truth. `PT-UI-RUNTIME-PLATFORM-010 — UiPlugin Render Publication` is completed through PR #101 and closeout truth. `PT-UI-RUNTIME-PLATFORM-011 — Scene/Debug Overlay Producer Migration and Retirement` is completed through PR #104 and closeout truth. The active runtime-platform focus is now `PT-UI-RUNTIME-PLATFORM-012 — Runtime Counter App Product` active planning only.
```

Next action:

```text
After the Phase 011 closeout record merges, open exactly one bounded `PT-UI-RUNTIME-PLATFORM-012 — Runtime Counter App Product` active-implementation authorization PR from current `main`. Do not return to SpatialCanvas implementation or later Component Platform milestones before the runtime platform product path is proven or explicitly deferred.
```

## PT-UI-RUNTIME-PLATFORM

Track ID: `PT-UI-RUNTIME-PLATFORM`

Title: Live UiPlugin Runtime Platform

Track type: architecture / runtime platform / public API

State: active planning for Phase 012

Lifecycle state: `active-planning` for `PT-UI-RUNTIME-PLATFORM-012` only

Goal:

```text
Live UiPlugin runtime and generic surface-frame rendering: app authors install `RenderPlugin`, `UiPlugin`, and their own app plugin; mount typed UI screens; handle typed actions through host-owned app state; produce source/program/evaluator-backed frames; publish through a producer-generic surface-frame seam that RenderPlugin prepares without owning UI semantics; retire prior render-owned UI producer paths; provide generic UI-runtime trace/history and agent operation; support source-reload/persistence contracts; and ship a runnable Counter app product.
```

Authority:

```text
Current-state investigation: docs-site/src/content/docs/reports/investigations/live-uiplugin-runtime-current-state-investigation.md
Runtime architecture: docs-site/src/content/docs/architecture/live-uiplugin-runtime-platform-architecture.md
Design-gate authority: docs-site/src/content/docs/design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
Full cutover plan: docs-site/src/content/docs/design/active/live-uiplugin-runtime-full-cutover-plan.md
Track orchestration routine: docs-site/src/content/docs/workspace/routines/track-orchestration-routine.md
Phase implementation spec: docs-site/src/content/docs/workspace/specs/phase-implementation-spec.md
Architecture spine: docs-site/src/content/docs/architecture/ui-framework-architecture.md
Workflow gates: complete-investigation-gate.md, complete-design-gate.md, complete-merge-readiness-gate.md
```

Milestones:

```text
001 Live UiPlugin runtime and generic surface-frame rendering investigation/design gate — completed through merged PR #74 docs-only hardening
002 Full platform cutover plan — completed through merged PR #76 docs-only planning
Workflow gate PT-WORKFLOW-TRACK-ORCHESTRATION-001 — completed through merged PR #77 and closeout truth
003 UiPlugin Foundation — completed through merged PR #79 and closeout truth
004 App Mounting API — completed through merged PR #82 and closeout truth
005 Typed Screen / Source / Action Contracts — completed through PR #85 and closeout truth
006 Mounted Surface Session Runtime — completed through PR #88 and closeout truth
007 Host Action Dispatch and Runtime Trace — completed through PR #91 and closeout truth
008 Runtime Evaluation, State Snapshot, and Invalidation — completed through PR #94 and closeout truth
009 SurfaceFrame Generic Producer Boundary — completed through PR #97 and closeout truth
010 UiPlugin Render Publication — completed through PR #101 and closeout truth
011 Scene/Debug Overlay Producer Migration and Retirement — completed through PR #104 and closeout truth
012 Runtime Counter App Product — active planning only; implementation PR not yet authorized
013 Source Reload and Persistence Contract — downstream implementation PR
014 Closeout and Adoption Lock — downstream closeout PR
```

Design gates:

```text
Complete investigation gate: complete for `PT-UI-RUNTIME-PLATFORM-001`; `PT-UI-RUNTIME-PLATFORM-002` added render/app-engine feature mapping, runtime architecture, agent/trace requirements, producer-generic render-boundary ordering, reload/persistence decisions, SDF-backend downstream ownership, phase-spec workflow decision, and product acceptance requirements.
Complete design gate: completed for `PT-UI-RUNTIME-PLATFORM-002` through merged PR #76.
Implementation authorization: none active. Phase 012 is active planning only until a separate authorization records exact owner files/crates, validation, evidence, principle checks, module decomposition, and stop conditions. Phase 013 and later remain forbidden until Phase 012 is reviewed, merged, and completion truth is recorded.
```

Evidence gates:

```text
Current evidence is `E3` source/design/planning inspection by path, `E5` local command validation for completed Phase 011, `E6` PR #104 merge/check metadata, `E8` accepted architecture/workflow/planning authority, and `E9` code/test plus validation plus authority alignment for Phase 011 closeout. Future implementation phases must provide focused crate tests, integration/proof tests, docs validation, dependency checks where applicable, runtime/proof report evidence, and for Phase 012 recorded human and agent Counter app commands.
```

Current blocker:

```text
Phase 011 is merged and closed out. Phase 012 is active planning only and not implementation-authorized. Source reload/persistence implementation and Phases 013-014 implementation remain blocked until Phase 012 is reviewed, merged, and completion truth is recorded.
```

Activation condition:

```text
Phase 012 may move to active implementation only after a separate authorization records the app/product crate inventory, human and agent command proof plan, exact allowed files, forbidden files, validation envelope, evidence expectation, principle checks, module decomposition, and stop conditions. Phase 013 may move only after Phase 012 is complete, reviewed/merged, and truthfully closed.
```

Next action:

```text
After the Phase 011 closeout record merges, open exactly one bounded Phase 012 active-implementation authorization PR from current `main`. Do not patch `apps/ui_counter_runtime` or product runtime code until that authorization is merged. Do not implement multiple runtime phases in one broad PR.
```

## Track shape

```text
Track ID:
Title:
Track type:
State:
Lifecycle state:
Goal:
Authority:
Milestones:
Design gates:
Evidence gates:
Current blocker:
Activation condition:
Next action:
```
