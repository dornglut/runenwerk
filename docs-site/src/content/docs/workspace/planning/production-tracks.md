---
title: Production Tracks
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-29
related_docs:
  - ../workflow-lifecycle.md
  - ../../design/active/ui-component-platform-generic-interaction-design.md
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
Make reusable Runenwerk UI controls and component surfaces story-proven, descriptor-backed, inspectable, and consumable by Gallery, Workbench, UI Designer, and future product tracks without moving reusable control semantics into those consumers.
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
010 Render surface / output — completed by user validation report through PR #34
011 Base control packages — completed through PR #37 and user validation report
012 Generic interaction — completed in PR #43 implementation evidence
013 Overlay / popup / layering — future
014 Minimum text editing — future
015 Generic text — future
016 Surface2D — future
017 SpatialCanvas — future
018 NodeCanvas — future
019 PortGraphCanvas — future
020 ProgressionTreeView — future
021 TrackSurface / Timeline — future
022 Transitions / effects — future
023 Adoption gates — future
024 Runtime-proven closeout — future
```

Design gates:

```text
009A corrected the owner-first rule before Phase 9 implementation.
009B proved generic layout vocabulary belongs in ui_layout.
009C proved ui_controls should bridge owner-crate vocabulary through per-control descriptors and read-only summaries.
010 proved render/output ownership across ui_render_data, ui_controls, ui_runtime, and engine render.
011 proved UI-local contribution/preset/lowering authoring for package-quality base controls while keeping full interaction behavior for Phase 12.
012 completed generic reusable interaction semantics, package/catalog/inspection visibility, normalized input replay/report evidence, renderer-neutral visible proof, and focus/keyboard/text-intent seams while preserving existing owner boundaries.
Each later milestone requires its own owning design or planning update before code.
```

Evidence gates:

```text
Phase 1: local validation and branch/PR evidence.
Phase 2: user reported the authoring-kit validation gate green.
Phase 3: user reported the story-proof validation gate green.
Phase 4: user reported the catalog validation gate green.
Phase 5: user reported the input validation gate green.
Phase 6: user reported the state/host-intent validation gate green.
Phase 7: user reported the theme/state/style validation gate green.
Phase 8: user reported the accessibility/focus/inspection validation gate green.
Phase 9: PR #29 merged 009A ownership realignment, 009B ui_layout layout foundation, 009C ui_controls layout bridge, read-only catalog inspection bridge, and focused tests; user reported the validation gate green.
Phase 10: PR #34 merged renderer-neutral output evidence, control render bridge, runtime output evidence generation, and engine render submission proof; user reported the validation gate green.
Phase 11: PR #37 merged the UI-local base-control contribution/preset/lowering proof; user reported the validation gate green.
Phase 12: PR #43 implements package-backed generic interaction descriptors, catalog/inspection visibility, normalized input facts, descriptor-driven mounted replay/report, `InteractionVisualProof`/`InteractionProofFrame` visible proof, negative proof cases, read-only text-intent probe behavior, and no-bypass assertions. Validation passed for the Phase 12 implementation gate recorded in the closeout report.
Later phases: overlays, text editing, rendering, adoption, diagnostics, docs evidence, and runtime-proof gates as appropriate.
```

Current blocker:

```text
No current blocker for Phase 12. Existing gallery/static mount infrastructure does not yet render the Phase 12 proof model as a product-facing gallery page, but PR #43 provides the renderer-neutral visible proof path required for completion. Phase 13 overlays/layering and later full text editing remain future work.
```

Activation condition:

```text
Each future milestone activates only through an accepted planning/design update and active-work transition.
```

Next action:

```text
Review/merge PR #43. Do not implement overlay/popup/layering, full text editing, host-specific command behavior, shared plugin framework extraction, generic plugin primitives, or foundation/meta as part of Phase 12. Open a separate active-work entry before Phase 13 or text-editing implementation begins.
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

## Rules

- Production tracks guide sequencing.
- Production tracks do not authorize code without an owned implementation scope.
- Strategic order must be readable without generated views.
- Legacy structured track files may remain as optional mirrors.
- Use `../workflow-lifecycle.md` before promoting a track to active planning or active implementation.
- Do not create a production track for every accepted design. Use a production track only for strategic multi-phase work.
