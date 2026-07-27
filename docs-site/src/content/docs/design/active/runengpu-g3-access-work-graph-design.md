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
```

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
ColorAttachment { load, store }
DepthStencilAttachment { access, load, store }
Present
```

Attachment operations separate:

```text
load:  Load | Clear
store: Store | Discard
```

`Load` reads prior content. `Clear` establishes complete initialized attachment coverage. `Store` preserves post-node coverage. `Discard` removes later readable coverage.

### Queries and samplers

Timestamp query use has a checked query-index range. Samplers are immutable input evidence and do not create data hazards by themselves.

## Initialization flow

Graph-time initialization is region-aware:

```text
Zeroed descriptor         -> complete initialized coverage
Prepared descriptor       -> checked prepared coverage
Uninitialized descriptor  -> no initialized coverage
pure write                -> initialize written coverage
copy destination          -> initialize destination coverage
read-write                -> require prior coverage, then preserve/write it
attachment Load           -> require prior coverage
attachment Clear          -> establish full attachment coverage
attachment Store          -> preserve coverage
attachment Discard        -> remove later readable coverage
```

Imported or retained prior-epoch content enters only through explicit `GpuWorkResourceInput` evidence. Lifetime, labels, or the presence of a current runtime allocation never imply initialized content.

G3 validates the evidence. G5 later proves whether execution actually uploaded, preserved, synchronized, completed, or retired it.

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

Overlapping cross-fragment writers without one unique producer are rejected as ambiguous rather than ordered by array position.

## Same-node access

The authoring boundary normalizes repeated access declarations:

- duplicate identical access is deduplicated;
- compatible storage read plus storage write over the same range becomes read-write;
- disjoint accesses remain separate;
- incompatible overlapping roles fail, including sampled read plus attachment write or overlapping copy source/destination on the same resource unless a later accepted operation proves legality.

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

G3 nodes are pre-admission work intent. They include operation kind, exact access, capability requirements, backend-neutral operation shape, execution preference, label, and provenance.

Current render shader/pipeline/draw payload remains in a temporary render-owned sidecar keyed by prepared node identity. G4 replaces this seam with admitted generic shader/pipeline/interface authority. The sidecar cannot alter G3 hazard truth.

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

It is only for constraints not representable by data access. An explicit edge that opposes an inferred data edge or creates a cycle fails.

Cross-fragment explicit node edges are deferred. Existing render passes needing explicit non-data order lower into one fragment. Independent fragments compose through resources and exports.

## Prepared graph

`GpuPreparedWorkGraph::prepare(...)` is the single advanced authority. There is no public mutable graph and no reduced validator.

Preparation:

1. accepts immutable fragments;
2. validates identities, descriptors, usages, ranges, view parents, queries, and imports/exports;
3. normalizes access;
4. derives initial coverage;
5. infers RAW/WAR/WAW edges;
6. adds explicit non-data edges;
7. rejects ambiguity, conflict, and cycles;
8. produces deterministic prepared IDs and topological order;
9. publishes normalized access, edges with typed causes, coverage summaries, exports, diagnostics, and provenance.

Independent ready nodes are ordered deterministically for inspection without promising concurrent or parallel execution.

Preparation performs no backend admission or execution. G5 ordinary submission must invoke the same preparation authority internally.

## Render and GPU-primitive adapter

One temporary adapter is added:

```text
engine/src/plugins/render/adapters/gpu_work.rs
```

It:

- lowers current render role fields into exact G3 access;
- maps current whole-resource authoring to checked whole ranges only where no narrower fact exists;
- translates history/import/runtime-entry assumptions into explicit input evidence;
- groups passes requiring explicit non-data order into one fragment;
- maps prepared node order back to render-owned execution payload;
- lowers primitive temporary storage to typed G2 transient resources;
- removes primitive string resource/dependency authority.

It does not move renderer semantics, shader paths, fixed-time policy, UI/product meaning, or WGPU behavior into RunenGPU.

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
- encoding, submission, upload/update, completion, readback, cancellation, and retirement;
- surface acquisition and presentation execution;
- extraction or a new package;
- aliasing, pass fusion, multi-queue scheduling, or graph visualization.

## Acceptance

G3 planning is accepted only when issue `#174`, its investigation, this design, and the implementation specification are independently reviewed and merged with canonical validation green.

Only then may one bounded G3 implementation issue be created. The planning issue itself authorizes no Rust implementation.