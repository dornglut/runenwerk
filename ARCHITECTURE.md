# Runenwerk Architecture

## Purpose

This document defines the repository-wide architecture doctrine for Runenwerk: layer ownership, dependency direction, state authority, and long-term architectural constraints.

## Layer Model

```text
foundation -> domain crates -> engine/runtime -> apps/adapters/tools
```

## Core Doctrine

```text
AI proposes.
Domains validate.
Ratifiers check.
Diagnostics explain.
Tests protect.
Schemas describe.
Inspection views expose.
Commands mutate.
```

Runenwerk should not be AI-shaped. It should be truth-shaped: clear boundaries, explicit contracts, precise errors, stable IDs, and testable behavior.

## SDF-First Field Product Architecture

Runenwerk is SDF-first and product-driven.

SDF and field products are the primary production substrate for world geometry,
editable spatial truth, material/world fields, product lineage, rendering
inputs, collision/query formation, diagnostics, and long-term simulation
handoffs.

SDF-first does not mean SDF-only. Meshes remain valid as imported source
assets, derived render/debug/export/preview/fallback products, and specialized
representations when an owning design explicitly accepts that role.

Renderer caches, GPU resources, UI projections, editor overlays, and debug
products are derived state. They must not silently become authoritative world,
simulation, or gameplay truth.

## Foundation

Foundation crates provide reusable vocabulary and low-level contracts that are not specific to editor, runtime, renderer, or application behavior.

Current foundation crates:

- `foundation/id`: typed identity primitives and allocators.
- `foundation/id_macros`: attribute macro support for typed ID wrappers.
- `foundation/diagnostics`: structured diagnostic reporting vocabulary.
- `foundation/ratification`: shared ratification report vocabulary.
- `foundation/schema`: portable schema identity, version, path, value, shape, field, constraint, metadata, descriptor, and optional diagnostics-bridge vocabulary.
- `foundation/commands`: portable command contract identity, schema reference, descriptor, proposal, metadata, hint, issue, and optional diagnostics-bridge vocabulary.

Foundation must not own LLM clients, prompt logic, app policy, editor policy, runtime orchestration, or backend integrations.

## Domain Crates

Domain crates own engine concepts and their invariants. Other layers may validate, inspect, or mutate domain state only through explicit APIs.

Examples include ECS, scheduler, scene, editor domains, geometry, SDF, spatial/chunking/world domains, and UI contracts.

## Engine / Runtime

Runtime crates compose domain descriptions into executable behavior. They own scheduling, plugin composition, runtime resources, rendering execution, and app lifecycle.

Runtime must not become editor-shaped.

## Apps / Adapters / Tools

Apps and tools compose the engine for concrete use cases. AI integrations belong here, not in foundation or pure domain crates.

## Architectural Invariants

- Foundation crates do not depend on domain, engine, editor, app, adapter, or AI-integration crates.
- Domain crates do not depend on runtime, applications, concrete backends, or AI integrations.
- Important mutations go through commands, builders, import pipelines, or controlled transactions.
- Projected state is derived unless explicitly documented otherwise.
- Persisted formats are versioned from version `1`.
- Inspection APIs expose read-only DTOs, not mutable internals.
- Generated, imported, migrated, or AI-proposed state is ratified by the owning domain before acceptance.

## Description vs Execution

Prefer separate description/model and execution/runtime layers:

```text
RenderGraphDefinition -> RenderGraphExecutionPlan
AnimationGraph        -> AnimationRuntime
SceneAsset            -> LoadedScene
UiSurfaceDefinition   -> MountedSurfaceInstance
CommandDescriptor     -> CommandExecutor
```

Descriptions should be serializable, inspectable, ratifiable, diffable, testable, and suitable for AI-assisted editing. Execution objects may be optimized, backend-aware, resource-owning, and non-serializable.

## Source of Truth

The canonical documentation tree is `docs-site/src/content/docs`.

Root markdown files are repository-entry summaries for humans and agents working from the repo root. When root docs and docs-site pages overlap, update the docs-site page first, then keep the root summary aligned.

Design docs and ADRs live under:

- `docs-site/src/content/docs/design/`
- `docs-site/src/content/docs/adr/`

The accepted SDF-first architecture starts at:

- `docs-site/src/content/docs/adr/accepted/0008-adopt-sdf-first-field-product-architecture.md`
- `docs-site/src/content/docs/design/accepted/sdf-first-field-world-platform-design.md`
- `docs-site/src/content/docs/design/accepted/field-product-contracts-diagnostics-and-residency-design.md`

Active SDF-first character and procedural animation architecture is tracked in:

- `docs-site/src/content/docs/design/active/sdf-procedural-animation-and-animated-models-design.md`
- `docs-site/src/content/docs/adr/accepted/0011-animated-sdf-authoring-graphs-lower-before-runtime.md`
- `docs-site/src/content/docs/adr/proposed/animated-sdf-lowering-and-purpose-specific-products.md`
