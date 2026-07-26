---
title: RunenGPU Public API Ergonomics Review
description: Critical review and accepted design pressure for a simple common RunenGPU API, an inspectable advanced path, typed references, resource semantics, and human-readable failures.
status: active
owner: gpu
layer: investigation
canonical: false
last_reviewed: 2026-07-26
related_docs:
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/active-work.md
  - ./runengpu-industry-comparison.md
  - ./runengpu-render-s0-inventory.md
---

# RunenGPU Public API Ergonomics Review

## Purpose

This review tests whether the proposed RunenGPU architecture can become a public Rust framework that is understandable and efficient to use, rather than merely internally correct.

The architecture is accepted directionally:

```text
Runenwerk adapters
    -> RunenRender semantic plans or non-render adapters
        -> RunenGPU resources and work
            -> WGPU backend
```

The first proposed usage shape exposed too much of the internal model to every caller:

```text
context admission
capability negotiation
resource declarations
work fragments
epoch construction
graph validation
submission
completion
retirement
```

Those concepts remain necessary internally. They must not all be mandatory public ceremony.

## Core decision

The validated work graph is the internal correctness and inspection model. It is not the headline API.

RunenGPU requires two public paths that produce the same prepared execution authority:

```text
simple path
    build work
    -> submit
    -> automatic validation

advanced path
    build work
    -> prepare
    -> inspect diagnostics/plan
    -> submit prepared work
```

Conceptual ordinary usage:

```rust
let simulation = simulation.gpu_work(&gpu, &simulation_state)?;
let rendering = renderer.gpu_work(&gpu, &prepared_scene, request)?;
let submission = gpu.submit("frame 42", [simulation, rendering])?;
```

Conceptual advanced usage:

```rust
let prepared = gpu.prepare("frame 42", [simulation, rendering])?;
inspect(prepared.diagnostics());
let submission = gpu.submit_prepared(prepared)?;
```

Exact names remain future specification decisions. The experience and ownership split are binding design pressure.

## Progressive disclosure

The public surface should expose increasing control only when requested.

### Level 1 — domain facade

Ordinary application and domain code calls an owner-specific facade:

```rust
let work = particle_simulation.gpu_work(&gpu, &state)?;
```

The facade owns simulation semantics and lowers them into generic GPU work.

### Level 2 — generic work authoring

Reusable compute libraries and advanced consumers construct generic GPU work through a typed builder.

```rust
let work = GpuWork::build("particle simulation", |work| {
    let params = work.upload("parameters", &params)?;

    work.compute("integrate", &pipeline, |pass| {
        pass.bind(pipeline.params(), params);
        pass.bind(pipeline.particles(), particles.read_write());
        pass.dispatch_1d(active_particles, 64);
    })?;

    Ok(())
})?;
```

This example is illustrative. G2-G4 must decide exact types and names.

### Level 3 — preparation and inspection

Tools, tests, diagnostics, and sophisticated schedulers may explicitly prepare and inspect work before submission.

### Level 4 — backend implementation

WGPU resource realization, command recording, barriers, queue submission, and native handles remain internal. They are not the ordinary public API.

## Accepted ergonomic principles

### Automatic validation

`submit` validates automatically. Callers must not be required to remember a separate `.validate()` call.

Preparation remains available when the caller wants diagnostics, graph inspection, or deterministic planning evidence before execution.

### Typed authority

Strings are labels for diagnostics and traces. They are not authoritative resource IDs, binding keys, pipeline interfaces, or dependency references.

After shader/pipeline admission, binding references should be validated and typed rather than repeatedly named with arbitrary strings.

### Lexical builders

Closure-scoped work/pass builders are preferred over nested builder chains that require repeated `.finish()` calls.

Lexical scope:

- makes pass boundaries visible;
- prevents unfinished builders from escaping;
- supports ordinary `?` error propagation;
- keeps formatting manageable;
- reduces invalid intermediate states.

### Inferred ordering

Data dependencies are inferred from declared accesses:

```text
work A writes resource R
work B reads resource R
    -> A precedes B
```

Explicit ordering is reserved for real non-data dependencies. Ambiguous writers and cycles remain structured errors.

### RAII resource handles

Dropping the last public handle schedules safe backend retirement after relevant submissions complete. Ordinary callers do not manually track in-flight destruction.

Advanced explicit release or collection APIs may exist for memory-budget control, but they do not replace safe default RAII behavior.

### Human-readable diagnostics

Errors retain machine-readable identities and provenance but display application vocabulary.

Expected shape:

```text
Cannot prepare “frame 42”.

Work “draw particles” reads buffer “particles”, but no preceding work
initializes or writes that buffer.

Reader: draw particles
Resource: particles
Correction: provide initial contents or add work that writes the buffer
```

Generic strings, panics, backend-only enum dumps, and opaque numeric IDs are not sufficient public failures.

## Resource model correction

The earlier candidate model combined unrelated concepts under one lifetime classification. G2 must separate them.

### Kind

```text
Buffer
Texture
TextureView
Sampler
QuerySet
```

Additional kinds require current consumer evidence.

### Lifetime

```text
Transient
Retained
```

More refined lifetime terms require proven semantic value.

### Ownership

```text
RunenGPU-owned
Imported
Surface-acquired
```

Ownership determines who may destroy, reconstruct, synchronize, or replace a realization.

### Transfer and observation

```text
initial data
upload/update
copy
readback request
export relationship
```

Upload and readback are operations or relationships, not resource lifetime variants.

### Reconstruction

```text
source-backed
externally reconstructed
non-reconstructable
```

The semantic owner retains authoritative data and recovery policy.

The initial stable API must not expose every theoretical combination. It should include only combinations justified by the exact G2 consumer inventory.

## Capability-model pressure

The initial proposal used:

```text
Required
Preferred
Optional
Forbidden
```

`Optional` may be redundant because an unmentioned capability is normally irrelevant. `Forbidden` may be clearer as `Disabled` for user-facing configuration.

G2 must compare at least:

```text
Required
Preferred
Disabled
```

against the four-state candidate and select the smallest model that can express current consumer requirements and fallback behavior without ambiguity.

Capability profiles should be convenience recipes, not a second authority over individual requirements.

## Typed GPU data

A single broad `GpuData` concept risks implying that one Rust representation is universally valid as uniform, storage, vertex, indirect, and readback data.

G2 and G4 must preserve distinctions between:

```text
byte-safe transfer representation
uniform layout
storage layout
vertex layout
binding interface
readback decoding
```

Any derive or macro must prove layout, alignment, padding, stride, supported field types, nested values, dependency renaming, compile-pass behavior, and compile-fail behavior. Rust memory layout must not be copied implicitly.

A common derive may generate multiple validated representations, but it must not erase their semantic differences.

## RenderFlow migration pressure

Current RenderFlow ergonomics should be retained only where the meaning remains correct.

Keep or reproduce:

- readable human labels;
- concise resource and pass authoring;
- typed resource handles;
- straightforward compute-to-render composition;
- one understandable validation/submission result.

Remove or relocate:

- direct ECS resource projection from framework APIs;
- shader filesystem paths as GPU shader identity;
- built-in UI and product policy;
- surface/window policy;
- render semantics from generic GPU work;
- stringly resource and binding authority;
- mandatory graph terminology in ordinary code;
- nested `finish()` ladders.

A future RunenRender or Runenwerk convenience facade may remain fluent while lowering into neutral `GpuWork`.

## Rejected headline shape

The following is too framework-internal for ordinary callers:

```rust
let epoch = gpu
    .begin_epoch("frame")
    .add(simulation)
    .add(rendering)
    .validate()?;

let submission = gpu.submit(epoch)?;
```

It may remain an internal or advanced representation, but the common path should be one submission call.

## Ergonomic acceptance criteria

A G2 specification is not decision-complete unless it binds all of the following:

1. A basic compute example contains no WGPU, ECS, renderer, application, or raw-ID types.
2. The ordinary path submits in one call and validates automatically.
3. A separate prepare/inspect path exposes deterministic diagnostics without creating duplicate authority.
4. Resource kind, lifetime, ownership, transfer, and reconstruction are modeled independently.
5. Strings are human labels, not resource or binding authority.
6. The intended work builder requires no repeated nested `.finish()` sequence.
7. Resource dependencies are inferred from declared access.
8. Explicit ordering is limited to non-data dependencies.
9. Public handles have safe RAII behavior and delayed backend retirement.
10. Errors name the human operation and resource, explain the failure, and suggest correction.
11. Typed GPU data has explicit layout guarantees and does not imply one universal representation.
12. One render and one independent non-render consumer use the same generic work contract without renderer terminology.
13. A Runenwerk adapter proves ECS/domain preparation remains outside RunenGPU.
14. Current RenderFlow ergonomics are retained only where they do not preserve mixed ownership.
15. Simple and advanced paths compile into the same validated preparation authority.
16. The API does not require users to understand work graphs, epochs, admission, realization, or retirement for common tasks.

## Phase implications

### G2

Bind resource dimensions, capability requirements, typed handles, prepared-value ownership, public vocabulary, simple/advanced examples, diagnostics requirements, and exact RenderFlow disposition.

### G3

Implement generic work/access semantics and dependency inference behind the public builder. Keep the graph inspectable but not mandatory in ordinary code.

### G4

Bind admitted shader/pipeline interfaces, typed binding keys, context/device authority, data-layout policy, and WGPU realization.

### G5

Implement automatic prepare-and-submit, uploads, updates, completion, asynchronous readback, cancellation, and RAII retirement behavior.

### G6-G8

Prove the same public experience across rendering, non-render compute, surfaces, diagnostics, shutdown, and standalone conformance.

## Final verdict

The architecture remains sound. The public framework should be shaped as:

```text
simple domain or generic facade
    -> immutable typed GpuWork
        -> hidden validated preparation graph
            -> WGPU backend
```

It must not require every caller to manually author and submit a GPU work graph.
