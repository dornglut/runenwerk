---
title: Runenwerk Draw Rendering Foundation Roadmap
description: App-level implementation sequence for Draw ink rendering foundations between texture-backed CPU ink and later paper or watercolor work.
status: active
owner: drawing
layer: app
canonical: true
last_reviewed: 2026-05-14
related_designs:
  - ../../design/active/drawing-authoring-and-comic-layout-platform-design.md
  - ../../design/active/drawing-domain-crate-design.md
  - ../../design/active/render-product-surface-foundation-bundle-design.md
related_roadmaps:
  - ../../engine/plugins/render/docs/roadmap.md
  - ../../workspace/roadmap-index.md
related_reports:
  - ../../reports/closeouts/drawing-phase-5/closeout.md
  - ../../reports/closeouts/runtime-product-job-rpj4-rpj6/closeout.md
---

# Runenwerk Draw Rendering Foundation Roadmap

## Goal

Make `runenwerk_draw` ink rendering production-ready enough for later paper,
watercolor, live effects, and export work without moving drawing semantics into
the renderer.

The current baseline is Phase 5.1 plus the runtime-job responsiveness and cache
identity proof:
immediate pen feedback is projected through `UiPrimitive::Stroke`, while
deterministic CPU RGBA8 preview and committed ink tiles are formed by
`domain/drawing` through runtime jobs, published through product and query
snapshot barriers, uploaded through generic engine dynamic texture uploads, and
projected as neutral product-surface UI primitives. Preview/final quality now
participates in tile identity, descriptor generation, cache identity, and render
selection; app-visible final tile lifecycle is still a later phase.

## Foundation Policy

- CPU tile formation remains the canonical correctness oracle.
- GPU formation is a production acceleration path, not drawing truth.
- GPU output becomes the default visible path only for a tile generation that
  passes shadow comparison against the CPU reference.
- The first GPU/CPU proof uses strict tolerance: max RGBA channel delta <= 2 and
  changed pixels <= 1%.
- Preview and final profiles are both part of the foundation: preview tiles stay
  64x64 px, final tiles are 256x256 px, and both use the same canvas tile
  footprint.
- `domain/drawing` must not contain backend handles, render-flow APIs, shader
  code, or runtime cache state.
- `engine/src/plugins/render` stays generic: render-flow execution, dynamic
  targets, capture/readback, texture diff, and inspection only.
- `apps/runenwerk_draw` owns app cache policy, visibility promotion, CPU/GPU
  validation state, fallback policy, and Draw-specific render-flow requests.

## Contract Alignment Status

The current contracts are aligned with the future preview/final split:

- Preview/final quality must participate in product identity, descriptor
  generation, cache key construction, and render product scale/selection. This
  is implemented in `domain/drawing`, `domain/product`, and Draw render
  selection/upload keys.
- Final-quality ink tiles are a roadmap target, not current app-visible
  behavior.
- Publication and query-snapshot rejection diagnostics must remain visible and
  retryable, so cache/GPU phases cannot silently lock a failed generation as
  complete.

## Phase DRF1 - Preview And Final Tile Profiles

Owner: `domain/drawing`.

Status: implemented for deterministic CPU tile contracts.

Target modules:

- `domain/drawing/src/tile/formation.rs::DrawingTileFormationPolicy`
- `domain/drawing/src/tile/product.rs::ProductQualityClass`
- `domain/drawing/src/tile/product_contracts.rs`

Implementation requirements:

- Keep the existing preview default at 64x64 px.
- Add an explicit final-quality policy at 256x256 px over the same
  `tile_size_canvas_units` footprint.
- Preserve tile identity, source document revision, source output, lineage,
  formation version, descriptor generation, and quality class in every preview
  and final product descriptor.
- Keep CPU formation deterministic and complete before introducing GPU output.
- Reject invalid profile dimensions loudly through drawing-owned diagnostics.

Acceptance:

- `cargo test -p drawing --test ink_tile` proves preview and final profile
  determinism, descriptor separation, invalidation bounds, and quality-class
  lineage.
- Existing visible preview behavior remains immediate through
  `UiPrimitive::Stroke`, with preview tile products catching up asynchronously.

## Phase DRF2 - App-Derived Tile Cache

Owner: `apps/runenwerk_draw`.

Status: next app-owned runtime cache phase; not persistent storage.

Target modules:

- `apps/runenwerk_draw/src/app/ink.rs::DrawingInkRuntimeState`
- `apps/runenwerk_draw/src/runtime/resources.rs::DrawingInkUploadTrackerResource`

Implementation requirements:

- Add an app-derived tile cache for preview and final products with a 512 MiB
  default memory budget.
- Track cache entries by tile id, quality class, descriptor generation, source
  revision, formation version, payload size, and last access frame.
- Evict with LRU-style policy while protecting current visible tiles, active
  preview tiles, pending candidate tiles, and last-good committed fallback
  tiles.
- Preserve the existing last-good visibility lifecycle when formation,
  publication, query snapshot, upload, or cache insertion fails.
- Do not add native package sidecars, persisted cache archives, or package
  pruning in this phase.

Acceptance:

- `cargo test -p runenwerk_draw --test app_shell` covers long strokes, repeated
  strokes, preview dirty-tail updates, cache eviction, visible/current tile
  protection, and last-good fallback.
- `cargo test -p engine --test render_dynamic_texture_uploads` still proves the
  upload registry rejects bad payloads and preserves the previous valid
  snapshot.

## Phase DRF3 - Preview/Final Product-Surface Bridge

Owner: `apps/runenwerk_draw` with generic render substrate from
`engine/src/plugins/render`.

Target modules:

- `apps/runenwerk_draw/src/runtime/systems.rs::submit_draw_frame_system`
- `apps/runenwerk_draw/src/runtime/app.rs::register_draw_render_flow`
- `apps/runenwerk_draw/src/app/presentation.rs::build_workspace_frame_with_ink`

Implementation requirements:

- Submit preview and final dynamic texture target descriptors through the
  existing engine dynamic target registry.
- Submit render product selections for accepted committed products without
  inventing drawing-specific renderer APIs.
- Keep product-surface UI primitives backend-neutral and bound by dynamic
  texture keys, not payload bytes or backend handles.
- Route final-quality tiles through the same visibility and last-good policy as
  preview tiles, with quality class explicit in app cache and product selection
  state.
- Use existing render capture/readback and texture diff infrastructure for proof
  hooks instead of a Draw-local comparison path.

Acceptance:

- `cargo test -p runenwerk_draw --test app_shell` proves preview and final tile
  surfaces stay visible through accepted product/query snapshots.
- `cargo test -p engine --test render_dynamic_targets`
- `cargo test -p engine --test render_runtime_inspect`

## Phase DRF4 - GPU Ink Formation Proof

Owner: `apps/runenwerk_draw` for Draw-specific flow requests and
`engine/src/plugins/render` for generic execution only.

Target modules:

- `apps/runenwerk_draw/src/runtime/app.rs::register_draw_render_flow`
- future `apps/runenwerk_draw/src/runtime/gpu_ink.rs::register_drawing_ink_gpu_flow`
- `engine/src/plugins/render/api/flow.rs::RenderFlow`
- `engine/src/plugins/render/inspect/config.rs::RenderTextureDiffRequest`

Implementation requirements:

- Implement the first GPU ink proof through public `RenderFlow` primitives:
  compute, storage texture or storage buffer, target alias, copy, capture, and
  product-surface sampling.
- Do not add custom renderer executors, renderer-private drawing rasterizers, or
  drawing semantics inside `engine`.
- Produce GPU preview and final tile outputs for the same product descriptors
  that CPU formation produced.
- Capture GPU and CPU reference outputs and compare them with strict tolerance:
  max RGBA channel delta <= 2 and changed pixels <= 1%.
- Keep CPU output visible until GPU validation passes for the same tile
  generation.

Acceptance:

- `cargo test -p engine --test render_flow_v2`
- `cargo test -p engine --test render_runtime_inspect`
- `cargo test -p runenwerk_draw --test app_shell`
- A GPU/CPU texture diff pass proves preview and final tile outputs within
  tolerance without changing drawing document truth.

## Phase DRF5 - GPU Promotion And Fallback

Owner: `apps/runenwerk_draw`.

Target modules:

- `apps/runenwerk_draw/src/app/ink.rs::DrawingInkRuntimeState`
- `apps/runenwerk_draw/src/runtime/resources.rs`
- `apps/runenwerk_draw/src/runtime/systems.rs::submit_draw_frame_system`

Implementation requirements:

- Track GPU validation state per tile generation.
- Promote GPU output to the default visible path only after shadow comparison
  passes for that tile generation.
- On GPU execution or validation failure, show CPU current tiles or CPU
  last-good committed tiles, record diagnostics, and keep GPU disabled for that
  failed generation.
- Never block new visibility solely because GPU output failed when accepted CPU
  output exists.
- Do not show stale GPU output for affected tiles unless its descriptor
  generation still matches the visible product generation.

Acceptance:

- `cargo test -p runenwerk_draw --test app_shell` covers GPU-pass promotion,
  GPU-fail CPU fallback, stale GPU rejection, and last-good CPU preservation.
- `cargo check --workspace`
- `./quiet_full_gate.sh` after this phase because it crosses drawing app,
  product selection, render runtime, and diagnostics behavior.

## Validation Gates

Run the smallest relevant gate during each phase:

```text
cargo test -p drawing --test ink_tile
cargo test -p runenwerk_draw --test app_shell
cargo test -p engine --test render_dynamic_targets
cargo test -p engine --test render_dynamic_texture_uploads
cargo test -p engine --test render_flow_v2
cargo test -p engine --test render_runtime_inspect
cargo check --workspace
python3 tools/docs/validate_docs.py
```

Run `./quiet_full_gate.sh` after any phase that changes cross-domain contracts,
render runtime behavior, product publication/query behavior, or app-visible GPU
fallback semantics.

After every completed phase, run the phase completion drift-check routine before
starting the next phase.

## Explicitly Deferred

- Native package sidecars, persisted cache archives, cache pruning, and package
  migration.
- Paper height response and procedural paper products.
- Watercolor, eraser compositing, painterly simulation, decorative finishes,
  technical drawing effects, and live layer/effect composition.
- Export adapters, material-map output, OpenRaster, Blender texture-set
  manifests, PDF/CBZ/webcomic export, and comic layout authority.
- Renderer-owned drawing semantics or backend handles in `domain/drawing`.
