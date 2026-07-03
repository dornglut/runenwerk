---
title: Production Tracks
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ../workflow-lifecycle.md
  - ../../design/active/ui-component-platform-spatial-canvas-design.md
  - ../../design/active/ui-component-platform-overlay-popup-layering-design.md
  - ../../design/active/ui-component-platform-text-editing-design.md
  - ../../design/active/ui-component-platform-generic-text-design.md
  - ../../design/active/ui-component-platform-surface2d-design.md
  - ../../reports/investigations/phase-17-spatialcanvas-source-investigation.md
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
017 SpatialCanvas — active planning/design intake; implementation not authorized
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
Phase 16 future-pressure extraction: PR #64 merged docs-only extraction of still-useful stale Surface2D branch material into docs-site/src/content/docs/reports/investigations/surface2d-future-pressure-branch-review.md at merge commit 05c51375986cf08e360884ebf44702ec62662c1e. Current branch inspection during Phase 17 intake shows origin/surface2d-phase-16 is absent.
Phase 17: SpatialCanvas is open as active planning/design intake only. The source investigation is docs-site/src/content/docs/reports/investigations/phase-17-spatialcanvas-source-investigation.md and the design intake is docs-site/src/content/docs/design/active/ui-component-platform-spatial-canvas-design.md. It records Surface2D dependency, candidate owners ui_controls/ui_runtime/ui_static_mount, explicit non-owners, principle compliance, module decomposition expectations, and implementation stop conditions. It does not authorize implementation.
```

Current blocker:

```text
No Phase 16 product blocker remains after PR #61, PR #63 closeout, and PR #64 future-pressure extraction. Phase 17 implementation remains blocked until the SpatialCanvas design is accepted and planning explicitly records exact owner files, allowed files/crates, forbidden files/crates, implementation contract, module decomposition, validation envelope, evidence expectation, and stop conditions.
```

Next action:

```text
Review the Phase 17 SpatialCanvas planning/design intake. Do not start Phase 17 implementation until its complete design gate is accepted and active planning promotes it to active implementation.
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
