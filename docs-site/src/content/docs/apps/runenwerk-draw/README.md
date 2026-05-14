---
title: Runenwerk Draw
description: Current architecture overview for the focused Runenwerk drawing app and first visible ink path.
status: active
owner: drawing
layer: app
canonical: true
last_reviewed: 2026-05-14
related_designs:
  - ../../design/active/drawing-authoring-and-comic-layout-platform-design.md
  - ../../design/active/drawing-domain-crate-design.md
  - ../../design/active/runenwerk-draw-pen-first-radial-tablet-ux-design.md
related_roadmaps:
  - ./roadmap.md
related_reports:
  - ../../reports/closeouts/runtime-product-job-rpj4-rpj6/closeout.md
  - ../../reports/closeouts/runtime-product-job-rpj7a-cache-policy/closeout.md
---

# Runenwerk Draw

`apps/runenwerk_draw` is the focused drawing application shell. It reuses the
shared engine runtime, UI frame contracts, render submission registry, and pure
`domain/drawing` contracts while keeping drawing product composition outside
`apps/runenwerk_editor`.

## Entry Points

- `apps/runenwerk_draw/src/main.rs`: binary entry point.
- `apps/runenwerk_draw/src/lib.rs`: public app crate surface.
- `apps/runenwerk_draw/src/runtime/app.rs`: engine app construction and render
  flow registration.
- `apps/runenwerk_draw/src/runtime/plugin.rs`: drawing app runtime plugin.
- `apps/runenwerk_draw/src/app/state.rs`: app-owned drawing state, committed
  stroke routing, and ink visibility facade.
- `apps/runenwerk_draw/src/app/ink.rs`: app-owned ink product publication and
  query snapshot journals.
- `apps/runenwerk_draw/src/app/presentation.rs`: canvas chrome, paper, and
  accepted/preview ink projection into product-surface UI primitives plus
  immediate `StrokePrimitive` feedback.
- `apps/runenwerk_draw/src/runtime/ink.rs`: product publication and query
  snapshot barrier handlers for drawing ink tiles and preview catch-up job
  processing.
- `apps/runenwerk_draw/src/runtime/ink_jobs.rs`: committed and preview CPU ink
  tile runtime jobs.
- `apps/runenwerk_draw/src/runtime/systems.rs`: runtime submission of UI frames,
  dynamic ink texture targets, upload requests, and committed render product
  selections.

## Next Rendering Foundation

- [Runenwerk Draw Rendering Foundation Roadmap](./roadmap.md): preview/final
  tile profiles, app-derived tile caching, product-surface bridging, GPU ink
  proof through public render-flow APIs, and CPU current or last-good fallback
  when GPU validation fails.

## Current Behavior

The app starts independently, opens a ratified minimal `DrawingDocument`,
projects a canvas-first workspace, presents visible paper/chrome shell UI
through the shared render UI pipeline, and routes pointer/stylus-compatible
`ui_input` events into drawing state.

The Phase 5 ink path is now present and hardened for last-good visibility:

- pointer-down/move appends ordered preview samples, marks preview tile catch-up
  dirty, and projects immediate screen-space pen feedback through
  `UiPrimitive::Stroke`;
- preview ink tile products are visual catch-up products formed asynchronously
  by runtime jobs from owned document/stroke snapshots, then applied on the main
  thread;
- pointer-up commits the preview into `DrawingDocument` through
  `DrawingTransaction` and `DrawingCommand::{BeginStroke, AppendStrokeSample,
  CommitStroke}`;
- deterministic CPU ink tiles form from committed stroke truth in bounded
  runtime job batches;
- committed tiles can be served from the app-owned in-memory tile payload cache
  when engine runtime cache metadata accepts the matching `ProductCacheKey`;
- formed tile descriptors publish through the `ProductPublication` barrier;
- strict renderer query snapshots publish through the `QuerySnapshotPublication`
  barrier;
- last accepted committed ink stays visible until a newer committed generation
  is accepted by both product publication and query snapshot publication;
- a just-released preview overlay remains visible above last accepted ink until
  the committed generation replaces it through immediate stroke feedback and,
  when available, preview tile catch-up products; formation or publication
  failure preserves the last-good committed tiles with diagnostics;
- accepted current and preview tile payloads are uploaded through the generic
  engine dynamic texture upload path and projected as neutral product-surface UI
  primitives, one primitive per visible tile.

The first visible ink path still uses deterministic CPU brush formation for
preview-quality products. `StrokePrimitive` is immediate UI projection only; it
is not authoritative drawing state, not a tile product, and not cache identity.
Preview/final quality is already part of tile identity, descriptor generation,
cache identity, texture target keys, and committed render selection. The
renderer path is texture-backed for tile products, but this is not yet GPU
product formation, persistent tile cache, watercolor/paper simulation, eraser
compositing, package save format, or advanced layer/effect renderer.

## Current Limits / Known Gaps

- The visible app path is immediate CPU/UI stroke feedback plus deterministic
  CPU preview-quality ink tiles. Final-quality CPU tile contracts and cache
  identity exist, and committed preview tiles have an in-memory cache-hit proof.
  App-visible final tile lifecycle, preview/final cache budgeting and eviction,
  persistent tile cache, GPU formation, and GPU promotion/fallback are planned
  by the roadmap, not implemented app behavior yet.
- Touch input currently routes through the same drawing path as fallback pointer
  input. The pen-first UX target is different: touch drawing should be disabled
  by default and enabled only through explicit profile/input policy.
- Active strokes keep app-level capture for outside-canvas move/up samples.
  Window/OS lost-capture diagnostics and recovery policy are still not a full
  UX surface.
- Native tablet packet routing and fallback suppression exist, but backend
  arbitration, real Windows Ink/Wacom/macOS hardware acceptance, and device
  setup UX are still open.

## Ownership Boundary

`runenwerk_draw` owns product-level wiring, canvas-first workspace setup, app
state, runtime plugin registration, and shell-level input routing. It should not
own drawing document semantics, graph ratification, brush/paper contracts,
native tablet APIs, renderer-private tile formation, package IO, or export
adapters.

Those belong to their owning crates:

- drawing truth and ratification: `domain/drawing`;
- platform-neutral input vocabulary: `domain/ui/ui_input`;
- native tablet packet normalization: `adapters/native_tablet_input`;
- render execution and UI frame composition: `engine`;
- later package and export contracts: future focused designs.
