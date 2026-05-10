# Crate Inventory

## Purpose

This document records the active workspace crates and their intended layer. The canonical docs tree is `docs-site/src/content/docs`; this root file is the quick crate map for contributors and agents.

Regenerate or audit this list from `cargo metadata --no-deps` whenever workspace members change.

## Layer Rules

- `foundation`: low-level reusable primitives with no domain/runtime/app dependencies.
- `domain`: engine-agnostic reusable contracts and logic.
- `engine/runtime`: runtime composition, plugins, renderer/backend integration, and app loop glue.
- `net`: transport/session/replication/history contracts and transport adapters.
- `app`: runnable applications and tools.
- `adapter/tool`: external host integrations and tooling glue.

## Active Workspace Crates

| Crate | Path | Layer | Purpose | Public API |
| --- | --- | --- | --- | --- |
| `id` | `foundation/id` | foundation | Typed identity primitives and allocators. | evolving |
| `ecs` | `domain/ecs` | domain | Entity/component/resource world, query, messaging, ownership, and system runtime contracts. | evolving |
| `ecs_macros` | `domain/ecs_macros` | domain | Derive macros for ECS component/resource/bundle/reflection contracts. | evolving |
| `id_macros` | `foundation/id_macros` | foundation | Attribute macro support for typed ID wrappers. | evolving |
| `diagnostics` | `foundation/diagnostics` | foundation | Structured diagnostic reporting vocabulary. | evolving |
| `ratification` | `foundation/ratification` | foundation | Shared ratification report, issue, status, ratifier, and diagnostics-bridge vocabulary. | evolving |
| `schema` | `foundation/schema` | foundation | Portable schema identity, version, path, value, shape, field, constraint, metadata, and descriptor vocabulary. | evolving |
| `commands` | `foundation/commands` | foundation | Portable command contract identity, schema reference, descriptor, proposal, metadata, hint, issue, and optional diagnostics-bridge vocabulary. | evolving |
| `geometry` | `domain/geometry` | domain | Explicit geometric primitives and geometric queries. | evolving |
| `spatial` | `domain/spatial` | domain | World/grid/chunk/clipmap/ring spatial coordinate contracts. | evolving |
| `spatial_index` | `domain/spatial_index` | domain | Spatial index traits and spatial hash implementation. | evolving |
| `chunking` | `domain/chunking` | domain | Chunk streaming focus, policy, set, diff, and streamer logic. | evolving |
| `asset` | `domain/asset` | domain | Engine-agnostic asset identity, source/artifact descriptors, deterministic import planning, dependency graph, diagnostics, and ratification contracts. | evolving |
| `sdf` | `domain/sdf` | domain | Signed-distance-field primitives, transforms, composition, and queries. | evolving |
| `world_ops` | `domain/world_ops` | domain | Chunk/world operation logs, dirty tracking, build queues, invalidation, and replication deltas. | evolving |
| `world_sdf` | `domain/world_sdf` | domain | SDF chunk payloads, collision query contracts, formed field-product descriptors, ratification, and cave/sector storage summaries. | evolving |
| `scheduler` | `domain/scheduler` | domain | Deterministic schedule planning, graph validation, labels, access, and system execution plans. | evolving |
| `graph` | `domain/graph` | domain | Domain-neutral graph definitions, typed ports, validation, traversal, and cycle policy. | evolving |
| `texture` | `domain/texture` | domain | Texture2D, Texture3D/volume, generated texture product, preview descriptor, sampler, color-space, compression, ratification, and lineage contracts. | evolving |
| `material_graph` | `domain/material_graph` | domain | Authored material graph documents, node catalog boundaries, ratification, lowering, source maps, cache keys, and formed material product descriptors. | evolving |
| `drawing` | `domain/drawing` | domain | Drawing document, stroke, brush, paper, layer/composition graph, command, ratification, diagnostic, and tile-lineage contracts. | evolving |
| `scene` | `domain/scene` | domain | Scene transform value contracts and domain-owned transform schema descriptors. | evolving |
| `ui_math` | `domain/ui/ui_math` | domain | UI geometry primitives. | evolving |
| `ui_input` | `domain/ui/ui_input` | domain | UI input, focus, routing, keyboard, pointer, and shortcut contracts. | evolving |
| `ui_layout` | `domain/ui/ui_layout` | domain | Stateless UI layout algorithms and constraints. | evolving |
| `ui_text` | `domain/ui/ui_text` | domain | Text styles, buffers, atlas metrics, layout, cursor, and selection contracts. | evolving |
| `ui_theme` | `domain/ui/ui_theme` | domain | UI color, spacing, radius, typography, and theme tokens. | evolving |
| `ui_render_data` | `domain/ui/ui_render_data` | domain | Renderer-facing UI frame, layer, primitive, color, and batching contracts. | evolving |
| `ui_surface` | `domain/ui/ui_surface` | domain | Surface definition, mount, observation, session, presentation, intent, capability, and ratification contracts. | evolving |
| `ui_definition` | `domain/ui/ui_definition` | domain | Authored UI templates, slots, repeaters, embeds, menus, availability, validation, normalization, and retained UI formation. | evolving |
| `ui_tree` | `domain/ui/ui_tree` | domain | Retained UI tree, widget IDs, nodes, and computed layout records. | evolving |
| `ui_runtime` | `domain/ui/ui_runtime` | domain | Retained UI runtime orchestration, input dispatch, layout, frame output, and runtime state. | evolving |
| `ui_widgets` | `domain/ui/ui_widgets` | domain | Widget constructors over `ui_tree` contracts. | evolving |
| `editor_core` | `domain/editor/editor_core` | domain | Editor command, capability, transaction, ratification, migration, selection, session, sharing, and workflow contracts. | evolving |
| `editor_definition` | `domain/editor/editor_definition` | domain | Editor-owned definition documents, lifecycle validation, workspace/profile/layout, menu, shortcut, theme, command binding, panel registry, tool-surface, toolbar, shell chrome, route, availability, and surface template schemas. | evolving |
| `editor_preview` | `domain/editor/editor_preview` | domain | Engine-agnostic preview session, command, event, reload decision/status, runtime product, and bootstrap DTOs for external runtime preview. | evolving |
| `editor_inspector` | `domain/editor/editor_inspector` | domain | Inspector model, editing, target, resolution, bridge, schema interop, session, and validation contracts. | evolving |
| `editor_scene` | `domain/editor/editor_scene` | domain | Editor scene model, commands, command descriptors, proposal adapter, command bridge, and scene command contracts. | evolving |
| `editor_viewport` | `domain/editor/editor_viewport` | domain | Editor viewport camera, expression, hit, overlay, snap, and viewport contracts. | evolving |
| `editor_shell` | `domain/editor/editor_shell` | domain | Editor shell composition, workspace identity, observation, expression, command routing, and view models. | evolving |
| `editor_persistence` | `domain/editor/editor_persistence` | domain | Editor persistence formats, RON codec, scene migration/normalization/formation, and change-log contracts. | evolving |
| `engine` | `engine` | engine/runtime | App/runtime/plugin composition, render/input/time/scene/world/net integration, and renderer execution. | evolving |
| `engine_render_macros` | `engine_render_macros` | engine/runtime | Render derive macros for GPU parameter contracts. | evolving |
| `engine_sim` | `net/engine_sim` | net | Simulation identity, tick, hash, profile, RNG, command-frame, and codec vocabulary. | evolving |
| `engine_net` | `net/engine_net` | net | Transport-agnostic protocol, replication, session, runtime, simulation, and transport contracts. | evolving |
| `engine_net_macros` | `net/engine_net_macros` | net | Attribute macros for network component/entity metadata. | evolving |
| `engine_net_quic` | `net/engine_net_quic` | net | QUIC transport/runtime adapter for `engine_net`. | evolving |
| `engine_replay` | `net/engine_history` | net | Replay archive, recorder, controller, policy, and validation substrate. | evolving |
| `runenwerk_editor` | `apps/runenwerk_editor` | app | Runnable editor app, shell bridge, runtime viewport/render integration, persistence, and authoring workflows. | internal/evolving |
| `runenwerk_runtime_preview` | `apps/runenwerk_runtime_preview` | app | External runtime preview child process, loopback QUIC bootstrap host, preview/play app shell, and headless preview test harness. | internal/evolving |
| `godot_chunking_addon` | `adapters/godot_chunking_addon` | adapter/tool | Godot bridge for chunking/spatial integration. | internal |

## Documentation Status

The docs-site crate index should treat crate documentation as one of:

- `current`: factual current architecture or usage.
- `roadmap`: target work or migration plan.
- `historical`: proposal or decision record that is not current-state documentation.
- `missing`: no useful crate-specific docs yet.

Missing or thin current docs should be tracked in `docs-site/src/content/docs/workspace/crate-docs-status.md`.
