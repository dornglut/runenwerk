---
title: Domain Layer Overview
description: Overview of Runenwerk's engine-agnostic domain layer, ownership rules, and domain documentation map.
status: active
owner: domain
layer: domain
canonical: true
last_reviewed: 2026-05-16
---

# Domain Layer Overview

## Purpose

The domain layer contains engine-agnostic reusable contracts, models, invariants, and domain logic.

Domain crates define what concepts mean and what rules they must obey. They do not own app wiring, backend integration, renderer execution, editor application policy, or runtime orchestration.

## Dependency Rule

Domain crates may depend on foundation crates and carefully selected lower-level domain contract crates.

Domain crates must not depend on:

- runtime/app code;
- backend adapters;
- editor application wiring;
- AI integrations;
- concrete rendering, windowing, input, or audio backends unless the domain explicitly owns that backend.

## Current Domain Areas

| Area | Crates | Primary docs |
| --- | --- | --- |
| ECS | `domain/ecs`, `domain/ecs_macros` | [`ecs/00-overview.md`](./ecs/00-overview.md), [`ecs/README.md`](./ecs/README.md), [`ecs-macros/README.md`](./ecs-macros/README.md) |
| Scheduler | `domain/scheduler` | [`scheduler/README.md`](./scheduler/README.md), [`scheduler/design-goals.md`](./scheduler/design-goals.md) |
| Scene | `domain/scene` | [`scene/README.md`](./scene/README.md) |
| Asset and product contracts | `domain/asset`, `domain/product` | Workspace-level current-state contract summaries live in [`../workspace/sdf-first-execution-roadmap.md`](../workspace/sdf-first-execution-roadmap.md), [`../design/accepted/field-product-contracts-diagnostics-and-residency-design.md`](../design/accepted/field-product-contracts-diagnostics-and-residency-design.md), and [`../design/accepted/sdf-first-production-capability-map.md`](../design/accepted/sdf-first-production-capability-map.md) until deeper crate guides are written. |
| Geometry | `domain/geometry` | [`geometry/README.md`](./geometry/README.md), [`geometry/ownership-boundary.md`](./geometry/ownership-boundary.md), [`geometry/api-notes.md`](./geometry/api-notes.md) |
| SDF | `domain/sdf` | [`sdf/index.md`](./sdf/index.md), [`sdf/README.md`](./sdf/README.md), [`sdf/query-model.md`](./sdf/query-model.md) |
| Materials and textures | `domain/material_graph`, `domain/texture` | [`material-graph/README.md`](./material-graph/README.md), [`texture/README.md`](./texture/README.md) |
| Procedural generation | `domain/procgen` | [`procgen/README.md`](./procgen/README.md) for generator documents, planning lifecycle, reservations, deterministic lowering, and product-output boundaries |
| Drawing | `domain/drawing` | [`drawing/README.md`](./drawing/README.md) |
| Spatial / chunking / world data | `domain/spatial`, `domain/spatial_index`, `domain/chunking`, `domain/world_streaming`, `domain/world_ops`, `domain/world_sdf` | [`spatial/README.md`](./spatial/README.md), [`spatial-index/README.md`](./spatial-index/README.md), [`chunking/README.md`](./chunking/README.md), [`world-streaming/README.md`](./world-streaming/README.md), [`world-ops/README.md`](./world-ops/README.md), [`world-sdf/README.md`](./world-sdf/README.md) |
| UI substrate and definitions | `domain/ui/*`, including `domain/ui/ui_definition` | [`ui/README.md`](./ui/README.md), [`ui/architecture.md`](./ui/architecture.md), [`ui/roadmap.md`](./ui/roadmap.md) |
| Editor domains and definitions | `domain/editor/*`, including `domain/editor/editor_definition` | [`editor/README.md`](./editor/README.md) |

## Planned Domain Areas

These areas are roadmap-level intent, not implemented workspace members.
Do not add crate metadata for them until their implementation milestone lands.

- particles and VFX;
- physics and collision authoring;
- animation and procedural motion, with active architecture tracked in [`../design/active/sdf-procedural-animation-and-animated-models-design.md`](../design/active/sdf-procedural-animation-and-animated-models-design.md);
- simulation/world processes;
- gameplay graph orchestration after narrower gameplay event/action/state/quest contracts exist.

## What Belongs in Domain

Domain documentation should define:

- ownership boundaries;
- domain concepts;
- invariants;
- allowed dependencies;
- command or mutation contracts;
- ratification and validation rules;
- data model semantics;
- engine-agnostic usage examples;
- integration contracts consumed by engine/runtime or apps.

## What Does Not Belong in Domain

Domain documentation should not own:

- renderer backend details;
- windowing details;
- app startup wiring;
- editor UI implementation policy;
- transport backend configuration;
- LLM, prompt, or agent behavior;
- production runtime orchestration unless the domain explicitly owns the abstraction.

## Documentation Map

Start here when working in the domain layer:

- ECS: [`ecs/00-overview.md`](./ecs/00-overview.md)
- ECS usage: [`ecs/usage-guide.md`](./ecs/usage-guide.md)
- ECS advanced guide: [`ecs/advanced-guide.md`](./ecs/advanced-guide.md)
- Scheduler: [`scheduler/README.md`](./scheduler/README.md)
- Geometry: [`geometry/README.md`](./geometry/README.md)
- Asset/product contracts: [`../design/accepted/field-product-contracts-diagnostics-and-residency-design.md`](../design/accepted/field-product-contracts-diagnostics-and-residency-design.md)
- SDF: [`sdf/index.md`](./sdf/index.md)
- Material graph: [`material-graph/README.md`](./material-graph/README.md)
- Texture: [`texture/README.md`](./texture/README.md)
- Procgen contract: [`procgen/README.md`](./procgen/README.md)
- Drawing: [`drawing/README.md`](./drawing/README.md)
- UI substrate: [`ui/README.md`](./ui/README.md)

For workspace-wide ownership, see:

- `DOMAIN_MAP.md`
- `CRATES.md`
- [`../workspace/crate-docs-status.md`](../workspace/crate-docs-status.md)

## Known Gaps

The following domain areas still need deeper crate-level usage and architecture docs beyond the current landing pages:

- `domain/scene`
- `domain/spatial`
- `domain/spatial_index`
- `domain/chunking`
- `domain/world_streaming`
- `domain/asset`
- `domain/product`
- `domain/world_ops`
- `domain/world_sdf`
- `domain/editor/editor_core`
- `domain/editor/editor_shell`
- `domain/editor/editor_viewport`
- `domain/editor/editor_scene`
- `domain/editor/editor_inspector`
- `domain/editor/editor_persistence`

These gaps should be filled with crate-level `README.md`, architecture, usage, and ownership-boundary docs only when the implementation is stable enough to document truthfully.
