# Runenwerk Architecture

## Purpose

This root document is the short repository-wide architecture summary. Canonical long-form architecture lives under `docs-site/src/content/docs`.

For the UI framework architecture spine, see `docs-site/src/content/docs/architecture/ui-framework-architecture.md`.

## Layer model

```text
foundation -> domain -> engine/runtime -> apps/adapters/tools
```

## Core doctrine

```text
Domains define meaning.
Contracts cross boundaries.
Commands mutate important state.
Ratifiers validate candidates.
Diagnostics explain outcomes.
Tests protect invariants.
Projections stay derived.
```

Runenwerk should be truth-shaped: clear owners, explicit contracts, precise errors, stable IDs, and testable behavior.

## Seven programming principles

Use `docs-site/src/content/docs/guidelines/programming-principles.md` as the engineering lens for architecture and code review:

1. KISS: keep the owned path simple.
2. DRY: remove duplicate authority.
3. YAGNI: do not build speculative surfaces.
4. SOLID: keep responsibility and dependency boundaries honest.
5. Separation of Concerns: organize by responsibility.
6. Avoid Premature Optimization: prove the bottleneck first.
7. Law of Demeter: depend on direct contracts.

## Foundation

Foundation crates provide reusable vocabulary and low-level contracts that are not specific to editor, runtime, renderer, application behavior, or AI workflow.

Current foundation crates:

```text
foundation/id             typed identity primitives and allocators
foundation/id_macros      attribute macro support for typed ID wrappers
foundation/diagnostics    structured diagnostic reporting vocabulary
foundation/ratification   shared ratification report vocabulary
foundation/schema         portable schema identity, value, shape, constraint, and descriptor vocabulary
foundation/commands       portable command contract vocabulary
foundation/resource_ref   portable external resource references
```

Foundation must not own LLM clients, prompt logic, app policy, editor policy, runtime orchestration, backend integrations, or domain-specific invariants.

## Domain crates

Domain crates own engine-agnostic concepts and invariants. Other layers may validate, inspect, or mutate domain state only through explicit APIs.

## Engine and runtime

Runtime crates compose domain descriptions into executable behavior. They own scheduling, plugin composition, runtime resources, rendering execution, and app lifecycle.

Runtime must not become editor-shaped.

## Apps, adapters, and tools

Apps and tools compose the engine for concrete use cases. Adapters own external host integration. AI integrations belong in apps, tools, or adapters, not in foundation or pure domain crates.

## Architectural invariants

- Foundation crates do not depend on domain, engine, editor, app, adapter, or AI-integration crates.
- Domain crates do not depend on runtime, applications, concrete backends, or AI integrations.
- Important mutations go through commands, builders, import pipelines, or controlled transactions.
- Projected state is derived unless explicitly documented otherwise.
- Persisted formats are versioned from version `1`.
- Inspection APIs expose read-only DTOs, not mutable internals.
- Generated, imported, migrated, or AI-proposed state is ratified by the owning domain before acceptance.

## Description vs execution

Prefer separate description/model and execution/runtime layers:

```text
RenderGraphDefinition -> RenderGraphExecutionPlan
AnimationGraph        -> AnimationRuntime
SceneAsset            -> LoadedScene
UiSurfaceDefinition   -> MountedSurfaceInstance
CommandDescriptor     -> CommandExecutor
```

Descriptions should be serializable, inspectable, ratifiable, diffable, testable, and suitable for AI-assisted editing. Execution objects may be optimized, backend-aware, resource-owning, and non-serializable.

## Source of truth

The canonical documentation tree is:

```text
docs-site/src/content/docs
```

Root Markdown files are repository-entry summaries. When root docs and docs-site pages overlap, update the docs-site page first, then keep the root summary aligned.

Design docs and ADRs live under:

```text
docs-site/src/content/docs/design/
docs-site/src/content/docs/adr/
```
