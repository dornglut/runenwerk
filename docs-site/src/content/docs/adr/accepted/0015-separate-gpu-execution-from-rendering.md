---
title: Separate GPU Execution from Rendering
description: Accepted ownership and dependency decision establishing standalone RunenGPU as the shared GPU execution framework beneath RunenRender.
status: accepted
owner: workspace
layer: architecture
canonical: true
last_reviewed: 2026-09-13
related_designs:
  - ../../architecture/repository-family-architecture.md
  - ../../design/accepted/runenrender-decomposition-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../design/active/shader-authoring-and-canonical-artifact-policy.md
related_reports:
  - ../../reports/design/runengpu-phase-requirements-proof-matrix.md
related_roadmaps:
  - ../../workspace/planning/roadmap.md
---

# ADR 0015: Separate GPU Execution from Rendering

## Decision

RunenGPU and RunenRender are distinct framework owners:

```text
product       repository                  package       crate
RunenGPU      dornglut/runen-gpu          runen-gpu     runen_gpu
RunenRender   dornglut/runen-render       runen-render  runen_render
```

The required dependency direction is:

```text
Runenwerk adapters and host policy
    -> RunenRender semantic image/result formation
        -> standalone RunenGPU generic GPU execution
            -> private backend implementation
```

Non-render consumers may use RunenGPU directly.

RunenGPU owns reusable generic GPU execution semantics and its backend realization.
RunenRender owns renderer-semantic meaning and lowers only through public RunenGPU
contracts. Runenwerk owns host, source-domain, product, authoring, artifact, and recovery
policy around those framework boundaries.

This ADR amends the older renderer-owned-GPU shape in ADR 0014. ADR 0014 remains
authoritative for repository independence, Runenwerk integration ownership, clean
cutover, provenance, and removal of duplicate source authority.

Current reusable RunenGPU API and conformance detail is **not defined by this ADR**.
After the accepted standalone transfer, that authority belongs only to
[`dornglut/runen-gpu`](https://github.com/dornglut/runen-gpu/blob/main/ARCHITECTURE.md)
and its repository-owned source/tests/CI.

## Current state

The RunenGPU standalone transfer and Runenwerk consumer cutover are complete. Runenwerk
currently consumes exact accepted RunenGPU revision:

```text
77c7c8d5ad6922b6f46c6b25e31b1a224c1314a4
```

That exact pin plus Runenwerk integration validation is the current product compatibility
claim. It does not freeze standalone RunenGPU evolution and it does not authorize a
Runenwerk-local duplicate semantic authority.

The historical internal G1A-G8/GX designs and proof program were used to establish the
public boundary before transfer. They are predecessor evidence now, not active
architecture. Their proof taxonomy is retained noncanonically in
[the historical RunenGPU proof report](../../reports/design/runengpu-phase-requirements-proof-matrix.md).

RunenRender remains Runenwerk-owned until its separately accepted external cutover. Its
current semantic architecture is owned by the
[RunenRender architecture design](../../design/accepted/runenrender-decomposition-design.md)
and its active execution plan.

## Why the split exists

A renderer-only GPU layer would force simulations, tools, image processing, procedural
generation, bakers, and other non-render consumers either to depend on renderer
vocabulary or to create parallel backend paths. A backend-only wrapper would fail to
provide the ownership, validation, typed composition, lifecycle, diagnostics, and
portable public execution contract needed by independent consumers.

The durable split is therefore:

```text
RunenGPU
    generic validated physical GPU execution

RunenRender
    renderer-semantic scene/request/planning/admission/result meaning

Runenwerk
    application/domain/host/product integration and policy
```

## RunenGPU ownership boundary

Standalone RunenGPU owns reusable generic GPU execution meaning, including the public
contracts by which consumers express, validate, realize, execute, inspect, and observe
GPU work. Exact resource kinds, program descriptors, access models, submission states,
surface states, error variants, compiler-derived facts, backend mappings, and conformance
requirements are defined by the standalone repository at the relevant revision.

RunenGPU does **not** own:

- renderer scenes, views, materials, transport, reconstruction, or output semantics;
- simulation, field/SDF, world, ECS, UI, or product semantics;
- Runenwerk frame/fixed/domain scheduling;
- windows/event-loop policy;
- shader filesystem/source-root/package policy;
- authoring compiler selection and source watching;
- product last-known-good/reload UX;
- persisted Runenwerk capture/build/media artifacts;
- image/video encoding or product recovery.

Runenwerk and RunenRender must not recreate reusable RunenGPU validation or backend
semantics merely because they integrate it.

## RunenRender ownership boundary

RunenRender owns how renderer-semantic inputs become renderer-semantic results. That
includes renderer-local scene meaning, observations, output meaning, representations,
query protocols, appearance, transport, renderer methods, device-independent planning,
semantic admission, derived renderer state, reconstruction, overlay composition, and
renderer-specific lowering into generic RunenGPU work.

RunenRender does not own generic GPU resources, hazard/lifetime validation, backend
program realization, generic submission/progress/readback mechanics, low-level surface
execution, or device-loss semantics. It must not depend directly on private backend
types merely to bypass the public RunenGPU contract.

## Runenwerk ownership boundary

Runenwerk retains:

- application and engine lifecycle;
- frame, fixed-time, and domain scheduling;
- windows, event loops, DPI/monitor/visibility/presentation product policy;
- ECS, scene, world, material-authoring, field/SDF, UI, editor, and simulation adapters;
- shader source discovery, source roots, revision, filesystem watching, and reload
  scheduling;
- authoring compiler/frontend selection and deterministic canonical artifact production;
- product quality/capability selection and cross-framework composition;
- capture/reproducibility bundle policy and persisted artifact schemas;
- offline job ordering, filenames/manifests, retry/failure policy, and media encoding;
- diagnostics presentation and product recovery.

Runenwerk may create one shared RunenGPU context and compose work from RunenRender and
non-render consumers. That integration role does not transfer reusable GPU semantic
ownership back into Runenwerk.

The current detailed source/compiler/artifact boundary is owned by the
[Shader Authoring and Canonical Artifact Policy](../../design/active/shader-authoring-and-canonical-artifact-policy.md).

## Shader/program ownership

The durable split is:

```text
source-domain / renderer kernel meaning
    owned by the semantic producer

Runenwerk authoring/toolchain policy
    source roots, frontend/compiler selection, deterministic canonical WGSL,
    dependency invalidation, watching/reload, publication, persisted provenance

standalone RunenGPU
    canonical program admission, compiler-derived/effective program facts,
    requirements, binding/layout/pipeline compatibility, backend realization
```

A higher-level shader frontend may lower to canonical WGSL before RunenGPU admission.
It does not become a second RunenGPU runtime source/interface authority.

## Resource and identity law

The original decomposition established a durable conceptual rule that remains relevant
to integration even though exact RunenGPU types are now standalone-owned:

```text
semantic/domain identity
!= RunenRender identity
!= RunenGPU runtime resource identity
!= persisted artifact identity
```

Unrelated properties such as resource kind, lifetime, ownership/import relation,
transfer/readback intent, reconstruction source, and memory intent must not be collapsed
merely for integration convenience. Exact current representations of those properties
are owned by standalone RunenGPU.

Labels, debug names, process-local type identities, runtime handles, backend objects, and
device generations are not silently promoted into persistence, replay, wire, cache,
binding, or semantic authority.

## Public-boundary consequence

RunenRender and non-render consumers use only accepted public RunenGPU contracts.
Runenwerk integration may provide ergonomic composition, but both ordinary and advanced
paths converge on the same standalone validation/execution owner.

Current source and tests, rather than historical G-phase pseudocode, define exact public
API shape. Compatibility aliases, private reach-through, mirrored validation, or a
Runenwerk-local GPU facade are not accepted migration strategies.

## Framework independence

RunenSDF, RunenECS, and RunenUI do not depend on RunenGPU or RunenRender merely because
an application may accelerate or display their outputs.

The default shape remains:

```text
RunenSDF ----+
RunenECS ----+--> Runenwerk adapters/integration
RunenUI -----+
                  |
                  +--> RunenRender --> RunenGPU
                  +--> non-render RunenGPU work
```

Cross-framework translation stays Runenwerk-owned until an independently reusable
adapter boundary is separately proved.

## RunenUI relationship

RunenUI owns semantic UI, state, actions, focus, accessibility, layout, style, text,
hit testing, and renderer-neutral paint output. A Runenwerk bridge may translate
accepted paint primitives into a RunenRender overlay contribution without exposing
widget state to RunenRender or forcing RunenUI to depend on RunenRender/RunenGPU.

## RunenSDF relationship

RunenSDF remains backend-neutral field authority. GPU or renderer realization is derived
integration state. RunenSDF does not depend on RunenGPU or RunenRender merely because a
product evaluates or displays fields on a GPU.

## RunenECS relationship

RunenECS remains generic ECS authority. ECS storage/query/scheduling and entity/component
meaning stay outside RunenGPU and RunenRender. Runenwerk adapters project prepared domain
facts across framework boundaries.

## Historical G-phase material

The predecessor G-phase designs and matrix remain valuable for chronology, why particular
boundaries were selected, and how acceptance was proven. They no longer answer current
RunenGPU API or implementation questions.

Use:

- standalone `dornglut/runen-gpu` for current reusable semantics and conformance;
- this ADR for the durable GPU/render/Runenwerk ownership split;
- current RunenRender design/plan for renderer semantics and execution work;
- the historical RunenGPU proof report and Git history for predecessor evidence.

## Clean-cutover consequence

After accepted standalone transfer:

- standalone RunenGPU is the only writable reusable GPU semantic authority;
- Runenwerk pins an exact accepted revision for integration;
- original internal implementation and temporary migration seams are removed;
- active Runenwerk documentation does not retain G-phase semantic authority;
- historical evidence may remain only when explicitly noncanonical;
- no mirror, compatibility package, forwarding namespace, source include, submodule,
  moving-branch dependency, or parallel runtime path is retained.

## Rejected alternatives

Rejected:

- keeping generic GPU execution inside RunenRender;
- keeping WGPU/backend authority in Runenwerk and extracting only helpers;
- renaming a mixed renderer implementation to RunenGPU;
- copying mixed RenderFlow/runtime ownership wholesale into a framework;
- wrapping every backend type one-for-one without added ownership/validation meaning;
- preserving active Runenwerk G-phase semantic documents after standalone transfer;
- maintaining compatibility facades or private reach-through after a public boundary;
- making source-root/filesystem/reload/product artifact policy reusable RunenGPU semantics;
- using visual demonstrations as substitutes for deterministic conformance.

## Consequences

### Positive

- rendering and non-render GPU consumers share one reusable execution authority;
- RunenRender remains semantic and backend-independent;
- Runenwerk integration remains explicit without duplicating framework semantics;
- standalone RunenGPU can evolve and validate independently;
- exact-revision product compatibility remains auditable;
- shader authoring/tooling and runtime admission have one owner each.

### Costs

- Runenwerk adapters must translate rather than rely on private backend reach-through;
- RunenRender implementation must conform to an external public GPU boundary;
- changes to reusable GPU semantics require upstream work in the standalone repository;
- product compatibility must deliberately repin and revalidate newer accepted RunenGPU
  revisions rather than following a moving branch.
