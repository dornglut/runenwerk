# Runenwerk Architecture

## Purpose

This root document is the short repository-wide architecture summary. Canonical
long-form architecture lives under `docs-site/src/content/docs`.

Current repository-family authority:

```text
docs-site/src/content/docs/architecture/repository-family-architecture.md
docs-site/src/content/docs/adr/accepted/0014-repository-family-extraction-boundaries.md
docs-site/src/content/docs/adr/accepted/0015-separate-gpu-execution-from-rendering.md
```

## Target repository family

```text
RunenSDF ----+
RunenECS ----+--> Runenwerk adapters/integration --> applications
RunenUI -----+
                  |
                  +--> RunenRender --> RunenGPU
                  +--> non-render RunenGPU workloads
```

Runenwerk remains the integration and product repository. Framework repositories
must not depend on Runenwerk.

The accepted direct framework dependency is:

```text
RunenRender -> RunenGPU
```

RunenGPU owns general GPU execution. RunenRender owns image formation and lowers
render work through RunenGPU.

## Initial repository packages

```text
repository                 package       crate
Crystonix/runen-sdf        runen-sdf     runen_sdf
Crystonix/runen-ecs        runen-ecs     runen_ecs
Crystonix/runen-gpu        runen-gpu     runen_gpu
Crystonix/runen-render     runen-render  runen_render
Crystonix/runen-ui         runen-ui      runen_ui
```

Each framework begins with one public package. Internal modules carry boundaries
until an independently useful dependency, backend, release, ABI, or compile-time
constraint proves that another package is needed.

Do not create speculative `core`, `wgpu`, `gpu`, facade, macro, testing, capture,
or compatibility packages merely to draw architecture boundaries.

## Current in-repository layer model

Until each clean cutover completes:

```text
foundation -> domain -> engine/runtime -> apps/adapters/tools
```

Code location is current implementation fact, not permanent ownership authority.
Extraction proceeds only after the future public boundary is corrected and proven
inside Runenwerk.

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

## Engineering principles

Use `docs-site/src/content/docs/guidelines/programming-principles.md` as the review
lens:

1. KISS: keep the owned path simple.
2. DRY: remove duplicate authority.
3. YAGNI: do not build speculative surfaces.
4. SOLID: keep responsibility and dependency boundaries honest.
5. Separation of Concerns: organize by responsibility.
6. Avoid Premature Optimization: prove the bottleneck first.
7. Law of Demeter: depend on direct contracts.

## Foundation

Foundation crates provide low-level vocabulary that is not specific to editor,
runtime, rendering, application behavior, or repository workflow.

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

## Framework ownership

### RunenSDF

Owns reusable field mathematics, numerical contracts, bounds, composition,
transforms, capabilities, and CPU reference queries.

It does not own Runenwerk world, rendering, GPU, ECS, material, or product policy.

### RunenECS

Owns entity/component/resource lifecycle, storage/query semantics, deferred
structural mutation, system access, explicit reflection, and ECS-local scheduling
integration.

It does not own general spatial indexing, engine lifecycle, rendering extraction,
networking, replay, world streaming, or product policy.

### RunenGPU

Owns GPU contexts, normalized capabilities, resources, access/lifetimes, hazards,
workloads, submissions, uploads/readback, low-level surfaces, WGPU realization,
backend outcomes, and GPU diagnostics.

It does not own image formation, field/simulation algorithms, ECS, UI, windows and
event loops, shader filesystem policy, or product recovery.

### RunenRender

Owns prepared render scenes, views, providers/interactions, materials/media,
emitters, visibility, transport, radiance caches, history, reconstruction,
overlays, color, presentation intent, and lowering into RunenGPU workloads.

It does not own WGPU directly, general GPU execution, ECS extraction, SDF
mathematics, UI semantics, native-window policy, or Runenwerk lifecycle.

### RunenUI

Owns semantic UI, state/actions, focus/accessibility, layout/style/text, hit
testing, and renderer-neutral paint output.

RunenUI remains independent of RunenRender and RunenGPU. A Runenwerk bridge may
translate accepted paint output into a RunenRender overlay contribution.

### Runenwerk

Retains:

- application and engine lifecycle;
- frame/tick and domain scheduling;
- windows/event-loop policy;
- ECS and domain extraction;
- scene, world, material-authoring, SDF, UI, editor, and simulation adapters;
- shader source discovery/revision/watch/reload policy;
- product capability and quality selection;
- cross-framework composition;
- diagnostics presentation, artifacts, recovery, and runtime evidence;
- applications and tools.

## Current GPU/render implementation

The current `engine/src/plugins/render` location combines several future owners:

```text
general GPU execution             -> future RunenGPU
image formation                   -> future RunenRender
windows/lifecycle/domain adapters -> retained Runenwerk
source-domain semantics           -> owning domain/framework
```

Moving the directory unchanged is forbidden.

The required sequence is:

```text
S0 complete inventory
-> internal RunenGPU boundary proof
-> external RunenGPU clean cutover
-> internal RunenRender boundary proof on RunenGPU
-> external RunenRender clean cutover
-> reusable adapter review
-> advanced renderer work
```

No implementation phase is authorized before S0 classifies current files,
consumers, identities, lifecycles, shaders, macros, and validation evidence.

## Apps, adapters, and tools

Apps and tools compose the engine for concrete use cases. Adapters own external
host and cross-framework translation.

An adapter may depend on Runenwerk and framework public contracts. Frameworks do
not depend back on the adapter or Runenwerk.

Adapters translate identities, prepared inputs, outputs, lifecycle facts,
diagnostics, provenance, and ownership. They do not mirror algorithms or create
parallel authority.

## Architectural invariants

- Foundation crates do not depend on domain, engine, editor, app, or adapter code.
- Framework repositories do not depend on Runenwerk.
- RunenGPU contains no renderer or domain meaning.
- RunenRender depends on RunenGPU and does not depend on WGPU directly.
- RunenRender does not depend on ECS, SDF, UI, scene/material authoring, Winit, or
  Runenwerk.
- RunenSDF, RunenECS, and RunenUI do not depend on RunenGPU or RunenRender merely
  because applications display or accelerate their output.
- Important mutations go through commands, builders, import pipelines, or
  controlled transactions.
- Projected state is derived unless explicitly documented otherwise.
- Persisted formats have explicit owners and versions.
- Inspection APIs expose read-only products, not mutable internals.
- Generated, imported, migrated, or proposed state is ratified by its owner.
- No source mirror, submodule, compatibility package, forwarding namespace, or
  moving-branch dependency survives a completed extraction.

## Description versus execution

Prefer separate description/model and execution/runtime layers:

```text
RenderContribution -> PreparedRenderScene -> RenderPlan -> GpuWorkFragment
AnimationGraph      -> AnimationRuntime
SceneAsset          -> LoadedScene
CommandDescriptor   -> CommandExecutor
```

Descriptions should be serializable where appropriate, inspectable, ratifiable,
diffable, and testable. Execution objects may be optimized, backend-aware,
resource-owning, and non-serializable.

## Source of truth

The canonical documentation tree is:

```text
docs-site/src/content/docs
```

Root Markdown files are entry summaries. Update canonical documents first, then
align root summaries.
