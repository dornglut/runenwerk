---
title: WR-019 Field Visualizer Product Workflow Contract
description: Promotion and implementation-readiness contract for routing field visualization through viewport products.
status: active
owner: apps/runenwerk_editor
layer: app-runtime / editor-ui
canonical: true
last_reviewed: 2026-05-16
related_designs:
  - ../../../design/active/field-visualizer-product-workflow-design.md
  - ../../../design/active/editor-rendered-world-and-multi-entity-viewport-design.md
  - ../../../design/implemented/render-product-surface-foundation-bundle-design.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-track-index.md
  - ../../../workspace/roadmap-index.md
---

# WR-019 Field Visualizer Product Workflow Contract

## Goal

Establish implementation readiness for `WR-019` under the `PM-SDF-OW-001`
production product spine. Field visualization must become a viewport product
workflow: users select field, atlas, volume, brickmap, and history debug
products through existing viewport product routing, and unavailable products
stay inspectable through diagnostics.

This contract establishes implementation readiness; code work starts only from a
later accepted implementation task. It does not implement product behavior and
does not complete `WR-019`.

## Source Of Truth

- `docs-site/src/content/docs/workspace/production-tracks.yaml` owns
  `PT-SDF-OW` and active milestone `PM-SDF-OW-001`. The milestone links
  `WR-019`, `WR-026`, and `WR-021`, and requires product workflows to avoid
  parallel viewers, asset truth stores, and renderer-owned semantic sources.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` owns `WR-019`.
  The row is `ready_next`, blocker `B2`, depends on `WR-018:completed` and
  `WR-003:support_only`, and names selected field preview routing, controls,
  and unavailable-product diagnostics as the next evidence.
- `docs-site/src/content/docs/design/active/field-visualizer-product-workflow-design.md`
  is the active owning design. It requires the visualizer to route through
  viewport products, not through a separate field viewer path.
- `docs-site/src/content/docs/reports/closeouts/wr-018-rendered-world-v1/closeout.md`
  provides the completed rendered-world V1 evidence that WR-019 was waiting on:
  viewport scene packet extraction, product target handoff, render/picking
  consistency, and explicit deferral of Field Visualizer work.
- `docs-site/src/content/docs/engine/roadmaps/fully-featured-renderer-roadmap.md`
  maps `WR-019` to FR-2 product routing and keeps renderer work subordinate to
  prepared product selection.

Readiness checks completed before this contract was written:

- `task production:plan -- --milestone PM-SDF-OW-001 --roadmap WR-019`
  classified the next action as `write_promotion_contract`.
- `task production:validate` passed.
- `task production:check` passed.
- `task roadmap:validate` passed.
- `task roadmap:check` passed.
- `task docs:validate` passed.

## Readiness

Promotion verdict: `WR-019` can honestly carry a bounded implementation
contract, but remains `ready_next` until implementation, validation, and closeout
evidence land.

The promotion is valid because the blocking WR-018 rendered-world packet and
viewport product handoff evidence exists, the production milestone is active,
the WR row links to the milestone, and the active Field Visualizer design names
the owner modules. `WR-003` remains `support_only`; the implementation may use
its product-selection and residency substrate, but must not reopen WR-003 as
part of this slice.

The remaining `B2` blocker is the bounded WR-019 proof itself: route field
debug products through the existing viewport product target, presentation, and
query-snapshot path, with controls and diagnostics. If implementation requires
a new product authority model, new renderer-owned semantic state, or a separate
viewer path, stop and write design work instead.

## Implementation Scope

Owning domain and crate boundaries:

- `domain/editor/editor_viewport` owns reusable viewport expression product and
  presentation contracts.
- `domain/editor/editor_shell` owns shell view models, viewport controls, and
  workspace persistence shape.
- `apps/runenwerk_editor` owns app-runtime product descriptors, target routing,
  shell provider integration, command dispatch, and diagnostics.
- `engine` may be consumed through existing render product-selection APIs only;
  it must not become field-product semantic truth.

Required V1 interface and implementation steps:

1. Add `domain/editor/editor_viewport/src/expression/field_visualizer.rs` and
   re-export it from `domain/editor/editor_viewport/src/expression/mod.rs`.
   The module owns the engine-agnostic field visualizer contract:
   `ViewportFieldVisualizerSettings`,
   `ViewportFieldVisualizerComponent`, `ViewportFieldVisualizerColorRamp`, and
   `ViewportFieldVisualizerDebugMode`.
2. Define `ViewportFieldVisualizerSettings` with exactly these fields:
   `component`, `slice_index`, `color_ramp`, and `debug_mode`. Defaults are
   `component=Auto`, `slice_index=0`, `color_ramp=Grayscale`, and
   `debug_mode=Values`.
3. Define `ViewportFieldVisualizerComponent` values `Auto`, `X`, `Y`, `Z`,
   `W`, and `Magnitude`; `ViewportFieldVisualizerColorRamp` values
   `Grayscale`, `Heat`, and `DivergingSigned`; and
   `ViewportFieldVisualizerDebugMode` values `Values`, `Availability`, and
   `Freshness`.
4. In `domain/editor/editor_viewport/src/expression/presentation.rs`, store
   `ViewportFieldVisualizerSettings` inside `ViewportPresentationState`.
   `selected_primary_product_id` remains the only selected product identity;
   field visualizer settings only parameterize how a selected field-compatible
   product is presented.
5. In `domain/editor/editor_viewport/src/camera.rs::ViewportRuntimeSettings`,
   store the same `ViewportFieldVisualizerSettings` so workspace and runtime
   restoration use one neutral contract.
6. In `domain/editor/editor_shell/src/workspace/persisted.rs`, extend
   `PersistedViewportSettingsV1`, `persisted_viewport_settings`, and
   `workspace_viewport_settings` additively. Old workspaces must decode with
   the V1 defaults, and existing `selected_primary_product_id` round-trips
   unchanged.
7. In `apps/runenwerk_editor/src/runtime/viewport/product_registry.rs`, keep
   `initial_product_descriptors` and producer helpers responsible for stable
   product ids, source reality class, freshness, dimensions, format, producer
   label, and channel/layer/slice descriptor metadata. Product descriptors do
   not encode the visualizer settings as product identity.
8. In `apps/runenwerk_editor/src/runtime/viewport/product_targets.rs`, preserve
   `product_target_record_for_descriptor`, `product_target_slots`, and
   `sync_viewport_product_targets_system` as the only dynamic target route for
   Field Visualizer products. Changing component, slice, ramp, or debug mode
   must not create a new dynamic target key.
9. In `apps/runenwerk_editor/src/runtime/viewport/presentation_resolver.rs`,
   keep `resolve_product_to_surface_slot` as the presentation resolver for
   field, atlas, volume, brickmap, and history products.
10. In `domain/editor/editor_shell/src/composition/build_viewport_panel.rs`
    module `viewport_options_popup`, add controls for component, slice index,
    color ramp, and debug mode. These controls must route through shell
    actions; no local-only UI state is allowed.
11. In `domain/editor/editor_shell/src/surfaces/viewport.rs`, add the viewport
    action and mutation variants needed to update
    `ViewportFieldVisualizerSettings`. Wire them through
    `apps/runenwerk_editor/src/shell/providers/scene/viewport.rs::build_frame`
    and `map_action`, then through
    `apps/runenwerk_editor/src/shell/dispatch/viewport.rs::dispatch_domain_mutation`.
12. In `apps/runenwerk_editor/src/shell/providers/field_product_viewer.rs`
    function `build_frame`, keep the Field Product Viewer status-only:
    descriptors, asset catalog runtime lines, SDF preview lines, procgen
    preview lines, stale-product notes, and reload status may be shown there,
    but product selection and field visualizer parameter mutation remain owned
    by the viewport action path.
13. In `apps/runenwerk_editor/src/shell/dispatch/viewport.rs` function
    `dispatch_select_product`, keep structural target resolution, capability
    checks, availability checks, and ratification as the gate for product
    selection.
14. In `apps/runenwerk_editor/src/runtime/viewport/render_product_selection.rs`
    function `prepare_viewport_render_product_selections`, keep strict
    query-snapshot consumption, residency requests, and diagnostics for selected
    field visualizer products.

Non-goals:

- No parallel field texture viewer or second viewport presentation path.
- No renderer-owned field source, product truth, material truth, or asset truth.
- No new product publication path outside product jobs and query snapshots.
- No terrain, material lab, prefab, water, vegetation, or world simulation work.
- No roadmap completion or production milestone completion update during
  implementation.
- No dynamic target identity changes when only component, slice, ramp, or debug
  mode changes.

ADR requirement: no new ADR is required while implementation stays inside
editor viewport projection contracts. Start architecture governance before code
if field products become authoritative domain truth, product publication rules
change, or renderer ownership expands.

## Acceptance Criteria

- Product selection maps scalar field, vector field, atlas, volume slice,
  brickmap debug, and history color products to
  `ViewportSurfacePresentationSlot::Primary` through existing dynamic target
  routing.
- Unavailable products remain visible in the viewport product list with
  availability and producer-health diagnostics; they do not disappear or
  silently fall back to scene color.
- Field Visualizer controls select or parameterize the product already presented
  by the viewport. Controls do not create duplicate product target identity.
- Per-viewport selected product state persists through existing viewport
  settings. Component, slice, ramp, and debug-mode settings persist per
  viewport/tool surface through `ViewportRuntimeSettings` and workspace
  persistence.
- Viewport option controls produce routed actions through the provider,
  shell-surface action, dispatch, and runtime presentation state path; no button
  is inert and no field visualizer setting is provider-local.
- Render selection consumes strict current query snapshots and emits blocking
  diagnostics for missing or stale selected products.
- The Field Product Viewer provider reports product workflow status without
  becoming semantic product truth or a separate viewer.
- Existing scene color, picking ids, overlay, viewport details, statistics,
  debug-stage, and root-background controls keep working.

## Validation

Focused validation required for the implementation slice:

```text
cargo test -p runenwerk_editor viewport
cargo test -p runenwerk_editor render_product_selection
cargo test -p engine render_product_selection
task roadmap:validate
task production:validate
task docs:validate
```

Broader validation required if public viewport contracts, persistence schema,
or production/roadmap evidence changes:

```text
cargo check -p runenwerk_editor
cargo test -p runenwerk_editor viewport
cargo test -p runenwerk_editor render_product_selection
cargo test -p engine render_product_selection
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task docs:validate
```

Required test scenarios:

- selecting each field visualizer product changes the selected primary product
  and resolves to a sampleable viewport surface when available;
- selecting an unavailable product is rejected or disabled with visible
  diagnostics;
- selected product and field visualizer settings survive viewport lifecycle
  synchronization and workspace persistence round-trip;
- viewport option controls produce routed actions, not inert buttons;
- changing component, slice, ramp, or debug mode updates presentation state
  without changing product target identity;
- render product selection reports missing query snapshots for selected field
  products instead of silently rendering another product;
- Field Product Viewer status reflects descriptor, preview, stale product, and
  reload diagnostics without mutating product selection directly.

## Stop Conditions

Stop implementation and report before writing more code if any of these become
true:

- the implementation needs a new renderer-owned source of field truth;
- product publication must bypass product jobs or query snapshots;
- Field Product Viewer must become a separate texture viewer to satisfy the UI;
- controls require product identity semantics not represented by the active
  design;
- persistence requires a breaking workspace schema change beyond additive
  optional viewport settings;
- field visualizer parameters cannot be represented by
  `ViewportFieldVisualizerSettings`;
- the work expands into WR-021 Material Lab, WR-026 asset adapters, prefab,
  terrain, water, vegetation, or world simulation.

If a stop condition is hit, write the design or ADR/governance work needed
instead of continuing implementation.

## Closeout Requirements

After implementation and validation, create
`docs-site/src/content/docs/reports/closeouts/wr-019-field-visualizer-product-workflow/closeout.md`
with:

- exact files and functions/modules changed;
- evidence for product routing, controls, diagnostics, persistence, and render
  selection;
- validation commands and results;
- explicit deferred work for WR-021, WR-026, prefab, terrain, and later
  production milestones.

Only after closeout evidence exists, update
`docs-site/src/content/docs/workspace/roadmap-items.yaml` for `WR-019`, then
run:

```text
task roadmap:render
task roadmap:validate
task roadmap:check
task docs:validate
```

Update production-track evidence only if `PM-SDF-OW-001` evidence changes, then
run:

```text
task production:render
task production:validate
task production:check
task docs:validate
```

`WR-021` Material Lab remains downstream until WR-019 closeout evidence exists.
