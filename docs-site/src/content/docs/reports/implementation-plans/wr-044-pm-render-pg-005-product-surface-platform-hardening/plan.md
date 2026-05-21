---
title: WR-044 PM-RENDER-PG-005 Product Surface Platform Hardening Plan
description: Promotion and implementation-readiness contract for the bounded PM-RENDER-PG-005 shared product-surface hardening slice.
status: active
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
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-044 PM-RENDER-PG-005 Product Surface Platform Hardening Plan

## Goal

Promote and implement `PM-RENDER-PG-005` as one bounded shared
product-surface hardening slice.

The slice makes flow-backed and upload-backed product-surface producers share
one typed, inspectable manifest and diagnostics vocabulary for dynamic targets,
dynamic uploads, prepared views, prepared flow invocations, UI bindings,
history signatures, and product-surface status.

The renderer remains an execution and presentation layer. It must not own
product truth, product selection, freshness, authority, fallback legality,
rebuild policy, dependency truth, drawing semantics, material truth, field
truth, or residency policy.

## Source Of Truth

- Production milestone: `PM-RENDER-PG-005`.
- Bounded implementation row: `WR-044`.
- Accepted PM-005 design:
  `docs-site/src/content/docs/design/accepted/product-surface-platform-hardening-design.md`.
- Boundary design:
  `docs-site/src/content/docs/design/accepted/render-product-graph-platform-design.md`.
- PM-002 prerequisite closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-render-pg-002-render-contract-ergonomics/closeout.md`.
- PM-004 prerequisite closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-render-pg-004-render-execution-graph-compiler-maturity/closeout.md`.
- `WR-003` remains support-only context. It is not the PM-005 implementation
  row.

## Readiness

`task production:plan -- --milestone PM-RENDER-PG-005 --roadmap WR-044`
reported:

- milestone state: `ready_next`;
- WR state: `ready_next`;
- dependencies `WR-041:completed` and `WR-043:completed`;
- next action: `write_promotion_contract`;
- promotion preflight: `promotable`.

Promotion command, after this contract is linked and validation passes:

```text
task roadmap:promote -- --id WR-044 --state current_candidate --evidence "Accepted PM-RENDER-PG-005 product-surface platform hardening design and promotion/readiness contract at docs-site/src/content/docs/reports/implementation-plans/wr-044-pm-render-pg-005-product-surface-platform-hardening/plan.md"
```

Do not promote if validation fails, if source files changed enough that
`task ai:goal -- --track PT-RENDER-PG` must be rerun, or if another current
candidate blocks promotion and the workflow requires
`task roadmap:switch-current`.

## Implementation Scope

Owned areas:

```text
engine/src/plugins/render
engine/tests
domain/ui/ui_render_data
apps/runenwerk_editor/src/runtime/viewport
apps/runenwerk_editor/src/runtime/systems/material_preview.rs
apps/runenwerk_editor/src/runtime/systems/texture_preview.rs
apps/runenwerk_editor/src/shell/providers
apps/runenwerk_editor/tests
apps/runenwerk_draw/src
apps/runenwerk_draw/tests
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/engine/roadmaps/fully-featured-renderer-roadmap.md
```

Expected implementation modules or nearby owners:

```text
engine/src/plugins/render/frame/product_surface.rs
engine/src/plugins/render/frame/packet.rs
engine/src/plugins/render/resource/dynamic_target.rs
engine/src/plugins/render/resource/dynamic_upload.rs
engine/src/plugins/render/inspect/prepared_frame.rs
engine/src/plugins/render/graph/prepared_validation.rs
domain/ui/ui_render_data/src/primitives/product_surface.rs
domain/ui/ui_render_data/src/primitives/viewport_surface_embed.rs
apps/runenwerk_editor/src/runtime/viewport/render_jobs.rs
apps/runenwerk_editor/src/runtime/viewport/product_targets.rs
apps/runenwerk_editor/src/runtime/systems/material_preview.rs
apps/runenwerk_editor/src/runtime/systems/texture_preview.rs
apps/runenwerk_editor/src/shell/providers/field_product_viewer.rs
apps/runenwerk_draw/src/app/presentation.rs
```

Use nearby module names if implementation shows a better local fit, but keep
the boundaries explicit. Do not create catch-all helper files.

## Required Contracts

The implementation must add or refine typed, inspectable contracts for:

- a producer-scoped product-surface manifest or request batch;
- flow-backed surface requests with dynamic targets, prepared views, prepared
  flow invocations, target aliases, and history signatures;
- upload-backed surface requests with dynamic upload descriptors and UI binding
  traceability;
- producer-scoped diagnostics for missing, duplicate, stale, fallback,
  rejected, unavailable, conflicting, or non-sampleable product surfaces;
- UI binding traceability for viewport embeds and generic product-surface
  primitives;
- prepared-frame and product-surface inspection by producer id, product family,
  target key, upload key, view id, invocation id, UI binding source, history
  signature, status, and diagnostics.

Helpers must remain return-only. Producer systems must still explicitly publish
into ECS resources and UI binding registries.

## Implementation Steps

1. Extend `engine/src/plugins/render/frame/product_surface.rs` with typed
   producer manifest/request-batch DTOs that can represent both flow-backed and
   upload-backed product surfaces.
2. Add typed product-surface diagnostics and status inspection without moving
   product policy into the renderer.
3. Preserve the PM-002 viewport and material preview helper path while adapting
   it to the richer manifest shape.
4. Add upload-backed manifest coverage for texture/debug preview surfaces.
5. Add field/debug status traceability so field visualizer availability,
   freshness, fallback, rejected, and unavailable states remain producer-owned
   but visible through the shared product-surface diagnostics path.
6. Add drawing product-surface traceability for preview/final tile surfaces
   without moving drawing semantics out of `domain/drawing` or
   `apps/runenwerk_draw`.
7. Extend prepared-frame/product-surface inspection to expose the new manifest,
   diagnostics, UI binding, upload, and history fields.
8. Update render public API docs and the renderer roadmap with the shared
   product-surface producer contract.
9. Add focused tests for helper-built manifests, diagnostics, UI binding
   traceability, upload-backed surfaces, field/debug status, drawing surfaces,
   and inspection.
10. After validation passes, create closeout evidence and only then update
    `PM-RENDER-PG-005` completion metadata.

## Explicit Non-Goals

Do not implement:

- native multi-window, surface-scoped swapchains, or presentation lifecycle
  from `PM-RENDER-PG-006`;
- render fragments, fragment assets, fragment registry, merge provenance, hot
  reload, or last-good fragment promotion from `PM-RENDER-PG-007`;
- production readiness budgets, capture/replay policy, final examples, release
  inspection, or broad observability closeout from `PM-RENDER-PG-008`;
- SDF brick/page-table, clipmap, raymarch acceleration, mesh/material truth,
  material lowering, drawing semantics, or product-family source ownership;
- renderer-owned product selection, freshness, authority, fallback legality,
  rebuild policy, dependency truth, or residency policy;
- renderer-private backend handles in apps, domains, UI, fragments, or product
  producer contracts.

## Acceptance Criteria

- Flow-backed and upload-backed product-surface producers use the same manifest
  and diagnostics vocabulary.
- Viewport and material preview continue to use the shared helper path.
- Texture/debug preview surfaces are traceable to declared dynamic product
  surface requests.
- Field visualizer status remains viewport/product-owned and does not create a
  parallel renderer viewer path.
- Drawing preview/final product surfaces stay drawing-owned while using the
  shared product-surface contract.
- UI bindings for viewport embeds and product-surface primitives are
  backend-neutral and traceable to declared producer requests.
- Non-sampleable, missing, stale, fallback, rejected, and unavailable surfaces
  are diagnosable without moving product policy into the renderer.
- Prepared-frame/product-surface inspection exposes producer, target, upload,
  view, invocation, UI binding, history, status, and diagnostic data.
- No out-of-scope multi-window, fragment, material-lowering, SDF acceleration,
  production-readiness, or renderer-owned product-policy work lands in WR-044.

## Validation

Focused implementation validation must include:

```text
cargo test -p engine render_dynamic_targets
cargo test -p engine render_runtime_inspect
cargo test -p engine render_flow
cargo test -p runenwerk_editor viewport
cargo test -p runenwerk_editor material_preview
cargo test -p runenwerk_editor texture_preview
cargo test -p runenwerk_draw product_surface
```

Add or extend tests for:

- helper-built product-surface manifests;
- flow-backed and upload-backed request batches;
- duplicate, missing, stale, fallback, rejected, unavailable, and
  non-sampleable diagnostics;
- viewport and material preview helper-path preservation;
- texture/debug preview dynamic product-surface traceability;
- field/debug producer status diagnostics;
- drawing product-surface tile traceability;
- prepared-frame/product-surface inspection over manifest-built requests;
- submit/preflight still consuming prepared data only.

Workflow validation:

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

## Stop Conditions

Stop and report instead of coding if:

- `task ai:goal -- --track PT-RENDER-PG` no longer selects PM-005/WR-044;
- `task production:plan -- --milestone PM-RENDER-PG-005 --roadmap WR-044`
  no longer reports a promotable or actionable row;
- promotion fails for anything other than exact metadata repair or a required
  current-candidate switch;
- implementation needs native multi-window, render fragments, material
  lowering, SDF acceleration, drawing semantic changes, or product policy;
- diagnostics would require renderer-owned product truth or backend handle
  leakage;
- validation fails and cannot be repaired inside the bounded WR-044 scope.

## Closeout Requirements

Closeout evidence must be created only after implementation and validation
pass.

Closeout path:

```text
docs-site/src/content/docs/reports/closeouts/pm-render-pg-005-product-surface-platform-hardening/closeout.md
```

After closeout:

- archive `WR-044` with completed evidence;
- add the closeout path to WR-044 write scopes before archival;
- update `PM-RENDER-PG-005` evidence gates, completion audit, and completion
  quality;
- rerun roadmap, production, docs, planning, and goal validation.

## Perfectionist Closeout Audit

Expected completion quality is `bounded_contract`.

This slice can prove a shared product-surface producer contract, typed
diagnostics, UI binding traceability, upload-backed and flow-backed coverage,
inspection, and ownership preservation. It should not claim `runtime_proven`
unless closeout includes runtime evidence that exercises the accepted path
through backend execution for the migrated producer families. It must not claim
`perfectionist_verified` while PM-006 through PM-008 remain incomplete.

Known quality gaps expected at closeout unless later evidence proves otherwise:

- PM-RENDER-PG-006 still owns multi-surface presentation.
- PM-RENDER-PG-007 still owns render fragments and hot reload.
- PM-RENDER-PG-008 still owns production readiness, capture/replay, budgets,
  final examples, and final inspection evidence.
- PM-005 must not move product truth or product policy into the renderer.
