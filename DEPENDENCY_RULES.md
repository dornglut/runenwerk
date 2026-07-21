# Dependency Rules

## Target repository direction

```text
RunenSDF ----+
RunenECS ----+--> Runenwerk adapters/integration --> applications
RunenUI -----+
                  |
                  +--> RunenRender --> RunenGPU
                  +--> non-render RunenGPU workloads
```

Framework repositories must not depend on Runenwerk.

The default rule is framework independence. A direct dependency requires an
accepted ADR proving independent value, correct ownership, and that the dependency
is not merely avoiding a Runenwerk adapter.

ADR 0015 accepts:

```text
RunenRender -> RunenGPU
```

RunenRender requires general GPU execution. RunenGPU remains independently useful
for compute, simulation, field realization, procedural tools, bakers, and
offscreen workloads.

In particular:

- RunenGPU does not depend on RunenRender, RunenSDF, RunenECS, RunenUI, Winit, or
  Runenwerk;
- RunenRender may depend on RunenGPU but not directly on WGPU, Winit, RunenSDF,
  RunenECS, RunenUI, scene/material authoring, or Runenwerk;
- RunenSDF, RunenECS, and RunenUI do not depend on RunenGPU or RunenRender merely
  because applications accelerate or display their output;
- Runenwerk owns cross-framework translations and product composition.

## Initial package rule

Each framework repository begins with one public package:

```text
runen-sdf
runen-ecs
runen-gpu
runen-render
runen-ui
```

Do not create speculative `core`, `wgpu`, `gpu`, facade, macro, testing, capture,
or compatibility packages for extraction convenience.

A package split requires concrete evidence such as:

- an independently reusable dependency subset;
- a second backend with distinct dependency pressure;
- a required proc-macro compile boundary with proven public value;
- a distinct release/versioning unit;
- platform/no-std separation;
- externally reused conformance/test support.

Prefer private modules and explicit internal seams before package extraction.

## Current in-repository direction

Until each clean cutover completes:

```text
foundation -> domain crates -> engine/runtime -> apps/adapters/tools
```

Current source location is transitional implementation fact, not permanent
ownership authority.

The current render plugin contains future RunenGPU, RunenRender, Runenwerk, and
other-domain responsibilities. Moving it unchanged is forbidden.

## Foundation rules

Foundation may depend only on justified foundation crates and appropriate
low-level external libraries.

Foundation must not depend on domain, runtime, editor, app, adapter, workflow, UI,
or concrete backend code.

Do not create a universal shared-core/meta repository to avoid explicit adapter
boundaries.

## Domain rules

Domain crates may depend on foundation and carefully selected lower-level domain
contracts.

Domain crates must not depend on runtime, app code, backend adapters, editor app
wiring, or concrete rendering/windowing/input/audio backends unless the domain
explicitly owns that backend.

During extraction, accidental dependencies are removed rather than copied into the
new repository.

## RunenGPU rules

RunenGPU may own WGPU internally as its initial backend.

Its public contracts may express:

- normalized capabilities and requirements;
- context-scoped identities;
- resources and views;
- access, initialization, lifetime, hazard, and retirement;
- compute/render/copy/clear/resolve/present workloads;
- shader/pipeline admission and realization outcomes;
- uploads, readback, submission, completion, surfaces, and device outcomes;
- GPU diagnostics and provenance.

RunenGPU public contracts must not express:

- render providers, materials, emitters, visibility, transport, or reconstruction;
- simulation, field, ECS, UI, world, editor, or product semantics;
- Winit windows or event-loop policy;
- shader filesystem watching;
- product recovery or diagnostics presentation.

Backend-specific facts are capability-gated or explicitly unstable. WGPU types do
not become universal semantics by default.

## RunenRender rules

RunenRender depends downward on RunenGPU.

RunenRender may own:

- prepared render scenes and contributions;
- views and logical targets;
- providers/interactions;
- materials, media, emitters, environments;
- visibility, transport, caches, history, reconstruction, overlays, color, and
  presentation intent;
- lowering render plans into RunenGPU workloads.

RunenRender must not:

- depend directly on WGPU;
- construct a device, queue, surface, allocator, or competing GPU resource model;
- import ECS, RunenSDF, RunenUI, Runenwerk, Winit, source scene/material/editor
  domains, or application lifecycle;
- reach back into host state during planning/execution;
- perform UI hit testing or text shaping;
- reinterpret SDF numerical semantics without an explicit adapter.

## RunenSDF, RunenECS, and RunenUI rules

RunenSDF remains backend-neutral and owns field/numerical semantics.

RunenECS owns ECS lifecycle/query/system semantics, not rendering or GPU execution.

RunenUI owns semantic UI, layout/style/text, focus/accessibility, hit testing, and
renderer-neutral paint output.

GPU/render integrations use Runenwerk-owned adapters until independent reuse proves
a separate bridge package.

## Engine/runtime rules

Runtime may depend on foundation, current domains, extracted frameworks, and
backend/runtime implementation dependencies.

Runtime owns lifecycle and integration. It must not move product/editor/domain
semantics into generic framework APIs.

Runenwerk owns:

- windows/event loops and native handles;
- application/frame/domain scheduling;
- ECS/domain extraction;
- shader source discovery/revision/watch/reload policy;
- cross-framework work composition;
- product capability/quality/fallback/recovery policy;
- diagnostics presentation and runtime evidence.

## Adapter rules

A Runenwerk adapter may depend on Runenwerk and the framework contracts it
translates between. No framework depends back on the adapter.

Adapters translate:

- identities;
- prepared inputs and outputs;
- lifecycle facts;
- diagnostics and provenance;
- source/resource ownership.

Adapters must not duplicate algorithms, mirror source, expose mutable internals,
become writable parallel authorities, or hide dependency cycles.

Keep a bridge in Runenwerk until an independent consumer proves stable reusable
ownership.

## App/tool rules

Apps and tools may compose higher layers but must not define core framework or
domain invariants.

A tool may depend directly on RunenGPU for independently useful compute/offscreen
work without depending on RunenRender.

## Test-support rules

Reusable fixtures should live in explicit test-support modules or packages only
when external reuse proves that boundary. Production APIs must not be widened
solely for tests.

Cross-repository conformance belongs in the framework repository for public
consumer proof and in Runenwerk for integration compatibility.

## Version and revision rules

Before stable publication, Runenwerk pins an exact Git revision or exact
pre-release version. Moving branch dependencies are forbidden.

A direct framework dependency such as RunenRender -> RunenGPU also uses an exact
accepted revision before stable publication.

## Clean-cutover rules

A completed extraction leaves:

- one external source authority;
- one-way dependency direction;
- exact dependency pinning;
- every active consumer migrated;
- no original source copy;
- no forwarding package or namespace;
- no submodule or source include;
- no branch dependency;
- no long-lived migration facade;
- no duplicate runtime path.

Temporary duplication is allowed only on an unmerged extraction branch.

If Runenwerk has no active consumer, deleting the internal implementation does not
require adding an unused external dependency.

## Boundary escalation

When one owner wants another owner's internals, first determine whether the
missing boundary is:

- a public value/DTO;
- command or request;
- diagnostic/report;
- adapter;
- capability contract;
- workload/contribution contract;
- test-support contract.

Do not solve boundary pressure by creating a universal abstraction, speculative
package, compatibility facade, or exposed mutable internals.
