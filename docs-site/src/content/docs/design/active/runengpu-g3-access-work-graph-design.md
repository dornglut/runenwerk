---
title: RunenGPU G3 Access and Work Graph Design
description: Canonical phase design for checked resource access, initialization flow, hazards, immutable GPU work fragments, and deterministic graph preparation.
status: active
owner: gpu
layer: framework/gpu
canonical: true
last_reviewed: 2026-07-27
related_docs:
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ./runengpu-architecture-design.md
  - ./runenrender-internal-decomposition-execution-plan.md
  - ../../reports/investigations/runengpu-g3-access-work-graph-investigation.md
  - ../../workspace/specs/pt-runengpu-g3-access-work-graph.ron
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
---

# RunenGPU G3 Access and Work Graph Design

## Status

```text
G2 logical resources and prepared data     accepted
G3 decision phase                          active through issue #174
G3 Rust implementation                    not authorized
G4-G7                                      deferred
external runen-gpu package                 not authorized
```

This document binds G3 architecture. The implementation specification binds exact modules, types, migration, deletion, tests, validation, and stop conditions.

## Mission

G3 answers:

> Given immutable GPU work contributed by independent consumers, which precise resource regions are read or written, which content is initialized, which nodes depend on which others, and is the resulting work valid and deterministic before a backend exists?

G3 does not create a GPU context, realize WGPU objects, admit shaders or pipelines, encode commands, submit work, preserve execution state, read back data, retire resources, or present surfaces.

## Public experience

Ordinary consumers contribute immutable work through lexical builders:

```rust
let simulation = GpuWorkFragment::build("simulation.update", |work| {
    work.compute("integrate", |node| {
        node.storage_read(&positions, GpuBufferRange::whole(&positions)?)?;
        node.storage_write(&next_positions, GpuBufferRange::whole(&next_positions)?)?;
        node.dispatch([groups, 1, 1])?;
        Ok(())
    })?;
    Ok(())
})?;
```

The advanced path prepares the same authority later used by ordinary submission:

```rust
let prepared = GpuPreparedWorkGraph::prepare(
    "frame 42",
    [simulation, rendering],
)?;
inspect(prepared.diagnostics());
```

Required qualities:

- no mandatory graph ceremony for domain facades;
- no nested `finish()` ladders;
- strings are labels or diagnostic reasons, not resource/node/dependency authority;
- resources and nodes use typed identities;
- data edges are inferred from access;
- explicit order is limited to genuine non-data constraints;
- one preparation/validation authority serves simple and inspectable paths.

## Access model

### Buffers

`GpuBufferRange` is a concrete checked half-open byte interval. Whole-buffer convenience resolves immediately against the descriptor. Partial construction rejects zero size, overflow, and out-of-bounds coverage.

Buffer access categories are explicit:

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
QueryResolveDestination
```

`QueryResolveDestination` is distinct from `CopyDestination`. G3 extends the accepted G2 buffer-usage vocabulary with normalized `GpuBufferUsage::QueryResolve` because the current timestamp consumer resolves opaque query data into a buffer before an ordinary copy/readback path.

### Textures and views

Work-time texture access uses `GpuTextureSubresourceRange` for mip, layer, and aspect coverage.

A texture-view access is normalized to:

```text
parent GpuTextureHandle
    + intersection(view range, requested range)
```

Views are not independent storage for hazards or initialization.

Texture categories are explicit:

```text
SampledRead
StorageRead
StorageWrite
StorageReadWrite
CopySource
CopyDestination
ColorAttachment { load_kind, store }
MultisampleResolveDestination
DepthStencilAttachment { access, load_kind, store }
Present
```

Attachment operation values remain in the render operation rather than the access enum:

```text
color load
    Load
    Clear(GpuColorClearValue)

depth load
    Load
    Clear(GpuDepthClearValue)

store
    Store
    Discard
```

The derived access records only `Load` versus `Clear`, because the clear value does not change hazards or initialization coverage. `Load` reads prior content. `Clear(value)` establishes complete initialized attachment coverage with an exact canonical value. `Store` preserves post-node coverage. `Discard` removes later readable coverage.

The accepted G2 format vocabulary currently has color formats and `Depth32Float`, but no stencil format. G3 therefore defines no stencil load, clear value, access mode, or standalone stencil clear authority.

Multisample texture resolution is not standalone work. A color attachment may name an optional single-sampled resolve destination as part of the same `GpuRenderOperation`. The render operation derives the source color-attachment access and destination `MultisampleResolveDestination` write. The resolve destination is written regardless of source attachment store policy.

### Queries and samplers

Timestamp query use has a checked query-index range.

```text
WriteTimestamp
    writes and initializes exact query indices

ResolveSource
    reads initialized query indices through a typed query-set resolve operation
```

A query-set resolve writes an exact destination buffer byte range. Timestamp results occupy one `u64` per query, so the logical range size is checked as `query_count * 8`. G3 validates count, overflow, destination bounds, `QueryResolve` usage, and source initialization. G4/G5 validate backend-specific offset alignment and encode the operation.

Samplers are immutable input evidence and do not create data hazards by themselves.

## Initialization flow

Graph-time initialization is region-aware:

```text
Zeroed descriptor                 -> complete initialized coverage
Prepared descriptor               -> checked prepared coverage
Uninitialized descriptor          -> no initialized coverage
pure write                        -> initialize written coverage
buffer copy destination           -> initialize destination coverage
standalone buffer zero            -> initialize exact destination bytes
query timestamp write             -> initialize written query indices
query resolve destination         -> initialize exact destination buffer bytes
attachment Load                   -> require prior attachment coverage
attachment Clear(value)           -> establish attachment coverage
attachment write/draw             -> preserve/write attachment coverage
multisample resolve destination   -> establish destination coverage
attachment Store                  -> preserve source attachment coverage
attachment Discard                -> remove source attachment coverage
read-write                        -> require prior coverage, then preserve/write it
```

Query sets begin with no initialized indices unless explicit graph-entry evidence exists. `ResolveSource` requires initialized coverage established by accepted timestamp writes or explicit input evidence.

Imported or retained prior-epoch content enters only through explicit `GpuWorkResourceInput` evidence. Lifetime, labels, or the presence of a current runtime allocation never imply initialized content.

G3 validates graph-time evidence. G5 later proves whether execution actually uploaded, preserved, synchronized, completed, resolved, copied, or retired backend state.

## Overlap and hazards

Overlap is evaluated on normalized storage:

- buffer byte intervals;
- parent texture mip/layer/aspect ranges;
- query index ranges.

For overlapping regions:

| Earlier access | Later access | Dependency |
|---|---|---|
| read | read | none |
| write | read | read-after-write |
| read | write | write-after-read |
| write | write | write-after-write when producer order is unique |
| read-write | any | both read and write rules |

Disjoint ranges do not conflict.

Within one fragment, lexical node declaration order orients access-derived hazards. This is not a second manual dependency API.

Fragment collection order is not semantic scheduling authority. Cross-fragment causality requires:

```text
shared typed resource
    + matching typed export/import relation
```

Any conflicting cross-fragment access with at least one write and no matching producer/consumer relation is rejected as missing causality. This includes read/write and write/write conflicts; preparation never guesses whether an unordered read should occur before or after a write.

Overlapping cross-fragment writers with more than one possible producer are rejected as ambiguous.

A timestamp write followed by query resolution produces a data dependency through the query-set range. Query resolution followed by a buffer copy produces a data dependency through the resolved destination byte range.

## Same-node access

The authoring boundary normalizes repeated access declarations:

- duplicate identical access is deduplicated;
- compatible storage read plus storage write over the same range becomes read-write;
- disjoint accesses remain separate;
- incompatible overlapping roles fail, including sampled read plus attachment write, source attachment aliasing its resolve destination, or overlapping copy source/destination on the same resource unless a later accepted operation proves legality.

## Immutable work

A `GpuWorkFragment` contains:

```text
declared resources
resource inputs/imports
resource outputs/exports
immutable nodes
explicit non-data order
provenance
```

Initial node kinds:

```text
Compute
Render
Copy
Clear
Resolve
Present
```

`Clear` initially means checked buffer-zero work only. Arbitrary color/depth clear values belong to render attachment load operations. A general standalone texture-clear node is deferred because the accepted backend exposes zero-only texture clearing behind separate capability pressure and no current consumer requires it.

`Resolve` is standalone query-set-to-buffer resolution. Multisample texture resolution belongs to a `Render` node's color-attachment operation because WGPU/WebGPU resolves the multisample attachment into its resolve target as part of that render pass.

`GpuRenderOperation` owns exact ordered color-attachment operations, an optional depth attachment operation, ordered draw intents, and render-side query writes. Attachment operation constructors derive mandatory access facts; callers do not separately restate attachment reads/writes.

G3 nodes are pre-admission work intent. They include operation kind, exact access, capability requirements, backend-neutral operation shape, execution preference, label, and provenance.

Current render shader/pipeline payload remains in a temporary render-owned sidecar keyed by prepared node identity. G4 replaces this seam with admitted generic shader/pipeline/interface authority. The sidecar cannot alter G3 hazard truth.

An empty render draw list is valid only when an attachment uses `Clear(value)` or render-side query writes make the pass meaningful. `Store` alone preserves content and is not work.

## Capability requirements

Operations and accesses derive the normalized requirements they structurally need:

```text
Compute operation                    -> Compute
Render operation                     -> RenderPipeline
Copy or buffer-zero Clear            -> Copy
Indirect draw                        -> IndirectDraw
Storage texture access               -> StorageTexture
Depth attachment                     -> DepthAttachment
Timestamp write or query resolution  -> TimestampQuery
Present                              -> Presentation
```

Consumers may add semantic requirements not inferable from operation shape. Preparation merges caller-declared and derived requirements through the accepted G2 requirement authority. A caller cannot neutralize an operation-implied requirement with `Disabled`; conflicts fail deterministically. G4 performs actual capability admission.

## Identity

`GpuWorkNodeId` is fragment-local, nonzero, and privately allocated by the fragment builder.

Composition produces:

```text
GpuPreparedWorkNodeId {
    fragment_ordinal,
    local_node,
}
```

Node identities are process-local typed references and diagnostics. They are not stable persistence, replay, network, wire, ABI, or cache values and do not reuse `GpuWorkResourceId`.

The existing `RenderFlowId`-derived resource-owner bridge remains exactly one crate-private adapter seam during G3. G4 context/work-scope authority must delete it. G3 does not introduce a global mutable context or public owner-scope constructor.

## Imports and exports

`GpuExportKey` is a validated semantic newtype, not a diagnostic label.

A fragment export binds:

```text
typed resource
export key
required final access
final initialized coverage
provenance
```

A consumer import binds the same typed resource and export key plus required initial access.

Preparation rejects duplicate export keys, kind mismatch, access mismatch, coverage mismatch, multiple producers, and unbound imports.

## Explicit order

`GpuExplicitOrder` is fragment-local and references typed node IDs. A non-empty reason is required for diagnostics.

It is only for constraints not representable by data access. An explicit edge that duplicates an inferred data edge is rejected as redundant; the correction is to remove the explicit edge and rely on typed access. An explicit edge that opposes an inferred data edge or creates a cycle also fails.

Cross-fragment explicit node edges are deferred. Existing render passes needing explicit non-data order lower into one fragment. Independent fragments compose through resources and exports.

## Prepared graph

`GpuPreparedWorkGraph::prepare(...)` is the single advanced authority. There is no public mutable graph and no reduced validator.

Preparation:

1. accepts immutable fragments;
2. validates identities, descriptors, usages, ranges, view parents, attachments, clear values, queries, query resolves, and imports/exports;
3. normalizes operation-derived and caller-declared access;
4. derives initial coverage and merged capability requirements;
5. infers RAW/WAR/WAW edges;
6. adds non-redundant explicit non-data edges;
7. rejects missing cross-fragment causality, ambiguity, conflict, redundant explicit order, and cycles;
8. produces deterministic prepared IDs and topological order;
9. publishes normalized access, edges with typed causes, coverage summaries, requirements, exports, diagnostics, and provenance.

Independent ready nodes are ordered deterministically for inspection without promising concurrent or parallel execution.

Preparation performs no context admission, backend realization, command encoding, submission, runtime retirement check, or surface action. G5 ordinary submission must invoke the same preparation authority internally; G4/G5 reject stale-generation or retired backend values at admission/submission time.

## Render and GPU-primitive adapter

One temporary adapter is added:

```text
engine/src/plugins/render/adapters/gpu_work.rs
```

It:

- lowers current render role and attachment fields into exact G3 operations/access;
- maps current attachment clear colors/depth values into canonical render attachment load operations;
- maps current whole-resource authoring to checked whole ranges only where no narrower fact exists;
- translates history/import/runtime-entry assumptions into explicit input evidence;
- maps current pass timestamp writes to exact query-index writes;
- emits a typed query-set resolve operation and exact destination buffer range for current timing resolution;
- maps the later resolve-buffer-to-readback copy through ordinary typed buffer-copy work;
- groups passes requiring explicit non-data order into one fragment;
- maps prepared node order back to render-owned execution payload;
- lowers primitive temporary storage to typed G2 transient resources;
- removes primitive string resource/dependency authority.

It does not move renderer semantics, shader paths, fixed-time policy, UI/product meaning, timing presentation/readback decoding, or WGPU behavior into RunenGPU.

## Deletion boundary

The G3 implementation deletes or removes as generic authority:

```text
CompiledResourceAccessKind
CompiledResourceLifetimeWindow
compile_resource_lifetime_windows
diagnose_resource_lifetime_windows
renderer-local generic topological ordering
GpuPrimitiveResourceAccessKind
GpuPrimitiveResourceAccess
GpuPrimitiveDispatchResource::Temporary(String)
string primitive stage dependencies
generic depends_on authority
```

Render-only shape and execution payload may remain until their owner phases, but they consume the one G3 prepared graph.

## Phase boundaries

G3 stops before:

- context/device/backend admission;
- shader/pipeline/interface and binding-layout authority;
- WGPU resource realization and barriers;
- encoding, submission, upload/update, completion, readback, cancellation, and backend retirement;
- stale-generation and runtime-retirement admission;
- backend query-resolve offset alignment and command encoding;
- standalone texture-clear capability/realization;
- surface acquisition and presentation execution;
- extraction or a new package;
- aliasing, pass fusion, multi-queue scheduling, or graph visualization.

## Acceptance

G3 planning is accepted only when issue `#174`, its investigation, this design, and the implementation specification are independently reviewed and merged with canonical validation green.

Only then may one bounded G3 implementation issue be created. The planning issue itself authorizes no Rust implementation.
