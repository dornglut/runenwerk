# Runenwerk Architecture

## Purpose

This root document is the short repository-wide architecture summary. Canonical
long-form architecture lives under `docs-site/src/content/docs`.

Current repository-family authority:

```text
docs-site/src/content/docs/architecture/repository-family-architecture.md
docs-site/src/content/docs/adr/accepted/0014-repository-family-extraction-boundaries.md
```

## Target repository family

```text
RunenSDF -------+
RunenECS -------+--> Runenwerk integration/adapters --> applications
RunenRender ----+
RunenUI --------+   (separate external workstream)
```

Runenwerk remains the integration and product repository. Extracted framework
repositories must not depend on Runenwerk. Adapters map framework-local contracts
to Runenwerk lifecycle, domains, rendering, and applications.

RunenUI work is governed separately and is not part of the current SDF/ECS/render
extraction program.

## Current in-repository layer model

Until each clean cutover completes:

```text
foundation -> domain -> engine/runtime -> apps/adapters/tools
```

Code location is current implementation fact, not permanent ownership authority.
Extraction proceeds only after a boundary is corrected and independently proven.

## Core doctrine

```text
Domains define meaning.
Contracts cross boundaries.
Commands mutate important state.
Ratifiers validate candidates.
Diagnostics explain outcomes.
Tests protect invariants.
Projections stay derived.
Adapters translate without duplicating authority.
```

Runenwerk should be truth-shaped: clear owners, explicit contracts, precise
errors, stable IDs, testable behavior, and one-way dependencies.

## Seven programming principles

Use `docs-site/src/content/docs/guidelines/programming-principles.md` as the
engineering lens for architecture and code review:

1. KISS: keep the owned path simple.
2. DRY: remove duplicate authority.
3. YAGNI: do not build speculative surfaces.
4. SOLID: keep responsibility and dependency boundaries honest.
5. Separation of Concerns: organize by responsibility.
6. Avoid Premature Optimization: prove the bottleneck first.
7. Law of Demeter: depend on direct contracts.

## Foundation

Foundation crates provide low-level vocabulary that is not specific to editor,
runtime, renderer, application behavior, or AI workflow.

Current foundation crates:

```text
foundation/id
foundation/id_macros
foundation/diagnostics
foundation/ratification
foundation/schema
foundation/commands
foundation/resource_ref
```

Do not create a universal `RunenCore`, shared meta-framework, universal ID crate,
or universal diagnostics crate for extraction convenience.

## Domain and framework ownership

Domain crates currently own engine-agnostic concepts and invariants. The active
program will move selected mature boundaries to independent repositories:

```text
domain/sdf                         -> future RunenSDF
domain/ecs + ecs_macros + scheduler -> future RunenECS
engine render-neutral/WGPU parts   -> future RunenRender
```

Runenwerk retains scene, world, material, editor, app, lifecycle, and adapter
policy.

## Engine and runtime

Runtime code composes domains/frameworks into executable behavior. It owns:

- app lifecycle;
- frame/tick policy;
- plugin composition;
- windows/event-loop policy;
- cross-domain adapters;
- product feature selection;
- integration diagnostics and tests.

Runtime must not become editor-shaped, and framework cores must not become
Runenwerk-shaped.

## Apps, adapters, and tools

Apps and tools compose the engine for concrete use cases. Adapters own external
host and cross-repository translation.

An adapter may depend on both Runenwerk and one framework. The framework must not
depend back on the adapter or Runenwerk.

## Architectural invariants

- Foundation crates do not depend on domain, engine, editor, app, or adapter code.
- Framework repositories do not depend on Runenwerk.
- RunenRender core does not depend on ECS, SDF, UI, scene, material authoring,
  Winit, WGPU, or Runenwerk.
- RunenECS core does not depend on Runenwerk geometry or spatial policy.
- RunenSDF does not depend on Runenwerk geometry, world, renderer, or ECS.
- Important mutations go through commands, builders, import pipelines, or
  controlled transactions.
- Projected state is derived unless explicitly documented otherwise.
- Persisted formats have explicit owners and versions.
- Inspection APIs expose read-only products, not mutable internals.
- Generated, imported, migrated, or AI-proposed state is ratified by its owner.
- No source mirror, submodule, compatibility package, or moving-branch dependency
  survives a completed extraction.

## Description versus execution

Prefer separate description/model and execution/runtime layers:

```text
RenderGraphDefinition -> RenderGraphExecutionPlan
AnimationGraph        -> AnimationRuntime
SceneAsset            -> LoadedScene
CommandDescriptor     -> CommandExecutor
```

Descriptions should be serializable, inspectable, ratifiable, diffable, and
testable. Execution objects may be optimized, backend-aware, resource-owning, and
non-serializable.

## Source of truth

The canonical documentation tree is:

```text
docs-site/src/content/docs
```

Root Markdown files are entry summaries. Update canonical docs first, then keep
root summaries aligned.
