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
006 State binding / host intent — completed by user validation report
007 Theme / state / style — completed by user validation report
008 Accessibility / focus / inspection — completed by user validation report
009 Layout / container / virtualization — completed by user validation report through 009A/009B/009C
010 Render surface / output — completed through PR #34 and user validation report
011 Base control packages — completed through PR #37 and user validation report
012 Generic interaction — completed through PR #43 and user validation report
012A Executable interaction story — completed through PR #43 and user validation report
013 Overlay / popup / layering — completed through PR #44 and local validation report
014 Text Editing / Editable Text Behavior — completed through PR #46 and local validation report
015 Generic Text — completed through PR #48 baseline and PR #49 hardening
016 Surface2D — completed through PR #61 after PR #62 workflow hardening
017 SpatialCanvas — future, blocked behind runtime platform implementation planning
018 NodeCanvas — future
019 PortGraphCanvas — future
020 ProgressionTreeView — future; may be reframed as a generic tree/hierarchical graph package before implementation
021 TrackSurface / Timeline — future
022 Transitions / effects — future
023 Adoption gates — future final hardening, not the first adoption proof
024 Runtime-proven closeout — future
```

Evidence gates:

```text
Phase 13: PR #44 merged into main at merge commit 6f2d3827f315191d7aeaf68a64f523627197cad8. Local validation passed on 2026-07-02 with the full Phase 13 cargo/docs/diff gate.
Phase 14: PR #46 merged into main at merge commit 6d9bf983c77a32c701681ff55a05e1f9ebcdeed1. Local validation passed on 2026-07-02 with the recorded Phase 14 cargo/docs/diff gate before merge.
Phase 15: PR #48 merged into main at merge commit 91cea8b8f0dfc38143de77ba931bc81ffc91dcff. PR #49 merged into main at merge commit 338a8092d534dbb412da89363d50a46cd5efeae9. Final local validation passed with the recorded cargo workspace/docs/diff gate.
Phase 16: PR #62 merged docs-only workflow, principle, decomposition, and merge-readiness hardening into main at merge commit 6cfb82b81aa5478496ff6cbf3fa2eea607777aaf. PR #61 squash-merged Surface2D into main at merge commit 2e803620c91726fb599c5e5c4eee4b3984cd4a9d. Post-merge validation from main passed with the recorded focused Surface2D commands, cargo test --workspace, docs validation, and diff check.
```

Current blocker:

```text
No Phase 16 product blocker remains. The bounded ECS-backed app-integration proof is completed through PR #72 and closeout report `../../reports/closeouts/pt-ui-framework-app-integration-002-closeout.md`.

The remaining strategic blocker is the full implementation-planning contract for `PT-UI-RUNTIME-PLATFORM-002`. PR #74 completed design-gate hardening for `PT-UI-RUNTIME-PLATFORM-001`; runtime implementation, public AppUiExt code, render adapter code, SurfaceFrame migration code, SDF/world-space/SpatialCanvas work, foundation/meta, domain/app_program, and generic plugin framework work remain blocked until the full cutover plan is accepted.
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
Live UiPlugin runtime and generic surface-frame rendering: app authors install `RenderPlugin`, `UiPlugin`, and their own app plugin; mount typed UI screens; handle typed actions through host-owned app state; produce source/program/evaluator-backed frames; and publish generic surface-frame submissions that RenderPlugin prepares without owning UI semantics.
```

Authority:

```text
Current-state investigation: docs-site/src/content/docs/reports/investigations/live-uiplugin-runtime-current-state-investigation.md
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
007 Host Action Dispatch — future implementation PR
008 Runtime Evaluation to Frame — future implementation PR
009 UiPlugin Render Publication — future implementation PR
010 Scene/Debug Overlay Compatibility Cutover — future implementation PR
011 SurfaceFrame Genericization — future staged migration PR
012 Live Counter App Proof — future implementation/proof PR
013 Closeout and Adoption Lock — future closeout PR
```

Design gates:

```text
Complete investigation gate: complete for `PT-UI-RUNTIME-PLATFORM-001`.
Complete design gate: in progress for `PT-UI-RUNTIME-PLATFORM-002` full cutover plan.
Implementation authorization: forbidden until the full cutover plan is accepted and the next phase PR records exact scope, owner modules, allowed files/crates, forbidden files/crates, validation envelope, evidence expectation, principle compliance, acceptance criteria, and stop conditions.
```

Evidence gates:

```text
Current evidence is `E2` connector metadata/file inspection, `E3` source/design/planning inspection by path, and `E8` accepted architecture/workflow/planning authority. No `E5` local command validation is available from this connector-only planning session. Future implementation phases must provide focused crate tests, integration/proof tests, docs validation, dependency checks where applicable, and runtime/proof report evidence.
```

Current blocker:

```text
The full platform cutover plan is not yet accepted. Runtime implementation, public AppUiExt code, render adapter code, SurfaceFrame migration code, and compatibility-producer migration code remain blocked until `PT-UI-RUNTIME-PLATFORM-002` is reviewed and merged.
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
