---
title: Production Tracks
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-30
related_docs:
  - ../workflow-lifecycle.md
  - ../../design/active/ui-component-platform-generic-interaction-design.md
  - ../../design/active/ui-component-platform-executable-interaction-story-design.md
  - ../../design/active/ui-component-platform-executable-interaction-story-implementation-scope.md
  - ../../design/active/ui-component-platform-overlay-popup-layering-design.md
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
Make reusable Runenwerk UI controls and component surfaces story-proven, descriptor-backed, inspectable, executable, and consumable by Gallery, Workbench, UI Designer, and future product tracks without moving reusable control semantics into those consumers.
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
012 Generic interaction — completed through PR #43 and user validation report
012A Executable interaction story — completed through PR #43 and user validation report
013 Overlay / popup / layering — active planning / design intake
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
012 merged PR #43 implementation evidence for generic reusable interaction semantics, package/catalog/inspection visibility, normalized input replay/report evidence, renderer-neutral visible proof, static mount frame evidence, and focus/keyboard/text-intent seams while preserving existing owner boundaries.
012A merged PR #43 implementation evidence for one executable story that supports deterministic replay, live proof-host input, semantic replay/live parity, static frame validation, and no-bypass counters.
013 is active design intake for reusable overlay, popup, dropdown, tooltip, modal-like, and layering semantics. It must consume the Phase 12/12A interaction substrate and the older Interaction V2 popup-stack lessons without moving product/editor/game behavior into generic UI.
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
Phase 12: PR #43 merged package-backed generic interaction descriptors, catalog/inspection visibility, normalized input facts, descriptor-driven mounted replay/report, `InteractionVisualProof`/`InteractionProofFrame` visible proof, `InteractionProofRenderFrame`/`UiFrame` static mount proof, negative proof cases, read-only text-intent probe behavior, and no-bypass assertions; user reported the validation gate green before the next phase started.
Phase 12A: PR #43 merged an Executable UI Interaction Story with replay mode, live proof-host mode, shared normalized input path, semantic replay/live parity, static frame artifact, and zero host-command/product-mutation/overlay/text-edit boundary assertions; user reported the validation gate green before the next phase started.
Phase 13: design intake must define owner crates/files, durable vocabulary, static/story proof shape, replay/report evidence, no-bypass assertions, validation commands, and stop conditions before implementation.
Later phases: text editing, rendering, adoption, diagnostics, docs evidence, and runtime-proof gates as appropriate.
```

Current blocker:

```text
Phase 13 implementation is blocked until the overlay/popup/layering design is accepted and an implementation-scope section names exact owner crates/files, non-goals, proof scenarios, negative scenarios, evidence contracts, validation commands, no-bypass assertions, and stop conditions.
```

Activation condition:

```text
Each future milestone activates only through an accepted planning/design update and active-work transition.
```

Next action:

```text
Review and accept, revise, or reject `docs-site/src/content/docs/design/active/ui-component-platform-overlay-popup-layering-design.md`. Do not implement UI Gallery exposure, full UI Designer, authored UI editing, full text editing, host-specific command behavior, shared plugin framework extraction, generic plugin primitives, or foundation/meta as part of Phase 13 design intake.
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
