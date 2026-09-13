---
title: UI Component Platform Surface2D Design
description: Implemented Runenwerk-local renderer-neutral 2D coordinate, navigation, transform, bounds, input-fact, overlay-fact, proof, and budget contract.
status: implemented
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-09-13
related_designs:
  - ../active/runenwerk-ui-story-driven-golden-workflow-design.md
  - ./editor-ui-runtime-v2-and-interaction-formation-design.md
  - ../active/runenwerk-typed-app-composition-plugin-framework-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../reports/closeouts/phase-16-surface2d-closeout.md
  - ../../reports/investigations/phase-16-surface2d-source-investigation.md
  - ../../reports/investigations/surface2d-future-pressure-branch-review.md
---

# UI Component Platform Surface2D Design

## Status

Implemented through the completed Phase 16 Surface2D delivery. The former `active_phase_evidence` exception is no longer needed because this document now resides in implemented lifecycle authority.

`Surface2D` is a reusable renderer-neutral coordinate/navigation substrate. It is not Gallery-specific, GraphCanvas-specific, a product editor command system, app composition, or a renderer backend.

## Implemented ownership

```text
ui_controls
  package-backed Surface2D declarations, descriptor validation,
  catalog projection, inspection facts, and control-facing support summaries.

ui_runtime
  runtime-local Surface2D state projection, transform/navigation/input-fact
  consumption, proof report, budget/no-bypass evidence, and proof-frame output.

ui_static_mount
  renderer-neutral static validation of the produced proof frame.

host/product/editor/game
  product commands, selection/data mutation, persistence, specialized graph or
  timeline truth, renderer resources, and external effects.
```

`ui_surface` remains a separate semantic surface/mount owner. Surface2D is lower-level coordinate/navigation vocabulary and does not rename, replace, remove, or absorb `ui_surface`.

## Implemented contract

The current Surface2D proof covers:

- stable surface/control identity;
- content and viewport bounds;
- world-to-screen and screen-to-world transforms with invalid-transform diagnostics;
- pan, zoom, and fit-content request/evidence;
- hover-coordinate facts;
- transient selection-rectangle facts without product selection mutation;
- pointer-capture and gesture cancel/commit facts;
- grid/background and diagnostic-overlay facts;
- large-content bounds and LOD/virtualization-readiness facts;
- explicit budget evidence;
- package/catalog/inspection visibility;
- runtime proof/report/frame formation;
- static mount proof and no-bypass assertions.

Current source includes `domain/ui/ui_controls/src/surface2d/` and `domain/ui/ui_runtime/src/surface2d/` plus focused package/runtime/static-mount tests.

## Downstream boundary

Specialized future surfaces such as SpatialCanvas, NodeCanvas, PortGraphCanvas, progression trees, and track/timeline surfaces are not part of this implemented classification. They remain separately lifecycle-owned and must consume the generic substrate without pushing their domain semantics into Surface2D.

Typed App Composition remains proposed Runenwerk architecture direction only and does not become implemented by relation to Surface2D.

## Boundary rules

- Surface2D emits renderer-neutral facts/intents; it does not mutate product/editor/game truth.
- Graph and timeline semantics stay out of Surface2D.
- Renderer resources/backend handles stay outside the domain contract.
- `ui_surface` semantics remain distinct.
- Runtime proof must preserve deterministic transform/navigation evidence and fail closed on invalid transforms.
- Large-content/LOD facts are readiness/budget evidence, not a universal rendering algorithm.

## Evidence

The retained Phase 16 closeout and investigation reports preserve delivery/provenance detail. Current `ui_controls` and `ui_runtime` source plus focused tests are implementation truth.
