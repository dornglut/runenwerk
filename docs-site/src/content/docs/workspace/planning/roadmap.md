---
title: Roadmap
description: Maintained high-level sequencing for Runenwerk and its peer frameworks.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-26
related_docs:
  - ../engineering-workflow.md
  - ./active-work.md
  - ./completed-work.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../reports/investigations/runengpu-industry-comparison.md
  - ../../reports/investigations/runengpu-public-api-ergonomics-review.md
  - ../../reports/investigations/runengpu-proof-workload-strategy.md
  - ../../reports/investigations/runengpu-g2-capabilities-resources-investigation.md
  - ../../reports/investigations/runengpu-render-s0-inventory.md
  - ../../reports/investigations/runengpu-render-s0-file-disposition.md
  - ../../reports/investigations/runengpu-render-s0-identity-consumer-lifecycle.md
  - ../../reports/closeouts/pt-runengpu-g1a-closeout.md
  - ../specs/pt-runengpu-g2-capabilities-resource-descriptors.ron
---

# Roadmap

This roadmap records high-level sequence and dependencies. GitHub issues and pull requests own live delivery.

## Repository family

```text
RunenSDF ----+
RunenECS ----+--> Runenwerk adapters/integration --> applications
RunenUI -----+
                  |
                  +--> RunenRender --> RunenGPU
                  +--> non-render RunenGPU workloads
```

Runenwerk remains the integration and product repository. Framework repositories do not depend on Runenwerk.

## Current priorities

1. Merge the decision-complete RunenGPU G2 planning PR `#171`, closing issue `#168`.
2. Execute issue `#172` as the only G2 implementation slice after the planning authority is accepted.
3. Migrate and delete the capability/resource authority replaced by G2 without compatibility aliases, forwarding modules, or duplicate descriptors.
4. Continue G3-G8 as individually specified internal boundary slices, migrating and deleting replaced authority in each phase rather than deferring a broad cutover to G8.
5. Extract RunenGPU and perform a clean Runenwerk cutover only after internal conformance and extraction-readiness gates pass.
6. Prove RunenRender internally on RunenGPU, then extract and cut over RunenRender.
7. Resume RunenECS boundary repair as separately bounded work.

The RunenSDF cutover, RunenGPU S0, and RunenGPU G1A gates are complete. G2 implementation owns the serialized active queue after PR `#171` merges; read-only investigation may still proceed independently.

## RunenSDF

The maintained standalone framework is:

```text
repository: dornglut/runen-sdf
maintained revision: ffa970f3eb7fd9ebaa1cfc67665e3e3128cd0676
source-transfer revision: d52badefc640d6dc6dcdd40268af3aea1bb8eefe
package: runen-sdf
crate: runen_sdf
```

Issue `#133` completed the retirement-only cutover after an exact-head census proved zero real consumers of the internal package. Runenwerk now contains no `domain/sdf` package, workspace member, lockfile package, local framework documentation mirror, compatibility namespace, submodule, source include, or unused external dependency.

Current authority is split deliberately:

- reusable signed-field mathematics and CPU reference queries: `dornglut/runen-sdf`;
- Runenwerk world/product integration and payloads: `domain/world_sdf` and Runenwerk-owned adapters.

The repository audit rejects return of the retired internal package and duplicate authority. Detailed evidence is recorded in the PT-RUNENSDF-004 closeout report.

## RunenGPU and RunenRender

Accepted dependency:

```text
RunenRender -> RunenGPU
```

Target layering:

```text
Runenwerk adapters and host policy
    -> RunenRender semantic image plan
    -> RunenGPU workload/resource contracts
    -> WGPU backend
```

Non-render consumers may lower directly into RunenGPU workloads without depending on RunenRender.

The [industry comparison](../../reports/investigations/runengpu-industry-comparison.md) confirms the intended middle ground: a backend-neutral device layer similar in placement to Godot RenderingDevice or bgfx, plus a validated workload graph informed by Unreal, Unity, and Filament, without making render semantics or ECS integration part of the GPU framework.

### Public API experience

The internal work graph is a correctness and inspection model, not the required entry point for ordinary callers.

The intended common path is conceptually:

```rust
let simulation = simulation.gpu_work(&gpu, &simulation_state)?;
let rendering = renderer.gpu_work(&gpu, &prepared_scene, request)?;
let submission = gpu.submit("frame 42", [simulation, rendering])?;
```

The inspectable path is separate:

```rust
let prepared = gpu.prepare("frame 42", [simulation, rendering])?;
inspect(prepared.diagnostics());
let submission = gpu.submit_prepared(prepared)?;
```

Exact names remain implementation decisions only where the G2 specification leaves them open. Required qualities are fixed:

- ordinary submission validates automatically;
- graph, epoch, admission, realization, and retirement terminology remain advanced or internal unless the caller needs them;
- strings are diagnostic labels, not resource or binding authority;
- typed handles and validated binding keys replace stringly references;
- builders use lexical/closure scope rather than repeated nested `finish()` ladders;
- ordering is inferred from declared resource access, with explicit ordering reserved for non-data dependencies;
- public handles use safe RAII and delayed backend retirement;
- errors name the human label, operation, cause, and corrective action;
- simple and advanced paths compile to the same validated authority.

The [public API ergonomics review](../../reports/investigations/runengpu-public-api-ergonomics-review.md) is binding design pressure for G2-G5 specifications.

### Resource-model correction

Resource kind, lifetime, ownership, transfer/observation, reconstruction, and memory intent are separate dimensions.

```text
kind
    buffer, texture, texture view, sampler, query set

lifetime
    transient, retained

ownership
    RunenGPU-owned, imported, surface-acquired

transfer and observation
    initial data, update/upload, copy, readback request, export relationship

reconstruction
    source-backed, externally reconstructed, non-reconstructable

memory intent
    device, upload buffer, readback buffer
```

`Imported`, `Exported`, `Readback`, and `SurfaceOwned` must not be treated as interchangeable lifetime classes. Upload/readback memory intent applies to buffers; textures remain device resources and participate through explicit copy relationships.

Buffer and texture initialization are distinct. Texture initialization binds format, extent, `bytes_per_row`, and `rows_per_image`. Texture-view validity cannot exceed the parent texture lease or checked subresource range.

### Typed-data boundary

```text
Runenwerk or source-domain adapter
    ECS or domain state
        -> explicit prepared value or bytes
            -> RunenGPU upload/update contract
```

Uniform, storage, vertex, indirect, transfer, texture-initialization, and readback-decoding semantics remain distinct. `TypeId` and type names are process-local diagnostics, not layout, binding, persistence, replay, wire, cache, or shader-interface authority. G4 owns backend layout and derive/macro disposition. G5 owns upload/update/readback execution.

### Current RenderFlow disposition

The current `RenderFlow` API is a transitional combined facade. It remains operational during migration but is not copied wholesale into either framework.

```text
Current RenderFlow responsibility                 Target owner
--------------------------------------------------------------------------
GPU resource identity/descriptors/access/work      RunenGPU
WGPU context/resource/pipeline/submission           RunenGPU backend
render views/targets/image-formation semantics      RunenRender
ECS state projection/fixed-time/application policy  Runenwerk
shader discovery/hot reload/window/UI integration   Runenwerk adapters
```

The current mixed graph is decomposed incrementally. Each phase migrates consumers of the authority it replaces and deletes that authority in the same accepted slice. G8 is a final conformance and residual-reach-through audit, not the first broad migration.

### Completed foundation

- architecture correction through PR `#126`;
- deterministic S0 inventory through PR `#128`;
- original G1A implementation specification through PR `#130`;
- corrected owner-scoped and fallible-authoring G1A specification;
- G1A implementation through PR `#164`, merged as `5bbdab36ae661d99432bfe5d215062c397aac975`;
- G1A completion evidence in the [PT-RUNENGPU-G1A closeout](../../reports/closeouts/pt-runengpu-g1a-closeout.md);
- G2 industry/API/proof-workload planning through PRs `#169` and `#170`;
- G2 current-main census and decision-complete specification through issue `#168` and PR `#171`.

G1A delivered:

```text
RenderResourceId
    -> GpuWorkResourceId { private owner scope, nonzero local value }

RenderResourceIdSequence
    -> owner-controlled GpuWorkResourceIdAllocator
```

The owner scope closes the confirmed cross-flow collision seam. Resource-allocating authoring now propagates structured failure, foreign-flow handles are rejected, and the renderer-owned identity names and allocator authority are deleted without aliases.

### Extraction sequence

```text
G1A owner-scoped logical GPU work-resource identity (completed)
-> G2 capabilities, logical resources, typed handles, prepared-data seam, and ownership split
-> G3 access, initialization flow, hazards, immutable generic work, and internal graph
-> G4 context/device admission, shader/pipeline admission, binding/layout, and WGPU realization
-> G5 headless execution, uploads, updates, submission, completion, readback, cancellation, and retirement
-> G6 offscreen graphics and shared render/non-render consumer proof
-> G7 surfaces, generations, thread affinity, and device outcomes
-> G8 final diagnostics, shutdown, residual anti-cheating audit, and conformance
-> GX external dornglut/runen-gpu transfer and clean Runenwerk cutover
-> internal RunenRender proof on RunenGPU
-> external dornglut/runen-render transfer and cutover
```

### Phase closure rules

Every G2-G7 phase must:

- start from current `main` and an exact declaration/consumer inventory;
- establish one future-transferable owner boundary;
- preserve the simple and advanced API paths without exposing lower-level machinery unnecessarily;
- migrate all consumers of the authority replaced by that phase;
- delete the replaced authority without aliases or parallel paths;
- preserve working RenderFlow-facing behavior only through explicit Runenwerk or RunenRender adapters;
- run focused checks plus exact-head `cargo validate` and `git diff --check`;
- update the parent issue slice index and durable planning state.

The temporary crate-private bridge that scopes `GpuWorkResourceIdAllocator` from `RenderFlowId` must be replaced by GPU-owned context/work-graph or epoch authority during G3/G4 and must not survive G8.

The proof portfolio selected by G2 is:

- G5 deterministic compute conformance: exact inclusive/exclusive 4,097-element `u32` prefix scan with complete output comparison;
- G5 stateful integration: headless fixed-seed Game of Life with full-grid CPU oracle, exact live count/checksum, and selected-cell assertions;
- G5 conditional texture proof: deterministic integer compute-to-texture, padded readback normalization, and Runenwerk-owned PNG encoding;
- G6 graphics conformance: offscreen known-pattern draw with selected-pixel evidence;
- G6 GPU-driven composition: compute-generated indirect draw with ordering inferred from resource access;
- G6 visual showcase: offscreen boids with structural, finite, bounded, and overflow evidence rather than exact cross-backend equality;
- G7 surface proof: reuse accepted G6 workloads;
- first RunenRender semantic proof: procedural sky/SDF terrain;
- later temporal/history proof: the SDF history flow.

G1A is a completed internal future-transferable slice. It does not by itself make RunenGPU extraction-ready, authorize an external package, or predefine exact later Rust contracts.

## RunenECS

RunenECS remains a separate workstream. Continue only through bounded internal boundary-repair changes that do not conflict with active GPU/render, SDF cutover, manifest, identity, or lifecycle work.

## RunenUI

RunenUI is governed in `dornglut/runen-ui`. Runenwerk eventually owns only the integration adapter between accepted renderer-neutral UI output and RunenRender contributions after both public boundaries stabilize.

## Sequencing rule

Structural changes sharing manifests, lockfiles, identities, dependency direction, lifecycle ownership, repository guards, or canonical planning authority are serialized or explicitly rebased. A completed planning or architecture change proves only its own scope and never implies external extraction readiness.
