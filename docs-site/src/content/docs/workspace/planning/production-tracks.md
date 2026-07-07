---
title: Production Tracks
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-07
related_docs:
  - ../workflow-lifecycle.md
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
017 SpatialCanvas — future, blocked behind runtime platform cutover
018 NodeCanvas — future
019 PortGraphCanvas — future
020 ProgressionTreeView — future; may be reframed as a generic tree/hierarchical graph package before implementation
021 TrackSurface / Timeline — future
022 Transitions / effects — future
023 Adoption gates — future final hardening, not the first adoption proof
024 Runtime-proven closeout — future
```

Current blocker:

```text
No Phase 16 product blocker remains. The bounded ECS-backed app-integration proof is completed through PR #72 and closeout report `../../reports/closeouts/pt-ui-framework-app-integration-002-closeout.md`.

The remaining strategic blocker is the full implementation-planning contract for `PT-UI-RUNTIME-PLATFORM-002`. Runtime implementation, public AppUiExt code, render adapter code, SurfaceFrame migration code, old scene/debug overlay migration work, runnable Counter product code, source-reload/persistence code, SDF/world-space/SpatialCanvas work, foundation/meta, domain/app_program, and generic plugin framework work remain blocked until the full cutover plan is accepted.
```

Next action:

```text
Review the `PT-UI-RUNTIME-PLATFORM-002` full cutover-plan PR. Do not return to SpatialCanvas implementation, standalone public AppUiExt ergonomics, authoring/execution strategy work, or later Component Platform milestones as implementation before the runtime platform cutover contract exists.
```

## PT-UI-RUNTIME-PLATFORM

Track ID: `PT-UI-RUNTIME-PLATFORM`

Title: Live UiPlugin Runtime Platform

Track type: architecture / runtime platform / public API

State: active-planning full cutover contract

Lifecycle state: `active-planning`; implementation not authorized until the full cutover-plan PR is accepted

Goal:

```text
Live UiPlugin runtime and generic surface-frame rendering: app authors install `RenderPlugin`, `UiPlugin`, and their own app plugin; mount typed UI screens; handle typed actions through host-owned app state; produce source/program/evaluator-backed frames; publish generic surface-frame submissions that RenderPlugin prepares without owning UI semantics; retire old render-owned UI producer paths; provide generic trace/history and agent operation; support source-reload/persistence contracts; and ship a runnable Counter app product.
```

Authority:

```text
Current-state investigation: docs-site/src/content/docs/reports/investigations/live-uiplugin-runtime-current-state-investigation.md
Runtime architecture: docs-site/src/content/docs/architecture/live-uiplugin-runtime-platform-architecture.md
Design-gate authority: docs-site/src/content/docs/design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
Full cutover plan: docs-site/src/content/docs/design/active/live-uiplugin-runtime-full-cutover-plan.md
Architecture spine: docs-site/src/content/docs/architecture/ui-framework-architecture.md
Workflow gates: complete-investigation-gate.md, complete-design-gate.md, complete-merge-readiness-gate.md
```

Milestones:

```text
001 Live UiPlugin runtime and generic surface-frame rendering investigation/design gate — completed through merged PR #74 docs-only hardening
002 Full platform cutover plan — active docs-only planning PR
003 UiPlugin Foundation — future implementation PR
004 App Mounting API — future implementation PR
005 Typed Screen / Source / Action Contracts — future implementation PR
006 Mounted Surface Session Runtime — future implementation PR
007 Host Action Dispatch and Runtime Trace — future implementation PR
008 Runtime Evaluation, State Snapshot, and Invalidation — future implementation PR
009 UiPlugin Render Publication — future implementation PR
010 Legacy Scene/Debug Overlay Migration and Removal — future implementation PR with no permanent old path
011 SurfaceFrame Genericization Cutover — future staged migration PR
012 Source Reload and Persistence Contract — future implementation PR
013 Runtime Counter App Product — future implementation/proof PR
014 Closeout and Adoption Lock — future closeout PR
```

Design gates:

```text
Complete investigation gate: complete for `PT-UI-RUNTIME-PLATFORM-001`; `PT-UI-RUNTIME-PLATFORM-002` adds render/app-engine feature mapping, runtime architecture, agent/trace requirements, reload/persistence decisions, and product acceptance requirements.
Complete design gate: in progress for `PT-UI-RUNTIME-PLATFORM-002` full cutover plan.
Implementation authorization: forbidden until the full cutover plan is accepted and the next phase PR records exact scope, owner modules, allowed files/crates, validation envelope, evidence expectation, principle compliance, acceptance criteria, and stop conditions.
```

Evidence gates:

```text
Current evidence is `E2` connector metadata/file inspection, `E3` source/design/planning inspection by path, and `E8` accepted architecture/workflow/planning authority. No `E5` local command validation is available from this connector-only planning session. Future implementation phases must provide focused crate tests, integration/proof tests, docs validation, dependency checks where applicable, runtime/proof report evidence, and for Phase 013 recorded human and agent Counter app commands.
```

Current blocker:

```text
The full platform cutover plan is not yet accepted. Runtime implementation, public AppUiExt code, render adapter code, SurfaceFrame migration code, old overlay migration work, source reload/persistence implementation, and runnable Counter product code remain blocked until `PT-UI-RUNTIME-PLATFORM-002` is reviewed and merged.
```

Activation condition:

```text
Promote to active implementation only after `PT-UI-RUNTIME-PLATFORM-002` is accepted and `PT-UI-RUNTIME-PLATFORM-003 — UiPlugin Foundation` opens as a bounded implementation PR.
```

Next action:

```text
Review the full cutover-plan PR. If accepted, merge it and open Phase 003. Do not implement multiple runtime phases in one broad PR.
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
