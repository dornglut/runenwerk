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

G3 can proceed as a documentation and specification phase without a new ADR, external package, dependency, or premature G4/G5/G7 implementation.

```text
repository: dornglut/runenwerk
branch inspected: main
baseline: 709aa6aced020ee99405e1e1c3dde7703c77a4d4
planning issue: #174
planning branch: docs/runengpu-g3-access-work-graph
```

G2 is accepted through issue `#172` and merged PR `#173`. G3 extends its normalized descriptors, typed resource handles, initialization declarations, and export relationships. It must not recreate resource or backend authority.

The current renderer contains useful correctness evidence, but generic access is split across broad pass reads/writes, role-specific pass fields, whole-resource lifetime windows, explicit dependency lists, and a second GPU-primitive plan. Buffer ranges, texture subresources, partial initialization, normalized texture-view overlap, and cross-fragment causality are absent.

## Ownership boundary

G3 owns:

- checked work-time buffer ranges and texture/query subranges;
- graph-entry initialization evidence and region-aware initialized coverage;
- overlap, hazards, inferred data dependencies, and explicit non-data order;
- immutable generic work fragments and nodes;
- deterministic preparation and inspection facts.

G3 does not own shader files, pipeline/binding admission, WGPU objects, realization, encoding, submission, completion, readback, retirement, surfaces, renderer meaning, ECS projection, fixed-time scheduling, or product policy.

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

### Whole-resource lifetime windows

`engine/src/plugins/render/graph/resource_lifetimes.rs` records whole-resource first/last read/write and reports only transient read-before-write.

Useful invariant: uninitialized transient content cannot be read.

Missing authority:

- byte and subresource coverage;
- partial initialization;
- RAW, WAR, and WAW causes;
- texture-view parent normalization;
- cross-fragment composition;
- order inference.

Disposition: replace it with prepared-graph initialized-coverage and dependency summaries and delete the file in the G3 implementation slice.

### Compiled render planning

`engine/src/plugins/render/graph/planning.rs` embeds renderer resources, explicit pass order, execution data, lifetime windows, and diagnostics.

Disposition: a temporary adapter produces one `GpuPreparedWorkGraph`; the render plan maps prepared node order to render-owned execution payloads. No second lifetime/hazard result remains.

### GPU primitives

`engine/src/plugins/render/gpu_primitives/plan.rs` defines a duplicate generic access model, string temporary-resource references, string stage dependencies, and sequential stage chaining.

The prefix-scan hierarchy proves that typed transient resources and inferred multi-stage dependencies are required outside rendering.

Disposition: primitive descriptors remain source conveniences but lower to G3 typed work. Delete primitive-local access types, `Temporary(String)`, and string stage dependency authority. Shader asset discovery remains outside RunenGPU until G4 program admission.

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

Buffer access distinguishes uniform, storage read/write/read-write, vertex, index, indirect, and copy source/destination.

Texture access distinguishes sampled, storage read/write/read-write, copy source/destination, color attachment, depth/stencil attachment, and present.

Attachment semantics require separate load and store facts:

```text
load:  Load | Clear
store: Store | Discard
```

`Load` requires initialized coverage. `Clear` establishes full initialized coverage for the attachment subresources. `Store` preserves post-node coverage. `Discard` removes later readable coverage. Discard is not a load mode.

### Initialization flow

- `Zeroed` and complete G2 `Prepared` initialization begin initialized.
- `Uninitialized` begins with no initialized coverage.
- pure writes and copy destinations initialize only covered regions;
- read-write requires prior initialized coverage;
- attachment `Load` requires prior coverage;
- attachment `Clear` establishes coverage before work;
- attachment `Discard` invalidates post-node readable coverage;
- imported and retained prior-epoch contents require explicit graph-entry evidence;
- G3 validates evidence but does not claim G5 actually preserved, uploaded, synchronized, or retired content.

### Hazard rules

For overlapping normalized regions:

| Earlier | Later | Result |
|---|---|---|
| read | read | no edge |
| write | read | RAW edge |
| read | write | WAR edge |
| write | write | WAW edge only with unique producer order; otherwise error |
| read-write | overlap | acts as both read and write |

Disjoint byte ranges and disjoint mip/layer/aspect ranges do not conflict.

Within one fragment, lexical node order orients data hazards. Callers do not restate those edges.

Fragment collection position is not semantic scheduling authority. Cross-fragment causality requires the same typed resource plus a matching typed import/export relation. Overlapping cross-fragment writers without one unique producer are rejected as ambiguous.

Explicit order is fragment-local, typed, and only for non-data constraints. Contradiction with inferred order or a cycle is an error.

### Work and graph shape

`GpuWorkFragment` is immutable after closure-scoped construction. It contains resources, imports, exports, nodes, explicit non-data edges, outputs, and provenance.

G3 nodes are immutable pre-admission operation intent. They contain node kind, exact accesses, capability requirements, backend-neutral operation shape, execution preference, label, and provenance. Render shader/pipeline/draw payload remains in a temporary render-owned sidecar keyed by prepared node ID. G4 replaces that seam with admitted generic program/interface authority.

Initial node kinds remain:

```text
Compute
Render
Copy
Clear
Resolve
Present
```

The single advanced authority is `GpuPreparedWorkGraph::prepare(...)`. It composes immutable fragments, validates and normalizes accesses, tracks initialized coverage, infers edges, incorporates explicit non-data order, rejects ambiguity/cycles, and produces deterministic prepared node order plus structured diagnostics. It performs no backend admission or execution.

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

This is a fourth temporary render adapter. It owns translation only, keeps render execution payload outside RunenGPU, and is deleted incrementally through G4-G6.

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
```

Transitive consumers include render runtime/preflight/inspection, fragment composition, procedural code, boids, Game of Life, SDF flows, compositor examples, editor/draw applications, render-flow tests, primitive tests, and the planning benchmark. The implementation issue must bind an exact current-main file inventory before source changes.

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

Render-only shape and execution payload may remain until their owner phases, but they must consume the one prepared G3 graph.

## Alternatives rejected

- Keep explicit pass dependencies: duplicates resource knowledge and remains renderer-shaped.
- Whole-resource inference only: over-serializes disjoint work and cannot validate partial initialization.
- Treat input array order as cross-fragment semantics: hides writer ambiguity and couples scheduling to collection order.
- Mandatory public graph DSL: violates progressive disclosure.
- Backend barriers in G3: belongs to G4/G5 translation and execution.
- Aliasing, pass fusion, or multi-queue scheduling: premature optimization without correctness evidence.

## Stable-format and dependency audit

No current evidence makes G3 ranges, nodes, edges, or prepared graphs persisted, replay, network, wire, cache, or external formats. Inspection output is process-local.

No dependency, package, workflow, lockfile, raw WGPU public type, ECS type, renderer semantic type, Winit type, SDF/UI/product type, or codec is required.

## Conclusion

Proceed with the decision-complete G3 specification and one documentation-only planning PR. Do not create the implementation issue until this planning authority is independently reviewed and merged. Issue `#174` does not authorize Rust implementation.