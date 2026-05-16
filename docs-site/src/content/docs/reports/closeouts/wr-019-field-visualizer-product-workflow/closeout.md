---
title: WR-019 Field Visualizer Product Workflow Closeout
description: Completion and drift-check record for viewport-owned field visualizer product workflow controls, routing, persistence, and diagnostics.
status: completed
owner: editor
layer: app-runtime / editor-ui
canonical: true
last_reviewed: 2026-05-16
related_designs:
  - ../../../design/active/field-visualizer-product-workflow-design.md
  - ../../../design/active/editor-rendered-world-and-multi-entity-viewport-design.md
  - ../../../design/implemented/render-product-surface-foundation-bundle-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-index.md
  - ../../../engine/roadmaps/fully-featured-renderer-roadmap.md
---

# WR-019 Field Visualizer Product Workflow Closeout

## Status

Complete as of 2026-05-16.

WR-019 completes the Field Visualizer product workflow slice. Field products now flow through viewport product descriptors, viewport presentation state, routed viewport option controls, strict query-snapshot diagnostics, product target records, and persisted viewport settings. The field product viewer remains diagnostics/status-only and does not become a parallel product selection surface.

This closeout does not implement real field producer algorithms, renderer-owned field truth, material graph preview products, storage-buffer scene packets, prefab runtime instancing, terrain production, or source-backed asset editor adapters.

## Owning Scope

- `domain/editor/editor_viewport/src/expression/field_visualizer.rs::ViewportFieldVisualizerSettings` owns the durable V1 field visualizer settings shape.
- `domain/editor/editor_viewport/src/expression/field_visualizer.rs::ViewportFieldVisualizerSettingsPatch::apply_to` owns granular settings mutation and product-aware slice clamping.
- `domain/editor/editor_viewport/src/expression/product.rs::ExpressionChannelLayerSliceMetadata` owns optional product slice bounds.
- `domain/editor/editor_viewport/src/expression/presentation.rs::ViewportPresentationState` owns selected product identity and visualizer presentation settings together without mixing their semantics.
- `domain/editor/editor_viewport/src/expression/observation.rs::ArtifactObservationFrame` exposes selected product identity, availability, producer health, and visualizer settings to editor surfaces.
- `domain/editor/editor_viewport/src/camera.rs::ViewportRuntimeSettings` carries field visualizer settings through retained viewport runtime settings.
- `domain/editor/editor_shell/src/surfaces/viewport.rs::ViewportSurfaceAction` and `ViewportDomainMutation` own the typed `PatchFieldVisualizerSettings` surface/domain contract.
- `domain/editor/editor_shell/src/composition/build_viewport_panel.rs::viewport_options_popup` owns the visible component, slice, ramp, and debug-mode controls.
- `domain/editor/editor_shell/src/workspace/persisted.rs::persisted_viewport_settings`, `workspace_viewport_settings`, `persisted_viewport_field_visualizer_settings`, and `workspace_viewport_field_visualizer_settings` own workspace round-trip defaults and schema compatibility.
- `apps/runenwerk_editor/src/shell/providers/scene/viewport.rs::SceneViewportProvider::build_frame` and `map_action` own provider-local route creation and mapping to shell domain mutations.
- `apps/runenwerk_editor/src/shell/dispatch/viewport.rs::dispatch_domain_mutation` and `dispatch_patch_field_visualizer_settings` own command target resolution, stale-route fail-closed behavior, patch application, and selected-product identity preservation.
- `apps/runenwerk_editor/src/runtime/viewport/settings_hydration.rs::ViewportRuntimeSettingsHydrationResource` owns one-shot persisted settings hydration so old workspace settings do not overwrite later user-dispatched presentation changes.
- `apps/runenwerk_editor/src/runtime/systems/viewport_lifecycle.rs::sync_viewport_instances_system` and `hydrate_presentation_from_viewport_settings` own lifecycle restore and persistence composition.
- `apps/runenwerk_editor/src/runtime/viewport/product_registry.rs::initial_product_descriptors`, `producer_field.rs`, and `producer_volume.rs` own field, vector, atlas, volume, brickmap, and history debug descriptor metadata.
- `apps/runenwerk_editor/src/runtime/viewport/product_targets.rs::sync_viewport_presentation_products_system` owns propagation of field visualizer settings into observation frames while keeping dynamic target identity stable.
- `apps/runenwerk_editor/src/shell/providers/mod.rs::build_viewport_observation_frame` owns field visualizer settings in viewport surface observations.
- `apps/runenwerk_editor/src/runtime/resources.rs::EditorViewportRenderState::viewport_settings` owns runtime settings composition for workspace persistence.
- `apps/runenwerk_editor/src/runtime/systems/bootstrap.rs::seed_viewport_runtime_contracts_system` seeds bootstrap observations with the default presentation settings.

## Completion Evidence

- `ViewportFieldVisualizerSettings` defines the V1 contract: `component` (`Auto`, `X`, `Y`, `Z`, `W`, `Magnitude`), `slice_index`, `color_ramp` (`Grayscale`, `Heat`, `DivergingSigned`), and `debug_mode` (`Values`, `Availability`, `Freshness`).
- `ViewportFieldVisualizerSettingsPatch` replaces whole-setting UI snapshots with granular patches: `SetComponent`, `SetSliceIndex`, `StepSliceIndex`, `SetColorRamp`, and `SetDebugMode`.
- `dispatch_patch_field_visualizer_settings` resolves the active viewport binding before mutation, applies a patch to the current presentation state, preserves selected product identity, and clamps slice changes against the selected product descriptor when `slice_count` is known.
- `ExpressionChannelLayerSliceMetadata::slice_count` creates the product-metadata seam for bounded slices. Existing products publish `None` unless they know a count; the current volume slice debug product publishes one slice.
- `SceneViewportProvider::build_frame` routes every field visualizer control as a `PatchFieldVisualizerSettings` action. Buttons no longer carry stale full-settings snapshots.
- `viewport_options_popup` renders component, slice, color ramp, and debug-mode controls from viewport surface view models, including disabled decrement at slice `0`.
- `ViewportRuntimeSettingsHydrationResource` tracks restored persisted settings by tool surface, viewport, selected product, and visualizer settings. Lifecycle hydration restores persisted non-default settings once, then user-dispatched presentation changes win until persistence records new settings.
- `persisted.rs` round-trips field visualizer settings additively and defaults older workspace files to `Auto`, slice `0`, `Grayscale`, and `Values`.
- Product target synchronization carries visualizer settings through observation frames without using them to create dynamic target keys.
- Unavailable field products remain visible through existing product descriptor and diagnostics paths, and selected unavailable products still fail closed through strict query-snapshot diagnostics.

## Drift Findings

- The accepted implementation stayed within the design boundary: viewport presentation state owns visualizer parameters; product descriptors and target records own stable product identity.
- The first implementation snapshot had two long-term risks: stale whole-setting UI routes could overwrite newer settings, and lifecycle restore only hydrated when presentation state was missing. Both are repaired by granular patches and explicit hydration tracking.
- Slice controls now have a durable metadata seam. Unknown slice counts remain unbounded above while always flooring at `0`; known counts clamp to descriptor bounds.
- No ADR is required because field product truth did not move into renderer code and product publication authority did not change.
- The Field Product Viewer did not gain direct selection authority; it remains diagnostics/status-only as required by the contract.

## Validation

Implementation validation completed on 2026-05-16:

- `cargo test -p editor_viewport field_visualizer_patch` passed.
- `cargo test -p runenwerk_editor dispatch_shell_command_updates_viewport_field_visualizer_settings_without_changing_product` passed.
- `cargo test -p runenwerk_editor dispatch_shell_command_clamps_field_visualizer_slice_step_to_selected_product_metadata` passed.
- `cargo test -p runenwerk_editor provider_local_viewport_field_controls_are_routed_actions` passed.
- `cargo test -p runenwerk_editor lifecycle_hydration` passed.
- `cargo test -p runenwerk_editor render_product_selection` passed.
- `cargo test -p runenwerk_editor viewport` passed: 96 unit tests, the split viewport resize smoke, 29 viewport architecture guards, and the viewport branch truth smoke passed; the windowed GPU truth smoke remained ignored because it requires a real windowed GPU device.
- `cargo test -p engine render_product_selection` passed.
- `task roadmap:validate` passed.
- `task roadmap:check` passed.
- `task docs:validate` passed.
- `task batch:validate -- --batch docs-site/src/content/docs/reports/batches/2026-05-16-next-current-candidate-roadmap-batch-wr-019/batch.toml` passed from the main repository, validating the WR-019 worktree.

Closeout and roadmap validation completed after evidence updates:

- `task docs:validate`
- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task docs:validate`
- `task production:validate` passed from the main repository Taskfile.
- `task production:check` passed from the main repository Taskfile.

Production validation was requested after implementation. The WR-019 worktree Taskfile does not contain `production:validate` or `production:check`; the production gates were run from the main repository Taskfile that contains those tasks.

## Deferred Work

- Real field product producers remain later work. WR-019 landed the viewport workflow, contracts, routing, persistence, and diagnostics path that those producers must use.
- Future producers should publish real `slice_count` metadata when known instead of relying on the open-ended fallback.
- Material Lab (`WR-021`), SDF Prefab V2 runtime instancing (`WR-022`), source-backed asset editor adapters (`WR-026`), terrain production, storage-buffer scene packets, and renderer-owned SDF/world work remain separate roadmap slices.
