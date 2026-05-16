---
title: WR-018 Rendered World V1 Closeout
description: Completion and drift-check record for the rendered-world V1 viewport scene packet, shader packet, and picking consistency slice.
status: completed
owner: editor
layer: app-runtime / engine-render
canonical: true
last_reviewed: 2026-05-16
related_designs:
  - ../../../design/active/editor-rendered-world-and-multi-entity-viewport-design.md
  - ../../../design/implemented/render-product-surface-foundation-bundle-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-index.md
  - ../../../apps/runenwerk-editor/viewport-expression-implementation-roadmap.md
---

# WR-018 Rendered World V1 Closeout

## Status

Complete as of 2026-05-16.

WR-018 completes the SDF-first editor rendered-world V1 slice: editor SDF primitive entities render and pick through one extracted viewport scene packet. The repaired closeout hardens the initial implementation by removing shader-owned ground geometry, making the default ground plane a normal authored entity, and moving viewport grid visuals into the overlay product path. This closeout does not start Field Visualizer product routing, material preview work, prefab runtime instancing, source-backed asset editor adapters, terrain production, renderer-owned ECS extraction, or storage-buffer scene packet work.

## Owning Scope

- `apps/runenwerk_editor/src/runtime/systems/frame_submit.rs::extract_viewport_scene_render_packet` owns app-runtime scene packet extraction from `RunenwerkEditorRuntime`.
- `apps/runenwerk_editor/src/runtime/resources.rs::EditorViewportSceneRenderPacket` and `EditorViewportRenderState::compose_scene_product_uniform` own the CPU packet and bounded uniform serialization contract.
- `apps/runenwerk_editor/src/runtime/systems/picking.rs::picking_scene_context_for_viewport`, `compose_picking_hit`, and `pick_entity_hit` own CPU picking from the render-state packet.
- `apps/runenwerk_editor/src/editor_runtime/mvp_scene.rs::bootstrap_mvp_scene_if_empty` owns the empty-scene MVP bootstrap for the authored `Graybox` and `Ground Plane` entities.
- `apps/runenwerk_editor/src/runtime/viewport/render_jobs.rs::build_viewport_render_job` owns per-viewport prepared render invocation uniform overrides.
- `assets/shaders/editor_viewport_scene_product.wgsl`, `assets/shaders/editor_viewport_picking_product.wgsl`, and `assets/shaders/editor_viewport_overlay_product.wgsl` consume the viewport uniform data for scene color, picking-id, and overlay products.

## Completion Evidence

- `apps/runenwerk_editor/src/editor_runtime/mvp_scene.rs::bootstrap_mvp_scene_if_empty` now creates two authored entities in an empty runtime document: `Graybox` and `Ground Plane`. The ground plane uses `EditorPrimitiveKind::Plane`, persists through scene files, appears in `EditorViewportSceneRenderPacket`, and is selectable/pickable as an entity.
- `apps/runenwerk_editor/src/runtime/systems/frame_submit.rs::extract_viewport_scene_render_packet` extracts all entities with `scene::LocalTransform` and `EditorPrimitive`, applies stable `EntityId` ordering through `EditorViewportSceneRenderPacket::from_primitives`, and marks selected and hovered primitives in packet flags.
- `apps/runenwerk_editor/src/runtime/resources.rs::EditorViewportSceneRenderPacket` assigns packet-local pick slots, maps pick slots back to full `EntityId` values, and reports the number of primitives omitted past the 64-slot uniform cap.
- `apps/runenwerk_editor/src/runtime/resources.rs::EditorViewportRenderState::set_scene_packet` makes `has_primitive` reflect packet emptiness while retaining the first primitive mirror only for branch trace compatibility.
- `apps/runenwerk_editor/src/runtime/resources.rs::EditorViewportRenderState::compose_scene_product_uniform` serializes the bounded primitive slot transforms, params, and flags into `EditorViewportSceneProductUniform`.
- `apps/runenwerk_editor/src/runtime/viewport/render_jobs.rs::build_viewport_render_job` passes target-local scene uniform bytes into each prepared viewport flow invocation instead of sharing a global viewport packet.
- `assets/shaders/editor_viewport_scene_product.wgsl` raymarches only serialized authored primitive slots and applies selected/hovered visual hints from the shared packet. It no longer contains hardcoded ground geometry or grid fallback rendering.
- `assets/shaders/editor_viewport_picking_product.wgsl` decodes the same primitive kind set and primitive slot layout, then returns the nearest hit packet-local pick slot instead of a truncated entity id.
- `assets/shaders/editor_viewport_overlay_product.wgsl` owns viewport grid rendering as a non-persistent overlay aid.
- `apps/runenwerk_editor/src/runtime/app.rs::register_editor_render_flow` clears scene and overlay product targets before drawing, so transparent overlay pixels and scene ray misses cannot retain old frame contents.
- `apps/runenwerk_editor/src/runtime/systems/picking.rs::picking_scene_context_for_viewport` reads the packet from `ViewportRenderStateResource` and returns no scene context before a render-state packet exists, so CPU picking fails closed instead of independently scanning runtime entities.
- `apps/runenwerk_editor/src/runtime/systems/picking.rs::pick_entity_hit` uses packet-shape hit semantics for V1 primitives, including bounded plane slabs and torus holes, before falling back to `EditorPickingTarget::Grid` only when no entity hit exists.

## Drift Findings

- The active design already matched implementation ownership: app runtime extracts scene reality, render consumes prepared product inputs, and the renderer does not become world truth.
- The initial WR-018 implementation still had a shader-owned ground/floor shortcut. This repair removed that shortcut so authored scene data is the only scene geometry path.
- The viewport expression roadmap already recorded the rendered-world packet, scene shader, and picking shader as implemented foundation state.
- Workspace roadmap state still marked `WR-018` as the current candidate after validation. This closeout updates `WR-018` to completed evidence and removed the rendered-world blocker for `WR-020`; WR-020 completion is recorded separately in `docs-site/src/content/docs/reports/closeouts/wr-020-source-backed-asset-core-contracts/closeout.md`.
- Later work remains deferred: Field Visualizer (`WR-019`), Material Lab (`WR-021`), SDF Prefab V2 runtime instancing (`WR-022`), and Source-backed Asset Editor Adapters (`WR-026`) have not started in this closeout.

## Validation

Required WR-018 validation completed on 2026-05-16:

- `cargo check -p runenwerk_editor` passed.
- `cargo test -p runenwerk_editor viewport` passed: 89 unit tests, 1 scene authoring filtered smoke, 28 viewport architecture guards, and the viewport branch truth smoke passed; the windowed GPU truth smoke remained ignored because it requires a real GPU window.
- `task docs:validate` passed.

Closeout and roadmap validation completed after evidence updates:

- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task docs:validate`

## Deferred Work

- `WR-020` later completed the domain-owned asset source/catalog contracts.
- `WR-026` editor adapters remain deferred until a separate roadmap slice selects editor adapter load/save, import orchestration, diagnostics surfacing, and prior-valid artifact UI over the landed domain contracts.
- Field Visualizer and Material Lab must consume the viewport product routing and rendered-world packet evidence rather than introduce parallel viewer or canvas-truth paths.
