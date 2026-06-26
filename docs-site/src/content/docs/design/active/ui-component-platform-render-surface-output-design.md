---
title: UI Component Platform Render Surface Output Design
description: Owner-first Phase 10 planning design for renderer-neutral output contracts, render evidence, and surface output summaries.
status: active
owner: ui_render_data
layer: domain
canonical: true
last_reviewed: 2026-06-26
related_designs:
  - ./ui-component-platform-ownership-realignment-design.md
  - ./ui-component-platform-layout-container-virtualization-design.md
  - ./ui-runtime-rendering-pipeline-roadmap.md
  - ./runenwerk-ui-story-driven-golden-workflow-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
---

# UI Component Platform Render Surface Output Design

## Status

This is the planning and acceptance design for `PT-UI-COMPONENT-PLATFORM-010-PLANNING`.

It opens Phase 10 after the completed owner-first Phase 9 closeout. It does not authorize Rust implementation, renderer behavior, runtime behavior, or mount eligibility changes by itself.

## Purpose

Phase 10 defines how reusable controls describe render/surface/output evidence without moving generic renderer or output vocabulary into `ui_controls`.

The design preserves the Phase 9 ownership correction:

```text
Owning crates define reusable UI vocabulary and contracts.
ui_controls defines per-control requirements and summaries that reference those contracts.
Catalog/inspection exposes read-only summaries.
Runtime, renderer, apps, editor, and game execute behavior later.
```

## Current owner investigation

Repository authority and current code place output ownership in multiple direct owners:

```text
ui_render_data
  owns renderer-facing UiFrame, UiSurface, UiLayer, UiPrimitive, primitive
  families, draw keys, sort keys, product surface primitives, and viewport
  surface embed payload contracts.

ui_runtime
  owns retained-tree/runtime output generation into UiFrame data.

ui_surface
  owns semantic surface, mount, observation, session, presentation, intent,
  capability, and ratification contracts. It does not own renderer primitive
  output.

engine/src/plugins/render
  owns backend renderer extraction, GPU/backend resources, renderer execution,
  and draw behavior.

apps/editor/game hosts
  own product data, runtime resources, concrete host state, persistence,
  fixture loading, and command execution.

ui_controls
  owns control packages, control kinds, per-control render evidence
  requirements, and read-only summaries that reference owner-crate output
  contracts.
```

The Phase 10 owner is therefore `ui_render_data` for renderer-facing output vocabulary. `ui_runtime` and `engine/src/plugins/render` are adjacent execution owners. `ui_controls` is not the source of truth for generic render/output vocabulary.

## Decision

Phase 10 must be owner-first:

```text
010A Render Output Owner Map
  planning/design only;
  confirm output vocabulary owner split and future slice boundaries.

010B Renderer-Neutral Output Foundation
  future implementation owner: ui_render_data;
  add or refine reusable renderer-neutral output contracts only if the
  existing UiFrame/UiPrimitive/ProductSurface contracts are insufficient.

010C Runtime Output Evidence Bridge
  future implementation owner: ui_runtime or an accepted runtime-view/output
  owner;
  derive output evidence from runtime artifacts/views/retained tree output,
  not from authored files or control package descriptors alone.

010D Control Render Evidence Bridge
  future implementation owner: ui_controls;
  add per-control render requirements/summaries that reference ui_render_data
  and story/runtime evidence. Do not define primitive/source-of-truth output
  vocabulary here.

010E Backend Proof
  future implementation owner: engine/src/plugins/render and app/story host;
  prove backend consumption of renderer-neutral output without renderer-owned
  UI semantics.
```

This planning pass opens 010A only. Later slices require explicit implementation planning.

## Vocabulary ownership

### `ui_render_data`

`ui_render_data` owns renderer-facing output contracts such as:

```text
UiFrame
UiSurface
UiLayer
UiPrimitive
RectPrimitive
BorderPrimitive
ClipPrimitive
ImagePrimitive
GlyphRunPrimitive
StrokePrimitive
ProductSurfacePrimitive
ProductSurfaceTextureBindingSource
ViewportSurfaceEmbedPrimitive
UiDrawKey
UiSortKey
```

Future Phase 10 additions, if needed, should also start here when they describe renderer-neutral output facts:

```text
render output provenance
expected primitive counts
primitive family summaries
surface output summaries
render-data diagnostics
texture binding summaries
layer/sort ordering summaries
```

These are output facts, not control semantics and not backend renderer behavior.

### `ui_runtime`

`ui_runtime` owns conversion from retained/runtime UI state into renderer-facing frame data.

It may produce evidence that a runtime view or retained tree emitted expected output, but it must not own backend renderer resources or authored control package semantics.

### `ui_surface`

`ui_surface` owns semantic surface and mount contracts. It may be referenced when Phase 10 needs surface identity or presentation context, but renderer primitive output stays in `ui_render_data`.

### `engine/src/plugins/render`

Engine render owns execution:

```text
UI frame extraction
prepared UI payloads
GPU/backend resources
texture upload/materialization
draw pass behavior
backend diagnostics
```

It must not own UI source truth, control package truth, route semantics, app/editor/game mutation, or authored definition semantics.

### `ui_controls`

`ui_controls` may describe only per-control output requirements and summaries:

```text
ControlRenderEvidenceRequirement
ControlRenderCapabilitySummary
ControlRenderInspectionFact
per-control expected output evidence ids
per-control required primitive families by reference
per-control story/runtime evidence links
catalog/inspection projection
```

`ui_controls` must not define source-of-truth generic output vocabulary such as primitive families, draw keys, surfaces, frame ordering, renderer diagnostics, texture binding semantics, or backend execution concepts.

## Required bridge shape

The bridge should follow the Phase 9 pattern:

```text
owner crate vocabulary
  -> per-control descriptor or evidence requirement
  -> read-only capability summary
  -> catalog/inspection projection
  -> story/runtime evidence gate
```

`ui_controls` may only reference owner-crate types once those contracts exist and are accepted.

## Non-goals

Do not implement in this phase:

```text
Rust code for Phase 10
renderer behavior
runtime behavior
mount eligibility changes
backend materialization
GPU upload changes
layout execution
new control packages
new story runner behavior
first-class ControlInspectionSection::Layout cleanup
generic render/output vocabulary in ui_controls
direct rendering from authored .ron files
rendering from ControlPackageDescriptor alone
renderer-owned UI semantics
app/editor/game mutation
```

## Acceptance criteria for this planning pass

`PT-UI-COMPONENT-PLATFORM-010-PLANNING` is ready for implementation split only when:

```text
- Phase 9 is marked complete in planning records with user validation evidence.
- PR #30 and feature/ui-component-platform-009-layout remain superseded and unused.
- The render/output owner split is recorded.
- The design names ui_render_data as renderer-facing output vocabulary owner.
- The design names ui_runtime and engine render as adjacent execution owners.
- The design states ui_controls is bridge-only.
- Planning records make Phase 10 active planning, not implementation.
- No Rust implementation is included.
```

## Future implementation validation envelope

When implementation is explicitly authorized later, expected validation should be narrowed by slice. Likely commands include:

```text
cargo fmt --all --check
cargo check -p ui_render_data
cargo check -p ui_runtime
cargo check -p ui_controls
cargo test -p ui_render_data
cargo test -p ui_runtime output
cargo test -p ui_controls control_render
cargo test -p ui_story
git diff --check
```

Backend proof slices should add engine/app/story validation only when they touch those owners.

## Stop conditions

Stop and redesign if any Phase 10 slice would:

```text
add generic render/output vocabulary to ui_controls;
make ui_controls own primitive families, draw keys, surface ordering, or texture binding semantics;
render from authored files directly;
render from ControlPackageDescriptor alone;
make renderer code infer control semantics from strings;
make engine render own UI package, route, host mutation, editor, game, or product semantics;
change runtime mount eligibility without story/runtime evidence;
add backend behavior before renderer-neutral output evidence exists;
reuse PR #30 or feature/ui-component-platform-009-layout.
```

## Next step

Review this design. If accepted, split Phase 10 into owner-first implementation slices. Start with `ui_render_data` only if existing renderer-facing output contracts are insufficient for the required evidence model.
