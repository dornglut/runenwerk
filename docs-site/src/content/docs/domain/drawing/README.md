---
title: Drawing Domain
description: Current ownership boundary for pure drawing documents, strokes, brushes, paper descriptors, composition graph contracts, command DTOs, ratification, diagnostics, deterministic ink tile products, and tile lineage.
status: active
owner: drawing
layer: domain
canonical: true
last_reviewed: 2026-05-19
related_docs:
  - ../../design/active/drawing-domain-crate-design.md
  - ../../design/active/drawing-authoring-and-comic-layout-platform-design.md
  - ../graph/README.md
---

# Drawing Domain

`domain/drawing` owns the engine-agnostic drawing contract slice for the
drawing platform roadmap. It defines authored drawing document DTOs, committed
stroke truth, brush and paper descriptors, stack-first composition graph
semantics, domain-owned commands, ratification reports, diagnostic code
families, deterministic CPU ink tile products, product-substrate helpers, and
derived tile-product lineage metadata.

The crate is not a drawing application. It can form deterministic CPU tile
payloads from committed stroke truth and from non-authoritative preview stroke
facts through the same rasterization path, but it does not own runtime
presentation, GPU upload, native tablet APIs, package IO, or app workflow
state.

## Tile Ownership

`domain/drawing` owns deterministic CPU tile formation, tile metadata,
lineage, invalidation, source cache key construction, and product-substrate
descriptor helpers. It does not own app-visible tile lifecycle or runtime cache
state.

App/runtime layers own current/preview/final visibility, dynamic texture upload
tracking, runtime cache acceptance, cache policy, GPU validation,
fallback/promotion decisions, and renderer submission. Drawing quality class
participates in product scale, descriptor generation, cache identity, and
render selection. Persistent caches and package sidecars remain app/package or
engine-runtime work according to the product cache roadmap.

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
- `DrawingTileFormationPolicy`, `DrawingInkTilePayload`,
  `DrawingInkTileProduct`, `DrawingInkTileFormation`,
  `DrawingInkTileInvalidation`, `DrawingInkPreviewStroke`,
  `form_drawing_ink_tiles`, `form_drawing_ink_tiles_for_ids`,
  `form_drawing_ink_preview_tiles`,
  `form_drawing_ink_preview_tiles_for_ids`, and
  `drawing_ink_tile_invalidation_for_strokes` for deterministic committed and
  live preview CPU ink tile formation;
- `drawing_committed_ink_tile_source_cache_key` for app/runtime cache lookup
  without moving payload cache policy into the drawing domain;
- drawing ink tile product helpers that build `domain/product`
  `ProductJobDescriptor`, `ProductDescriptorCore`, `ProductPublicationOutcome`,
  and `QuerySnapshotProductDescriptor` values for renderer strict consumption;
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
- native drawing package IO;
- OpenRaster, PSD, Blender, or web reader export;
- watercolor, decorative finish, or live effect formation;
- comic layout authority.

## Ratification

All externally supplied or command-produced drawing state should be ratified
before acceptance. The ratifier rejects invalid stroke samples, empty
committed strokes, invalid brush and paper ranges, missing references, invalid
layer stack state, graph structural errors, missing drawing semantics,
pass-through groups, outputs without semantics, declared effects saved as
authored state, and invalid tile lineage. Ink tile formation calls this ratifier
before producing payloads and fails closed for invalid documents, invalid
formation policy, unsupported eraser-only strokes, and oversized requested tile
batches. Whole-stroke invalidation can report more tiles than one interactive
batch; app/runtime callers should split those tile ids and call the bounded
formation APIs.

## Serialization Status

The current contracts are DTO-shaped and include schema or revision fields where
the Phase 2 design requires them, but the crate does not yet define serde
derives, a codec, or native package IO. Those decisions remain deferred until
the native package design also defines how wrapped `domain/graph` structures are
serialized and migrated.

## Next Phase

The next drawing phases should build on the existing command and product path:
paper response, native package persistence, persistent tile cache policy,
broader GPU acceleration work, eraser compositing, and richer layer/effect
formation. Native tablet and Wacom details still belong outside this crate.
