---
title: Roadmap
description: Maintained high-level sequencing for Runenwerk and its peer frameworks.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-27
related_docs:
  - ../engineering-workflow.md
  - ./active-work.md
  - ./completed-work.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runengpu-g3-access-work-graph-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../reports/investigations/runengpu-industry-comparison.md
  - ../../reports/investigations/runengpu-public-api-ergonomics-review.md
  - ../../reports/investigations/runengpu-proof-workload-strategy.md
  - ../../reports/investigations/runengpu-g2-capabilities-resources-investigation.md
  - ../../reports/investigations/runengpu-g3-access-work-graph-investigation.md
  - ../../reports/investigations/runengpu-render-s0-inventory.md
  - ../../reports/investigations/runengpu-render-s0-file-disposition.md
  - ../../reports/investigations/runengpu-render-s0-identity-consumer-lifecycle.md
  - ../../reports/closeouts/pt-runengpu-g1a-closeout.md
  - ../../reports/closeouts/pt-runengpu-g2-implementation-closeout.md
  - ../specs/pt-runengpu-g2-capabilities-resource-descriptors.ron
  - ../specs/pt-runengpu-g3-access-work-graph.ron
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

1. Complete and independently review the documentation-only G3 decision phase owned by issue `#174`.
2. Accept or correct the [G3 design](../../design/active/runengpu-g3-access-work-graph-design.md), [investigation](../../reports/investigations/runengpu-g3-access-work-graph-investigation.md), and [implementation specification](../specs/pt-runengpu-g3-access-work-graph.ron).
3. Create one bounded G3 implementation issue only after that planning PR is merged.
4. Keep G4-G7 deferred to their existing owners and implement each only through its own accepted planning slice.
5. Extract RunenGPU and perform a clean Runenwerk cutover only after internal conformance and extraction-readiness gates pass.
6. Prove RunenRender internally on RunenGPU, then extract and cut over RunenRender.
7. Resume RunenECS boundary repair as separately bounded work.

The RunenSDF cutover, RunenGPU S0, RunenGPU G1A, and RunenGPU G2 are complete. G2 merged through PR `#173` as `709aa6aced020ee99405e1e1c3dde7703c77a4d4`; issue `#172` is closed. G3 planning is active through issue `#174`. G3 Rust implementation remains unauthorized until its planning authority is accepted and a separate implementation issue exists.

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

Required qualities are fixed:

- ordinary submission validates automatically;
- graph, epoch, admission, realization, and retirement terminology remain advanced or internal unless the caller needs them;
- strings are diagnostic labels or reasons, not resource/node/dependency authority;
- typed handles, typed node identities, `GpuExportKey`, and validated binding keys replace stringly authority;
- builders use lexical/closure scope rather than repeated nested `finish()` ladders;
- ordering is inferred from declared resource access, with explicit ordering reserved for non-data dependencies and redundant data edges rejected;
- public handles use safe RAII and delayed backend retirement;
- errors name the human label, operation, cause, and corrective action;
- simple and advanced paths compile to the same validated authority.

The [public API ergonomics review](../../reports/investigations/runengpu-public-api-ergonomics-review.md) remains binding design pressure.

### Resource model

Resource kind, lifetime, ownership, transfer/observation, reconstruction, and memory intent remain separate dimensions.

```text
kind
    buffer, texture, texture view, sampler, query set

lifetime
    transient, retained

ownership
    RunenGPU-owned, imported, surface-acquired

transfer and observation
    initial data, update/upload, copy, query resolution, readback request, export relationship

reconstruction
    source-backed, externally reconstructed, non-reconstructable

memory intent
    device, upload buffer, readback buffer
```

`Imported`, `Exported`, `Readback`, and `SurfaceOwned` are not lifetime classes. Upload/readback memory intent applies to buffers; textures remain device resources and participate through explicit copy relationships.

Buffer and texture initialization are distinct. Texture initialization binds format, extent, `bytes_per_row`, and `rows_per_image`. Texture-view validity cannot exceed the parent texture lease or checked subresource range. Query-set indices are initialized only by explicit graph-entry evidence or accepted timestamp writes.

### Typed-data boundary

```text
Runenwerk or source-domain adapter
    ECS or domain state
        -> explicit prepared value or bytes
            -> RunenGPU upload/update contract
```

Uniform, storage, vertex, indirect, transfer, texture-initialization, and readback-decoding semantics remain distinct. Transitional render-adapter `TypeId` is process-local declared-Rust-type compatibility evidence for current uniform projection and indirect argument checks; it is not normalized GPU layout, descriptor, persistence, replay, wire, cache, or shader-interface authority. Type names are diagnostic display only. G4 owns backend layout and derive/macro disposition. G5 owns upload/update/readback execution.

### G3 access and work graph

The G3 planning authority binds:

```text
checked buffer byte ranges
checked texture mip/layer/aspect ranges
checked query ranges
normalized QueryResolve destination buffer usage
texture-view normalization to parent storage
render color/depth attachments
multisample resolve targets inside render attachments
attachment Load/Clear and Store/Discard semantics
region-aware graph-entry initialized coverage
RAW, WAR, and WAW dependency causes
operation/access-derived capability requirements
immutable GpuWorkFragment and GpuWorkNode values
standalone typed query-set resolve work
typed import/export causality
fragment-local explicit non-data order
deterministic GpuPreparedWorkGraph preparation
```

Within one fragment, lexical node order orients access-derived hazards. Fragment collection position is not semantic scheduling authority. Every cross-fragment overlap with at least one write requires shared typed resources plus matching typed imports/exports. Overlapping cross-fragment writers without one unique producer are rejected.

Multisample texture resolution remains a relation on a render color attachment because that is the accepted WGPU/WebGPU execution shape. Standalone `Resolve` work is query-set-to-buffer resolution.

Timestamp writes initialize exact query indices. Typed query resolution consumes initialized indices and writes an exact device-buffer range; the later copy to a readback buffer is ordinary typed buffer-copy work. G4/G5 retain backend alignment, command encoding, mapping, completion, stale-generation, and runtime-retirement authority.

G3 validates graph-time initialization and composition evidence only; it does not claim execution persistence or synchronization.

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
- G2 current-main census and decision-complete specification through issue `#168` and PR `#171`;
- G2 bounded implementation through issue `#172` and PR `#173`, merged as `709aa6aced020ee99405e1e1c3dde7703c77a4d4`, with evidence in the [implementation closeout](../../reports/closeouts/pt-runengpu-g2-implementation-closeout.md).

G1A delivered owner-scoped, fallible `GpuWorkResourceId` allocation and closed the confirmed cross-flow collision seam. G2 delivered normalized capabilities/resources, kind-typed logical handles, prepared-data contracts, explicit import lowering, and deletion of replaced renderer authority.

### Extraction sequence

```text
G1A owner-scoped logical GPU work-resource identity                     complete
-> G2 capabilities, resources, typed handles, prepared-data seam       complete
-> G3 decision-complete planning                                       active through #174
-> G3 access, initialization, hazards, immutable work, internal graph   blocked on planning acceptance
-> G4 context/device, shader/pipeline, binding/layout, WGPU              deferred
-> G5 execution, uploads, completion, readback, retirement              deferred
-> G6 offscreen graphics and shared render/non-render proof             deferred
-> G7 surfaces, generations, thread affinity, device outcomes           deferred
-> G8 final diagnostics, shutdown, residual audit                       deferred
-> GX external dornglut/runen-gpu transfer and clean cutover            blocked
-> internal RunenRender proof on RunenGPU
-> external dornglut/runen-render transfer and cutover
```

### Phase closure rules

Every G2-G7 phase must:

- start from current `main` and an exact declaration/consumer inventory;
- establish one future-transferable owner boundary;
- preserve simple and advanced API paths without exposing lower-level machinery unnecessarily;
- migrate all consumers of the authority replaced by that phase;
- delete replaced authority without aliases or parallel paths;
- preserve working RenderFlow-facing behavior only through explicit Runenwerk or RunenRender adapters;
- run focused checks plus exact-head `cargo validate` and `git diff --check`;
- update the parent issue slice index and durable planning state.

The temporary crate-private bridge that scopes `GpuWorkResourceIdAllocator` from `RenderFlowId` remains exactly one G3 adapter seam because live context ownership begins in G4. G4 context/work-scope authority must delete it; it cannot survive G8.

The proof portfolio remains:

- G5 deterministic compute conformance: exact inclusive/exclusive 4,097-element `u32` prefix scan with complete output comparison;
- G5 stateful integration: headless fixed-seed Game of Life with full-grid CPU oracle, exact live count/checksum, and selected-cell assertions;
- G5 conditional texture proof: deterministic integer compute-to-texture, padded readback normalization, and Runenwerk-owned PNG encoding;
- G6 graphics conformance: offscreen known-pattern draw with selected-pixel evidence;
- G6 GPU-driven composition: compute-generated indirect draw with ordering inferred from resource access;
- G6 visual showcase: offscreen boids with structural, finite, bounded, and overflow evidence;
- G7 surface proof: reuse accepted G6 workloads;
- first RunenRender semantic proof: procedural sky/SDF terrain;
- later temporal/history proof: the SDF history flow.

G2 acceptance does not make RunenGPU extraction-ready, authorize an external package, or predefine backend/execution contracts.

## RunenECS

RunenECS remains a separate workstream. Continue only through bounded internal boundary-repair changes that do not conflict with active GPU/render, SDF cutover, manifest, identity, or lifecycle work.

## RunenUI

RunenUI is governed in `dornglut/runen-ui`. Runenwerk eventually owns only the integration adapter between accepted renderer-neutral UI output and RunenRender contributions after both public boundaries stabilize.

## Sequencing rule

Structural changes sharing manifests, lockfiles, identities, dependency direction, lifecycle ownership, repository guards, or canonical planning authority are serialized or explicitly rebased. A completed planning or architecture change proves only its own scope and never implies external extraction readiness.
