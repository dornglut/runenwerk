---
title: Production Tracks
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-02
related_docs:
  - ../workflow-lifecycle.md
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
016 Surface2D — active-planning
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
Phase 16: Surface2D is active planning for reusable renderer-neutral 2D coordinate/navigation proof: surface identity, content and viewport bounds, world/screen transforms, pan, zoom, fit, selection rectangle, hover coordinate, pointer capture, gesture cancel/commit, overlay/diagnostic layers, grid/background vocabulary, large-content bounds, LOD readiness, budget evidence, accessibility/input acceptance, and host-intent/no-mutation boundaries.
```

Current blocker:

```text
No Phase 15 implementation blocker remains in local validation. Phase 16 has not been implementation-authorized; planning must first settle exact owner files, implementation scope, validation, evidence expectation, stop conditions, no-mutation boundaries, accessibility/input acceptance, budget evidence, and the current relationship to existing ui_surface vocabulary. Typed App Composition is proposed architecture direction only and does not authorize shared plugin framework work.
```

Next action:

```text
Harden the Phase 16 Surface2D design intake; preserve future order 017+ unchanged.
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
