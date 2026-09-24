---
title: Drawing Domain Crate Design
description: Focused active design for the first implementation-ready domain/drawing crate contracts, ratification, composition graph, stroke model, paper model, and tile lineage.
status: active
owner: drawing
layer: domain
canonical: true
last_reviewed: 2026-05-10
related_docs:
  - ../../guidelines/runenwerk-architecture.md
  - ../../domain/drawing/README.md
  - ../../domain/graph/README.md
  - ../../domain/texture/README.md
related_designs:
  - ./drawing-authoring-and-comic-layout-platform-design.md
  - ./semantic-graph-ir-and-compilation-design.md
---

# Drawing Domain Crate Design

## Status

Active design. Phase 2 has now created the first `domain/drawing` crate slice.
This document remains the implementation boundary for that crate until a later
design supersedes it.

## Purpose

The drawing platform design defines the long-term product and workflow shape.
This focused design defines the first pure-domain contract set needed before
`runenwerk_draw`, tablet adapters, render tile formation, watercolor simulation,
or export adapters are implemented.

The first `domain/drawing` slice proves:

- drawing document identity and revision;
- ordered stroke truth;
- ink brush and paper descriptors;
- stack-first layer composition graph truth;
- safe live composition descriptors;
- tile identity and source lineage;
- ratification and diagnostics for invalid authored state.

## Boundary

`domain/drawing` owns engine-agnostic drawing truth and invariants.

It may depend on foundation vocabulary and lower-level domain contracts such as
`domain/graph` where that is the established neutral graph substrate. It must
not depend on engine runtime, app crates, native tablet adapters, concrete GPU
resources, Wacom APIs, renderer-private passes, or editor shell state.

The crate does not own:

- GPU resources;
- compute shader code;
- renderer-private pass execution;
- app shell state;
- Wacom or operating-system tablet APIs;
- `apps/runenwerk_editor` workflow state;
- raw shader graph authoring;
- watercolor simulation implementation;
- comic layout authority.

## First Implementation Slice

The first slice should add only reusable domain contracts, ratifiers,
diagnostics, and unit tests. It should not draw pixels yet.

Required public concepts:

- `DrawingDocument`;
- `LayerStackEntry`;
- `LayerStackEntryId`;
- `StrokeRecord`;
- `StrokeId`;
- `StrokeSample`;
- `BrushDescriptor`;
- `InkBrushDescriptor`;
- `BrushDynamics`;
- `PaperDescriptor`;
- `PaperHeightSource`;
- `DrawingCompositeGraph`;
- `DrawingCompositeNode`;
- `CompositePort`;
- `LayerStackNode`;
- `GroupNode`;
- `PaintLayerSource`;
- `PaperSource`;
- `ReferenceImageSource`;
- `MaskNode`;
- `ClipNode`;
- `TransformNode`;
- `AdjustmentNode`;
- `CompositeOutput`;
- `CanvasCoordinate`;
- `CanvasTileId`;
- `TilePyramidLevel`;
- `DrawingTileProduct`;
- `DrawingTileProductSource`;
- drawing ratification reports, issue codes, diagnostics, and source lineage.

`EffectNode` should be declared as a future semantic family boundary, but the
first implementation should only persist descriptor-ready safe live composition
nodes. Declared effect families are catalog or roadmap concepts and must not
appear as authored document state.

Phase 2 contracts should be serialization-ready DTO-style domain contracts with
schema and revision fields where needed. The implemented Phase 2 slice uses
plain versioned DTO shapes, but does not add serde derives or a codec yet.
Native package chunks, `domain/graph` serialization policy, package migration,
compression, external asset resolution, and file IO are later package-design
work.

## Phase Product And Acceptance

The Phase 2 product is a pure `domain/drawing` crate, not a visible drawing app.

Acceptance requires:

- a new pure `domain/drawing` crate exists and is registered in the workspace;
- developers can construct drawing document DTOs with strokes, brush and paper
  descriptors, stack entries, and a composition graph;
- the crate can ratify valid and invalid drawing documents and return stable
  diagnostics;
- the crate can represent tile product descriptor and lineage metadata without
  pixel payloads.

Closeout note: the implemented tile product constructor uses
`DrawingTileProductSource` as a small parameter object for quality, source
revision, output, lineage, formation version, and invalidation bounds. This
keeps product construction explicit without a long positional-argument API.

This phase is not visual yet:

- no canvas UI;
- no rendered ink pixels;
- no tablet input;
- no app shell;
- no native package IO;
- no export path.

The practical outcome is stable drawing contract truth for later phases. Phase
3 and later work can consume real document, stroke, stack, graph, ratification,
diagnostic, and tile-lineage types instead of placeholders.

## Module Layout

The crate should preserve subdomain modules:

```text
domain/drawing/src/
|-- lib.rs
|-- document/
|   |-- mod.rs
|   |-- model.rs
|   `-- revision.rs
|-- stroke/
|   |-- mod.rs
|   |-- sample.rs
|   `-- record.rs
|-- brush/
|   |-- mod.rs
|   |-- descriptor.rs
|   `-- dynamics.rs
|-- paper/
|   |-- mod.rs
|   |-- descriptor.rs
|   `-- height_source.rs
|-- composition/
|   |-- mod.rs
|   |-- graph.rs
|   |-- node.rs
|   |-- port.rs
|   |-- stack.rs
|   |-- group.rs
|   |-- mask.rs
|   |-- clip.rs
|   |-- transform.rs
|   |-- adjustment.rs
|   `-- output.rs
|-- tile/
|   |-- mod.rs
|   |-- coordinate.rs
|   `-- product.rs
|-- product_lineage/
|   |-- mod.rs
|   `-- source_map.rs
|-- history/
|   |-- mod.rs
|   `-- operation.rs
|-- ratification/
|   |-- mod.rs
|   `-- ratifier.rs
`-- diagnostics/
    |-- mod.rs
    `-- issue.rs
```

Module names should describe subdomain responsibility. Do not use catch-all
files such as `utils.rs` or `_internal` module suffixes.

## Document Model

`DrawingDocument` is the authored document root.

It should contain:

- document identity;
- document schema version;
- document revision;
- canvas coordinate contract;
- source registries for strokes, brushes, papers, and references;
- `DrawingCompositeGraph` as layer/composition truth;
- current active output reference;
- tile lineage metadata for derived products.

`LayerStackEntry` is the authored stack item owned by `LayerStackNode`.

`LayerStackEntryId` identifies a layer-facing stack entry that UI and commands
can target. It must be stable across reorder operations.

`DrawingLayer` may exist later only as a UI/read-model projection label over
source nodes and stack entries. It must not become authored domain truth or a
second layer authority.

## Stroke Model

`StrokeRecord` owns one committed stroke.

It should contain:

- `StrokeId`;
- target paint source or layer id;
- brush descriptor reference;
- ink/color reference or inline color descriptor;
- ordered `StrokeSample` list;
- stroke bounds in canvas coordinates;
- source revision metadata.

`StrokeSample` should preserve platform-neutral drawing input facts:

- canvas position;
- timestamp or sample sequence;
- pressure when available;
- tilt when available;
- twist when available;
- tool kind when available.

The first ratifier should reject samples with invalid coordinates, invalid
pressure ranges, non-finite values, non-monotonic sample ordering, missing target
paint source, or empty committed strokes.

## Brush And Paper

`BrushDescriptor` is the base authored brush descriptor.

`InkBrushDescriptor` should cover the deterministic ink MVP:

- size range;
- opacity range;
- flow range;
- pressure-to-size behavior;
- pressure-to-opacity behavior;
- edge softness;
- optional paper response fields that are ratified but may be ignored by early
  non-paper formation products.

`BrushDynamics` should be explicit data, not hidden runtime behavior.

`PaperDescriptor` should describe authored paper intent:

- paper identity;
- roughness;
- absorbency;
- height source reference;
- procedural seed where applicable.

`PaperHeightSource` should support descriptor references for future formed paper
products, procedural noise, imported height fields, and SDF-derived sources. The
first implementation may ratify references without forming paper products yet.

## Composition Model

`DrawingCompositeGraph` is the authored composition source of truth. It should
wrap `domain/graph::GraphDefinition` for graph structure.

It should be:

- typed;
- acyclic;
- semantic;
- ratified by `domain/drawing`;
- source-mapped to formed drawing products.

`domain/graph` owns only structural graph facts: graph, node, port, edge,
direction, port type compatibility, cycle policy, validation, and traversal.
`domain/drawing` owns drawing node kinds, port semantics, stack meaning, masks,
clips, transforms, adjustments, outputs, ratification, diagnostics, and source
maps.

The first safe live composition core is:

- `LayerStackNode`;
- `GroupNode`;
- `PaintLayerSource`;
- `PaperSource`;
- `ReferenceImageSource`;
- `MaskNode`;
- `ClipNode`;
- `TransformNode`;
- `AdjustmentNode`;
- `CompositeOutput`.

`LayerStackNode` is the v1 layer-stack authority. Stack UI is a command and
projection surface over this node, not a separate layer model.

`GroupNode` owns group metadata, group mask, group clipping, isolation policy,
and a child `LayerStackNode`. The first supported group behavior is isolated
group blending. Pass-through group blending is deferred.

`ClipNode` clips by alpha coverage in the first slice. Luma clips, channel
clips, and material-map clips are later extensions.

`MaskNode` produces alpha mask semantics in the first slice. Rich mask channels
are later extensions.

`TransformNode` should carry non-destructive 2D transform intent for source
placement. Formation owns resampling details later.

`AdjustmentNode` should support cheap deterministic color adjustments first:

- opacity;
- brightness/contrast;
- HSV;
- threshold;
- channel remap;
- simple gradient map.

Curves, painterly effects, natural-media effects, and decorative finish effects
remain future effect families until descriptor, preview, final, and shippable
maturity contracts exist.

`CompositeOutput` declares output semantics explicitly:

- final canvas color;
- preview color;
- alpha or mask output;
- material-map output where supported later.

Outputs without explicit color, alpha, mask, or material-map semantics should be
rejected.

## Effect Maturity

Effect maturity controls what can be saved:

- Declared: roadmap or catalog only. Must not appear as authored document state.
- Descriptor: serializable semantic node contract exists. This is the first tier
  that may be saved in native documents.
- Preview: low-latency preview formation works. Preview products are derived.
- Final: deterministic final-quality tile or export formation works. Final
  products are derived.
- Shippable: diagnostics, invalidation, export degradation, tests, and
  performance budgets are stable.

The first crate slice should model this policy in descriptors and ratifiers even
if heavy effect nodes are not implemented.

## Tile And Lineage Model

`CanvasCoordinate` is stable logical drawing space. It is not a screen pixel and
not a GPU texture coordinate.

`TilePyramidLevel` identifies a bounded multiscale product level.

`CanvasTileId` identifies a tile in canvas space and pyramid level.

`DrawingTileProduct` is derived descriptor and lineage metadata in Phase 2. It
does not contain pixels, image buffers, GPU handles, or renderer resources.

It should include:

- product identity;
- tile id;
- quality class such as preview or final;
- source document revision;
- source graph/output reference;
- source layer or stroke ranges where applicable;
- brush and paper revisions where applicable;
- formation version;
- invalidation bounds;
- last-good relationship when a newer product fails.

Tile products must never be document truth. They are cacheable and rebuildable.

## Ratification

The first ratifier should reject:

- invalid stroke samples;
- empty committed strokes;
- invalid brush descriptor ranges;
- invalid paper descriptor ranges;
- missing paper height references;
- invalid layer ordering;
- composition graph cycles;
- incompatible composite ports;
- missing required node inputs;
- ambiguous paint targets;
- outputs without explicit semantics;
- `Declared` effect families saved as document state;
- invalid tile lineage;
- tile products with missing source revision or formation version.

Diagnostic issue families should be domain-owned and stable. Suggested first
families:

- `drawing.stroke.*`;
- `drawing.brush.*`;
- `drawing.paper.*`;
- `drawing.composition.*`;
- `drawing.tile.*`;
- `drawing.lineage.*`.

## Mutation Boundary

The first crate should expose domain-owned command or builder pathways for
normal edits instead of encouraging direct mutation of internals.

The concrete mutation style for Phase 2 is domain-owned command enums plus
transaction or builder helpers. `foundation/commands` descriptors may be added
later for editor discovery and tool-surface integration, but the first
implementation should not require that integration.

Initial command families should cover:

- begin, append, and commit stroke;
- create, rename, reorder, show, hide, and remove stack entries;
- create isolated group;
- attach or remove mask;
- add or remove alpha clip;
- set layer opacity or blend mode;
- set cheap adjustment descriptor;
- select composite output.

Commands should produce accepted domain state or structured diagnostics. UI and
app layers may present these commands, but they do not own the semantics.

## Tests

The first implementation should add behavior-named unit tests. `cargo test -p
drawing` should prove:

- invalid stroke samples are rejected;
- non-monotonic stroke samples are rejected;
- empty committed strokes are rejected;
- brush ranges are ratified;
- paper ranges and references are ratified;
- layer stack ordering is deterministic;
- stack entry identity is stable through reorder;
- no independent `DrawingLayer` authority is required or accepted;
- `GroupNode` contains a child `LayerStackNode`;
- pass-through groups are rejected until supported;
- `DrawingCompositeGraph` preserves structural graph validation and drawing
  semantic ratification as separate checks;
- composition graph cycles are rejected;
- incompatible composite ports are rejected;
- ambiguous paint targets are rejected;
- outputs without semantics are rejected;
- `Declared` effects cannot be saved as document state;
- tile product keys preserve source revision and formation version;
- tile product descriptors carry lineage metadata without pixel payloads;
- commands and transactions produce accepted state or structured diagnostics;
- last-good tile lineage is represented without making tile cache authoritative.

## Implementation Gate

Further `domain/drawing` changes must stay within this design until a focused
follow-up design supersedes it:

- pure domain contracts;
- no app shell;
- no renderer dependency;
- no native tablet dependency;
- no Wacom API;
- no watercolor simulation;
- no export adapter;
- no raw shader graph authority.

If implementation needs one of those dependencies, stop and split a follow-up
design first.
