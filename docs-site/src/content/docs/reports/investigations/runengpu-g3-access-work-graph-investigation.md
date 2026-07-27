---
title: RunenGPU G3 Access and Work Graph Investigation
description: Current-main access, initialization, hazard, dependency, generic-work, graph, render-adapter, query-resolution, and GPU-primitive census for the decision-complete G3 specification.
status: active
owner: gpu
layer: investigation
canonical: false
last_reviewed: 2026-07-27
related_docs:
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runengpu-g3-access-work-graph-design.md
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

G3 can proceed as a documentation and specification phase without a new ADR, external package, dependency, or premature G4/G5/G7 implementation.

```text
repository: dornglut/runenwerk
branch inspected: main
baseline: 709aa6aced020ee99405e1e1c3dde7703c77a4d4
planning issue: #174
planning branch: docs/runengpu-g3-access-work-graph
```

G2 is accepted through issue `#172` and merged PR `#173`. G3 extends its normalized descriptors, typed resource handles, initialization declarations, capability requirements, and export relationships. It must not recreate resource or backend authority.

The current renderer contains useful correctness evidence, but generic access is split across broad pass reads/writes, role-specific pass fields, whole-resource lifetime windows, explicit dependency lists, a second GPU-primitive plan, and renderer-owned GPU timestamp resolution. Buffer ranges, texture subresources, partial initialization, normalized texture-view overlap, query-resolution work, operation-implied capability requirements, and cross-fragment causality are absent from the future-transferable authority.

## Ownership boundary

G3 owns:

- checked work-time buffer ranges and texture/query subranges;
- graph-entry initialization evidence and region-aware initialized coverage;
- overlap, hazards, inferred data dependencies, and explicit non-data order;
- immutable generic work fragments and nodes;
- exact render attachment and multisample-resolve relationships;
- backend-neutral query-set resolution intent and exact logical destination coverage;
- operation/access-derived normalized capability requirements;
- deterministic preparation and inspection facts.

G3 does not own shader files, pipeline/binding admission, WGPU objects, realization, command encoding, submission, completion, readback decoding, runtime retirement, surfaces, renderer meaning, ECS projection, fixed-time scheduling, timing presentation, or product policy.

## Current declaration census

### Render pass graph

`engine/src/plugins/render/graph/pass_graph.rs` stores generic and render facts together:

```text
reads / writes / depends_on
sampled_textures / write_textures
uniform / vertex / index / instance / indirect roles
depth target
shader, raster, draw, view, fixed-step, and feature meaning
```

This proves the required access roles but is not the target authority. It has no byte ranges or texture subresources, can express contradictory role lists, and uses renderer pass IDs plus strings for order.

Disposition: `RenderPassNode` may remain temporary render authoring input. One render-owned adapter lowers it into G3 work. It cannot remain final access, initialization, hazard, dependency, or graph authority.

### Validation and ordering

`engine/src/plugins/render/graph/validation.rs` validates render shape and resource roles, then topologically sorts only explicit `depends_on` edges. Data order must be repeated manually.

Disposition: retain render-only shape checks above the boundary. Move generic resource/use validation, initialized coverage, hazards, inferred edges, cycles, and deterministic generic order to RunenGPU. Remove local generic topological authority after cutover.

Explicit order must remain genuinely non-data. Retaining an explicit edge that duplicates an inferred data edge would preserve redundant dependency authority and undermine the intended access-derived model. G3 therefore rejects redundant explicit order rather than retaining both causes or choosing between two normalization policies.

### Whole-resource lifetime windows

`engine/src/plugins/render/graph/resource_lifetimes.rs` records whole-resource first/last read/write and reports only transient read-before-write.

Useful invariant: uninitialized transient content cannot be read.

Missing authority:

- byte and subresource coverage;
- partial initialization;
- RAW, WAR, and WAW causes;
- texture-view parent normalization;
- query index initialization and resolution;
- cross-fragment composition;
- order inference.

Disposition: replace it with prepared-graph initialized-coverage and dependency summaries and delete the file in the G3 implementation slice.

Runtime use-after-retirement is not a G3 graph-time fact because no live context, backend generation, submission state, or retirement authority exists yet. G3 validates descriptor lifetime/ownership consistency and explicit graph-entry evidence. G4/G5 later reject stale-generation or retired backend values during admission/submission.

### Compiled render planning

`engine/src/plugins/render/graph/planning.rs` embeds renderer resources, explicit pass order, execution data, lifetime windows, and diagnostics.

Disposition: a temporary adapter produces one `GpuPreparedWorkGraph`; the render plan maps prepared node order to render-owned execution payloads. No second lifetime/hazard result remains.

### Render attachments and multisample resolution

WGPU/WebGPU multisample texture resolution is part of a render-pass color attachment through `resolve_target`; it is not a standalone command-encoder operation.

Official evidence:

- <https://docs.rs/wgpu/latest/wgpu/struct.RenderPassColorAttachment.html>
- <https://docs.rs/wgpu/latest/wgpu/struct.TextureFormatFeatureFlags.html#associatedconstant.MULTISAMPLE_RESOLVE>

The first planning draft incorrectly modeled multisample texture resolve as a standalone `Resolve` operation. That shape would not map honestly to the accepted first backend.

Disposition:

- `GpuRenderOperation` owns ordered color-attachment operations, optional depth/stencil attachment operation, draw intents, and render-side query writes;
- each color attachment may name an optional resolve destination;
- the operation derives source attachment and destination resolve accesses;
- source must be multisampled, destination single-sampled, and format/extent/subresources compatible;
- source and destination cannot alias;
- resolve destination is written regardless of source store policy;
- standalone `Resolve` work remains justified by query-set-to-buffer resolution.

This preserves backend-neutral operation meaning without fabricating an unavailable command.

### GPU primitives

`engine/src/plugins/render/gpu_primitives/plan.rs` defines a duplicate generic access model, string temporary-resource references, string stage dependencies, and sequential stage chaining.

The prefix-scan hierarchy proves that typed transient resources and inferred multi-stage dependencies are required outside rendering.

Disposition: primitive descriptors remain source conveniences but lower to G3 typed work. Delete primitive-local access types, `Temporary(String)`, and string stage dependency authority. Shader asset discovery remains outside RunenGPU until G4 program admission.

### GPU timestamp query resolution

`engine/src/plugins/render/renderer/render_flow/gpu_timing.rs` currently owns a complete backend-specific timestamp path:

```text
reserve two timestamp indices per pass
    -> attach beginning/end writes to the render or compute pass
    -> resolve_query_set into a QUERY_RESOLVE buffer
    -> copy the resolved bytes into a MAP_READ/COPY_DST readback buffer
    -> poll, map, decode u64 timestamps, and publish timing evidence
```

Current source evidence:

- `GpuPassTimingFrame` owns a `QuerySet`, resolve buffer, and readback buffer;
- the resolve buffer uses WGPU `QUERY_RESOLVE | COPY_SRC`;
- `CommandEncoder::resolve_query_set` writes one `u64` per timestamp query;
- a later buffer copy moves those bytes into the readback buffer;
- polling, mapping, timestamp-period conversion, and presentation of timing evidence are later-phase/runtime concerns.

Official WGPU evidence confirms that query resolution requires a dedicated query-resolve buffer usage and writes opaque query results as `u64` values:

- <https://docs.rs/wgpu/latest/wgpu/struct.BufferUsages.html#associatedconstant.QUERY_RESOLVE>
- <https://docs.rs/wgpu/latest/wgpu/struct.CommandEncoder.html#method.resolve_query_set>
- <https://docs.rs/wgpu/latest/wgpu/struct.QuerySet.html>

The accepted G2 normalized buffer usages currently omit query-resolution destination usage. The initial G3 draft modeled `WriteTimestamp` and `ResolveSource` ranges but omitted the operation and destination access that connect them. That state was not decision-complete for the current consumer.

Disposition:

- extend `GpuBufferUsage` with normalized `QueryResolve` during G3 implementation;
- add `GpuBufferAccessKind::QueryResolveDestination`;
- model timestamp writes as exact query-index writes that initialize those indices;
- model query-set resolution as a typed standalone `Resolve` operation consuming initialized query indices and writing an exact destination buffer range;
- calculate timestamp destination size as checked `query_count * 8` bytes;
- require the destination descriptor to admit `QueryResolve` usage;
- leave backend-specific destination-offset alignment and encoding to G4/G5;
- retain the later resolve-buffer-to-readback transfer as ordinary typed buffer-copy work;
- retain polling, map completion, timestamp-period conversion, artifacts, and diagnostic presentation outside G3.

This preserves one generic correctness graph without pulling execution or timing policy into RunenGPU prematurely.

### Capability requirements

The first planning draft allowed nodes to store capability requirements but did not bind which requirements follow mechanically from typed operations and accesses. Requiring every caller or adapter to restate obvious requirements would create another duplicate authority and permit impossible combinations such as timestamp work with timestamp queries explicitly disabled.

Disposition: G3 derives structural requirements and merges them through the accepted G2 requirement authority:

```text
Compute operation                    -> Compute
Render operation                     -> RenderPipeline
Copy or standalone Clear             -> Copy
Indirect draw                        -> IndirectDraw
Storage texture access               -> StorageTexture
Depth/stencil attachment             -> DepthAttachment
Timestamp write or query resolution  -> TimestampQuery
Present                              -> Presentation
```

Callers may add semantic requirements not inferable from operation shape. Operation-implied requirements cannot be disabled; incompatible merges fail before G4 admission.

### G2 seams

`GpuResourceAccessIntent` currently represents only export final access; its source explicitly defers ranges, subresources, and hazards to G3.

`GpuExportRelationship` correctly treats export identity as semantic and provenance as diagnostic. G3 should replace its raw `String` export key with a validated `GpuExportKey` newtype because the key participates in composition.

`GpuWorkResourceId` remains the one resource identity. The current `RenderFlowId`-derived owner-scope bridge cannot honestly become context-owned before G4. G3 retains exactly one crate-private adapter bridge and requires G4 to delete it. No global mutable context or public owner-scope constructor is introduced.

## Decision synthesis

### Checked ranges

Buffer access uses a concrete checked half-open interval:

```text
GpuBufferRange { offset, size }
```

`whole(&GpuBufferHandle)` resolves to descriptor size. Partial construction rejects zero size, overflow, and out-of-bounds coverage.

Texture access reuses `GpuTextureSubresourceRange`, adds semantic ordering/hashing and descriptor-bound checks, and normalizes texture-view accesses to the parent texture plus the checked range intersection.

Timestamp consumers justify a checked `GpuQueryRange`. Samplers are immutable input evidence and create no write hazard.

### Exact access roles

Buffer access distinguishes uniform, storage read/write/read-write, vertex, index, indirect, copy source/destination, and query-resolve destination.

Texture access distinguishes sampled, storage read/write/read-write, copy source/destination, color attachment, multisample resolve destination, depth/stencil attachment, and present.

Query access distinguishes timestamp writes from resolve-source reads.

Attachment semantics require separate load and store facts:

```text
load:  Load | Clear
store: Store | Discard
```

`Load` requires initialized coverage. `Clear` establishes full initialized coverage for the attachment subresources. `Store` preserves post-node coverage. `Discard` removes later readable coverage. Discard is not a load mode, and Store alone does not make an empty render operation meaningful.

### Initialization flow

- `Zeroed` and complete G2 `Prepared` initialization begin initialized.
- `Uninitialized` begins with no initialized coverage.
- query sets begin with no initialized indices unless explicit graph-entry evidence exists;
- pure writes and copy destinations initialize only covered regions;
- timestamp writes initialize exact query indices;
- query resolution requires initialized source indices and initializes its exact destination byte range;
- render attachment load requires prior initialized coverage;
- render attachment clear establishes source coverage;
- render draw/write preserves or updates source attachment coverage;
- a multisample resolve establishes destination coverage regardless of source store policy;
- attachment discard invalidates only the source attachment's post-node readable coverage;
- read-write requires prior initialized coverage;
- imported and retained prior-epoch contents require explicit graph-entry evidence;
- G3 validates evidence but does not claim G5 actually preserved, uploaded, synchronized, resolved, copied, mapped, or retired content.

### Hazard rules

For overlapping normalized regions:

| Earlier | Later | Result |
|---|---|---|
| read | read | no edge |
| write | read | RAW edge |
| read | write | WAR edge |
| write | write | WAW edge only with unique producer order; otherwise error |
| read-write | overlap | acts as both read and write |

Disjoint byte ranges and disjoint mip/layer/aspect/query ranges do not conflict.

Within one fragment, lexical node order orients data hazards. Callers do not restate those edges.

Fragment collection position is not semantic scheduling authority. Any cross-fragment conflict with at least one write requires a matching typed import/export producer-consumer relation. Read/write conflicts without that relation fail as missing causality rather than being oriented by access kind or input position. Overlapping cross-fragment writers without one unique producer are rejected as ambiguous.

Timestamp write -> query resolve ordering is inferred through the query range. Query resolve -> readback copy ordering is inferred through the destination buffer range.

Explicit order is fragment-local, typed, and only for non-data constraints. A redundant explicit edge duplicating an inferred edge, contradiction with inferred order, or a cycle is an error.

### Work and graph shape

`GpuWorkFragment` is immutable after closure-scoped construction. It contains resources, imports, exports, nodes, explicit non-data edges, outputs, and provenance.

G3 nodes are immutable pre-admission operation intent. They contain node kind, exact accesses, derived and caller-declared capability requirements, backend-neutral operation shape, execution preference, label, and provenance. Render shader/pipeline payload remains in a temporary render-owned sidecar keyed by prepared node ID. G4 replaces that seam with admitted generic program/interface authority.

Initial node kinds remain:

```text
Compute
Render
Copy
Clear
Resolve
Present
```

`Resolve` is query-set-to-buffer work. Multisample texture resolution is an optional relation on a render color attachment.

The single advanced authority is `GpuPreparedWorkGraph::prepare(...)`. It composes immutable fragments, validates and normalizes accesses and operation shape, derives capability requirements, tracks initialized coverage, infers edges, incorporates only non-redundant explicit non-data order, rejects missing causality/ambiguity/cycles, and produces deterministic prepared node order plus structured diagnostics. It performs no backend admission or execution.

G5 ordinary submission must call the same preparation authority internally.

### Identity

`GpuWorkNodeId` is private-construction, nonzero, and fragment-local. Composition deterministically creates `GpuPreparedWorkNodeId { fragment_ordinal, local_node }`.

Node identities are process-local diagnostics and typed references, not persistence, replay, network, wire, ABI, or cache formats. They do not reuse `GpuWorkResourceId`.

Cross-fragment explicit node edges are deferred. Current render passes requiring explicit non-data order lower into one fragment; independent fragments compose through resources and imports/exports.

## Module and adapter boundary

Future-transferable source:

```text
engine/src/plugins/gpu/api/access.rs
engine/src/plugins/gpu/api/work.rs
engine/src/plugins/gpu/api/graph.rs
engine/src/plugins/gpu/api/errors.rs
engine/src/plugins/gpu/api/resource.rs
engine/src/plugins/gpu/api/mod.rs
engine/src/plugins/gpu/mod.rs
```

Temporary integration:

```text
engine/src/plugins/render/adapters/gpu_work.rs
```

This is a fourth temporary render adapter. It owns translation only, keeps render execution and timing presentation payload outside RunenGPU, and is deleted incrementally through G4-G6.

## Consumer inventory to reverify at implementation start

Direct generic/render graph consumers:

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
engine/src/plugins/render/renderer/render_flow/gpu_timing.rs
```

Transitive consumers include render runtime/preflight/inspection, attachment construction, fragment composition, procedural code, boids, Game of Life, SDF flows, compositor examples, editor/draw applications, render-flow tests, primitive tests, timing evidence, and the planning benchmark. The implementation issue must bind an exact current-main file inventory before source changes.

## Required deletion target

```text
CompiledResourceAccessKind
CompiledResourceLifetimeWindow
compile_resource_lifetime_windows
diagnose_resource_lifetime_windows
renderer-local generic topological sorting as final authority
GpuPrimitiveResourceAccessKind
GpuPrimitiveResourceAccess
GpuPrimitiveDispatchResource::Temporary(String)
string primitive stage dependencies
generic RenderPassNode depends_on authority
broad renderer access/lifetime correctness as final authority
```

Render-only shape, execution payload, timing decoding, and diagnostics presentation may remain until their owner phases, but they must consume the one prepared G3 graph.

## Alternatives rejected

- Keep explicit pass dependencies: duplicates resource knowledge and remains renderer-shaped.
- Preserve a duplicate explicit edge alongside an inferred edge: violates the non-data-only rule and leaves two dependency authorities.
- Infer cross-fragment read/write direction without import/export causality: silently chooses between consuming initial contents and consuming another fragment's output.
- Model multisample texture resolve as standalone work: not realizable through the accepted WGPU backend, where resolve is a render color-attachment relation.
- Treat query resolution as `CopyDestination`: incorrect because WGPU requires dedicated query-resolve usage.
- Defer query resolution entirely to G5: leaves current timestamp work absent from the G3 graph and cannot infer write -> resolve -> copy dependencies.
- Put polling, mapping, or timing conversion in G3: crosses into G5 execution and Runenwerk presentation ownership.
- Require callers to restate operation-implied capabilities: creates duplicate authority and permits contradictory work/requirement combinations.
- Claim runtime use-after-retirement validation in G3: no context generation, backend registry, submission, or retirement state exists yet.
- Whole-resource inference only: over-serializes disjoint work and cannot validate partial initialization.
- Treat input array order as cross-fragment semantics: hides writer ambiguity and couples scheduling to collection order.
- Mandatory public graph DSL: violates progressive disclosure.
- Backend barriers in G3: belongs to G4/G5 translation and execution.
- Aliasing, pass fusion, or multi-queue scheduling: premature optimization without correctness evidence.

## Stable-format and dependency audit

No current evidence makes G3 ranges, nodes, edges, graphs, render attachment operations, query resolve operations, capability derivation, or prepared diagnostics persisted, replay, network, wire, cache, or external formats. Inspection output is process-local.

No dependency, package, workflow, lockfile, raw WGPU public type, ECS type, renderer semantic type, Winit type, SDF/UI/product type, or codec is required.

## Conclusion

Proceed with the decision-complete G3 specification and one documentation-only planning PR. Do not create the implementation issue until this planning authority is independently reviewed and merged. Issue `#174` does not authorize Rust implementation.
