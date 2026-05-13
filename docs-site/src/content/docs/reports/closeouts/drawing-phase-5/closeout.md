---
title: Drawing Phase 5 Closeout
description: Completion and drift-check record for deterministic ink tile formation and first visible ink in runenwerk_draw.
status: completed
owner: drawing
layer: app
canonical: true
last_reviewed: 2026-05-13
related_designs:
  - ../../../design/active/drawing-authoring-and-comic-layout-platform-design.md
  - ../../../design/active/drawing-domain-crate-design.md
related_domains:
  - ../../../domain/drawing/README.md
related_apps:
  - ../../../apps/runenwerk-draw/README.md
---

# Drawing Phase 5 Closeout

## Status

Complete as of 2026-05-13 for deterministic CPU ink tile formation and first
visible ink in `runenwerk_draw`.

This phase makes drawing on the canvas visible and hardens the first ink
visibility lifecycle. Committed strokes form CPU RGBA8 preview-quality tiles,
publish through product publication and query snapshot barriers, and remain
visible until a newer committed generation is fully accepted. Live preview
strokes use the same CPU brush rasterizer as committed ink.

Phase 5.1 replaced the original UI-rect projection bootstrap with texture-backed
product surfaces. The output is still deliberately bounded: it does not add GPU
product formation, persistent tile caches, paper response, watercolor, eraser
compositing, package IO, export adapters, or advanced layer/effect formation.

## Completion Evidence

- `domain/drawing/src/tile/formation.rs` forms deterministic ink tile payloads
  from committed `StrokeRecord` values and non-authoritative
  `DrawingInkPreviewStroke` values through the same rasterization path.
- `domain/drawing/src/tile/product_contracts.rs` builds product jobs,
  descriptors, publication outcomes, and query snapshots for strict renderer
  consumption.
- `apps/runenwerk_draw/src/app/state.rs` forms real preview tile products while
  routing pointer input, then commits pointer-up preview strokes through
  `DrawingTransaction`.
- `apps/runenwerk_draw/src/runtime/ink.rs` publishes formed ink products only at
  `ProductPublication` barriers, snapshots them only at
  `QuerySnapshotPublication` barriers, and promotes a candidate committed
  dirty-tile batch to visible ink only after both barriers accept it.
- `apps/runenwerk_draw/src/app/presentation.rs` projects accepted current ink
  tile snapshots and active or just-released preview tile products as neutral
  product-surface UI primitives.
- `engine/src/plugins/render/runtime/dynamic_texture_uploads.rs` and
  `engine/src/plugins/render/renderer/dynamic_targets.rs` provide the generic
  dynamic texture upload path used by drawing ink.

## Drift Corrections

- The drawing app README now states that paper/chrome, first visible CPU ink,
  real preview tile products, product-surface rendering, dirty tile batching,
  and last-good committed visibility are present.
- The drawing domain README now includes deterministic CPU ink tile payloads,
  preview tile formation, invalidation, and bounded tile-id formation in the
  current public scope.
- The active drawing platform design now records Phase 5.1 as the product-surface
  rendering hardening slice before paper response.

## Deferred Work

- GPU product formation and renderer-native drawing rasterization.
- Persistent tile cache and native drawing package save/load.
- Paper height response, watercolor, eraser compositing, live effects, final
  quality tile products, export adapters, and comic/page layout authority.
- Native OS/Wacom event capture beyond the existing adapter proof.

## Validation

Validation passed on 2026-05-13:

- `cargo test -p drawing --test ink_tile -- --nocapture`
- `cargo test -p engine --test render_dynamic_targets -- --nocapture`
- `cargo test -p engine --test render_dynamic_texture_uploads -- --nocapture`
- `cargo test -p runenwerk_draw --test app_shell -- --nocapture`

Closeout validation:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test -p drawing`
- `cargo test -p runenwerk_draw`
- `python3 tools/docs/validate_docs.py`
- `./quiet_full_gate.sh`
