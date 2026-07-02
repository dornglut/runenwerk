---
title: UI Component Platform Surface2D Design
description: Generic 2D coordinate, navigation, transform, bounds, overlay, input, and large-content primitive for reusable surfaces.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-07-02
related_designs:
  - ./runenwerk-ui-platform-capability-roadmap.md
  - ./runenwerk-ui-story-driven-golden-workflow-design.md
  - ./editor-ui-runtime-v2-and-interaction-formation-design.md
  - ./runenwerk-typed-app-composition-plugin-framework-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
  - ../../workspace/planning/decision-register.md
  - ../../workspace/workflow-lifecycle.md
---

# UI Component Platform Surface2D Design

## Status

This is the active Phase 16 planning design for `PT-UI-COMPONENT-PLATFORM-016` Surface2D.

Lifecycle state: `active-planning`.

This document does not authorize implementation by itself. Implementation requires the planning state to be promoted with exact owner files, minimum deliverable, validation envelope, evidence expectation, and stop conditions.

`Surface2D` is a reusable renderer-neutral coordinate and navigation substrate. It is not Gallery-specific, not GraphCanvas-specific, not a product editor command system, and not a renderer backend.

Typed App Composition documents are proposed architecture references only. They do not authorize plugin framework work, shared extraction, `foundation/meta`, or app-composition implementation in Phase 16.

## Decision

`Surface2D` is the generic 2D coordinate/navigation primitive underneath future reusable surfaces.

It owns reusable surface facts and intent vocabulary for:

```text
surface identity
content bounds
viewport bounds
world/screen transforms
pan and zoom state
fit requests
hover coordinate facts
selection rectangle facts
pointer capture facts
gesture cancel/commit facts
overlay and diagnostic layer facts
grid/background facts
large-content and LOD-readiness facts
budget evidence
```

It does not own product truth, graph truth, timeline truth, editor commands, renderer resources, authored UI mutation, or app recipe composition.

## Owner split to settle before implementation

Phase 16 implementation is blocked until this owner split is accepted:

```text
ui_controls:
  package-backed Surface2D declarations, descriptors, validation reasons,
  catalog projection, and inspection facts if Surface2D is exposed as a reusable package contract.

ui_runtime:
  runtime-local Surface2D state projection, input normalization consumption,
  pan/zoom/fit/hover/selection/capture intent evidence, proof report,
  and renderer-neutral proof-frame projection.

ui_static_mount:
  static proof that Surface2D declarations lower to mountable renderer-neutral evidence
  without bypassing package/catalog/inspection contracts.

ui_render_data / ui_render_primitives:
  only if the minimum proof needs new renderer-neutral primitive data.
  Do not add backend resources or renderer-owned truth.

host/product/editor/game layers:
  product commands, graph edits, timeline edits, selection mutation,
  persistence, project data, renderer resources, and external effects.
```

If the owner split would require a new crate, record that as an explicit planning decision before implementation. Do not create a new crate as an incidental implementation detail.

## Canonical vocabulary

- `Surface2D` - generic renderer-neutral 2D coordinate/navigation surface contract.
- `Surface2DId` - stable identity for a reusable surface instance or proof fixture.
- `Surface2DViewport` - visible screen-space or local-frame bounds.
- `Surface2DContentBounds` - renderer-neutral world/content bounds.
- `Surface2DTransform` - world-to-screen and screen-to-world mapping evidence.
- `Surface2DNavigationState` - pan/zoom/fit state facts.
- `Surface2DInteractionIntent` - normalized intent emitted by reusable surface behavior.
- `Surface2DGestureState` - pointer capture, drag, cancel, commit, and active gesture facts.
- `Surface2DSelectionBox` - transient rectangle evidence, not product selection mutation.
- `Surface2DOverlayLayer` - diagnostic/grid/background/adornment layer facts.
- `Surface2DBudgetEvidence` - explicit evidence for large bounds, LOD readiness, and report/runtime budgets.

## Relationship to existing `ui_surface` vocabulary

The current Typed App Composition proposal references `ui_surface` as semantic surface and mount compatibility vocabulary. Phase 16 must not silently conflict with that vocabulary.

Before implementation, choose and record one of these outcomes:

```text
A. Surface2D is lower-level coordinate/navigation vocabulary used by ui_surface.
B. Surface2D replaces part of ui_surface after an accepted migration decision.
C. Surface2D remains UI Component Platform-local and does not touch ui_surface.
```

Until that decision is recorded, `Surface2D` must not rename, remove, or absorb existing `ui_surface` contracts.

## Non-negotiable rules

- General coordinate/navigation contracts come before specialized canvases.
- Story/proof evidence comes before mount eligibility.
- `Surface2D` must not collapse into Gallery, Workbench, UI Designer, GraphCanvas, Timeline, or a product editor.
- `Surface2D` emits facts and host intents only; it does not mutate product/editor/game truth.
- Graph semantics stay out of `Surface2D`.
- Timeline semantics stay out of `Surface2D`.
- Renderer resources and backend handles stay out of `Surface2D`.
- Host/app/editor/game mutation remains outside `domain/ui` through explicit host intent or command paths.
- UI Story owns proof orchestration only.
- Gallery, Workbench, and UI Designer consume platform contracts; they do not own reusable surface semantics.
- Renderer output remains backend-neutral and must not become UI source truth.
- Typed App Composition remains proposed reference direction only unless accepted separately.

## Phase 16 minimum implementation scope

The first implementation pass should prove only the reusable substrate:

```text
surface id
content bounds
viewport bounds
world-to-screen transform
screen-to-world transform
pan state
zoom state
fit-content request/evidence
hover coordinate fact
selection rectangle fact
pointer capture fact
gesture cancel/commit fact
grid/background fact
diagnostic overlay fact
large-content bounds fact
budget evidence fact
invalid transform expected-failure diagnostic
```

The minimum proof must be visible through package/catalog/inspection projection, runtime proof/report evidence, and static mount proof if those owners are selected.

## Future extensions, not Phase 16 minimum

```text
specialized spatial item layout
nodes and links
ports and sockets
graph editor commands
timeline tracks
curve editing
material graph semantics
SDF graph semantics
gameplay graph semantics
animation graph semantics
particle graph semantics
UI Designer authored mutations
Workbench/provider redesign
full app recipe/plugin framework
renderer backend resources
```

## Accessibility and input acceptance

Surface2D planning must define acceptance for:

```text
keyboard pan
keyboard zoom
keyboard fit-content
focus-visible surface state
screen-reader/inspection-readable surface name and bounds
reduced-motion behavior for animated navigation
pointer capture cancellation
wheel and high-resolution scroll input
trackpad pinch or explicit deferral
touch pan/zoom or explicit deferral
controller navigation or explicit deferral
```

If an input mode is deferred, the descriptor/report must say so explicitly instead of silently omitting it.

## Performance and budget evidence

Phase 16 must add a budget-evidence shape before implementation. Exact numbers may be conservative, but the evidence model must cover:

```text
transform projection cost
pan/zoom update cost
hover coordinate update cost
selection rectangle update cost
fit-content calculation cost
large-content bounds projection cost
runtime report generation cost
static mount report generation cost
primitive count or fact count budget
```

Initial budgets should be recorded as p95 targets for deterministic fixtures where practical. If wall-clock budgets are too early, use deterministic operation/fact-count budgets and record why.

## Validation envelope

Planning validation:

```text
python tools/docs/validate_docs.py
git diff --check
```

Implementation validation must be set before moving to `active-implementation`. Expected shape:

```text
cargo test -p ui_controls surface2d
cargo test -p ui_runtime surface2d
cargo test -p ui_static_mount surface2d
cargo test -p ui_render_data    # only if renderer-neutral primitive data changes
cargo test -p ui_render_primitives # only if primitive contracts change
cargo test --workspace
python tools/docs/validate_docs.py
git diff --check
```

The exact package list must match the accepted owner split.

## Consumers

Consumers are future validation targets, not Phase 16 implementation scope:

```text
Gallery catalog inspection
UI Designer canvas
SpatialCanvas
NodeCanvas
PortGraphCanvas
TrackSurface
Timeline
CurveEditor
Material graph views
SDF graph views
Gameplay graph views
Animation graph views
Particle graph views
future drawing/editor surfaces
```

## Out of scope

```text
graph semantics
timeline semantics
Gallery catalog semantics
editor commands
renderer resources
product/editor/game mutation
authored UI editing
full app composition
plugin framework implementation
foundation/meta
shared plugin primitives
```

## Stop conditions

Stop and redesign if implementation requires:

```text
Surface2D mutating product/editor/game truth
Surface2D owning graph or timeline semantics
Surface2D owning renderer backend resources
Surface2D bypassing package/catalog/inspection projection
Surface2D replacing ui_surface vocabulary without an accepted migration decision
Surface2D depending on Typed App Composition as implementation authority
shared plugin framework extraction
foundation/meta
host command execution inside domain/ui
```
