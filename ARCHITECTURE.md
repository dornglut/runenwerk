# Runenwerk Architecture

## Purpose

This root document is the short repository-wide architecture summary. Canonical
long-form authority lives under `docs-site/src/content/docs`.

Current repository-family authority:

```text
docs-site/src/content/docs/architecture/repository-family-architecture.md
docs-site/src/content/docs/adr/accepted/0014-repository-family-extraction-boundaries.md
docs-site/src/content/docs/adr/accepted/0015-separate-gpu-execution-from-rendering.md
```

## Target repository family

```text
RunenSDF --------------------+
RunenECS --------------------+
RunenUI ---------------------+--> Runenwerk integration/adapters --> applications
                             |
RunenRender --> RunenGPU ----+
other GPU consumers -> RunenGPU
```

Runenwerk remains the integration and product repository. Extracted framework
repositories do not depend on Runenwerk.

RunenRender owns image formation and depends on RunenGPU for general GPU
execution. RunenGPU does not depend on RunenRender or domain frameworks.

RunenUI remains a separate external workstream. Its core/runtime do not depend on
RunenGPU or RunenRender merely because an optional backend or host may use them.

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
engineering lens:

1. KISS: keep the owned path simple.
2. DRY: remove duplicate authority.
3. YAGNI: do not build speculative surfaces.
4. SOLID: keep responsibility and dependency boundaries honest.
5. Separation of Concerns: organize by responsibility.
6. Avoid Premature Optimization: prove the bottleneck first.
7. Law of Demeter: depend on direct contracts.

## Foundation

Foundation crates provide low-level vocabulary not specific to editor, runtime,
renderer, application behavior, or backend execution.

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

RunenGPU is not a universal core. It owns one bounded responsibility: GPU
execution.

## Domain and framework ownership

Current and future ownership:

```text
domain/sdf                          -> future RunenSDF
domain/ecs + macros + scheduler     -> future RunenECS
general GPU execution               -> future RunenGPU
render semantics and image formation -> future RunenRender
reusable UI framework               -> external RunenUI
```

### RunenGPU

Owns:

- GPU runtime identities;
- normalized capabilities;
- backend-neutral resources, access, lifetimes, and work graphs;
- shader/pipeline realization contracts;
- compute/render/copy execution;
- headless execution, upload, readback, and completion;
- low-level surfaces and backend outcomes.

Does not own rendering, simulations, SDF, UI, ECS, windows, or product policy.

### RunenRender

Owns:

- prepared render scenes and contributions;
- views and logical targets;
- providers, materials, media, emitters, and environments;
- visibility, transport, radiance caches, and reconstruction;
- overlays, output color intent, and render diagnostics;
- render-specific lowering into RunenGPU work.

Does not own WGPU devices/queues/surfaces, ECS extraction, source authoring,
simulations, SDF mathematics, UI semantics, windows, or product recovery.

### Runenwerk

Retains scene, world, material-authoring, editor, app, lifecycle, windows,
adapters, shader source reload policy, product policy, diagnostics presentation,
and recovery.

## Engine and runtime

Runtime composes domains/frameworks into executable behavior. It owns:

- app lifecycle;
- frame/tick and domain scheduling;
- plugin composition;
- windows/event-loop policy;
- GPU context setup and selected capability profile;
- cross-domain adapters;
- product feature and quality selection;
- integration diagnostics and tests.

Runtime must not become editor-shaped, and framework cores must not become
Runenwerk-shaped.

## Apps, adapters, and tools

Apps and tools compose the engine for concrete use cases. Adapters own external
host and cross-repository translation.

An adapter may depend on Runenwerk and the frameworks it translates. Frameworks
must not depend back on the adapter or Runenwerk.

Initial UI/render and SDF/render bridges remain Runenwerk-owned until both public
sides stabilize and another independent host proves reusable bridge ownership.

## Architectural invariants

- Foundation does not depend on domain, engine, editor, app, adapter, UI, or
  concrete backend code.
- Framework repositories do not depend on Runenwerk.
- RunenGPU core does not depend on WGPU, Winit, Runenwerk, RunenRender, ECS, SDF,
  UI, scene, material, simulation, editor, or product semantics.
- RunenGPU WGPU backend does not depend on Winit, Runenwerk, a renderer, or domain
  semantics.
- RunenRender core does not depend on ECS, SDF, UI, scene, material authoring,
  Winit, WGPU, or Runenwerk.
- RunenRender GPU realization uses RunenGPU and does not own a second device,
  queue, surface, allocator, or WGPU error model.
- RunenECS core does not depend on Runenwerk geometry or spatial policy.
- RunenSDF does not depend on Runenwerk geometry, world, renderer, GPU, or ECS.
- RunenUI core/runtime do not depend on RunenGPU, RunenRender, or Runenwerk.
- Important mutations go through commands, builders, import pipelines, or
  controlled transactions.
- Projected state is derived unless documented otherwise.
- Persisted formats have explicit owners and versions.
- Inspection APIs expose read-only products, not mutable internals.
- Generated, imported, migrated, or AI-proposed state is ratified by its owner.
- No source mirror, submodule, compatibility package, duplicate execution path, or
  moving-branch dependency survives a completed extraction.

## Description versus execution

Prefer separate semantic description and runtime execution:

```text
PreparedRenderScene -> RenderPlan -> GpuWorkFragment -> GPU submission
AnimationGraph       -> AnimationRuntime
SceneAsset           -> LoadedScene
CommandDescriptor    -> CommandExecutor
```

Descriptions should be serializable where appropriate, inspectable, ratifiable,
diffable, and testable. Runtime execution objects may be optimized, backend-aware,
resource-owning, and non-serializable.

## Source of truth

Canonical documentation:

```text
docs-site/src/content/docs
```

Root Markdown files are entry summaries. Update canonical docs first, then keep
root summaries aligned.
