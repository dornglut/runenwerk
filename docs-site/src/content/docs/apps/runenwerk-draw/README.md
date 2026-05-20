---
title: Runenwerk Draw
description: Current architecture overview for the focused Runenwerk drawing app and first visible ink path.
status: active
owner: drawing
layer: app
canonical: true
last_reviewed: 2026-05-19
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
- `apps/runenwerk_draw/src/app/ink/mod.rs`: app-owned ink runtime aggregate for
  preview products, publication, visibility, cache, GPU validation, fallback
  state, and query snapshot journals.
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
- [Draw authority and product flow](./diagrams/draw-authority-and-product-flow.puml):
  PlantUML source for the current drawing truth, app runtime state, product
  publication, query snapshot, and render execution boundaries.

## Current Behavior

The app starts independently, opens a ratified minimal `DrawingDocument`,
projects a canvas-first workspace, presents visible paper/chrome shell UI
through the shared render UI pipeline, and routes pointer/stylus-compatible
`ui_input` events into app-owned drawing runtime state.

The current DRF1-DRF5 ink path is present and hardened for last-good
visibility:

- pointer-down/move appends ordered preview samples, marks preview tile catch-up
  dirty, and projects immediate screen-space pen feedback through a bounded
  `UiPrimitive::Stroke` tail until formed preview products catch up;
- preview ink tile products are visual catch-up products formed asynchronously
  by runtime jobs from owned document/stroke snapshots, then applied on the main
  thread. Compatible lagging preview jobs may advance formed sample coverage,
  while incompatible or stopped-session jobs are ignored;
- pointer-up commits the preview into `DrawingDocument` through
  `DrawingTransaction` and `DrawingCommand::{BeginStroke, AppendStrokeSample,
  CommitStroke}`;
- deterministic CPU ink tiles form from committed stroke truth in bounded
  runtime job batches;
- preview and final CPU tile profiles are part of tile identity, descriptor
  generation, cache identity, texture target keys, and render selection;
- committed tiles can be served from the app-owned in-memory tile payload cache
  when engine runtime cache metadata accepts the matching `ProductCacheKey`;
- preview and final products participate in app-owned cache budgeting,
  eviction, and visible/pending/last-good protection policy;
- formed tile descriptors publish through the `ProductPublication` barrier;
- strict renderer query snapshots publish through the `QuerySnapshotPublication`
  barrier;
- last accepted committed ink stays visible until a newer committed generation
  is accepted by both product publication and query snapshot publication;
- accepted committed query snapshots preserve any newer active preview stroke
  and its preview products, instead of clearing current drawing feedback for an
  earlier stroke;
- a just-released preview overlay remains visible above last accepted ink until
  the committed generation replaces it through preview tile catch-up products
  and, when needed, bounded immediate tail feedback; formation or publication
  failure preserves the last-good committed tiles with diagnostics;
- accepted current and preview tile payloads are uploaded through the generic
  engine dynamic texture upload path and projected as neutral product-surface UI
  primitives, one primitive per visible tile;
- Draw-owned GPU ink validation requests run through public render-flow APIs,
  compare GPU output against CPU reference captures, promote only matching
  tile generations to GPU-backed product surfaces, reject stale generations,
  and keep CPU current or last-good tiles visible when validation fails.

`DrawingDocument` is the source authority for drawing truth. `StrokePrimitive`
is immediate UI projection only; it is not authoritative drawing state, not a
tile product, and not cache identity. Renderer and GPU output are derived
execution state. GPU-backed product surfaces may become visible only through
app-owned validation and promotion policy, and they never replace CPU tile truth
or mutate the document.

The renderer path is texture-backed for tile products and has a strict GPU
proof/promotion/fallback slice, but this is not paper response, watercolor,
export, package IO, lasso, transform, fill, mask eraser compositing, radial
menu behavior, or new advanced GPU drawing behavior.

## Current Limits / Known Gaps

- Persistent tile cache storage, native package sidecars, and package cache
  pruning remain deferred. The implemented cache proof is app-owned in-memory
  payload caching with engine-owned metadata decisions.
- GPU validation, visible promotion, stale-generation rejection, and CPU
  current or last-good fallback are implemented as app-owned runtime policy.
  GPU output is still a derived execution product and must not become drawing
  truth.
- Paper response, watercolor, export, package IO, lasso, transform, fill,
  selection tooling, mask eraser compositing, radial menu behavior, and richer
  layer/effect formation remain out of this slice.
- Touch input currently routes through the same drawing path as fallback pointer
  input. The pen-first UX target is different: touch drawing should be disabled
  by default and enabled only through explicit profile/input policy.
- Active strokes keep app-level capture for outside-canvas move/up samples.
  Window/OS lost-capture diagnostics and recovery policy are still not a full
  UX surface.
- Native tablet packet routing and fallback suppression exist, but backend
  arbitration, real Windows Ink/Wacom/macOS hardware acceptance, and device
  setup UX are still open.

## Runenwerk Draw Authority Map

`DrawingDocument` in `domain/drawing` is the source authority. Every committed
stroke mutation must flow through `DrawingTransaction` and `DrawingCommand`
before derived products can be formed. `domain/drawing` owns drawing semantics,
ratification, document revision, deterministic CPU ink tile formation,
invalidation, tile lineage, and product descriptor/cache identity helpers.

`apps/runenwerk_draw` owns product-level wiring, canvas-first workspace setup,
tool input/session runtime, app state, runtime plugin registration, shell-level
input routing, dirty tile batching, app cache payloads, pending/current/last-good
visibility, product publication and query snapshot handling, GPU validation
records, visible product promotion, and fallback policy. These are runtime
state derived from document truth and current app input; they are not drawing
truth.

`engine/render` owns generic render-flow execution, dynamic texture allocation,
upload, capture/readback, texture diff, and UI composition execution. Renderer
and GPU output are derived execution state, not drawing truth. Engine code must
not own drawing semantics or decide whether GPU output supersedes CPU tile
truth.

`adapters/native_tablet_input` owns native tablet packet normalization only. It
may map Windows Pointer/Ink, Wintab, and macOS tablet facts into `ui_input`
pointer events and diagnostics, but it must not own drawing semantics, stroke
commit policy, brush behavior, or render products.

`domain/ui/ui_render_data` owns neutral UI primitives and product-surface
binding vocabulary. It projects app-selected products for rendering but must
not store drawing document truth, product lifecycle policy, or renderer backend
handles.

Workbench integration is deferred. It must not block standalone
`runenwerk_draw` work, and this app must not be migrated into Workbench as part
of drawing feature cleanup or rendering foundation work.

## DrawingInkRuntimeState Module Split

`apps/runenwerk_draw/src/app/ink/mod.rs::DrawingInkRuntimeState` is the
app-owned ink runtime aggregate. It delegates focused responsibilities to
submodules under `apps/runenwerk_draw/src/app/ink/`.

Current module shape:

- `apps/runenwerk_draw/src/app/ink/mod.rs`: public facade, re-exports, and the
  `DrawingInkRuntimeState` aggregate boundary.
- `apps/runenwerk_draw/src/app/ink/preview.rs`: preview tile products,
  preview dirty-tail bookkeeping, preview failure handling, and preview cache
  insertion calls.
- `apps/runenwerk_draw/src/app/ink/publication.rs`: candidate products,
  published descriptors, accepted query snapshot ids, formation/publication
  keys, and barrier acceptance transitions.
- `apps/runenwerk_draw/src/app/ink/visibility.rs`: visible committed products,
  candidate-to-visible promotion, cleared-tile handling, current/last-good
  visibility preservation, and `visible_surface_kind_for`.
- `apps/runenwerk_draw/src/app/ink/cache.rs`: in-memory tile payload cache,
  source-key lookup, budget tracking, LRU-style eviction, and protected cache
  key policy.
- `apps/runenwerk_draw/src/app/ink/gpu_validation.rs`: per-generation GPU
  validation records, request eligibility, target eligibility, pass/fail
  recording, stale-generation rejection helpers, and promotion/fallback queries.
- `apps/runenwerk_draw/src/app/ink/journal.rs`: bounded runtime journal entries
  and diagnostic stage recording.

The split preserves the product publication/query snapshot barrier lifecycle,
CPU tile authority, cache identity, GPU validation semantics, fallback
behavior, and public app tests. It does not add paper response, watercolor,
export, package IO, lasso, transform, fill, mask eraser compositing, radial
menu behavior, or new GPU behavior.
