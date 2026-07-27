---
title: RunenGPU G3 Access and Work Graph Investigation
description: Current-main access, initialization, hazard, dependency, generic-work, graph, render-adapter, and GPU-primitive census for the decision-complete G3 specification.
status: active
owner: gpu
layer: investigation
canonical: false
last_reviewed: 2026-07-27
related_docs:
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/specs/pt-runengpu-g2-capabilities-resource-descriptors.ron
  - ../../workspace/specs/pt-runengpu-g3-access-work-graph.ron
  - ./runengpu-g2-capabilities-resources-investigation.md
  - ../closeouts/pt-runengpu-g2-implementation-closeout.md
---

# RunenGPU G3 Access and Work Graph Investigation

## Outcome

G3 can be specified without a new ADR, external package, dependency, or premature G4/G5/G7 implementation.

The verified source baseline is:

```text
repository: dornglut/runenwerk
branch inspected: main
commit: 709aa6aced020ee99405e1e1c3dde7703c77a4d4
planning issue: #174
planning branch: docs/runengpu-g3-access-work-graph
```

G2 is accepted through issue `#172` and merged PR `#173`. Its normalized capabilities, logical descriptors, typed handles, prepared-data contracts, diagnostic/provenance rules, and render adapters are the required input. G3 must extend that authority rather than recreating resource descriptions or backend execution.

The current renderer contains useful correctness evidence, but not a transferable work model. Access is split across broad `reads`/`writes`, role-specific pass fields, whole-resource lifetime windows, explicit pass dependencies, and a second GPU-primitive access model. Buffer ranges, texture subresources, partial initialization, overlap, and cross-fragment causality are absent.

## Binding ownership

```text
Runenwerk adapters and host policy
    -> RunenRender semantic image planning
        -> RunenGPU generic work and validation
            -> G4/G5 WGPU admission and execution
```

G3 owns immutable generic work intent, work-time access ranges, initialization-flow facts, hazards, inferred data dependencies, explicit non-data ordering, deterministic graph preparation, and inspection facts.

G3 does not own shader files, pipeline realization, binding-layout admission, WGPU objects, command encoding, submission, completion, readback, retirement, surfaces, renderer semantics, ECS projection, fixed-time scheduling, or product policy.

## Current declaration census

### Render pass graph

`engine/src/plugins/render/graph/pass_graph.rs` currently defines renderer-shaped nodes with:

```text
reads / writes
depends_on
sampled_textures / write_textures
vertex_buffers / index_buffers / instance_buffers / indirect_buffers
depth_target
uniform_bindings
render shader, draw, raster, view, fixed-step, and product-facing fields
```

Useful evidence:

- access roles already matter independently from broad read/write classification;
- compute, graphics, copy, and present consumers exist;
- vertex, index, indirect, uniform, storage, sampled, storage-texture, attachment, and present uses are required;
- explicit order and data order are currently conflated.

Defects:

- no buffer byte ranges;
- no texture mip/layer/aspect access ranges;
- a texture view is not normalized to its parent resource for hazards;
- role lists can disagree with broad reads/writes;
- strings and renderer pass IDs own dependency authoring;
- renderer meaning and generic GPU correctness share one node.

Disposition: retain `RenderPassNode` only as transitional render authoring. Lower it through one render-owned G3 adapter into generic immutable work. It must not remain competing access, initialization, hazard, or ordering authority.

### Flow validation and ordering

`engine/src/plugins/render/graph/validation.rs` validates renderer pass shape, resource kind/usage, explicit `depends_on`, and then topologically sorts only those explicit edges.

Useful evidence:

- duplicate/unknown/foreign references, cycles, invalid roles, copy class mismatches, present constraints, and capability errors already need structured reporting;
- deterministic topological order is required by runtime planning.

Defects:

- data dependencies are not inferred;
- declaration order and explicit edges can accidentally hide hazards;
- a caller must manually reproduce ordering already implied by accesses;
- renderer-specific diagnostics are the only graph error authority.

Disposition: preserve renderer-only shape validation above the boundary. Move generic resource/use validation, initialization, hazard inference, cycle detection, and deterministic generic order to RunenGPU. Delete the local generic topological/lifetime authority when all consumers use the prepared GPU graph.

### Whole-resource lifetime windows

`engine/src/plugins/render/graph/resource_lifetimes.rs` defines `CompiledResourceAccessKind` and `CompiledResourceLifetimeWindow`. It records first/last use/read/write by pass index and reports only transient read-before-first-write.

Useful invariant: uninitialized transient content cannot be read.

Defects:

- whole-resource only;
- write classification is incomplete;
- no initialized-region coverage;
- no read-after-write, write-after-read, or write-after-write edge cause;
- no texture-view parent normalization;
- no cross-fragment composition;
- first/last windows are observations after an explicit order, not the authority that derives order.

Disposition: replace with G3 initialized-coverage and dependency summaries. Delete this renderer-owned generic authority in the G3 implementation slice without forwarding types or aliases.

### Compiled render planning

`engine/src/plugins/render/graph/planning.rs` embeds renderer resources, pass order, execution plan, lifetime windows, and diagnostics in `CompiledRenderFlowPlan`.

Disposition: the temporary render adapter produces one `GpuPreparedWorkGraph`, then maps its prepared node order back to render-owned execution payloads. `CompiledRenderFlowPlan` may retain render execution data, but it must consume rather than duplicate G3 correctness.

### GPU primitives

`engine/src/plugins/render/gpu_primitives/plan.rs` defines:

```text
GpuPrimitiveResourceAccessKind { Read, Write }
GpuPrimitiveResourceAccess
GpuPrimitiveDispatchResource::Temporary(String)
GpuPrimitiveDispatchStage { reads, writes, depends_on: Vec<String> }
GpuPrimitiveDispatchPlan
GpuPrimitiveExecutionPlan
```

Useful evidence:

- multi-stage prefix scan requires typed temporary storage;
- counter reset, scan, scatter, and indirect-argument generation require deterministic compute stages;
- temporary resources and access-derived stage order are real non-render use cases.

Defects:

- second generic access model outside RunenGPU;
- string temporary-resource identity;
- stage labels used as dependency authority;
- broad sequential chaining instead of resource-derived dependencies;
- render shader asset paths embedded in otherwise generic planning.

Disposition: primitive descriptors remain source-domain conveniences, but lower to G3 typed resources and nodes. Delete primitive-local access and string dependency authority in the G3 implementation slice. Shader source/pipeline realization remains G4/Runenwerk adapter ownership.

### G2 resource access and export seam

`GpuResourceAccessIntent` currently represents only an export relationship's required final read/write intent. Its source comment explicitly defers ranges, subresources, hazards, and work-time validation to G3.

`GpuExportRelationship` already proves that provenance is non-semantic and the consumer-owned export key is semantic. G3 should replace its raw `String` key with a validated `GpuExportKey` newtype because it participates in fragment composition and is not a diagnostic label.

### Resource identity bridge

`GpuWorkResourceId` is owner-scoped and accepted. The allocator's owner-scope constructor remains crate-private. Current render authoring derives the scope from `RenderFlowId`.

G3 cannot honestly replace this bridge with a live `GpuContext`, because context/device authority begins in G4. The accepted bounded decision is:

- do not add a process-global mutable context or public owner-scope constructor;
- retain exactly one crate-private render-adapter bridge during G3;
- allow G3 fragment-local node allocation without creating a second resource identity;
- require G4 context/work-scope authority to delete the `RenderFlowId` bridge.

## Required G3 model

### Access ranges

Buffer access uses a checked concrete half-open byte range:

```text
GpuBufferRange { offset, size }
```

`whole(&GpuBufferHandle)` resolves immediately to `offset = 0` and the descriptor size. `new(&GpuBufferHandle, offset, size)` rejects zero size, arithmetic overflow, and out-of-bounds coverage. No saturating arithmetic or unchecked `Range<u64>` becomes authority.

Texture access reuses `GpuTextureSubresourceRange`, extended with semantic ordering/hashing and descriptor-bound validation. Texture-view access is normalized to:

```text
parent texture handle + intersection of view range and requested range
```

Hazards are evaluated on the normalized parent texture, not the view handle alone.

Timestamp query use requires a checked `GpuQueryRange { first, count }`; sampler use is immutable input evidence and creates no write hazard.

### Access categories

One broad read/write enum is insufficient. Exact buffer categories:

```text
UniformRead
StorageRead
StorageWrite
StorageReadWrite
VertexRead
IndexRead
IndirectRead
CopySource
CopyDestination
```

Exact texture categories:

```text
SampledRead
StorageRead
StorageWrite
StorageReadWrite
CopySource
CopyDestination
ColorAttachment { load }
DepthStencilAttachment { access, load }
Present
```

Attachment load is `Load`, `Clear`, or `Discard`. `Load` requires initialized coverage. `Clear` and `Discard` do not read prior content and initialize the written coverage. Depth/stencil access distinguishes read-only from read-write.

### Initialization coverage

Initialization is region-aware and graph-time only:

- G2 `Zeroed` and complete `Prepared` initialization begin fully initialized;
- `Uninitialized` begins with no initialized coverage;
- pure write, clear, copy destination, and discard initialize only their covered range/subresources;
- read, read-write, attachment load, indirect, vertex, index, uniform, sampled, copy source, and present require initialized coverage;
- imported or retained prior-epoch content is accepted only through explicit initial-state evidence supplied by the adapter/consumer;
- G3 validates the evidence but does not claim G5 execution actually preserved or synchronized it;
- initialization state is not persisted, serialized, or inferred from a diagnostic label.

### Hazard and dependency rules

For overlapping normalized regions:

| Earlier access | Later access | Result |
|---|---|---|
| read | read | no edge |
| write | read | inferred RAW edge |
| read | write | inferred WAR edge |
| write | write | inferred WAW edge only when a unique order exists; otherwise ambiguous-writer error |
| read-write | any overlap | treat as both read and write |

Disjoint buffer ranges and disjoint texture mip/layer/aspect ranges do not conflict.

Within one fragment, lexical node declaration order is the stable orientation for access-derived hazards. This is not an extra manual dependency API.

Fragments are not ordered by their position in an input array. Cross-fragment causality must come from shared typed resources plus explicit import/export relationships. An overlapping cross-fragment write with no unique producer/export relation is rejected rather than ordered arbitrarily.

Explicit ordering is accepted only as `GpuExplicitOrder` between typed node IDs in the same fragment and must carry a non-empty diagnostic reason. It is for non-data constraints. If an explicit edge conflicts with inferred data order or creates a cycle, preparation fails.

### Immutable work and graph preparation

`GpuWorkFragment` is immutable after construction. It contains declared/imported/exported resources, nodes, explicit non-data edges, outputs, and provenance.

G3 nodes are immutable pre-admission operation intent. They own operation kind, access facts, capability requirements, execution-shape facts already independent of a backend, label, and provenance. Current render shader/pipeline/draw payload remains in a render-owned sidecar keyed by the prepared node ID. G4 replaces that sidecar seam with admitted generic program/interface authority.

The initial node kinds remain:

```text
Compute
Render
Copy
Clear
Resolve
Present
```

The single advanced preparation authority is `GpuPreparedWorkGraph::prepare(...)`. There is no public mutable graph and no separate reduced-validation graph. It produces deterministic prepared node IDs/order, dependency edges with typed causes, normalized accesses, initialized-coverage summaries, exports, and structured diagnostics.

Ordinary G5 submission must later invoke this same preparation authority internally.

## Identity decision

`GpuWorkNodeId` is fragment-local, nonzero, private-construction identity allocated by the fragment builder. Composition derives `GpuPreparedWorkNodeId { fragment_ordinal, local_node }` deterministically. Neither identity is stable persistence, replay, wire, ABI, network, or cache authority.

This avoids a process-global allocator and avoids reusing `GpuWorkResourceId` as a node ID. Resources continue to use the one accepted G2 identity.

Cross-fragment explicit node-ID edges are not accepted in G3. Current render flows that require explicit non-data order lower all involved passes into one fragment. Independent fragments compose through typed resources and exports.

## Proposed module boundary

Future-transferable source:

```text
engine/src/plugins/gpu/api/access.rs
engine/src/plugins/gpu/api/work.rs
engine/src/plugins/gpu/api/graph.rs
engine/src/plugins/gpu/api/errors.rs
engine/src/plugins/gpu/api/mod.rs
engine/src/plugins/gpu/mod.rs
```

Temporary integration:

```text
engine/src/plugins/render/adapters/gpu_work.rs
```

The G3 adapter is a fourth temporary render adapter. It lowers current render pass/resource declarations and primitive plans into G3 work, keeps render execution payloads outside RunenGPU, and is deleted incrementally through G4-G6. It is not future RunenGPU source.

## Consumer and migration census

Direct declaration/validation consumers include:

```text
engine/src/plugins/render/api/passes.rs
engine/src/plugins/render/graph/pass_graph.rs
engine/src/plugins/render/graph/validation.rs
engine/src/plugins/render/graph/planning.rs
engine/src/plugins/render/graph/execution_plan.rs
engine/src/plugins/render/graph/resource_lifetimes.rs
engine/src/plugins/render/graph/prepared_validation.rs
engine/src/plugins/render/graph/merge.rs
engine/src/plugins/render/gpu_primitives/plan.rs
```

Runtime/inspection consumers include:

```text
engine/src/plugins/render/renderer/render_flow/execute.rs
engine/src/plugins/render/renderer/render_flow/preflight_cache.rs
engine/src/plugins/render/renderer/render_flow/runtime_resources.rs
engine/src/plugins/render/inspect/plan.rs
engine/src/plugins/render/inspect/resource_inspector.rs
engine/src/plugins/render/inspect/graph_dump.rs
```

Representative tests/examples/benchmarks include render-flow, dynamic-target, fragment, primitive, boids, SDF, compositor, editor, and draw consumers. The implementation specification must bind the exact current-main file inventory before source changes because G2 altered this surface substantially.

## Deletion target

The G3 implementation slice must delete or remove as generic authority:

```text
CompiledResourceAccessKind
CompiledResourceLifetimeWindow
compile_resource_lifetime_windows
diagnose_resource_lifetime_windows
renderer-local generic topological sorting
GpuPrimitiveResourceAccessKind
GpuPrimitiveResourceAccess
GpuPrimitiveDispatchResource::Temporary(String)
string stage dependency authority
broad RenderPassNode access/lifetime correctness as final authority
```

Render-only shape and execution payload types may remain until their owning phases, but they must consume the one G3 prepared graph.

## Alternatives rejected

### Keep explicit render pass dependencies

Rejected. It preserves renderer-shaped ceremony, duplicates data knowledge, and cannot compose independent non-render fragments safely.

### Infer only whole-resource dependencies

Rejected. It unnecessarily serializes disjoint buffer ranges and texture subresources and cannot validate partial initialization.

### Make composition array order semantic

Rejected. It silently converts caller collection order into cross-consumer scheduling policy and hides ambiguous writers.

### Build a mandatory public graph DSL

Rejected. The graph remains advanced/inspectable authority; ordinary consumers contribute immutable fragments through domain or generic builders.

### Implement backend barriers in G3

Rejected. G3 produces backend-neutral dependencies and access facts. G4 maps them to WGPU realization and G5 encodes/submits them.

### Introduce resource aliasing or pass fusion

Rejected. They are optimizations without current correctness pressure and would obscure the initial hazard model.

## Stable-format and dependency audit

No current evidence makes the proposed ranges, node IDs, edges, or prepared graph a persisted, replay, network, wire, or external format. Existing inspection and diagnostic output is process-local. The specification must explicitly deny stability promises and add source guards.

No new dependency, package, workflow, lockfile change, raw WGPU public type, ECS type, renderer semantic type, Winit type, SDF/UI/product type, or codec is required.

## Planning conclusion

Proceed with one decision-complete G3 specification and documentation-only planning PR. Do not create the implementation issue until that planning authority is independently reviewed and merged. Do not start G3 Rust implementation from issue `#174`.