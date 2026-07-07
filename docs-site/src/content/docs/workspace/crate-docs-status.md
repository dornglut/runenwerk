---
title: "Crate Documentation Status"
description: "Current documentation coverage status for Runenwerk workspace crates."
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-07
---

# Crate Documentation Status

This page tracks whether each workspace crate has current, discoverable documentation. Use `CRATES.md` at the repository root for the quick crate inventory.

Status labels:

- `current`: factual current architecture or usage docs exist.
- `thin`: some docs exist, but they do not yet cover the crate's current public surface.
- `roadmap`: roadmap/proposal docs exist but current-state docs are missing.
- `missing`: no useful crate-specific docs yet.

## Foundation

| Crate | Path | Status | Primary docs |
| --- | --- | --- | --- |
| `id` | `foundation/id` | current | [`../foundation/id/README.md`](../foundation/id/README.md) |
| `id_macros` | `foundation/id_macros` | current | [`../foundation/id-macros/README.md`](../foundation/id-macros/README.md) |
| `diagnostics` | `foundation/diagnostics` | current | [`../foundation/diagnostics/current-state.md`](../foundation/diagnostics/current-state.md) |
| `ratification` | `foundation/ratification` | current | [`../foundation/ratification/README.md`](../foundation/ratification/README.md) |
| `schema` | `foundation/schema` | current | [`../foundation/schema/README.md`](../foundation/schema/README.md) |
| `commands` | `foundation/commands` | current | [`../foundation/commands/README.md`](../foundation/commands/README.md) |
| `resource_ref` | `foundation/resource_ref` | current | [`../foundation/resource-ref/README.md`](../foundation/resource-ref/README.md) |

## Domain

| Crate | Path | Status | Primary docs |
| --- | --- | --- | --- |
| `ecs` | `domain/ecs` | current | [`../domain/ecs/README.md`](../domain/ecs/README.md) |
| `ecs_macros` | `domain/ecs_macros` | thin | [`../domain/ecs-macros/README.md`](../domain/ecs-macros/README.md) |
| `geometry` | `domain/geometry` | current | [`../domain/geometry/README.md`](../domain/geometry/README.md) |
| `spatial` | `domain/spatial` | thin | [`../domain/spatial/README.md`](../domain/spatial/README.md) |
| `spatial_index` | `domain/spatial_index` | thin | [`../domain/spatial-index/README.md`](../domain/spatial-index/README.md) |
| `chunking` | `domain/chunking` | thin | [`../domain/chunking/README.md`](../domain/chunking/README.md) |
| `world_streaming` | `domain/world_streaming` | current | [`../domain/world-streaming/README.md`](../domain/world-streaming/README.md) |
| `asset` | `domain/asset` | thin | [`../domain/00-overview.md`](../domain/00-overview.md) |
| `product` | `domain/product` | thin | [`../domain/00-overview.md`](../domain/00-overview.md) |
| `sdf` | `domain/sdf` | current | [`../domain/sdf/README.md`](../domain/sdf/README.md) |
| `world_ops` | `domain/world_ops` | thin | [`../domain/world-ops/README.md`](../domain/world-ops/README.md) |
| `world_sdf` | `domain/world_sdf` | thin | [`../domain/world-sdf/README.md`](../domain/world-sdf/README.md) |
| `scheduler` | `domain/scheduler` | thin | [`../domain/scheduler/README.md`](../domain/scheduler/README.md) |
| `graph` | `domain/graph` | current | [`../domain/graph/README.md`](../domain/graph/README.md) |
| `texture` | `domain/texture` | thin | [`../domain/texture/README.md`](../domain/texture/README.md) |
| `material_graph` | `domain/material_graph` | thin | [`../domain/material-graph/README.md`](../domain/material-graph/README.md) |
| `procgen` | `domain/procgen` | thin | [`../domain/procgen/README.md`](../domain/procgen/README.md) |
| `drawing` | `domain/drawing` | current | [`../domain/drawing/README.md`](../domain/drawing/README.md) |
| `scene` | `domain/scene` | thin | [`../domain/scene/README.md`](../domain/scene/README.md) |
| `domain/ui/*` | `domain/ui` | current | [`../domain/ui/architecture.md`](../domain/ui/architecture.md) |
| `domain/editor/*` | `domain/editor` | thin | [`../domain/editor/README.md`](../domain/editor/README.md) |

## Engine And Net

| Crate | Path | Status | Primary docs |
| --- | --- | --- | --- |
| `engine` | `engine` | current | [`../engine/index.md`](../engine/index.md) |
| `engine_render_macros` | `engine_render_macros` | current | [`../engine/reference/plugins/render/render-macros.md`](../engine/reference/plugins/render/render-macros.md) |
| `engine_sim` | `net/engine_sim` | current | [`../net/engine-sim/README.md`](../net/engine-sim/README.md) |
| `engine_net` | `net/engine_net` | current | [`../net/engine-net/README.md`](../net/engine-net/README.md) |
| `engine_net_macros` | `net/engine_net_macros` | current | [`../net/engine-net-macros/README.md`](../net/engine-net-macros/README.md) |
| `engine_net_quic` | `net/engine_net_quic` | current | [`../net/engine-net-quic/README.md`](../net/engine-net-quic/README.md) |
| `engine_replay` | `net/engine_history` | current | [`../net/engine-history/README.md`](../net/engine-history/README.md) |

## Apps And Adapters

| Crate | Path | Status | Primary docs |
| --- | --- | --- | --- |
| `runenwerk_editor` | `apps/runenwerk_editor` | current | [`../apps/runenwerk-editor/current-architecture.md`](../apps/runenwerk-editor/current-architecture.md) |
| `runenwerk_draw` | `apps/runenwerk_draw` | current | [`../apps/runenwerk-draw/README.md`](../apps/runenwerk-draw/README.md) |
| `runenwerk_runtime_preview` | `apps/runenwerk_runtime_preview` | current | [`../reports/closeouts/m5-runtime-preview/closeout.md`](../reports/closeouts/m5-runtime-preview/closeout.md) |
| `ui_counter_runtime` | `apps/ui_counter_runtime` | current | [`../apps/ui-counter-runtime/README.md`](../apps/ui-counter-runtime/README.md) |
| `godot_chunking_addon` | `adapters/godot_chunking_addon` | current | [`../adapters/godot-chunking-addon/README.md`](../adapters/godot-chunking-addon/README.md) |
| `native_tablet_input` | `adapters/native_tablet_input` | current | [`../adapters/native-tablet-input/README.md`](../adapters/native-tablet-input/README.md) |

## Long-Term Maintenance

When workspace crates are added, removed, split, or renamed:

1. Update `Cargo.toml`.
2. Update root `CRATES.md`.
3. Update root `DOMAIN_MAP.md` when ownership changes.
4. Update this status page.
5. Run `task docs:validate`.
