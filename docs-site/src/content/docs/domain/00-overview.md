---
title: Domain Layer Overview
description: Overview of Runenwerk's engine-agnostic domain layer, ownership rules, and domain documentation map.
status: active
owner: domain
layer: domain
canonical: true
last_reviewed: 2026-05-06
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
| Geometry | `domain/geometry` | [`geometry/README.md`](./geometry/README.md), [`geometry/ownership-boundary.md`](./geometry/ownership-boundary.md), [`geometry/api-notes.md`](./geometry/api-notes.md) |
| SDF | `domain/sdf` | [`sdf/index.md`](./sdf/index.md), [`sdf/README.md`](./sdf/README.md), [`sdf/query-model.md`](./sdf/query-model.md) |
| Spatial / chunking / world data | `domain/spatial`, `domain/spatial_index`, `domain/chunking`, `domain/world_ops`, `domain/world_sdf` | [`spatial/README.md`](./spatial/README.md), [`spatial-index/README.md`](./spatial-index/README.md), [`chunking/README.md`](./chunking/README.md), [`world-ops/README.md`](./world-ops/README.md), [`world-sdf/README.md`](./world-sdf/README.md) |
| UI substrate | `domain/ui/*` | [`ui/README.md`](./ui/README.md), [`ui/architecture.md`](./ui/architecture.md), [`ui/roadmap.md`](./ui/roadmap.md) |
| Editor domains | `domain/editor/*` | [`editor/README.md`](./editor/README.md) |

## Planned Domain Areas

These areas are roadmap-level intent, not implemented workspace members.
Do not add crate metadata for them until their implementation milestone lands.

| Area | Planned crate | Primary docs | Notes |
| --- | --- | --- | --- |
| UI definition and formation | `domain/ui/ui_definition` | [`ui/roadmap.md`](./ui/roadmap.md), [`../design/active/ui-definition-formation-foundation-design.md`](../design/active/ui-definition-formation-foundation-design.md) | Planned M3.5 framework for authored UI templates, slots, repeaters, embeds, menus, availability, and retained UI formation. |
| Editor definition and self-authoring | `domain/editor/editor_definition` | [`../design/active/ui-definition-formation-foundation-design.md`](../design/active/ui-definition-formation-foundation-design.md), [`../design/active/editor-self-authoring-and-final-ui-design.md`](../design/active/editor-self-authoring-and-final-ui-design.md), [`../apps/runenwerk-editor/roadmap.md`](../apps/runenwerk-editor/roadmap.md) | Planned M3.5 editor binding layer for toolbar, menus, workspace catalogs, shell chrome, and common provider surface templates; M3.6 extends it into visual UI/style/layout authoring. |

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
- SDF: [`sdf/index.md`](./sdf/index.md)
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
- `domain/world_ops`
- `domain/world_sdf`
- `domain/editor/editor_core`
- `domain/editor/editor_shell`
- `domain/editor/editor_viewport`
- `domain/editor/editor_scene`
- `domain/editor/editor_inspector`
- `domain/editor/editor_persistence`

These gaps should be filled with crate-level `README.md`, architecture, usage, and ownership-boundary docs only when the implementation is stable enough to document truthfully.
