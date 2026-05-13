---
title: Runenwerk Draw
description: Current architecture overview for the focused Runenwerk drawing app and first visible ink path.
status: active
owner: drawing
layer: app
canonical: true
last_reviewed: 2026-05-13
related_designs:
  - ../../design/active/drawing-authoring-and-comic-layout-platform-design.md
  - ../../design/active/drawing-domain-crate-design.md
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
  accepted/preview ink projection into product-surface UI primitives.
- `apps/runenwerk_draw/src/runtime/ink.rs`: product publication and query
  snapshot barrier handlers for drawing ink tiles.
- `apps/runenwerk_draw/src/runtime/systems.rs`: runtime submission of UI frames,
  dynamic ink texture targets, upload requests, and committed render product
  selections.

## Current Behavior

The app starts independently, opens a ratified minimal `DrawingDocument`,
projects a canvas-first workspace, presents visible paper/chrome shell UI
through the shared render UI pipeline, and routes pointer/stylus-compatible
`ui_input` events into drawing state.

The Phase 5 ink path is now present and hardened for last-good visibility:

- pointer-down/move creates a low-latency preview overlay from domain-formed
  ink tile products;
- pointer-up commits the preview into `DrawingDocument` through
  `DrawingTransaction` and `DrawingCommand::{BeginStroke, AppendStrokeSample,
  CommitStroke}`;
- deterministic CPU ink tiles form from committed stroke truth in bounded dirty
  tile batches;
- formed tile descriptors publish through the `ProductPublication` barrier;
- strict renderer query snapshots publish through the `QuerySnapshotPublication`
  barrier;
- last accepted committed ink stays visible until a newer committed generation
  is accepted by both product publication and query snapshot publication;
- a just-released preview overlay remains visible above last accepted ink until
  the committed generation replaces it, and formation or publication failure
  preserves the last-good committed tiles with diagnostics;
- accepted current and preview tile payloads are uploaded through the generic
  engine dynamic texture upload path and projected as neutral product-surface UI
  primitives, one primitive per visible tile.

The first visible ink and live preview still use deterministic CPU brush
formation as the source of truth. The renderer path is now texture-backed, but
this is not yet GPU product formation, persistent tile cache, watercolor/paper
simulation, eraser compositing, package save format, or advanced layer/effect
renderer.

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
