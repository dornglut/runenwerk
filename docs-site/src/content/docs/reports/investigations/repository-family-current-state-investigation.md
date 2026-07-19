---
title: Repository Family Current-State Investigation
description: Current source, dependency, ownership, and extraction-readiness evidence for RunenSDF, RunenECS, and RunenRender.
status: active
owner: workspace
layer: investigation
canonical: true
last_reviewed: 2026-07-19
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../design/active/runensdf-extraction-design.md
  - ../../design/active/runenecs-extraction-boundary-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../workspace/planning/active-work.md
---

# Repository Family Current-State Investigation

## Investigation question

Which Runenwerk subsystems can become independent repositories, what boundaries
must change first, and which work may proceed safely in parallel?

## Baseline

Repository: `Crystonix/Runenwerk`

Reviewed published `main` head:

```text
c078bd8609dc407d68269e86a1472c9234932213
```

The investigation also inspected rejected commit
`b5e9624c594c9f1e3f2a0929bf84028f13fde860` as historical evidence of an
incomplete source deletion. That commit is not an implementation base for this
program.

## Evidence limits

This investigation used GitHub repository, source, manifest, commit, and pull
request inspection. It did not run the local Cargo workspace or GPU/runtime
commands. Command validation remains required in each implementation phase.

Source classification in this report is sufficient to establish track order and
required design gates. It is not by itself authorization for ECS or renderer
source movement.

## Workspace facts

The root workspace currently contains domain crates for SDF, ECS, geometry,
spatial indexing, scheduler, scene, materials, world systems, applications, and
the monolithic engine package.

The extraction candidates are not equivalent in maturity:

| Candidate | Current location | Immediate extraction readiness |
|---|---|---|
| RunenSDF | `domain/sdf` | high after a bounded geometry/query correction |
| RunenECS | `domain/ecs`, `domain/ecs_macros`, parts of `domain/scheduler` | medium/low until ownership decisions close |
| RunenRender | `engine/src/plugins/render`, `engine_render_macros` | low until internal decomposition is proven |

## RunenSDF findings

### Current package

`domain/sdf/Cargo.toml` defines one unpublished `sdf` package with only:

```text
glam
geometry
```

The crate exports:

```text
bounds
combine
epsilon
field
gradient
normal
ops
primitives
queries
sample
transform
util
```

Its public spine is `SdfField3`, `SdfSample`, and `FieldBounds`.

### Confirmed coupling

The `geometry` dependency is public rather than incidental:

- `FieldBounds::Bounded` stores `geometry::Aabb3`;
- `FieldBounds::bounded` accepts `Aabb3`;
- `FieldBounds::as_aabb` returns `Aabb3`;
- raymarch queries accept `geometry::Ray3`.

This means moving the crate unchanged would make RunenSDF depend on another
Runenwerk domain package and would expose Runenwerk geometry as part of its API.

### Boundary assessment

The coupling is narrow and replaceable. SDF needs only a small repository-local
bounds and ray/query vocabulary based on `glam::Vec3`.

The initial extraction should remain one `runensdf` crate. Current evidence does
not justify separate core, query, macro, GPU, shader, or program crates.

### Readiness

```text
COMPLETE_INVESTIGATION: active; consumer and test inventory still required
DESIGN: target direction fixed; detailed conformance inventory still required
IMPLEMENTATION: blocked until the design gate and local baseline validation pass
EXTRACTION_ORDER: first
```

## RunenECS findings

### Current package graph

`domain/ecs` directly depends on:

```text
anyhow
ecs_macros
geometry
scheduler
thiserror
```

The public crate root exports substantially more than storage and queries:

- entity, bundle, component, resource, world, command, and query APIs;
- reflection;
- spatial hash configuration and spatial indexes;
- configured systems and runtime-plan reports;
- broadcast streams;
- tick buffers;
- work queues;
- ownership transfer and ownership descriptors;
- component/resource change extraction and structural deltas;
- telemetry.

The separate `domain/scheduler` crate has no ECS dependency and exports access,
builder, DAG, label, node, plan, scheduler-core, system, telemetry, and utility
modules. Its low-level package shape suggests possible independent value, but
semantic ownership cannot be inferred from package direction alone.

### Confirmed boundary problems

1. ECS core publicly exports geometry-dependent spatial indexing.
2. ECS scheduling semantics and generic scheduler/executor semantics are not yet
   classified.
3. Reflection ownership and registry lifetime require explicit review.
4. Broadcast, tick-buffer, and work-queue families may represent distinct
   semantics under one public surface.
5. Change extraction, ownership, structural deltas, networking, replay, and
   replication boundaries require consumer mapping.
6. The public root is broad enough that moving it unchanged would freeze current
   accidental aggregation.

### Readiness

```text
COMPLETE_INVESTIGATION: incomplete
DESIGN: incomplete
IMPLEMENTATION: forbidden
EXTRACTION_ORDER: second, after SDF workflow proof and boundary repair
```

## RunenRender findings

### Current package location

Rendering is not an independent package. It is a large engine plugin rooted at:

```text
engine/src/plugins/render
```

The public module tree includes:

```text
api
backend
composition
features
frame
gpu_primitives
graph
inspect
material_compiler
params
pipelines
procedural
renderer
residency
resource
shader
runtime
texture_upload
plugin
```

The engine package directly depends on WGPU, Winit, Naga, image, rusttype,
material graph, world, spatial, ECS, scheduler, networking/history packages, and
many former/current UI packages.

### Confirmed boundary problems

The current renderer location combines at least these ownership categories:

1. backend-neutral graph and frame planning;
2. WGPU backend state and command execution;
3. window/surface integration;
4. Runenwerk plugin scheduling and resources;
5. ECS extraction;
6. scene/world preparation;
7. material graph compilation;
8. SDF rendering integration;
9. editor and product features;
10. UI feature integration;
11. diagnostics and proof infrastructure.

Moving `engine/src/plugins/render` unchanged would create a physically separate
repository that still depends on Runenwerk product semantics.

### Required direction

RunenRender must first be decomposed inside Runenwerk into:

```text
backend-neutral render contracts and graph planning
WGPU backend
Runenwerk host/plugin integration
Runenwerk ECS/scene/material/SDF/editor/UI adapters
```

Runenwerk must consume the separated renderer through the same boundary intended
for external consumers before repository transfer is authorized.

### Readiness

```text
COMPLETE_INVESTIGATION: incomplete; full per-module and control-flow review required
DESIGN: incomplete
INTERNAL_DECOMPOSITION: blocked by design gate
EXTERNAL_EXTRACTION: forbidden
EXTRACTION_ORDER: last
```

## Parallel-work decision

Three tracks may proceed concurrently, but not as three simultaneous source
moves:

```text
RunenSDF    complete investigation/design, then implementation and extraction
RunenECS    complete investigation and decision closure
RunenRender complete semantic inventory and decomposition design
```

This avoids concurrent large changes to root manifests, lockfiles, engine
imports, architecture summaries, and shared planning authority.

## RunenUI disposition

RunenUI is explicitly excluded from this program. Another workstream owns it.
This investigation makes no maturity claim and creates no RunenUI dependency.

RunenRender must expose generic producer and render-consumer contracts so that a
future RunenUI adapter can integrate without UI semantics in renderer core.

## Rejected extraction attempt

Commit `b5e9624c...` removed large source areas before completing workspace,
dependency, consumer, lockfile, renderer, test, asset, and authority cutover.
That demonstrates why deletion is the final step of a governed cutover rather
than its starting point.

The commit is retained as historical evidence only and is classified:

```text
REJECTED_EXTRACTION_ATTEMPT
```

## Recommended next evidence

### RunenSDF

- inspect every source and test file;
- inventory every direct and transitive consumer;
- map all geometry leakage;
- decide numerical and error policy;
- define independent conformance;
- produce exact move/stay/redesign/delete map.

### RunenECS

- inspect storage, world, queries, systems, reflection, indexing, messaging,
  change extraction, ownership, macros, scheduler, networking, replay, renderer,
  and application consumers;
- classify scheduler and spatial ownership;
- define public compatibility and concurrency policy.

### RunenRender

- inspect every module, shader, example, benchmark, and integration test;
- trace frame, resource, graph, surface, device-loss, and plugin control flow;
- classify every module by future owner;
- identify generic seams before file movement.

## Conclusion

RunenSDF is the only candidate suitable for near-term extraction after one
bounded design and boundary-correction program. RunenECS requires architecture
closure. RunenRender requires internal decomposition and must move last.
