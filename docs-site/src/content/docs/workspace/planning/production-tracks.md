---
title: Production Tracks
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-06
related_docs:
  - ../workflow-lifecycle.md
  - ../../architecture/ui-framework-architecture.md
  - ../../design/active/ui-framework-app-integration-direction-review.md
  - ../../design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
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
017 SpatialCanvas — future
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
Phase 14: PR #46 merged into main at merge commit 6d9bf983c77a32c701681ff55a05e1f9ebcdeed1. Main contains package-backed editable-text declarations, InspectorField text-editing lowering, package descriptor wiring, package validation, catalog projection, inspection projection, normalized text edit/composition/selection facts, ui_runtime::text_editing replay/report/value/caret/selection/composition/suppression/no-bypass proof, proof-frame projection, static mount validation, focused tests, and final proof-frame cleanup. Local validation passed on 2026-07-02 with the recorded Phase 14 cargo/docs/diff gate before merge.
Phase 15: PR #48 merged into main at merge commit 91cea8b8f0dfc38143de77ba931bc81ffc91dcff. The validated implementation commit 32e402b108d1e72d7cc5b4113af29d8d29626680 covers renderer-neutral `ui_text` block/run/span/layout/style/line/glyph/overflow/fallback/diagnostic contracts, `TextBlockLayoutRequest` / `TextBlockLayoutResult` / `TextLayouter`, package-backed Generic Text descriptors and validation, catalog projection, separate `TextDisplay` inspection projection, runtime proof report/frame, static mount proof, renderer-neutral frame/extract adaptation to `TextVisualRun` / `TextGlyph` evidence, and removal of the old `ui_text::GlyphRun` / `PositionedGlyph` compatibility path. PR #49 merged into main at merge commit 338a8092d534dbb412da89363d50a46cd5efeae9 and completed the hardening pass: source-run/cluster evidence correction, height overflow evidence, stable-ID constructors, homogeneous visual-run segmentation, button-label policy cleanup, runtime text-emission naming, Generic Text direction-policy inspection, and large output-emission file splits. Final local validation passed with the recorded cargo workspace/docs/diff gate.
Phase 16: PR #62 merged docs-only workflow, principle, decomposition, and merge-readiness hardening into main at merge commit 6cfb82b81aa5478496ff6cbf3fa2eea607777aaf. PR #61 squash-merged Surface2D into main at merge commit 2e803620c91726fb599c5e5c4eee4b3984cd4a9d. Main contains the renderer-neutral Surface2D package/catalog/inspection contract, runtime proof report/frame, and static mount proof across ui_controls, ui_runtime, and ui_static_mount. Post-merge validation from main passed with the focused Surface2D package/runtime/static-mount commands, cargo test --workspace, docs validation, and diff check.
```

Current blocker:

```text
No Phase 16 product blocker remains. The bounded ECS-backed app-integration proof is completed through PR #72 and closeout report `../../reports/closeouts/pt-ui-framework-app-integration-002-closeout.md`.

The remaining strategic blocker is PR #74 / `PT-UI-RUNTIME-PLATFORM-001` intake review and hardening: Live UiPlugin runtime, generic surface-frame rendering, public plugin/runtime API shape, render-target ownership, AppUiExt pressure, external templates, DSL/compiler frontends, retained/immediate/reactive strategies, and SDF/game/world-space targets must be positioned without bypassing ui_definition, UiProgram, UiStory, or host-owned mutation.
```

Next action:

```text
Keep Phase 16 and PT-UI-FRAMEWORK-APP-INTEGRATION-002 as completed dependencies. Keep Phase 17 SpatialCanvas as future planning only. Review and harden PR #74 / PT-UI-RUNTIME-PLATFORM-001 intake before returning to SpatialCanvas implementation, public AppUiExt ergonomics, authoring/execution strategy work, or later Component Platform milestones.
```

## PT-UI-RUNTIME-PLATFORM

Track ID: `PT-UI-RUNTIME-PLATFORM`

Title: Live UiPlugin Runtime Platform

Track type: architecture / runtime platform / public API

State: track candidate / active-planning intake review

Lifecycle state: `active-planning` intake review; implementation not authorized

Goal:

```text
Live UiPlugin runtime and generic surface-frame rendering: app authors install `RenderPlugin`, `UiPlugin`, and their own app plugin; mount typed UI screens; handle typed actions through host-owned app state; produce source/program/evaluator-backed frames; and publish generic surface-frame submissions that RenderPlugin prepares without owning UI semantics.
```

Authority:

```text
Primary proposed design: docs-site/src/content/docs/design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
Architecture spine: docs-site/src/content/docs/architecture/ui-framework-architecture.md
Workflow gates: complete-investigation-gate.md and complete-design-gate.md
```

Milestones:

```text
001 Live UiPlugin runtime and generic surface-frame rendering intake — PR #74 intake review/hardening
002 UiPlugin skeleton and app mounting API — future, blocked until complete design gate and active planning
003 Typed UiScreen / IntoUi / UiActionHandler ergonomics — future
004 Mounted surface/session runtime using ui_surface — future
005 Typed event/action dispatch using ui_hosts contracts — future
006 Runtime/evaluator output to frame — future
007 UiPlugin render publication — future
008 Render genericization from UiFrame naming toward SurfaceFrame naming — future
009 Counter live app proof — future
010 Closeout and planning truth — future
```

Design gates:

```text
Complete investigation gate: required and not complete yet.
Complete design gate: required and not complete yet.
Implementation authorization: forbidden until accepted PR #74 intake plus active planning names exact implementation contract, allowed files/crates, forbidden files/crates, validation envelope, evidence expectation, principle compliance, module decomposition, and stop conditions.
```

Evidence gates:

```text
Current evidence is source/design/planning inspection and accepted/workflow authority only. Future implementation must provide focused crate tests, integration/proof tests, docs validation, dependency checks where applicable, and runtime/proof report evidence.
```

Current blocker:

```text
PR #74 remains intake review/hardening only. Complete investigation gate evidence, complete design gate evidence, and an exact implementation contract are still required before runtime implementation, public AppUiExt code, render adapter code, or SurfaceFrame migration code.
```

Activation condition:

```text
Promote beyond intake only after accepted PR #74 intake, complete investigation gate evidence, complete design gate evidence, and an exact implementation contract are recorded.
```

Next action:

```text
Review and harden the design intake. Do not open runtime implementation yet.
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
