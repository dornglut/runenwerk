---
title: PM-RENDER-PG-005 Product Surface Platform Hardening Closeout
description: Closeout evidence for the bounded shared product-surface platform hardening implementation slice.
status: completed
owner: engine
layer: engine-runtime / render product surfaces
canonical: false
last_reviewed: 2026-05-21
related_designs:
  - ../../../design/accepted/product-surface-platform-hardening-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
  - ../../../design/accepted/render-contract-ergonomics-design.md
  - ../../../design/accepted/render-execution-graph-compiler-maturity-design.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# PM-RENDER-PG-005 Product Surface Platform Hardening Closeout

## Result

`PM-RENDER-PG-005` completed as a bounded shared product-surface platform
hardening slice.

The implementation makes flow-backed and upload-backed product-surface
producers use one typed manifest and diagnostics vocabulary for dynamic
targets, dynamic uploads, prepared views, prepared flow invocations, UI binding
intents, history signatures, product-surface status, and inspection.

The helpers remain return-only. Producers still publish explicitly into
`RenderDynamicTextureTargetRequestRegistryResource`,
`RenderDynamicTextureUploadRegistryResource`,
`PreparedRenderFrameRequestResource`, and UI binding registries. Product truth,
selection, freshness, fallback legality, authority, rebuild policy, dependency
truth, drawing semantics, field semantics, material truth, and residency policy
remain outside the renderer.

The slice does not implement native multi-window presentation, render
fragments, hot reload, material lowering, drawing semantic changes, SDF
acceleration, production readiness budgets, capture/replay policy, or
renderer-owned product policy.

## Implementation Evidence

- `engine/src/plugins/render/frame/product_surface.rs` adds
  `RenderProductSurfaceManifest` with producer id, product family, dynamic
  targets, dynamic uploads, prepared views, prepared flow invocation requests,
  viewport/product UI binding intents, surface status, and typed diagnostics.
- `RenderProductSurfaceManifest::with_upload_backed_product_surface_binding`
  marks generic product-surface bindings that require a dynamic upload and
  reports `missing_upload` when the matching upload descriptor is absent.
- Product-surface diagnostics now cover duplicate target/upload keys, missing
  dynamic targets, missing uploads, non-sampleable UI bindings, conflicting
  history signatures, and producer-owned stale/fallback/rejected/unavailable
  status.
- `engine/src/plugins/render/inspect/prepared_frame.rs` adds
  `inspect_render_product_surface_manifest(...)` and inspection DTOs for
  producer, target, upload, view, invocation, UI binding, status, and
  diagnostic data.
- Editor viewport product targets now build a return-only
  `viewport_product_surface_manifest(...)` before explicitly publishing dynamic
  target descriptors and viewport surface bindings.
- Editor viewport and material preview request construction continue to use the
  PM-002 shared helper path for prepared views and flow invocations.
- Editor texture preview upload-backed surfaces now build a
  `texture_preview_product_surface_manifest(...)` and then explicitly publish
  extracted dynamic target and upload contributions.
- Drawing ink preview/final tile surfaces now build a
  `drawing_ink_product_surface_manifest(...)` and then explicitly publish
  extracted dynamic target and upload contributions.
- Render public API docs and the render flow usage guide document the shared
  product-surface manifest, upload-backed binding opt-in, diagnostics, and
  explicit producer publication boundary.

## Validation

Focused implementation validation passed:

```text
cargo test -p engine render_dynamic_targets
cargo test -p engine render_runtime_inspect
cargo test -p engine render_flow
cargo test -p runenwerk_editor viewport
cargo test -p runenwerk_editor material_preview
cargo test -p runenwerk_editor texture_preview
cargo test -p runenwerk_draw product_surface
cargo check -p engine -p runenwerk_editor -p runenwerk_draw
```

Observed focused-test coverage:

- `cargo test -p engine render_dynamic_targets`: 12 dynamic target,
  product-surface manifest, diagnostics, and prepared-frame request tests
  passed.
- `cargo test -p engine render_runtime_inspect`: 12 runtime inspection tests
  passed, including product-surface manifest inspection.
- `cargo test -p engine render_flow`: 17 render flow unit tests, 1 submit
  cutoff guard test, and 3 render flow compiler tests passed.
- `cargo test -p runenwerk_editor viewport`: 107 editor unit tests, 1 startup
  render smoke test, 30 viewport architecture guard tests, and 1 branch truth
  smoke test passed; the real-window GPU smoke remains intentionally ignored.
- `cargo test -p runenwerk_editor material_preview`: 27 material preview tests
  passed.
- `cargo test -p runenwerk_editor texture_preview`: 11 texture preview tests
  passed, including upload-backed product-surface manifest traceability.
- `cargo test -p runenwerk_draw product_surface`: 2 drawing manifest unit tests
  and 1 app-shell product-surface bridge test passed.

Workflow validation passed before closeout:

```text
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
task ai:goal -- --track PT-RENDER-PG
```

## Completion Quality

Completion quality is `bounded_contract`.

This is not `runtime_proven`: the slice proves shared product-surface
contracts, producer migrations, diagnostics, and inspection through focused
tests, but it does not claim backend pixel proof or production capture/replay
evidence across every future product family.

This is not `perfectionist_verified`: later production milestones still own
multi-surface presentation, render fragments and hot reload, and final
production readiness inspection.

## Known Gaps

- `PM-RENDER-PG-006` still owns native multi-surface presentation and
  surface-scoped submit/present lifecycle.
- `PM-RENDER-PG-007` still owns render fragments, fragment validation, hot
  reload, and last-good fragment fallback.
- `PM-RENDER-PG-008` still owns production readiness, capture/replay, budgets,
  final examples, and final inspection evidence.
- PM-005 did not claim `runtime_proven` or `perfectionist_verified` evidence.
