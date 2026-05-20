---
title: Editor Rendered World And Multi Entity Viewport Design
description: Active design for the SDF-first rendered editor world, multi-entity viewport scene packets, and picking/render alignment.
status: active
owner: apps/runenwerk_editor
layer: app-runtime / engine-render
canonical: true
last_reviewed: 2026-05-20
related_designs:
  - ../implemented/render-product-surface-foundation-bundle-design.md
  - ./workspace-viewport-expression-upgrade-design.md
  - ../accepted/sdf-first-production-capability-map.md
related_roadmaps:
  - ../../engine/roadmaps/fully-featured-renderer-roadmap.md
  - ../../apps/runenwerk-editor/roadmap.md
  - ../../apps/runenwerk-editor/viewport-expression-implementation-roadmap.md
  - ../../engine/plugins/render/docs/roadmap.md
---

# Editor Rendered World And Multi Entity Viewport Design

## Status

Active. V1 is the editor viewport rendered-world slice: authored editor entities with `scene::LocalTransform` plus `runenwerk_editor::editor_runtime::EditorPrimitive` are extracted into one deterministic viewport scene packet and rendered together up to the explicit uniform slot cap.

The default MVP ground plane is an authored `EditorPrimitiveKind::Plane` entity, not shader-only scene geometry. The viewport grid is a non-persistent overlay/navigation aid, not an authored entity.

This does not make the renderer own world truth. The editor app owns scene extraction; the engine render plugin only consumes prepared render-flow inputs and product targets.

## Problem

The previous viewport scene product rendered one selected-or-first primitive while analytic picking scanned all primitive entities. That made visual output, picking, statistics, and debug evidence diverge.

The owning code paths are:

- `apps/runenwerk_editor/src/runtime/systems/frame_submit.rs::populate_viewport_render_state`
- `apps/runenwerk_editor/src/runtime/resources.rs::EditorViewportRenderState`
- `apps/runenwerk_editor/src/runtime/systems/picking.rs::produce_editor_picking_system`
- `apps/runenwerk_editor/src/runtime/viewport/render_jobs.rs::build_viewport_render_job`
- `assets/shaders/editor_viewport_scene_product.wgsl`
- `assets/shaders/editor_viewport_picking_product.wgsl`

## Decision

Use an app-runtime viewport scene packet as the first rendered-world contract.

The packet is:

```text
RunenwerkEditorRuntime scene reality
  -> stable editor entity order
  -> EditorViewportSceneRenderPacket
  -> EditorViewportSceneProductUniform
  -> scene color and picking product shaders
```

V1 packet contents:

- stable `editor_core::EntityId`;
- local translation and safe scale;
- current editor SDF primitive kind (`Box`, `Sphere`, `Capsule`, `Cylinder`, `Torus`, and bounded `Plane`) and normalized primitive parameters;
- selected and hovered flags for viewport feedback;
- packet-local pick slots that map shader picking output back to full `editor_core::EntityId` values;
- bounded primitive slots for the current uniform-only render-flow path, with explicit overflow metadata when extraction omits primitives past the slot cap.

The V1 GPU packet is intentionally uniform-backed because prepared render invocations currently support per-invocation uniform overrides, not per-invocation storage-buffer overrides. A later render-flow extension may replace the bounded uniform slots with storage buffers without changing scene extraction ownership.

Long-term renderer sequencing for storage-buffer scene packets, depth/product
outputs, SDF/world rendering, material previews, fragments, and production
inspection is tracked in
[Fully Featured Renderer Roadmap](../../engine/roadmaps/fully-featured-renderer-roadmap.md).

## Invariants

- `apps/runenwerk_editor/src/runtime/systems/frame_submit.rs::extract_viewport_scene_render_packet` is the canonical V1 scene extraction helper.
- Scene packet ordering is sorted by stable editor entity id.
- Rendering and GPU picking products consume the same packet layout.
- Rendering and GPU picking products decode the same editor primitive kind set from that layout.
- Scene color products render only packet-backed authored scene primitives. They must not add shader-owned ground, floor, or grid geometry.
- Overlay products own viewport grid visuals. The grid remains an editor aid and a navigation fallback, not authored scene data.
- Viewport scene and overlay product passes clear their offscreen targets before drawing. Transparent miss pixels must never preserve prior-frame scene or overlay contents.
- GPU picking products return packet-local pick slots, and Rust maps those slots back to full editor entity identities.
- CPU picking uses the same extracted packet shape semantics, not an independent runtime scan or broad AABB approximation with different primitive normalization.
- CPU entity picking consumes the viewport render-state scene packet; when that packet is not available, it reports no entity hit instead of extracting a separate runtime view.
- `EditorViewportRenderState::has_primitive` means "scene packet has at least one renderable primitive".
- Viewport render jobs remain per viewport and target-local.

## V1 Scope

V1 includes:

- multi-entity SDF primitive rendering in editor viewport scene color;
- same primitive slots exposed to the picking-id product shader;
- CPU picking based on `EditorViewportSceneRenderPacket`;
- default empty-scene bootstrap of `Graybox` plus authored `Ground Plane`;
- viewport grid rendering through the overlay product path;
- explicit uniform-slot overflow reporting;
- selected/hovered primitive visual hints;
- viewport FPS/frame-time statistics projected from `engine::DebugMetricsState`;
- debug/branch trace compatibility for the first primitive mirror.

V1 excludes:

- terrain, materials, prefab instancing, and full world streaming;
- general mesh scene extraction;
- renderer-owned ECS extraction;
- per-invocation render storage buffers.

WR-030 may add a Material Lab Mesh Preview product surface for source-backed
model/mesh material proof without changing this V1 authored scene contract. That
preview product is app-owned Material Lab workflow state and renderer-prepared
product input, not general editor-scene mesh extraction. If future work adds
authored mesh entities to the editor scene packet, this design must be updated
through architecture governance before implementation.

## Migration Path

1. Keep app-owned extraction in `apps/runenwerk_editor/src/runtime/systems/frame_submit.rs`.
2. Add a render-flow prepared storage override contract in `engine/src/plugins/render/frame/packet.rs` only when field/material/prefab products need more than the bounded uniform packet.
3. Move shader packet reads from uniform slots to storage slots while preserving `EditorViewportSceneRenderPacket` as the CPU contract.
4. Add material/field/prefab contributions as product producers, not as renderer-owned world state.

## Tests

Required coverage:

- `apps/runenwerk_editor/src/runtime/resources.rs::EditorViewportRenderState::compose_scene_product_uniform`
- `apps/runenwerk_editor/src/runtime/systems/frame_submit.rs::extract_viewport_scene_render_packet`
- `apps/runenwerk_editor/src/runtime/systems/picking.rs::pick_entity_hit`
- `apps/runenwerk_editor/src/runtime/viewport/render_jobs.rs::build_viewport_render_job`
- `assets/shaders/editor_viewport_scene_product.wgsl`
- `assets/shaders/editor_viewport_picking_product.wgsl`
- `assets/shaders/editor_viewport_overlay_product.wgsl`
