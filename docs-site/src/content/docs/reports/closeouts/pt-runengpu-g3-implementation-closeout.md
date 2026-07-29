---
title: PT-RUNENGPU-G3 Access and Work Graph Implementation Closeout
description: Accepted implementation, migration, deletion, adapter, sidecar, validation, review, and remaining-phase evidence for the bounded RunenGPU G3 cutover.
status: completed
owner: workspace
layer: reports
canonical: true
last_reviewed: 2026-07-29
related_docs:
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runengpu-g3-access-work-graph-design.md
  - ../../design/active/runengpu-g4-context-program-realization-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../reports/investigations/runengpu-g3-access-work-graph-investigation.md
  - ../../reports/investigations/runengpu-g4-context-program-realization-investigation.md
  - ../../workspace/specs/pt-runengpu-g3-access-work-graph.ron
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/completed-work.md
  - ../../workspace/planning/roadmap.md
  - ../../engine/reference/plugins/render/public-api-reference.md
  - ../../engine/reference/plugins/render/render-flow-usage-guide.md
---

# PT-RUNENGPU-G3 Access and Work Graph Implementation Closeout

## Outcome

Issue `#177` / PR `#181` implemented and accepted the bounded RunenGPU G3
access/work-graph slice. The accepted squash merge is:

```text
39d6fe65a334502bdfba0b1a2ce3b365099fcf28
```

The implementation creates one future-transferable authority under
`engine::plugins::gpu` for:

- checked resource access and canonical regions;
- typed immutable GPU operations;
- region-aware graph-entry initialization;
- exact RAW/WAR/WAW hazards;
- typed cross-fragment causality;
- operation-derived requirements;
- immutable work fragments and nodes;
- deterministic prepared graphs.

Current render, timing, GPU-primitive, inspection, application, example, benchmark,
and test consumers were migrated through one temporary lowering adapter. Renderer
execution payload remains in a private sidecar over prepared-node identity and cannot
alter G3 truth. The replaced renderer-owned generic lifetime/access/topological
authority was deleted without aliases, forwarding modules, or a second graph.

The accepted slice created no external package and implemented no G4 backend admission
or realization, G5 GPU-owned execution/readback/retirement, or G7
surface/loss/reconstruction behavior.

## Revisions and acceptance

| Evidence | Revision or state |
|---|---|
| Parent program | issue `#167` |
| Owning implementation issue | issue `#177` |
| Implementation pull request | PR `#181` |
| Branch | `codex/runengpu-g3-access-work-graph` |
| Previous candidate base | `90d24abb93bff4b1d3f5b4743056bc00ff80d4b6` |
| Baseline repair | issue `#179` / PR `#180`, merge `1c645b2bbfcece44dd6ae151cc97559793afa2c2` |
| Accepted implementation base | `1c645b2bbfcece44dd6ae151cc97559793afa2c2` |
| Access-contract commit | `5f132c5c8d488d129057658dd40a275273f5bdb5` |
| Operation-contract commit | `2baed155c51869026376d5a2a25ed6e8041d38e5` |
| Prepared-graph commit | `9d52f7a1c95ee2aa78fa6b53689a80b83a6ea4a7` |
| Render cutover commit | `0c950b244cea799661468d4774d485b5fc2b5984` |
| First reviewed head | `38abac6bd234d9db3a4544aedbf2dba149538e36` |
| Semantic graph decomposition | `d78df27e769f65b1a3af6ce5e02d3ea053b342ea` |
| Dependency and authoring correction | `905c506e33202405d1bea8c160a05ac92c326c43` |
| Accepted squash merge | `39d6fe65a334502bdfba0b1a2ce3b365099fcf28` |
| Accepted current main after validation maintenance | `6bbd341691a34763ef54c8ca059940cac8981265` |

The commit after the accepted G3 merge changes only verified-head validation and
workflow authority. It changes no G3 semantics, RunenGPU/render source, dependency,
manifest, or lockfile.

## Public G3 authority

### Checked access and canonical regions

`engine::plugins::gpu` owns:

- `GpuBufferRange`, `GpuQueryRange`, and checked texture-subresource coverage;
- texture/view normalization to kind-preserving parent storage;
- explicit buffer, texture, query, and sampler access records;
- exact buffer-byte, texture mip/layer/aspect, and query-index overlap;
- D3 whole-mip-volume behavior;
- descriptor-usage validation, including distinct query-resolve destination authority;
- semantic equality, ordering, and hashing excluding labels and provenance.

Ranges and normalized access cannot be field-constructed or changed after validation.
Texture views do not become independent hazard storage.

### Typed operations

One `GpuWorkOperation` authority owns exact immutable shape for:

- compute dispatch;
- render color/depth attachments, canonical clear values, direct/indexed/indirect draw
  intents, multisample resolve targets, and timestamp writes;
- all four buffer/texture copy directions with checked logical coverage;
- exact-range buffer zero;
- query-set-to-buffer resolution;
- logical presentation intent.

Operations derive mandatory access and capability requirements. Callers may add
semantic requirements but cannot contradict or disable an operation-implied fact.
Standalone clear remains buffer zero only. Multisample resolve remains a render
attachment relationship.

### Immutable work and preparation

The accepted authority includes:

- private fragment-local `GpuWorkNodeId` and process-local
  `GpuPreparedWorkNodeId` values;
- typed inputs, imports, outputs, exports, and semantic export keys;
- explicit fragment-local non-data order only;
- lexical compute authoring and advanced checked-node authoring;
- immutable work fragments and nodes;
- exact dependency regions and reasons;
- prepared initialization summaries, requirements, outputs, diagnostics, and order;
- one immutable `GpuPreparedWorkGraph::prepare(...)` authority.

Preparation validates resource identity and kind, operation/access consistency, exact
initial coverage, imports/exports, requirements, same-node normalization, RAW/WAR/WAW
hazards, missing or ambiguous cross-fragment causality, redundant or opposing explicit
edges, and cycles. It emits deterministic semantic dependencies, regions,
requirements, initialization, and topological order.

Fragment collection order may affect diagnostic ordinals only. It cannot change
semantic edges or error outcomes.

### Ordinary authoring path

The accepted ordinary compute path is lexical and closure-scoped:

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

The builder stages mutation and commits only on success, so closure or downstream
validation failure cannot leave partial resources or consume node identity. The
advanced primitive and ordinary path converge on the same operation, access,
requirement, and execution-preference authority.

Other operation families retain their concrete checked operation types and advanced
`add_node(...)` path until a concrete ordinary contract is separately justified.

## Independent review corrections

The first independent review of
`38abac6bd234d9db3a4544aedbf2dba149538e36` found four merge-blocking issues. The
accepted correction addressed all four:

1. Duplicate resource declaration became transactional. An occupied entry returns
   `DuplicateResource` without replacing the original descriptor, and authoring may
   continue safely after the error.
2. RAW/WAR/WAW dependencies retain exact resource identity plus exact buffer, texture,
   or query intersection regions. Samplers create no data hazard and multiple regions
   remain distinct.
3. The accepted lexical `GpuWorkFragment::build` / `work.compute` path was restored
   without creating a second validation authority.
4. The 183,145-byte monolithic graph module was decomposed into semantic owner-focused
   modules. No numbered split, forwarding layer, duplicate preparation,
   initialization, or hazard implementation was added.

The complete fallible-mutation audit found no remaining G3 violation. Resource,
input/import/output/order/node/requirement/identity and sidecar mutation performs
fallible checks before reusable state changes. Preparation-local maps remain scratch
state discarded on failure.

Every authored graph file remained below the repository ceiling of 131,072 raw bytes.
The accepted module split preserved one public graph root and semantic private modules.

## Render migration and temporary adapter

Exactly one lowering seam exists:

```text
engine/src/plugins/render/adapters/gpu_work.rs
```

It lowers current render passes, attachments, clear values, projected dispatch,
timestamp writes, query resolution, readback-copy intent, and typed primitive
temporaries into checked G3 work. Whole-resource access is used only where the current
render declaration has no narrower fact. History and current-runtime assumptions become
explicit initialization evidence.

`RenderPassNode` retains render shape and payload inputs but no longer owns generic
reads, writes, or broad dependency correctness. Fragment-local non-data order is the
only retained authoring edge. Renderer execution, transient planning, capability
validation, and inspection consume the exact prepared graph.

The adapter is temporary. G4B removes program/interface/pipeline descriptor truth.
G4C removes WGPU realization/cache truth, synthetic logical handles, and the
`RenderFlowId`-derived owner bridge. G5 removes the residual execution payload. G7
removes the temporary surface compatibility seam.

The adapter is not a compatibility facade and contains no second hazard,
initialization, requirement, or scheduling implementation.

## Temporary execution sidecar

`RenderGpuWorkSidecar` is private and keyed only by `GpuPreparedWorkNodeId`. Labels,
pass indices, resource IDs, and strings are not lookup authority.

Accepted invariants:

1. every prepared node has exactly one payload and every payload names one prepared
   node;
2. duplicate insertion is a structured error and leaves the original map unchanged;
3. missing, foreign, and operation-kind-mismatched entries are structured errors;
4. insertion order is non-semantic;
5. runtime order comes only from the prepared graph;
6. the sidecar exposes no access, dependency, initialization, requirement, operation,
   or graph mutation path.

The sidecar still contains mixed future migration payload. Exact deletion split:

- G4B removes program, source, entry-point, interface, binding, layout,
  specialization, and pipeline descriptor truth;
- G4C removes backend resource, module, layout, bind-group, pipeline, cache,
  generation, and realization truth;
- G5 deletes the remaining execution-only payload during encoding/submission cutover.

## Consumer migration

Direct consumers migrated:

- render pass authoring, validation, merge, planning, compiled execution payload, and
  prepared-frame flow inputs;
- runtime execution, preflight cache/provenance, and per-invocation preparation;
- compiler, plan, provenance, graph inspection, and transient allocation planning;
- timestamp writes, typed query resolution, resolve-buffer/readback copy;
- GPU primitive counters, scans, compaction, scatter, indirect arguments, temporaries,
  and inferred stage order;
- procedural descriptors and lowering.

Transitive consumers migrated:

- `runenwerk_draw` render and GPU-ink flows;
- `runenwerk_editor` runtime and startup/viewport tests;
- boids, Game of Life, SDF render flow, debug inspection, fullscreen, postprocess, and
  fragment-compositor examples;
- render-flow planning benchmarks;
- render-flow, dynamic-target, fragment, resource-model, runtime-inspection,
  pipeline-fallback, timing, procedural-instance, import, cutoff, application, and
  documentation tests.

No current declaration, direct consumer, or transitive consumer remains on the retired
generic renderer graph authority.

## Replaced authority deleted

The implementation deleted
`engine/src/plugins/render/graph/resource_lifetimes.rs` and removed:

```text
CompiledResourceAccessKind
CompiledResourceLifetimeWindow
compile_resource_lifetime_windows
diagnose_resource_lifetime_windows
renderer-local generic topological ordering
generic reads/writes/depends_on correctness fields
PassDependencyCycleDetected
UnknownPassDependency
GpuPrimitiveResourceAccessKind
GpuPrimitiveResourceAccess
GpuPrimitiveDispatchResource::Temporary(String)
string primitive stage dependencies
```

Transient allocation windows remain derived policy over prepared G3 truth. They do not
restore generic lifetime/access authority. No compatibility alias, deprecated name,
forwarding module, source mirror, duplicate validator, dual graph, dependency,
manifest, lockfile, or workflow change remains.

## Validation evidence

Untouched accepted-base validation completed before G3 source edits:

```text
cargo validate
git diff --check
cargo test -p engine --test network_plugins \
  server_tracks_per_connection_baselines_across_reconnects --locked -- --test-threads=1
```

Focused observed evidence included:

| Command or suite | Observed result |
|---|---|
| `cargo test -p engine gpu:: --locked` | 84 GPU tests passed |
| adapter `gpu_work` unit filter | 4 adapter/sidecar tests passed |
| `render_flow_v2` | 16 passed |
| `render_dynamic_targets` | 15 passed |
| `render_flow_fragments` | 9 passed |
| `render_resource_model` | 15 passed |
| `render_runtime_inspect` | 23 passed |
| `procedural_instance` | 11 passed |
| `render_gpu_timing` | 4 passed |
| `render_pipeline_fallback` | 5 passed |
| focused GPU-primitive suite | 14 passed |
| `render_cutoff_guard` | 5 passed |
| `render_import_contract` | 4 passed |
| engine doctests | 9 ordinary and 21 compile-fail passed |
| strict engine Clippy | passed with warnings denied |
| `runenwerk_draw` | 12 library, 48 shell, and 7 guard tests passed |
| `runenwerk_editor` | 643 library, 2 startup, and 55 viewport tests passed; one manual environment-dependent GPU smoke remained ignored by design |

The final candidate also observed:

```text
cargo fmt --all --check                                      passed
cargo check -p engine --examples --benches --locked          passed
CI=true pnpm --dir docs-site build                           passed, 1,001 pages
cargo validate                                               passed
git diff --check                                             passed
```

PR `#181` and GitHub Actions own exact accepted-head remote evidence and the accepted
squash merge. No environment-dependent GPU execution result was claimed as new G3
proof.

## Complete-diff review

The complete implementation diff was reviewed for public contract shape, canonical
equality and hashing, initialization and hazards, operation/access agreement,
determinism, adapter ownership, sidecar containment, runtime order, direct/transitive
consumers, deletion, panic and warning suppression, dependency direction, and phase
boundaries.

Accepted structural findings:

- no production panic, unwrap, or expect was introduced in new G3 source before test
  modules;
- no warning suppression or broad allowance was introduced;
- retired generic symbols occur only in guards or historical records;
- renderer execution consumes prepared G3 order and operation identity;
- the sidecar is private and prepared-node-keyed;
- manifests, dependencies, lockfiles, workflows, packages, and external repositories
  were unchanged;
- no second validator, graph, compatibility alias, or forwarding path remains.

## Deliberate later-phase seams

### G4

- fixed legacy renderer capability translation remains until G4A admission;
- the crate-private `RenderFlowId`-derived resource-owner bridge remains until G4C;
- program, interface, binding, layout, pipeline, WGPU realization, and cache payload
  remains until G4B/G4C;
- backend copy/query alignment, format, feature, and limit admission remains G4A/G4C.

### G5

G3 preparation proves graph intent only. Upload, preserved prior-epoch state, backend
synchronization, command encoding, submission, progress, pressure, completion,
cancellation, readback, runtime retirement, and delayed destruction remain G5.

Current timing `map_async`/poll/channel behavior remains renderer-owned execution
evidence, not a reusable progress contract.

### G7

G3 present is logical work intent only. Surface acquisition, presentation,
surface/device generations, thread affinity, loss, and reconstruction remain G7.

These are accepted phase boundaries, not hidden implementation gaps or extraction
readiness.

## Accepted closure and next safe action

G3 implementation, review corrections, migration, deletion, validation,
documentation, and complete-diff review are accepted at
`39d6fe65a334502bdfba0b1a2ce3b365099fcf28`.

The exact next program step is the G4 decision phase. G4 implementation begins only
through the ordered accepted specifications and separate issues:

```text
G4A context admission
-> G4B program/interface/layout
-> G4C WGPU realization and cutover
```

No G5, G7, RunenRender implementation, package extraction, dependency, or compatibility
path is authorized by this G3 closeout.