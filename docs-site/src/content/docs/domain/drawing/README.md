---
title: Drawing Domain
description: Current ownership boundary for pure drawing documents, strokes, brushes, paper descriptors, composition graph contracts, command DTOs, ratification, diagnostics, and tile-lineage metadata.
status: active
owner: drawing
layer: domain
canonical: true
last_reviewed: 2026-05-10
related_docs:
  - ../../design/active/drawing-domain-crate-design.md
  - ../../design/active/drawing-authoring-and-comic-layout-platform-design.md
  - ../graph/README.md
---

# Drawing Domain

`domain/drawing` owns the engine-agnostic drawing contract slice for Phase 2 of
the drawing platform roadmap. It defines authored drawing document DTOs,
committed stroke truth, brush and paper descriptors, stack-first composition
graph semantics, domain-owned commands, ratification reports, diagnostic code
families, and derived tile-product lineage metadata.

The crate is not a drawing application and does not render pixels.

## Current Scope

The current public API covers:

- `DrawingDocument` identity, schema version, revision, canvas bounds, source
  registries, composition graph, committed strokes, pending stroke command
  state, and tile product descriptors;
- `StrokeRecord`, `StrokeSample`, `BrushDescriptor`, `InkBrushDescriptor`,
  `BrushDynamics`, `PaperDescriptor`, and `PaperHeightSource`;
- `DrawingCompositeGraph` as a drawing-owned semantic wrapper around
  `domain/graph::GraphDefinition`;
- `LayerStackEntry` and `LayerStackNode` as the v1 layer-facing authority;
- safe composition descriptors for paint sources, paper sources, reference
  images, masks, clips, isolated groups, transforms, cheap adjustments, effects
  by maturity tier, and explicit composite outputs;
- `DrawingTileProduct` and `DrawingTileProductSource` descriptors with source
  revision, output reference, quality class, formation version, invalidation
  bounds, lineage, and last-good fallback metadata;
- `DrawingCommand` and `DrawingTransaction` for normal mutations;
- `ratify_drawing_document` and drawing-owned issue codes/diagnostic codes.

## Ownership Boundary

`domain/drawing` may depend on foundation vocabulary and lower-level domain
contract crates such as `domain/graph`. It must stay independent from runtime,
apps, native tablet adapters, Wacom APIs, renderer-private execution, GPU
resources, package IO, export adapters, and editor shell state.

The crate intentionally does not own:

- `apps/runenwerk_draw`;
- tablet or Wacom input;
- GPU or compute shader execution;
- ink tile pixel formation;
- native drawing package IO;
- OpenRaster, PSD, Blender, or web reader export;
- watercolor, decorative finish, or live effect formation;
- comic layout authority.

## Ratification

All externally supplied or command-produced drawing state should be ratified
before acceptance. The Phase 2 ratifier rejects invalid stroke samples, empty
committed strokes, invalid brush and paper ranges, missing references, invalid
layer stack state, graph structural errors, missing drawing semantics,
pass-through groups, outputs without semantics, declared effects saved as
authored state, and invalid tile lineage.

## Serialization Status

The current contracts are DTO-shaped and include schema or revision fields where
the Phase 2 design requires them, but the crate does not yet define serde
derives, a codec, or native package IO. Those decisions remain deferred until
the native package design also defines how wrapped `domain/graph` structures are
serialized and migrated.

## Next Phase

The next roadmap phase is stylus input contracts. That phase should consume
`domain/drawing` stroke and command types, but it must not move native tablet or
Wacom details into this crate.
