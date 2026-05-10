---
title: Drawing Authoring and Comic Layout Platform Design
description: Active design for Runenwerk drawing, ink, watercolor, paper simulation, tablet input, multiscale canvas tiles, and later comic layout workflows.
status: active
owner: workspace
layer: cross-domain
canonical: true
last_reviewed: 2026-05-10
related_docs:
  - ../../guidelines/runenwerk-architecture.md
  - ../../domain/graph/README.md
  - ../../domain/material-graph/README.md
  - ../../domain/texture/README.md
  - ../../domain/ui/README.md
  - ../../domain/editor/README.md
related_designs:
  - ./drawing-domain-crate-design.md
  - ./semantic-graph-ir-and-compilation-design.md
  - ./editor-ui-workspace-tool-surface-architecture.md
  - ./render-fragment-data-driven-maturity-design.md
  - ./viewport-dynamic-product-target-allocation-design.md
  - ./render-product-surface-foundation-bundle-design.md
---

# Drawing Authoring and Comic Layout Platform Design

## Status

Active design. This document records the target architecture for a future
Runenwerk drawing product surface. It does not describe an implemented drawing
crate or app yet.

## Purpose

Runenwerk should grow toward a drawing program that can support:

- deterministic pressure-sensitive ink;
- ink colors and brush dynamics;
- paper grain, height, noise, and SDF-derived surface information;
- watercolor-style wetness, pigment movement, bleed, drying, and edge behavior;
- practical infinite-feeling zoom;
- Wacom and native tablet workflows;
- later manga, webcomic, text, balloon, panel, and page layout workflows.

This design defines the long-term ownership boundaries before those features are
implemented. The main goal is to avoid turning existing material graph, SDF world
brush, render, or editor shell concepts into accidental drawing architecture.

## Strategic Decision

Runenwerk should add a sibling product app named `runenwerk_draw`.

`runenwerk_draw` should be drawing-focused, but it should reuse the existing
workspace, UI, input, render, and runtime substrate where those contracts are
already reusable. It should not be a detached rewrite. It should also not force
drawing-specific tablet, tile, paint, paper, and canvas authority into
`apps/runenwerk_editor`.

The relationship is:

```text
shared domain/UI/render/editor substrate
  -> apps/runenwerk_editor for existing editor workflows
  -> apps/runenwerk_draw for drawing and comic authoring workflows
```

The main editor may later host drawing surfaces or launch drawing workflows, but
the drawing app keeps a product-level boundary for canvas-first interaction.

## Current Repository Anchors

Implemented anchors that this design must respect:

- `domain/graph` owns the neutral graph substrate only.
- `domain/material_graph` owns authored material graph semantics, ratification,
  lowering, and formed material products.
- `domain/texture` owns texture product descriptors, not GPU upload or shader
  binding policy.
- `domain/drawing` owns pure drawing document, stroke, brush, paper,
  layer/composition graph, command, ratification, diagnostic, and tile-lineage
  contracts.
- `domain/ui/*` owns reusable UI contracts, input vocabulary, stylus-capable
  pointer packets, layout, text, render data, surface mounting, definitions,
  trees, runtime, and widgets.
- `domain/editor/*` owns editor-facing contracts, workspace projections,
  document metadata, viewport semantics, shell composition, preview contracts,
  inspector models, and persistence boundaries.
- `engine/src/plugins/render` already has render-flow concepts that include
  compute, fullscreen, graphics, copy, and present passes.
- `apps/runenwerk_editor` wires the current editor runtime and should remain the
  existing editor app rather than becoming the semantic owner of drawing.
- `adapters/native_tablet_input` owns the first macOS/Wacom-oriented tablet
  packet normalization proof into platform-neutral `ui_input` events.
- Existing SDF authoring and `world_ops` brush concepts are field/world editing
  operations, not natural-media drawing strokes.

Remaining missing anchors today:

- no rendered ink tile formation or drawing product payloads;
- no `runenwerk_draw` product app;
- no comic/page layout domain.

## Core Doctrine

Drawing authoring follows the repository doctrine:

```text
authored drawing intent
  -> domain-owned ratification
  -> formed drawing products
  -> engine/app runtime presentation
  -> observation and diagnostics
```

Drawing source of truth must remain reconstructable and inspectable. Runtime
tile products and GPU resources are formed products, not the only authority.

## Non-Goals

The first architecture slice must not:

- make `domain/material_graph` a general drawing, brush, watercolor, or shader
  graph system;
- make `domain/graph` own drawing semantics;
- reinterpret existing SDF/world CSG brushes as drawing strokes;
- make `apps/runenwerk_editor` the owner of drawing document, paint simulation,
  tablet, or tile invariants;
- introduce a raw universal `shader_graph` as authored source of truth;
- require every mark to remain fully vector/procedural;
- implement full watercolor simulation before deterministic ink and tile
  formation are proven;
- put native Wacom or operating-system details into pure domain crates.

## Product Shape

### `runenwerk_draw`

The future app should own product wiring for:

- canvas-first workspace layout;
- drawing document opening, saving, and persistence integration;
- tablet routing and fallback pointer routing;
- brush, color, layer, paper, and tile preview surfaces;
- render target composition for drawing surfaces;
- import/export workflows for drawing and comic outputs.

It should reuse editor/UI/runtime substrate where possible, especially:

- UI widgets and theme contracts;
- tool surface and workspace concepts where they are generic;
- render product surface and dynamic target contracts;
- diagnostics and inspection patterns;
- persistence patterns once drawing document formats exist.

### Relationship to the main editor

The main editor should remain the modular host for existing editor workflows.
It may later expose drawing documents or embedded drawing surfaces, but that is a
hosting concern. Drawing semantics belong in drawing domains, not in the editor
shell.

### Workflow packaging strategy

Workflow implementation should be tool-surface first and app-shell second.

Runenwerk should build each workflow once around domain-owned semantics,
provider surfaces, command paths, observation models, diagnostics, and formed
products. Those workflow surfaces can then be hosted in:

- `runenwerk_editor` as the full integrated host;
- a focused standalone app when product UX or shipping needs justify it;
- later packaged editor profiles or specialized editor bundles.

The rule is:

```text
domain semantics + tool surfaces
  -> editor-hosted workflow
  -> focused app host only when needed
```

`runenwerk_draw` is the early focused app because canvas-first layout, tablet
latency, native tablet setup, and direct app distribution are product-critical.
Future `runenwerk_ui_designer` and `runenwerk_material_lab` should be thin
packaged hosts over the same UI-definition and material-graph tool surfaces, not
separate implementations. UI authoring should start inside `runenwerk_editor`.
Material authoring should also start editor-hosted, with a standalone material
lab only if preview/export workflows become useful outside the full editor.

### Standalone shipping

`runenwerk_draw` should ship as a standalone application binary, not only as a
tool surface inside `runenwerk_editor`.

The repository does not yet have a complete desktop app packaging pipeline. The
future shipping path should be explicit:

- development run through `cargo run -p runenwerk_draw`;
- release build through `cargo build -p runenwerk_draw --release`;
- macOS `.app` bundle generation;
- macOS code signing and notarization;
- macOS `.dmg` distribution;
- first public release as a signed and notarized direct-download macOS app
  outside the Mac App Store;
- later Windows signed installer;
- later Linux AppImage, package, or archive distribution;
- later Mac App Store evaluation only after native tablet, package IO, and
  sandbox constraints are understood.

Native tablet integration affects shipping. Operating-system permissions, Wacom
SDK linkage, signing requirements, and capability diagnostics belong in the
native tablet adapter and app packaging layer, not in `domain/drawing`.

The MVP platform target is macOS with Wacom/native tablet support and mouse
fallback. Windows tablet support is important, but it should not block the first
macOS-focused public release.

### Drawing app experience

The drawing app should feel canvas-first rather than like a general editor with
a drawing panel attached.

The first experience target should prioritize:

- a large central canvas with minimal chrome;
- low-latency stroke feedback;
- a low-latency preview stroke path while ratified tile products form;
- Wacom/native tablet detection;
- tablet setup and diagnostics;
- pressure test pad;
- tilt preview;
- cursor offset and pressure calibration;
- eraser and barrel-button mapping;
- brush library, favorites, recents, search, and preview strokes;
- paper preset gallery with grain, height, and lighting previews;
- ink/color swatches, opacity, flow, and wet/dry state display;
- a normal layer-stack panel backed by semantic composition contracts;
- paint layers, reference layers, paper layers, effect layers, and masks;
- hidden or advanced graph inspection for layer composition once the graph model
  is stable;
- visible autosave, recovery, and operation history state;
- tile rebuild status and low-resolution preview while high-resolution tiles
  form;
- export profile selection for PNG, OpenRaster, texture sets, and later reader
  bundles.

Advanced simulation controls should be hidden by default until brush and paper
presets are stable. The first UI should expose artist-facing controls such as
flow, spread, staining, granulation, dry-brush behavior, and paper absorbency.

## Truth Model

The canonical representation is hybrid:

```text
editable strokes + brush/paper recipes + deterministic raster tile products
```

This is intentionally not full vector and not raw raster only.

### Editable authored truth

The drawing domain should retain:

- document metadata;
- semantic layer/composition graph intent, including `LayerStackNode` state,
  visibility, opacity, compositing, clipping, group, mask, adjustment, and
  effect intent;
- ordered stroke records;
- stroke samples with time, position, pressure, tilt, and tool data;
- brush descriptors;
- ink descriptors and color data;
- paper descriptors;
- procedural paper/noise/SDF references;
- tile product lineage and invalidation metadata.

### Formed raster products

The drawing domain should form deterministic tile products for:

- ink coverage;
- pigment/color accumulation;
- alpha and mask data;
- paper interaction;
- wetness or drying state later;
- preview/composite products;
- multiscale zoom products.

Tiles are runtime-friendly and cacheable. They must preserve source lineage back
to document, layer, stroke, brush, paper, and formation version.

### Tile product contract

Tile products need explicit identity before implementation.

The drawing domain should define:

- stable canvas coordinate space;
- tile size policy;
- tile pyramid level identity;
- tile product key;
- source document revision;
- source layer and stroke ranges;
- source brush and paper revisions;
- formation version;
- invalidation bounds;
- preview versus export quality class.

Tile caches are derived state. Missing tiles should trigger rebuilds. Stale
tiles should be rejected or replaced through explicit lineage checks, not used
silently.

### Vector-owned workflows

Vector or layout authority remains appropriate for:

- comic panels;
- page frames;
- webcomic flow;
- speech balloons;
- text boxes;
- rulers, guides, crop marks, and layout handles;
- transformable non-paint objects.

Those concepts should not be forced into paint tiles unless raster export or
preview requires it.

### Undo and history model

The native package should support flexible history without making history the
only source of truth.

Required layers:

- current authoritative document state;
- optional versioned operation log for undo, redo, review, and reconstruction;
- autosave or session recovery state;
- derived tile cache state.

Save profiles should allow:

- compact save with current state only;
- working save with current state and recent operation history;
- archive save with current state, full operation log, and optional embedded
  tile cache.

Undo and redo should operate on semantic drawing commands where possible:

- stroke begin, append, and commit;
- stroke erase or edit;
- layer create, delete, reorder, rename, and visibility changes;
- brush, ink, and paper assignment changes;
- tile invalidation generated from accepted mutations.

Tile caches must never be the undo authority. Undo restores document state and
then invalidates or rebuilds affected tiles.

## Layer and Composition Model

Black Ink by Bleank is useful prior art for non-destructive layer workflows:
its Layer Editor uses a node graph, and its Layer Stack operator preserves a
familiar stacked layer interaction inside that graph. Runenwerk should take the
architectural lesson, not the exact product model.

References:

- [Black Ink Layer Editor](https://blackink.bleank.com/Learn/Doc/UI/Panels/LayerEditor/)
- [Black Ink Operators](https://blackink.bleank.com/Learn/Doc/Operators/)
- [Black Ink Layer Stack](https://blackink.bleank.com/Learn/Doc/Operators/LayerStack/)

The long-term drawing layer source of truth should be:

```text
paint/layer sources + semantic composition graph + deterministic formed tile products
```

The composition graph should be typed, acyclic, semantic, ratified, and owned by
`domain/drawing`. It may reuse `domain/graph` for neutral graph structure, but
the drawing domain owns node meaning, port types, ratification, diagnostics,
formation, and source maps.

The graph should not be a raw shader graph. Authors describe drawing and layer
intent. The drawing domain ratifies it. Formation lowers it to tile, preview,
material-map, or export products. Compute shaders remain valid backend
formation targets, not authored graph truth.

Future contracts should include:

- `DrawingCompositeGraph` for the authored composition document;
- `DrawingCompositeNode` for semantic drawing composition nodes;
- `CompositePort` for typed input and output channels;
- `LayerStackNode` for ordered artist-facing layer stacks;
- `PaintLayerSource` for editable paint and stroke layer sources;
- `PaperSource` for authored paper descriptors or formed paper products;
- `ReferenceImageSource` for non-authoritative image references;
- `MaskNode` for reusable mask composition;
- `ClipNode` for clipping one source by another;
- `GroupNode` for group metadata, masks, clipping, isolation policy, and a child
  `LayerStackNode`;
- `TransformNode` for non-destructive transforms;
- `AdjustmentNode` for color, tone, threshold, or channel adjustments;
- `EffectNode` for deterministic non-destructive effects;
- `CompositeOutput` for final canvas, preview, export, and material-map
  outputs.

`LayerStackNode` should be the default v1 operator and the layer-stack authority
inside the composition graph. The normal layer panel in `runenwerk_draw` should
manipulate a `LayerStackNode` through commands, so common layer work remains
familiar: add a layer, reorder it, rename it, toggle visibility, change opacity,
choose supported blend modes, attach masks, add clipping masks, create isolated
groups, attach cheap adjustment layers, and select the active paint target.

The first safe live composition core should include:

- stack-backed layer composition;
- clipping masks;
- isolated group clipping and group masks;
- reusable masks;
- non-destructive transforms;
- cheap deterministic adjustment layers;
- preview and final quality tile classes;
- last-good products while recomputation catches up.

The first group blending mode should be isolated group blending. `GroupNode`
owns group metadata, group masks, group clipping, isolation policy, and a child
`LayerStackNode`. The child stack composites internally, then emits one typed
output into its parent stack. Photoshop-style pass-through groups can be added
later, but they should not be required for the first safe live composition
phase.

The first adjustment layers should be cheap deterministic color operators:
opacity, brightness/contrast, HSV, threshold, channel remap, and simple gradient
map. Curves, painterly effects, material finish effects, and natural-media
operators remain planned, but they should mature behind explicit descriptor,
formation, cache, and performance contracts.

The node graph view should stay hidden, experimental, or advanced until the
stack projection is stable. A user should not need to understand graph editing
to draw, ink, mask, or export a normal document.

The graph power becomes important later for:

- reusable masks shared by multiple layers or outputs;
- non-destructive adjustments and effects;
- procedural paper and texture inputs;
- transformable reference and paint sources;
- alternate export composites;
- material-map outputs such as height, normal, roughness, wetness, and pigment
  masks;
- comic-specific composites for print, CBZ, scrolling webcomic, and enhanced
  reader exports.

The layer-stack panel is a projection over semantic graph state. It must not
create hidden layer semantics outside the graph. If a graph can no longer be
faithfully represented as a simple stack, the UI should either show a bounded
advanced state, lock unsupported stack edits with diagnostics, or open a future
composition graph inspector.

Composition graph ratification should reject:

- cycles;
- missing required inputs;
- incompatible port types;
- unsupported blend modes or effect nodes for the target formation profile;
- ambiguous paint targets;
- nondeterministic effect parameters;
- outputs without explicit color, alpha, mask, or material-map semantics;
- graph constructs that cannot report source lineage to formed products.

Live non-destructive effects should be normal editing behavior, not an
apply-and-bake workflow. Manual bake, freeze, or flatten operations may be added
later as explicit optimization or export tools, but they must not be required to
keep a document editable.

Live effect formation should use:

- fast interactive preview tiles;
- asynchronous final-quality tile formation;
- export-forced final-quality formation;
- last-good tiles while recomputation catches up;
- tile-level invalidation instead of whole-document recomputation where
  possible;
- explicit quality class, formation version, and source lineage in every formed
  product.

Every non-trivial effect node should define:

- explicit inputs and outputs;
- deterministic parameters and seed policy;
- formation version;
- target capability checks;
- no hidden runtime resources;
- tile invalidation bounds;
- source-lineage reporting.

Native project packages should preserve the full composition graph. Layered
interchange formats should receive an adapter projection. Unsupported graph
constructs must be flattened, degraded with diagnostics, or rejected by the
chosen export profile. They must never be silently discarded.

## Effect Families and Maturity Tiers

Long-term effect work should be planned as first-class semantic effect families,
not as one-off filters or raw shader snippets. Effects may be hidden,
experimental, or unavailable in early UI, but the native document model should
avoid dead ends that would make live painterly, technical, or decorative effects
impossible later.

Planned effect families:

- Core live effects: opacity, blend, clipping, masks, isolated groups,
  transforms, and basic adjustments.
- Painterly effects: ink bleed, edge darkening, pigment pooling, dry-brush
  breakup, feathering, and bristle or nib artifacts.
- Watercolor effects: wetness, diffusion, granulation, backruns, staining,
  evaporation, and pigment separation.
- Decorative and material finish effects: metallic ink, gold leaf, foil,
  holographic foil, glitter, pearlescent ink, varnish, gloss, spot-UV-style
  masks, and emboss/deboss.
- Paper and material effects: paper height, SDF or noise grain, pigment deposit
  height, generated normals, roughness maps, and wetness maps.
- Technical and procedural effects: halftone, screentone, speed lines, hatch
  fields, pattern fills, threshold, ramp, remap, and channel packing.
- Comic and reader effects: panel-local depth, parallax layers, reveal masks,
  lighting maps, impact overlays, and enhanced web reader maps.

Decorative finish effects should be semantic finish descriptors with masks and
parameters, not raw shader code. They may form:

- base color or albedo;
- metallic or finish masks;
- roughness or gloss;
- normal or height;
- flake or glitter masks;
- iridescence or hue-shift masks;
- optional light or view response metadata.

Flat PNG/TIFF exports should bake an approximation of gold, holographic,
glitter, metallic, varnish, or pearlescent effects. Native packages,
material-map exports, Runenwerk exports, Blender-oriented texture sets, and
interactive reader exports can preserve richer finish metadata where the target
supports it.

Effect maturity and persistence should be explicit:

- Declared: named future effect family with ownership and intended products.
  This is roadmap or catalog information only and must not appear as authored
  document state.
- Descriptor: serializable semantic node contract exists. Descriptor or higher
  is the first maturity tier that may be saved in native documents.
- Preview: low-latency preview formation works. Preview products are derived
  formed products, not authored truth.
- Final: deterministic final-quality tile or export formation works. Final
  products are also derived formed products, not authored truth.
- Shippable: diagnostics, cache invalidation, tests, export degradation, and
  performance budgets are stable.

Heavy live effects must satisfy the same formation discipline as drawing tiles:

- no apply-and-bake as the normal workflow;
- explicit inputs, outputs, seed policy, formation version, and quality class;
- tile-level invalidation and source lineage;
- CPU reference path for deterministic tests where practical;
- GPU production path for responsiveness where needed;
- last-good tiles remain visible while recomputation catches up.

## Proposed Domain Ownership

### `domain/drawing`

`domain/drawing` owns the first engine-agnostic drawing contract slice.

It should own:

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
- `PaintLayerSource`;
- `PaperSource`;
- `ReferenceImageSource`;
- `MaskNode`;
- `ClipNode`;
- `GroupNode`;
- `TransformNode`;
- `AdjustmentNode`;
- `EffectNode`;
- `EffectMaturityTier`;
- `DecorativeFinishDescriptor`;
- `CompositeOutput`;
- `CanvasCoordinate`;
- `CanvasTileId`;
- `TilePyramidLevel`;
- `DrawingTileProduct`;
- drawing issue codes, ratifiers, diagnostics, source maps, and product lineage.

The crate should be organized by drawing subdomain responsibility rather than
one large flat model. Expected internal subsystems:

- `document`;
- `stroke`;
- `brush`;
- `paper`;
- `composition`;
- `tile`;
- `simulation`;
- `product_lineage`;
- `history`;
- `ratification`;
- `diagnostics`.

It should not own:

- GPU resources;
- backend shader compilation;
- operating-system tablet APIs;
- Wacom SDK details;
- app shell state;
- editor graph canvas interaction state;
- renderer-private pass execution.

### `domain/ui/ui_input`

Extend platform-neutral input vocabulary for stylus-capable events.

The design target is to preserve:

- device identity;
- pointer source kind;
- tool kind;
- eraser state;
- pressure;
- tilt;
- twist or rotation when available;
- tangential pressure when available;
- timestamp;
- coalesced samples.

Mouse and trackpad input must remain valid fallback input. A pointer without
stylus data is still usable; drawing tools decide whether a missing tablet
capability is acceptable.

The input contract also needs a sampling and latency model:

- stable ordering for raw, coalesced, and predicted samples;
- raw sample timestamps;
- hover samples before contact where available;
- barrel button and eraser transitions;
- cursor offset and pressure calibration hooks;
- smoothing policy owned by drawing tools or brush dynamics, not by the OS
  adapter;
- low-latency preview stroke path that can be replaced by ratified committed
  stroke products.

Native packet loss, unsupported tilt/twist, and unavailable hover must be
reported as capabilities instead of silently normalized into fake values.

### `adapters/native_tablet_input`

`adapters/native_tablet_input` owns the first native tablet input adapter proof.

This adapter owns:

- macOS/Wacom-oriented packet DTOs;
- capability detection;
- packet normalization;
- mapping into `domain/ui/ui_input` events;
- diagnostics for unsupported or missing capabilities.

It should not own drawing semantics. It produces input facts.

Real macOS event tap, Wacom SDK, installer permission, and device-management
integration remain later adapter/app work.

### `apps/runenwerk_draw`

Add a future product app that composes:

- `domain/drawing`;
- reusable `domain/ui/*` crates;
- editor/workspace substrate where appropriate;
- engine runtime and render plugins;
- native tablet adapters;
- persistence and import/export workflows.

The app owns product experience and composition, not domain invariants.

### Future `domain/comic_layout`

Comic and webcomic layout should become a separate domain after drawing basics
are proven.

It should own:

- page and strip documents;
- panel geometry and reading order;
- text boxes;
- speech balloons;
- captions;
- layout guides;
- export intent for print pages and scrolling webcomic formats.

It should reference drawing documents or layers through explicit contracts, not
through editor shell state.

Comic text should not rely on basic UI label rendering long term. The comic
layout domain should define text and balloon contracts, while shaping and font
runtime behavior should be handled by an adapter or runtime layer using a real
text shaping stack when the comic phase begins.

The long-term typography target should support:

- font fallback;
- kerning and ligatures;
- CJK text;
- vertical text;
- ruby or furigana later;
- line breaking;
- text boxes and balloon-fit workflows;
- text-on-path later if needed.

## Graph and Shader Boundary

Runenwerk should not add a raw general `shader_graph` as the source of truth for
drawing.

The accepted graph model is semantic:

```text
domain/graph
  -> neutral graph structure

domain/material_graph
  -> material semantics

domain/drawing composition graph
  -> layer, mask, adjustment, effect, output, and export-composite semantics

future domain/drawing_graph
  -> brush, paper, procedural effect, or formation semantics if they outgrow
     the drawing composition graph

engine render/compute products
  -> backend execution targets
```

Compute shaders are valid implementation targets for tile formation,
pigment movement, wetness propagation, paper interaction, and preview
composition. They are not authored domain truth by themselves.

The rule is:

```text
Authors edit semantic intent.
Domains ratify semantic intent.
Lowering forms drawing/render/compute products.
Engine runtime executes formed products.
```

## Canvas and Zoom Model

The zoom model should be hybrid multiscale tiles, not unbounded raster memory and
not pure vector rendering.

The drawing domain should define stable canvas coordinates and tile identities.
The renderer/app may then use:

- visible tile windows;
- tile cache budgets;
- tile pyramids;
- clipmap-style residency;
- preview levels;
- high-resolution export formation.

At extreme zoom levels, vector stroke truth and procedural paper/brush recipes
can be resampled into the requested product level. Watercolor and paper
simulation may have practical resolution floors, but those limits should be
explicit product capabilities rather than hidden renderer behavior.

### First document scales

The first implementation should not make an infinite board the MVP.

Preferred first document presets:

- illustration page;
- manga/comic page at print DPI;
- vertical webcomic strip;
- square texture map at 2K, 4K, and 8K;
- custom bounded canvas.

This gives natural media and export code concrete size, DPI, and cache targets.
The coordinate and tile model should stay future-compatible with very large
canvases and deep zoom, but bounded pages and texture maps are easier to ratify,
cache, export, and test first.

## Ink MVP

The first usable feature slice should prove deterministic ink drawing.

Required capabilities:

- open a drawing document;
- choose ink color;
- choose an ink brush descriptor;
- capture pressure-sensitive stroke samples;
- commit ordered strokes to a layer;
- ratify stroke samples and brush ranges;
- form deterministic ink tile products;
- preview layer compositing;
- preserve source lineage from tile products to strokes and brushes;
- invalidate and rebuild affected tiles after edits.

Deferred from the MVP:

- full watercolor diffusion;
- advanced pigment chemistry;
- comic text and panel editing;
- graph-authored brushes;
- full cross-platform tablet parity;
- production export formats.

## Paper and Natural Media Simulation

Paper should be modeled as explicit authored input, not as hidden shader state.

The paper model should support:

- paper descriptor identity;
- roughness and absorbency parameters;
- height field references;
- deterministic procedural noise;
- SDF-derived masks or height-like fields where appropriate;
- product lineage from paper source to formed drawing tiles.

Ink simulation should be added before watercolor:

- coverage;
- opacity;
- pressure-width dynamics;
- edge softness;
- color;
- paper height modulation;
- viscosity;
- absorption response;
- dry-brush breakup;
- pooling and deposit buildup;
- nib or brush drag.

Watercolor should extend the same model later:

- wetness;
- pigment concentration;
- diffusion;
- viscosity;
- surface tension;
- absorption rate;
- evaporation rate;
- granulation;
- backruns;
- drying;
- layer interaction.

Simulation descriptors should be explicit and ratified. Likely parameters
include:

- `viscosity`;
- `diffusion_rate`;
- `surface_tension`;
- `absorption_rate`;
- `evaporation_rate`;
- `granulation`;
- `staining`;
- `pigment_density`;
- `paper_capillary_strength`.

Artist-facing controls should come first. Low-level simulation parameters should
exist in the document model from the start, but initial UI should expose named
brush and paper presets with controls such as flow, spread, staining,
granulation, and dry-brush behavior. Later advanced simulation controls can
unlock direct parameter editing once presets are stable.

Presets translate artist controls into deterministic formation parameters.
Formation parameters need explicit ranges, units or normalized interpretation,
solver version, seed policy, and timestep policy before they are used in export
or long-term documents.

Reaction-diffusion should be supported as an optional deterministic tile
formation operator, not as the core drawing source of truth. It is useful for
blooms, pigment separation, marbling-like effects, paper/pigment interaction,
and procedural texture behavior. It must run from explicit inputs, fixed seeds,
fixed step counts, and formation versions so previews and exports are
reproducible.

The first role for reaction-diffusion should be artistic watercolor and pigment
behavior. It may later become a procedural texture or material-map formation
operator if that proves useful, but it should not pull the initial drawing work
into a broad material or shader graph design.

Each simulation slice must define deterministic formation inputs and invalidation
rules before it becomes a runtime effect.

### CPU reference and GPU production

Natural-media formation should have both a CPU reference path and a GPU
production path.

The CPU path should own:

- small-tile reference formation;
- golden tests;
- deterministic debugging;
- cross-platform expected behavior.

The GPU path should own:

- real-time brush previews;
- large tile formation;
- interactive watercolor and paper simulation;
- production preview performance.

Both paths must consume the same drawing descriptors, paper descriptors, brush
descriptors, and formation inputs. GPU output may use tolerances where exact
numeric parity is unrealistic, but those tolerances must be explicit in tests.

## Export and Interchange

Drawing export should be split by target use. A single export format cannot
serve native editing, print/comic output, layered interchange, and game/material
texture output equally well.

### First color pipeline

The first color-management target should be sRGB defaults with explicit linear
formation math where needed.

Initial rules:

- UI previews and common flat exports default to sRGB;
- paint formation, compositing, filtering, and material-map formation may use
  linear internal math when required by the product;
- export manifests record color space, transfer function, bit depth, alpha mode,
  normal-map convention, height scale, channel packing, source document version,
  and export profile;
- 16-bit output is available where height, masks, or natural-media gradients need
  more precision;
- Display P3, ICC-heavy workflows, and CMYK/print color management are deferred
  until the basic sRGB/linear pipeline is stable.

### Native project package

The native document format should be versioned and retain:

- document metadata;
- layers;
- composition graph;
- strokes;
- brush descriptors;
- paper descriptors;
- comic layout objects when present;
- source lineage;
- tile cache metadata when retained.

The default native file should be a single zip-like document package. This keeps
documents easy to move, share, back up, and version casually while still allowing
the internal layout to remain folder-like and migration-friendly.

Authoritative strokes, layers, brush descriptors, paper descriptors, simulation
parameters, and metadata must always live in the native package. Large tile
caches should be optional and regenerable. The default should be:

- keep a small preview thumbnail or flattened composite in the package;
- keep large tile caches in sidecar or cache storage;
- offer an explicit archive option that embeds tile caches when needed.

The native package contract should include:

- package manifest version;
- document schema version;
- migration history;
- package capabilities;
- external asset references;
- embedded asset references;
- cache manifest;
- operation-log manifest when history is saved;
- recovery and partial-load behavior;
- last-good preview product metadata.

The package reader should be able to open current state even when optional caches
or recovery data are missing. Broken required authored state should produce
diagnostics instead of partial success.

### Flat drawing output

The app should support common flat drawing exports before comic-specific export:

- PNG;
- 16-bit PNG where useful;
- TIFF.

Vector/layout data should remain vector where the target format supports it.
Panels, text, balloons, guides, and crop marks should not be rasterized early
unless the chosen export target requires it.

### Layered interchange

Layered export is a separate target from native project persistence.

Preferred early target:

- OpenRaster (`.ora`) for Krita/GIMP-style workflows.

Desirable later targets:

- PSD/PSB import/export where practical;
- Clip Studio workflow support after comic layout requirements are clearer.

These formats should be treated as interchange adapters. They must not become
the internal authority for drawing documents. OpenRaster should be the first
layered interchange target because it is open and simpler to implement. PSD/PSB
should be best-effort compatibility, because Photoshop features will not map
cleanly to Runenwerk's future stroke, paper, and simulation model.

Layered interchange adapters must consume the native layer/composition graph and
project it into the target format. The first OpenRaster path should support the
stack-compatible subset first. Unsupported reusable masks, non-destructive
effects, transforms, material-map outputs, or alternate composites should be
flattened at explicit boundaries, degraded with an export report, or rejected by
the selected profile. PSD/PSB should follow the same rule later. Interchange
adapters must not become the layer authority.

### Game and material texture output

Natural-media drawing can also export texture maps for runtime or DCC use.

Useful outputs include:

- base color or albedo;
- alpha or coverage;
- height or displacement;
- normal map derived from paper height and pigment/ink deposition;
- roughness;
- metallic or finish mask;
- gloss or varnish mask;
- flake or glitter mask;
- iridescence or hue-shift mask;
- wetness mask;
- pigment or ink mask;
- paper height map;
- layer masks.

Material-map export should be a formed product family with explicit color-space,
bit-depth, channel packing, and source-lineage metadata. It should reuse
`domain/texture` descriptors where possible instead of inventing renderer-owned
texture truth.

The first material export targets should be Runenwerk itself and Blender.
Runenwerk consumes lossless native products; Blender consumes baked texture sets.
Unreal, Unity, Godot, and other engines should be handled later through export
profiles that change naming, color-space, normal convention, channel packing,
and manifest metadata without changing internal drawing truth.

Internal source products should remain separate:

- paper height;
- ink or pigment deposit height;
- wetness;
- coverage or alpha;
- pigment mask;
- generated normal.

Export profiles may then bake convenience outputs:

- combined height;
- combined normal;
- albedo or base color;
- roughness;
- packed ORM-style masks where useful.

The first Blender-oriented preset should produce:

- `base_color.png`;
- `alpha.png`;
- `height_16.png`;
- `normal.png`;
- `roughness.png`;
- `wetness_mask.png`;
- `pigment_mask.png`;
- `manifest.json`.

The manifest should record color space, bit depth, normal convention, height
scale, channel packing, source document version, and export profile.

Do not generate `.blend` files or require a Blender add-on for the first export
slice. Texture sets plus a manifest are portable, testable, and useful outside
Blender. A Blender import helper can be added later if the manifest stays stable.

### Comic and webcomic output

Comic and webcomic export should wait for `domain/comic_layout` contracts.
Without page, panel, text, balloon, and reading-order authority, comic export
would invent a hidden layout schema.

Later comic export targets should include:

- PDF for print pages;
- CBZ for comic page bundles;
- sliced PNG, JPEG, or WebP for scrolling webcomic output;
- optional vector-preserving outputs where text, panels, and balloons can remain
  editable or selectable.

### Interactive web reader export

Interactive web output should be an export artifact, not a browser-based
authoring target.

The native app should be able to export a self-contained reader bundle:

```text
Runenwerk drawing/comic document
  -> web reader bundle
  -> HTML/CSS/JS + image layers + maps + manifest
```

The bundle should have a static baseline and optional enhanced modes. A reader
must still work as ordinary images when scripts, GPU effects, animation, or
enhancement layers are unavailable.

Useful enhancement inputs include:

- separated foreground, midground, background, text, and effect layers;
- panel-local depth maps;
- paper height maps;
- ink or pigment height maps;
- normal maps;
- roughness maps;
- wetness masks;
- metallic, finish, glitter, or iridescence masks;
- particle or overlay layers;
- panel timing and reveal metadata.

Supported effects should be panel-authored and restrained:

- 2.5D parallax on scroll, pointer movement, or device tilt;
- material-aware ink, paper, and pigment lighting;
- subtle wet ink or watercolor shimmer from baked masks;
- cinematic panel and speech balloon reveals;
- speed-line, dust, rain, smoke, glint, or impact overlays;
- lightweight 3D or SDF-like panel effects where explicitly authored.

The reader export should support modes:

- static;
- enhanced;
- reduced motion;
- low power or mobile.

Interactive reader export must not become drawing document authority. Effects
are derived from authored comic layout metadata, drawing layers, and formed
maps. The native document remains the editable source of truth.

## Implementation Roadmap

Each phase has an explicit product and acceptance shape. Early phases are
deliberately practical rather than visual; visible drawing output starts only
after the app shell and tile formation phases exist.

### Phase 1: Documentation and Boundary Acceptance

Create this active design and keep it linked from the active design index.

Completion criteria:

- design states app shape, truth model, graph boundary, input boundary, and
  roadmap;
- design distinguishes drawing brushes from SDF/world brushes;
- design distinguishes semantic graphs from shader/compute execution targets;
- documentation validation passes.

### Phase 2: Drawing Domain Contracts

Phase 2 created the first pure `domain/drawing` crate slice.

Current status: implemented in `domain/drawing` with focused crate tests.

The phase product is a pure domain crate with serialization-ready DTO-shaped
drawing contracts, ratifiers, diagnostics, and tile-lineage descriptors. It is
not a visible drawing app and should not render pixels.

Completion criteria:

- drawing document, layer, stroke, sample, brush, ink, paper, tile, and product
  descriptors exist;
- drawing composition graph descriptors exist for the first safe live core:
  stack node, typed ports, paint/reference/paper sources, masks, clips, isolated
  groups, transforms, cheap adjustments, outputs, preview/final quality class,
  and last-good product metadata;
- long-term effect families are declared with ownership, intended products, and
  maturity status without requiring every future effect node to be implemented
  in the first descriptor slice;
- drawing crate modules are split by document, stroke, brush, paper, tile,
  composition, simulation, product lineage, history, ratification, and
  diagnostics responsibilities;
- current-state, operation-log, and recovery-state contracts are explicit;
- ratifiers reject invalid samples, invalid brush ranges, invalid paper
  references, invalid layer ordering, invalid composition graph cycles,
  incompatible composite ports, ambiguous paint targets, and invalid tile
  lineage;
- unit tests cover deterministic ordering and descriptor validation.

### Phase 3: Stylus Input Contracts

Extend `domain/ui/ui_input` and add the native tablet adapter proof.

Status: implemented as `domain/ui/ui_input` stylus-capable pointer packets and
`adapters/native_tablet_input` packet normalization proof. Real native OS/Wacom
event capture remains deferred to the drawing app/platform integration phases.
The next phase is Phase 4, `runenwerk_draw` App Shell.

Completion criteria:

- pointer fallback remains compatible;
- stylus event data can carry pressure, tilt, twist, eraser/tool kind, device id,
  timestamp, and coalesced samples;
- raw, coalesced, predicted, hover, eraser, barrel-button, and calibration
  behavior is specified as input capabilities;
- low-latency preview stroke behavior is separate from committed ratified
  strokes;
- adapter maps macOS/Wacom-oriented packets into platform-neutral input events;
- tests cover missing capability behavior and preservation of pressure/tilt.

### Phase 4: `runenwerk_draw` App Shell

Add the sibling drawing app.

Completion criteria:

- app starts independently;
- app reuses shared UI/render/runtime substrate;
- app owns drawing product composition and canvas-first workspace setup;
- app can open a minimal drawing document and route input to drawing tools.

### Phase 5: Deterministic Ink Tile Formation

Form ink tiles from stroke truth.

Completion criteria:

- stroke capture produces stable ordered records;
- affected tiles are invalidated deterministically;
- tile keys include coordinate, pyramid level, document revision, source ranges,
  formation version, and quality class;
- tile products preserve source lineage;
- `LayerStackNode` composition forms deterministic preview products for ink
  layers;
- the layer panel can manipulate the stack-backed composition path without
  creating hidden app-owned layer state;
- zoom uses tile levels rather than camera-distance clamps.

### Phase 6: Paper Height and Procedural Surface Inputs

Add paper interaction.

Completion criteria:

- paper descriptors can reference procedural noise, height fields, or SDF-derived
  sources through explicit product contracts;
- ink formation responds deterministically to paper properties;
- ink formation supports ratified viscosity and absorption parameters;
- CPU reference formation exists for small deterministic tiles;
- GPU production formation consumes the same descriptors and inputs as the CPU
  reference path;
- diagnostics explain unsupported paper sources and invalid product references.

### Phase 7: Live Layer Composition and Effects

Prove the first safe live composition core.

Completion criteria:

- layer-stack UI edits `LayerStackNode` truth through drawing commands;
- clipping masks, isolated group clipping, group masks, reusable masks, and
  cheap adjustment layers form deterministic preview and final-quality products;
- preview tiles update separately from final-quality tiles;
- preview and final-quality products carry quality class, formation version,
  source lineage, invalidation bounds, and last-good fallback metadata;
- last-good products remain visible while recomputation catches up;
- tile invalidation bounds are derived from graph, source, and parameter
  changes;
- manual bake or freeze is optional future workflow, not required for normal
  editing;
- diagnostics explain unsupported graph features and target capability gaps.

### Phase 8: Technical Drawing Effects

Add drawing-level technical and procedural effects without waiting for comic
layout authority.

Completion criteria:

- halftone, screentone, hatch fields, speed lines, pattern fills, threshold,
  ramp, remap, and channel packing are semantic effects with deterministic
  descriptor and formation versions;
- technical drawing effects form drawing-level preview and final products
  without requiring `domain/comic_layout`;
- formed products preserve source lineage, invalidation bounds, quality class,
  and last-good fallback metadata;
- later comic, material, and export profiles can consume these products without
  becoming their authority.

### Phase 9: Advanced Live Effects and Painterly Composition

Add heavier live painterly effect descriptors after the safe composition core is
stable.

Completion criteria:

- painterly effect families have explicit descriptor, preview, final, and
  shippable maturity states;
- ink bleed, edge darkening, pigment pooling, dry-brush breakup, feathering, and
  bristle or nib artifacts have deterministic inputs and formation versions
  before becoming active document features;
- CPU reference behavior exists where practical for deterministic tests;
- GPU production behavior exists where needed for responsiveness;
- source lineage and last-good fallback are preserved for every formed effect
  product.

### Phase 10: Watercolor Simulation

Add watercolor as deterministic tile formation over the same document model.

Completion criteria:

- wetness and pigment state are formed products with lineage;
- viscosity, diffusion, evaporation, staining, and granulation parameters are
  ratified inputs;
- artist-facing presets map to stored deterministic formation parameters;
- optional reaction-diffusion operators preserve fixed seeds, fixed step counts,
  and formation versions;
- simulation is deterministic for a given document, brush, paper, and formation
  version;
- invalidation and last-good product behavior are explicit.

### Phase 11: Decorative Finish and Material Effect Products

Add decorative finish and material-output effects as semantic drawing products.

Completion criteria:

- metallic ink, gold leaf, foil, holographic foil, glitter, pearlescent ink,
  varnish, gloss, spot-UV-style masks, and emboss/deboss are represented as
  semantic finish descriptors, not raw shader code;
- formed products can include base color, metallic or finish mask, roughness or
  gloss, normal or height, flake or glitter mask, iridescence or hue-shift mask,
  and optional light/view response metadata;
- formed finish products are target-neutral and do not require flat image,
  material-map, Blender-oriented, or interactive reader export to be implemented
  in this phase;
- later export phases decide whether a target bakes approximations, preserves
  material maps, or preserves dynamic light/view response metadata.

### Phase 12: Drawing, Material Export, and Packaging

Add drawing/material export and standalone app distribution.

Completion criteria:

- native drawing package format is a versioned zip-like document package;
- native package contract covers manifest, schema version, migration history,
  cache manifest, operation-log manifest, recovery behavior, and last-good
  preview metadata;
- flat image export supports at least PNG;
- OpenRaster is the accepted first layered interchange target;
- Blender texture-set export produces the first material export preset and
  manifest;
- flat PNG/TIFF exports can bake deterministic approximations of unsupported
  decorative finish products with export diagnostics;
- material-map export defines albedo, alpha, height, normal, roughness,
  wetness, finish, metallic, glitter, iridescence, mask, and related product
  descriptors where supported;
- color pipeline covers sRGB defaults, linear formation math where needed,
  bit-depth, alpha mode, normal-map convention, height scale, and channel
  packing metadata;
- direct-download macOS app bundle, signing, notarization, and `.dmg`
  distribution requirements are documented;
- tablet adapter shipping constraints are isolated from pure domain crates.

### Phase 13: Comic and Webcomic Layout

Add a separate layout domain after drawing basics are stable.

Completion criteria:

- panel, page, text, balloon, and guide authority is separate from paint tiles;
- layout can reference drawing content through explicit contracts;
- text contracts are separate from basic UI labels;
- typography plan covers shaping, font fallback, CJK, vertical text, line
  breaking, and future ruby/furigana;
- export paths can rasterize or preserve vector/layout data as needed.

### Phase 14: Comic and Webcomic Export

Add comic-specific export after comic layout contracts exist.

Completion criteria:

- PDF export consumes page, panel, text, balloon, and print metadata from
  `domain/comic_layout`;
- CBZ export consumes page order and flattened page products;
- scrolling webcomic export consumes strip layout and slice metadata;
- vector/layout preservation is used where the target format supports it;
- comic export does not invent its own page, panel, text, or reading-order
  authority.

### Phase 15: Interactive Web Reader Export

Add a generated web reader output for enhanced manga and webcomic reading.

Completion criteria:

- reader export consumes comic layout page, panel, layer, timing, and reveal
  metadata rather than inventing a separate hidden schema;
- reader bundle has a static image fallback;
- reader manifest records pages, panels, layers, enhancement maps, modes, and
  source document version;
- material/depth effects are optional formed products, not document authority;
- panel-local depth, reveal masks, parallax metadata, timing metadata, impact
  overlays, and enhancement masks consume comic layout metadata rather than
  inventing page, panel, timing, or reveal authority;
- reduced-motion and low-power modes are available;
- generated HTML/CSS/JS has no dependency on native editor runtime state.

## Roadmap Phase Products And Acceptance

Phase products are cumulative. A later visual phase may depend on earlier
non-visual contracts; phases 1 through 3 are successful when they make later
implementation testable and hard to misuse.

| Phase | Phase product | Testable, visual, or practical acceptance |
| --- | --- | --- |
| 1. Documentation and Boundary Acceptance | Active platform design linked from the active design index. | Documentation validation passes; ownership boundaries, stop conditions, and roadmap shape are explicit. Not visual. |
| 2. Drawing Domain Contracts | Pure `domain/drawing` crate with serialization-ready DTO-shaped drawing contracts, ratifiers, diagnostics, command/transaction helpers, and tile-lineage descriptors. | `cargo test -p drawing` proves valid and invalid documents, stack entries, graph ratification, brush/paper ranges, command results, and tile-lineage descriptors. Not visual. |
| 3. Stylus Input Contracts | Platform-neutral stylus input vocabulary plus a macOS/Wacom-oriented native tablet adapter proof. | Tests prove pointer fallback, pressure, tilt, twist, eraser/tool kind, barrel buttons, timestamps, coalesced samples, and missing-capability behavior. Practical input foundation, not a drawing app. |
| 4. `runenwerk_draw` App Shell | Standalone focused app shell reusing the editor/workspace/UI/render/runtime substrate. | App launches, opens a minimal drawing document, routes pointer input, and presents a canvas-first workspace. First visible shell; real ink formation is not required yet. |
| 5. Deterministic Ink Tile Formation | Stroke-to-ink tile formation with tile keys, invalidation, source lineage, and preview layer composition. | Users can make pressure-sensitive ink strokes and see deterministic ink tiles update while zoom uses pyramid levels. First real drawing output. |
| 6. Paper Height and Procedural Surface Inputs | Paper descriptors and procedural surface products that affect deterministic ink formation. | Ink visibly responds to supported paper/noise/SDF-derived height inputs; CPU reference and GPU production paths consume the same descriptors. |
| 7. Live Layer Composition and Effects | Safe live composition core: layer stack, clipping masks, isolated groups, masks, transforms, cheap adjustments, preview/final products, and last-good fallback. | Stack and effect edits remain live and non-destructive; deterministic final-quality products exist for the safe core without requiring manual bake as the normal workflow. |
| 8. Technical Drawing Effects | Drawing-level halftone, screentone, hatching, speed-line, pattern, threshold/ramp/remap, and channel-packing effects. | Technical effects form preview and final drawing products without depending on comic layout authority. |
| 9. Advanced Live Effects and Painterly Composition | Heavy painterly effect descriptors and formed products for bleed, pooling, dry brush breakup, feathering, and bristle or nib artifacts. | Painterly products preserve deterministic inputs, formation versions, source lineage, CPU/GPU tolerance policy, invalidation, and last-good fallback. |
| 10. Watercolor Simulation | Deterministic wetness, pigment, diffusion, granulation, backrun, staining, and drying products over stroke and paper truth. | Watercolor presets produce reproducible natural-media tiles with ratified simulation parameters, invalidation, and last-good behavior. |
| 11. Decorative Finish and Material Effect Products | Target-neutral finish descriptors and formed products for metallic ink, gold, foil, holographic foil, glitter, pearlescent ink, varnish, gloss, and emboss/deboss. | Finish products expose semantic masks, maps, and light/view response metadata without requiring any specific export target yet. |
| 12. Drawing, Material Export, and Packaging | Native package contract, flat exports, OpenRaster, Blender texture-set manifest, and macOS app distribution path. | Exports produce usable files and manifests; unsupported effects bake, degrade, or reject with diagnostics; macOS signing/notarization requirements are documented. |
| 13. Comic and Webcomic Layout | Separate `domain/comic_layout` authority for page, strip, panel, text, balloon, guide, reading-order, and layout objects. | Layout objects are editable and can reference drawing products without owning paint tiles or drawing composition semantics. |
| 14. Comic and Webcomic Export | PDF, CBZ, scrolling webcomic image export, and optional vector/layout-preserving outputs. | Export consumes comic layout authority and produces ordered page or strip outputs without inventing hidden page, panel, text, or reading-order schema. |
| 15. Interactive Web Reader Export | Static-plus-enhanced web reader bundle with optional maps, reader metadata, reduced-motion, low-power, and fallback paths. | Generated bundle works as static images first and enhanced reader second; fallback integrity and optional effect degradation are testable without native editor runtime state. |

## Future Design Splits

This document should remain the platform overview. Before implementation, split
the high-risk areas into focused designs:

- [Drawing Domain Crate Design](./drawing-domain-crate-design.md) for document,
  stroke, brush, paper, tile, layer/composition graph, simulation, history,
  ratification, and diagnostics contracts. Phase 2 created the first
  `domain/drawing` crate slice from this artifact;
- drawing layer and composition graph contract for stack projection, node
  catalog, typed ports, graph ratification, source maps, export projections, and
  advanced graph inspection;
- live effect family and maturity design for core, painterly, watercolor,
  decorative finish, paper/material, technical/procedural drawing, and
  comic/reader effect contracts;
- technical drawing effect design for halftone, screentone, hatching,
  speed-line, pattern, threshold/ramp/remap, and channel-packing products that
  do not depend on comic layout authority;
- decorative finish output design for metallic, gold, foil, holographic,
  glitter, pearlescent, varnish, gloss, emboss/deboss, material maps, and flat
  export approximation diagnostics;
- native package contract for manifest, schema version, migrations, operation
  logs, cache sidecars, recovery, and partial-load diagnostics;
- tablet input and latency contract for stylus capabilities, sampling,
  prediction, calibration, low-latency preview strokes, and adapter diagnostics;
- tile formation contract for tile identity, invalidation, cache policy, CPU
  reference formation, GPU production formation, and golden/tolerance testing;
- drawing app UX design for canvas-first layout, tablet setup, brush/paper/color
  workflows, layers, recovery, tile status, and export profile UX;
- workflow packaging and specialized app profile design for hosting the same
  tool surfaces inside `runenwerk_editor`, `runenwerk_draw`, future
  `runenwerk_ui_designer`, future `runenwerk_material_lab`, and later packaged
  editor profiles.

## Validation Plan

Documentation phase:

```sh
python3 tools/docs/validate_docs.py
```

Drawing domain phase:

```sh
cargo test -p drawing
```

Input phase:

```sh
cargo test -p ui_input
cargo test -p native_tablet_input
```

App/render phase:

```sh
cargo check -p runenwerk_draw
cargo test --workspace
./quiet_full_gate.sh
```

Export and packaging phase:

```sh
cargo check -p runenwerk_draw
```

Add export-specific tests once export adapters exist. At minimum, tests should
cover native package migration metadata, PNG export dimensions and color-space
metadata, material-map channel declarations, and layered interchange round-trip
behavior for supported features.

Drawing-domain tests should include history profile behavior, tile-key
determinism, tile invalidation bounds, layer-stack projection behavior,
`GroupNode` nesting, composition graph cycle rejection, composite port
compatibility, clipping masks, isolated group clipping, cheap adjustment
determinism, preview/final tile lineage, final-quality safe live effects,
last-good fallback, native package manifest parsing, and missing-cache recovery.
Simulation and effect tests should compare CPU reference output against expected
small-tile goldens and compare GPU production output against explicit tolerances
when GPU validation exists.

Effect tests should cover maturity persistence rules, especially that declared
effects are catalog or roadmap concepts only and do not appear as authored
native document state.

Decorative finish and material-output tests should cover finish descriptor
determinism, decorative finish map metadata, flat-export approximation reports,
material-map output metadata, and export degradation reports.

Technical drawing effect tests should cover halftone, screentone, hatch fields,
speed lines, pattern fills, threshold/ramp/remap, and channel packing without a
comic layout dependency.

Interactive reader export tests should cover static fallback generation, manifest
references, missing enhancement maps, reduced-motion mode, and asset path
integrity.

Use the smallest relevant validation during implementation loops. Use the full
gate when a phase changes workspace integration or cross-domain contracts.

## Stop Conditions

Stop and redesign before implementation if:

- app crates would own workflow semantics that belong in domain crates or tool
  surfaces;
- drawing semantics would need to live in `apps/runenwerk_editor`;
- material graph would need to accept drawing-only semantics;
- layer/composition semantics would require a raw shader graph as authored
  truth;
- a normal layer panel would need hidden app-owned layer state not represented in
  the semantic composition graph;
- composition effect nodes would require nondeterministic runtime-only state;
- declared effect families would need to be saved as authored document truth
  before descriptor contracts exist;
- live non-destructive effects would require destructive apply-and-bake as the
  normal editing workflow;
- safe live effects cannot produce deterministic final-quality formed products;
- live effects cannot preserve source lineage, tile invalidation bounds,
  preview/final quality classes, and last-good fallback;
- decorative finish effects would have to be authored as raw shader code instead
  of semantic finish descriptors and formed products;
- technical drawing effects would require comic layout authority before they can
  form drawing-level products;
- SDF/world brush concepts are being reused as natural-media brush authority;
- native tablet details would enter a pure domain crate;
- tile products cannot preserve source lineage;
- watercolor simulation requires hidden mutable runtime state as its only
  authority;
- reaction-diffusion would require non-deterministic runtime-only state;
- export adapters would become internal document authority;
- interactive web reader effects would become required for reading exported
  comics;
- interactive web reader export would need to invent page or panel metadata
  instead of consuming comic layout contracts;
- interactive web reader export would depend on native editor runtime state;
- native package loading cannot recover current authored state without tile
  caches;
- standalone packaging would require tablet SDK details in pure domain crates;
- comic typography would need to live in basic UI label rendering;
- zoom requires unbounded raster allocation instead of explicit product levels.

## Open Decisions

These decisions should be made when their implementation phase starts:

- exact native document extension and internal package layout;
- save profile names and default history retention policy;
- exact workflow packaging/profile contract for focused standalone hosts;
- exact descriptor field shapes for the first safe live composition core;
- accepted v1 blend modes, alpha rules, clipping rules, and layer-group
  behavior;
- pass-through group blending semantics and whether they are needed before or
  after isolated group blending is stable;
- first cheap adjustment layer parameter ranges and interpolation policy;
- first live-effect performance budgets for preview tiles, final tiles, and
  last-good fallback;
- first decorative finish descriptor set and material-map channel policy;
- first technical drawing effect ordering and which halftone, screentone,
  hatching, speed-line, pattern, or channel effects are v1 versus deferred;
- effect maturity promotion rules from declared to descriptor, preview, final,
  and shippable;
- composition graph serialization location inside the native package and its
  migration policy;
- graph-to-stack projection rules and when the advanced composition graph view
  becomes visible;
- layered interchange degradation, flattening, and diagnostic report policy;
- exact native tablet API or Wacom SDK integration path;
- tile size and tile cache budget defaults;
- cache sidecar location, pruning, and archive-embed policy;
- first paper descriptor parameter set;
- first viscosity, diffusion, absorption, and evaporation parameter ranges;
- CPU reference precision, GPU tolerance, and formation golden policy;
- whether graph-authored brushes need a separate `domain/drawing_graph` crate in
  the first watercolor phase or later;
- exact Blender texture-set manifest schema;
- typography shaping backend and adapter boundary;
- future Display P3, ICC, and CMYK/print color-management boundary;
- interactive web reader manifest schema and first enhancement mode set.
