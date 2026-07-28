---
title: PT-RUNENGPU-G3 Access and Work Graph Implementation Closeout
description: Implementation, migration, deletion, adapter, sidecar, validation, and remaining-phase evidence for the bounded RunenGPU G3 cutover.
status: completed
owner: workspace
layer: reports
canonical: true
last_reviewed: 2026-07-28
related_docs:
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runengpu-g3-access-work-graph-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../reports/investigations/runengpu-g3-access-work-graph-investigation.md
  - ../../workspace/specs/pt-runengpu-g3-access-work-graph.ron
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/completed-work.md
  - ../../workspace/planning/roadmap.md
  - ../../engine/reference/plugins/render/public-api-reference.md
  - ../../engine/reference/plugins/render/render-flow-usage-guide.md
---

# PT-RUNENGPU-G3 Access and Work Graph Implementation Closeout

## Outcome

Issue `#177` implements the complete bounded RunenGPU G3 access/work-graph slice in
draft PR `#181`. The implementation creates one future-transferable authority under
`engine::plugins::gpu` for checked resource access, typed immutable GPU operations,
region-aware graph-entry initialization, exact hazards, typed cross-fragment
causality, operation-derived requirements, immutable work fragments/nodes, and
deterministic prepared graphs.

Current render, timing, GPU-primitive, inspection, application, example, benchmark,
and test consumers are migrated through one temporary lowering adapter. Renderer
execution payload remains in a private sidecar over prepared node identity and cannot
alter G3 truth. The replaced renderer-owned generic lifetime/access/topological
authority is deleted without aliases, forwarding modules, or a second graph.

The candidate creates no external package and implements no G4 backend admission or
realization, G5 GPU-owned execution/readback/retirement, or G7 surface/loss/
reconstruction behavior. Draft PR `#181` remains open and unmerged for independent
review; this report deliberately asserts no G3 merge SHA.

## Revisions and publication state

| Evidence | Revision or state |
|---|---|
| Parent program | issue `#167` |
| Owning implementation issue | issue `#177` |
| Implementation pull request | draft PR `#181` |
| Branch | `codex/runengpu-g3-access-work-graph` |
| Previous candidate base | `90d24abb93bff4b1d3f5b4743056bc00ff80d4b6` |
| Baseline repair | issue `#179` / PR `#180`, merged as `1c645b2bbfcece44dd6ae151cc97559793afa2c2` |
| Verified accepted implementation base | `1c645b2bbfcece44dd6ae151cc97559793afa2c2` |
| Access-contract commit | `5f132c5c8d488d129057658dd40a275273f5bdb5` |
| Operation-contract commit | `2baed155c51869026376d5a2a25ed6e8041d38e5` |
| Prepared-graph commit | `9d52f7a1c95ee2aa78fa6b53689a80b83a6ea4a7` |
| Frozen code candidate | `0c950b244cea799661468d4774d485b5fc2b5984` |
| Final documentation head | the commit containing this report; its exact SHA is recorded in PR `#181` and issue `#177` after publication because a commit cannot contain its own SHA |
| Merge base against `origin/main` at publication | exact accepted implementation base |
| Acceptance record | repository Git history plus eventual closure of issue `#177` and merge of PR `#181`; no merge SHA asserted here |

The accepted-base diff at frozen code candidate
`0c950b244cea799661468d4774d485b5fc2b5984` contains exactly 66 changed files with
12,675 additions and 1,431 deletions. The final documentation candidate contains
exactly 78 changed files with 13,310 additions and
1,522 deletions. PR `#181` owns the later exact documentation-head SHA,
final remote-head equality, exact-head Actions URLs and conclusions, review state,
and final aggregate statistics after this self-contained report is committed.

The pre-start record is issue comment
`https://github.com/dornglut/runenwerk/issues/177#issuecomment-5096747870`. It records
the accepted-base fast-forward, exact current-main census, untouched-baseline
validation, and confirmation that PR `#180` changed only
`engine/tests/network_plugins/delta_and_reconnect.rs` without changing G3 authority,
consumers, manifests, dependencies, lockfiles, workflows, or architecture.

## Public G3 authority introduced

### Checked access and canonical regions

`engine::plugins::gpu` now owns:

- `GpuBufferRange`, `GpuQueryRange`, and checked texture-subresource coverage;
- texture/view normalization to kind-preserving parent storage;
- explicit buffer, texture, query, and sampler access records;
- exact buffer byte, texture mip/layer/aspect, and query-index overlap;
- D3 whole-mip-volume behavior;
- descriptor-usage validation, including distinct `QueryResolveDestination` and
  `GpuBufferUsage::QueryResolve` authority;
- semantic equality, ordering, and hashing that exclude labels and provenance.

Ranges and normalized access cannot be field-constructed or changed after validation.
Texture views do not become independent hazard storage.

### Typed operations

One `GpuWorkOperation` authority owns operation kind and exact immutable shape for:

- compute dispatch;
- render color/depth attachments, canonical clear values, direct/indexed/indirect
  draw intents, multisample resolve targets, and timestamp writes;
- all four buffer/texture copy directions with checked logical coverage;
- exact-range buffer zero;
- query-set-to-buffer resolution;
- logical present intent.

Operations derive their mandatory access and capability requirements. Callers may add
semantic requirements but cannot contradict or disable an operation-implied fact.
Color and depth clear values reject non-finite/out-of-range components and canonicalize
negative zero. Standalone clear remains buffer zero only; multisample resolve remains
part of a render attachment.

### Immutable work and preparation

The public work-graph authority includes:

- private fragment-local `GpuWorkNodeId` allocation and process-local
  `GpuPreparedWorkNodeId` values;
- `GpuWorkResourceInput`, `GpuWorkImport`, `GpuWorkOutput`, and semantic
  `GpuExportKey` causality;
- `GpuExplicitOrder` for fragment-local non-data constraints only;
- `GpuWorkNode`, `GpuWorkFragmentBuilder`, and immutable `GpuWorkFragment`;
- `GpuDependencyReason`, `GpuWorkDependency`, prepared initialization summaries,
  merged requirements, outputs, and diagnostics;
- one immutable `GpuPreparedWorkGraph::prepare(...)` authority.

Preparation validates resource identity/kind, operation/access consistency, exact
initial coverage, imports/exports, requirements, same-node normalization, RAW/WAR/WAW
hazards, missing or ambiguous cross-fragment causality, redundant/opposing explicit
edges, and cycles. It emits deterministic prepared identities, diagnostics,
dependencies, requirements, initialization, and topological order. Input fragment
enumeration may change diagnostic ordinals but cannot change semantic edges or error
outcomes.

## Render migration and temporary adapter

Exactly one new lowering seam exists:

```text
engine/src/plugins/render/adapters/gpu_work.rs
```

It lowers current render passes, attachments, clear values, projected dispatch,
timestamp writes, query resolution, readback copy intent, and typed primitive
temporaries into checked G3 work. Whole-resource access is used only where the current
render declaration has no narrower fact. History and current-runtime entry assumptions
become explicit initialization input evidence.

`RenderPassNode` retains render shape and payload inputs but no longer owns generic
`reads`, `writes`, or broad `depends_on` correctness. `non_data_order_after` is the
only retained render authoring edge and is lowered into fragment-local explicit order.
Lexical `authoring_index` remains provenance only. Renderer execution, transient
planning, capability validation, and inspection consume the exact prepared graph.

The adapter is temporary. G4 deletes its shader/pipeline/interface and render-derived
owner bridges when GPU context/admission/realization authority exists. It is not a
compatibility facade and contains no second hazard, initialization, or scheduling
implementation.

## Temporary execution-payload sidecar

`RenderGpuWorkSidecar` is private to the adapter and stores only current render
execution payload. Its sole key is `GpuPreparedWorkNodeId`; labels, pass indices,
resource IDs, and strings are not lookup authority.

Enforced invariants:

1. every prepared node has exactly one payload and every payload names one prepared
   node;
2. duplicate insertion is a structured error and leaves the original map unchanged;
3. missing entries are structured errors;
4. foreign prepared-node identities are structured errors;
5. payload/operation-kind mismatch is a structured error;
6. insertion order is non-semantic;
7. runtime order comes only from `GpuPreparedWorkGraph::topological_order()`;
8. the sidecar exposes no access, dependency, initialization, requirement, operation,
   or graph mutation path.

The four adapter unit tests cover lowering and the combined bijection, missing,
duplicate, foreign, kind-mismatch, prepared-order, and insertion-order cases. The
cutoff guard additionally inspects production source to prevent a second prepared
graph or payload lookup authority from returning.

## Consumer migration

### Direct consumers

- render pass authoring, validation, merge, planning, compiled execution payload, and
  prepared-frame flow inputs;
- runtime render execution, preflight cache/provenance, and per-invocation preparation;
- compiler/plan/provenance graph inspection;
- transient window and alias planning derived from prepared G3 access/order;
- timestamp query write, typed query resolution, and resolve-buffer/readback copy;
- GPU primitive counters, scans, compaction, scatter, indirect-argument generation,
  typed temporary buffers, and inferred stage order;
- procedural descriptors/lowering that consume migrated primitive plans.

### Transitive consumers

- `runenwerk_draw` render and GPU-ink flows;
- `runenwerk_editor` runtime and startup/viewport architecture tests;
- boids, Game of Life, SDF render flow, debug inspection, fullscreen, postprocess,
  and fragment-compositor examples;
- render-flow planning benchmarks;
- render-flow, dynamic-target, fragment, resource-model, runtime-inspection,
  pipeline-fallback, timing, procedural-instance, import, cutoff, application, and
  documentation tests.

No current declaration, direct consumer, or transitive consumer remains on the
retired generic graph authority.

## Replaced authority deleted

The implementation deletes
`engine/src/plugins/render/graph/resource_lifetimes.rs` and removes:

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

Transient allocation windows remain as derived allocation policy over prepared G3
truth; they do not restore lifetime-window access authority. No compatibility alias,
deprecated name, forwarding module, source mirror, duplicate validator, dual graph,
manifest/dependency/lockfile change, or workflow change remains.

## Exact changed-file inventory

### Added G3 code authority and adapter

- `engine/src/plugins/gpu/api/access.rs`
- `engine/src/plugins/gpu/api/graph.rs`
- `engine/src/plugins/gpu/api/work.rs`
- `engine/src/plugins/render/adapters/gpu_work.rs`

### Deleted renderer authority

- `engine/src/plugins/render/graph/resource_lifetimes.rs`

### Modified GPU and render code

- `engine/src/plugins/gpu/api/errors.rs`
- `engine/src/plugins/gpu/api/handles.rs`
- `engine/src/plugins/gpu/api/mod.rs`
- `engine/src/plugins/gpu/api/resource.rs`
- `engine/src/plugins/gpu/mod.rs`
- `engine/src/plugins/render/adapters/gpu_capabilities.rs`
- `engine/src/plugins/render/adapters/mod.rs`
- `engine/src/plugins/render/api/flow.rs`
- `engine/src/plugins/render/api/passes.rs`
- `engine/src/plugins/render/composition/fragment_validation.rs`
- `engine/src/plugins/render/composition/fragments.rs`
- `engine/src/plugins/render/frame/packet.rs`
- `engine/src/plugins/render/gpu_primitives/compaction.rs`
- `engine/src/plugins/render/gpu_primitives/counters.rs`
- `engine/src/plugins/render/gpu_primitives/draw_args.rs`
- `engine/src/plugins/render/gpu_primitives/plan.rs`
- `engine/src/plugins/render/gpu_primitives/scan.rs`
- `engine/src/plugins/render/graph/diagnostics.rs`
- `engine/src/plugins/render/graph/execution_plan.rs`
- `engine/src/plugins/render/graph/merge.rs`
- `engine/src/plugins/render/graph/mod.rs`
- `engine/src/plugins/render/graph/pass_graph.rs`
- `engine/src/plugins/render/graph/pass_shape.rs`
- `engine/src/plugins/render/graph/planning.rs`
- `engine/src/plugins/render/graph/prepared_validation.rs`
- `engine/src/plugins/render/graph/validation.rs`
- `engine/src/plugins/render/graph/validation_builtin_ui.rs`
- `engine/src/plugins/render/inspect/graph_dump.rs`
- `engine/src/plugins/render/inspect/pass_provenance.rs`
- `engine/src/plugins/render/inspect/plan.rs`
- `engine/src/plugins/render/procedural/descriptors.rs`
- `engine/src/plugins/render/procedural/lowering.rs`
- `engine/src/plugins/render/procedural/population/uniform_grid.rs`
- `engine/src/plugins/render/renderer/render_flow/execute.rs`
- `engine/src/plugins/render/renderer/render_flow/gpu_timing.rs`
- `engine/src/plugins/render/renderer/render_flow/mod.rs`
- `engine/src/plugins/render/renderer/render_flow/preflight_cache.rs`
- `engine/src/plugins/render/renderer/render_flow/provenance.rs`
- `engine/src/plugins/render/resource/transient.rs`
- `engine/src/plugins/render/runtime/frame_prepare.rs`

### Modified direct/transitive code consumers and tests

- `apps/runenwerk_draw/src/runtime/app.rs`
- `apps/runenwerk_draw/src/runtime/gpu_ink.rs`
- `apps/runenwerk_editor/src/runtime/app.rs`
- `apps/runenwerk_editor/tests/startup_render_smoke.rs`
- `apps/runenwerk_editor/tests/viewport_architecture_guards.rs`
- `engine/benches/render_flow_planning.rs`
- `engine/examples/boids_render_flow/rendering/evidence.rs`
- `engine/examples/boids_render_flow/rendering/graph.rs`
- `engine/examples/game_of_life_sdf/rendering/graph.rs`
- `engine/examples/render_flow_debug_inspect/main.rs`
- `engine/examples/render_flow_fullscreen_minimal/main.rs`
- `engine/examples/render_flow_postprocess_compositor/main.rs`
- `engine/examples/render_fragment_compositor.rs`
- `engine/examples/sdf_render_flow/rendering/graph.rs`
- `engine/tests/render_cutoff_guard.rs`
- `engine/tests/render_dynamic_targets.rs`
- `engine/tests/render_flow_fragments.rs`
- `engine/tests/render_flow_v2.rs`
- `engine/tests/render_import_contract.rs`
- `engine/tests/render_pipeline_fallback.rs`
- `engine/tests/render_runtime_inspect.rs`

### Documentation in the final pull request

- `docs-site/src/content/docs/architecture/repository-family-architecture.md`
- `docs-site/src/content/docs/design/active/README.md`
- `docs-site/src/content/docs/design/active/runengpu-architecture-design.md`
- `docs-site/src/content/docs/design/active/runengpu-g3-access-work-graph-design.md`
- `docs-site/src/content/docs/design/active/runenrender-internal-decomposition-execution-plan.md`
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`
- `docs-site/src/content/docs/engine/reference/plugins/render/render-flow-usage-guide.md`
- `docs-site/src/content/docs/reports/closeouts/README.md`
- `docs-site/src/content/docs/reports/closeouts/pt-runengpu-g3-implementation-closeout.md`
- `docs-site/src/content/docs/workspace/planning/active-work.md`
- `docs-site/src/content/docs/workspace/planning/completed-work.md`
- `docs-site/src/content/docs/workspace/planning/roadmap.md`

## Validation evidence

### Accepted-base and pre-start validation

On the untouched accepted base, these completed successfully before G3 source edits:

```text
cargo validate
git diff --check
cargo test -p engine --test network_plugins server_tracks_per_connection_baselines_across_reconnects --locked -- --test-threads=1
```

The exact `90d24abb93bff4b1d3f5b4743056bc00ff80d4b6..1c645b2bbfcece44dd6ae151cc97559793afa2c2`
inspection contained only the accepted reconnect-test repair.

### Focused implementation validation

Observed successful focused commands and results include:

| Command or suite | Observed result |
|---|---|
| `cargo test -p engine gpu:: --locked` | 75 GPU tests passed; all filtered binaries clean |
| adapter `gpu_work` unit filter | 4 adapter/sidecar tests passed |
| `cargo test -p engine --test render_flow_v2 --locked` | 16 passed |
| `cargo test -p engine --test render_dynamic_targets --locked` | 15 passed |
| `cargo test -p engine --test render_flow_fragments --locked` | 9 passed |
| `cargo test -p engine --test render_resource_model --locked` | 15 passed |
| `cargo test -p engine --test render_runtime_inspect --locked` | 23 passed |
| `cargo test -p engine --test procedural_instance --locked` | 11 passed |
| `cargo test -p engine --test render_gpu_timing --locked` | 4 passed |
| `cargo test -p engine --test render_pipeline_fallback --locked` | 5 passed |
| focused GPU-primitive suite | 14 passed, including exact 4,097-element prefix-scan planning and runtime-adapter coverage |
| `cargo test -p engine --test render_cutoff_guard --locked` | 5 passed |
| `cargo test -p engine --test render_import_contract --locked` | 4 passed |
| `cargo test -p engine --doc --locked` | 8 ordinary doctests and 21 compile-fail doctests passed |
| `cargo clippy -p engine --lib --tests --benches --examples --locked -- -D warnings` | passed with warnings denied |
| `cargo test -p runenwerk_draw --locked` | 12 library, 48 application-shell, and 7 guard tests passed |
| `cargo test -p runenwerk_editor --locked` | 643 library, 2 startup, and 55 viewport tests passed; one existing manual environment-dependent GPU smoke remained ignored by design |

Access tests cover exact zero/overflow/bounds causes, query ranges, texture aspect/
mip/layer bounds, D2 overlap/disjoint behavior, view-parent normalization, D3
whole-volume behavior, descriptor usage, and diagnostic-free equality/order/hash.
Operation tests cover canonical clears, dispatch/draw/indirect paths, all copy
directions, attachments, multisample resolve failures, buffer zero, query resolution,
timestamp-to-resolve-to-copy, kind, and derived requirements. Graph tests cover G2
initialization modes, partial coverage, RAW/WAR/WAW/read-read, disjoint regions,
same-node normalization, typed imports/exports, fragment reorder invariance,
deterministic identities/order/diagnostics/requirements, explicit-order success and
failure, foreign identities, and attachment/query initialization transitions.

Compile-fail rustdoc proves private range/node construction, absent raw-ID recovery,
cross-kind rejection, immutable fragments/prepared graphs, private render-operation
fields, and no public duplicate node-kind mutation path. Source/dependency guards scan
the complete GPU subtree for renderer, ECS, WGPU, Winit, UI, SDF, application,
editor, product, codec, `TypeId`, `Any`, shader-filesystem, source-include, compatibility,
and forwarding reach-through. Retired authority names and paths are also guarded.

Examples and `render_flow_planning` benchmark targets compile under the strict Clippy
command. The final explicit example/benchmark compilation command and result are
recorded below with the final documentation candidate.

### Final documentation candidate

| Evidence | Result |
|---|---|
| `cargo fmt --all --check` | passed |
| `cargo check -p engine --examples --benches --locked` | passed; only the existing `block 0.1.6` future-incompatibility notice was emitted |
| `CI=true pnpm --dir docs-site build` | passed; Astro built 1,001 pages and emitted only the existing missing-site sitemap warning |
| `cargo validate` | passed: workspace formatting, locked tests/doctests, strict Clippy, documentation validation, and repository audit; only the existing `block 0.1.6` future-incompatibility notice was emitted |
| `git diff --check` | passed; tracked and new-file content checked before commit and repeated at the documentation head |
| `git diff --check 90d24abb93bff4b1d3f5b4743056bc00ff80d4b6..<final-documentation-head>` | passed at the documentation head |
| complete accepted-base and previous-base diff review | passed with no unresolved local correctness, ownership, migration, deletion, dependency, or phase-boundary finding |

Exact-head GitHub Actions and Documentation Build are publication facts that occur
only after the documentation commit changes the PR head. PR `#181` records the final
head SHA, workflow run URLs/IDs, conclusions, comments, submitted reviews, and
unresolved-thread count. A stale earlier-head run is not accepted as final evidence,
and merge remains blocked until the exact head is green. This separation avoids a
self-referential report that changes the head while claiming to describe it.

## Complete-diff review

The complete implementation diff was reviewed for public contract shape, canonical
equality/hashing, exact initialization/hazards, operation/access agreement,
determinism, adapter ownership, sidecar containment, runtime order, direct and
transitive consumers, deletion, panic/warning suppression, dependency direction, and
phase boundaries.

Structural audit results:

- no production `panic!`, `unwrap`, or `expect` was introduced in new G3 source
  before test modules;
- no new warning suppression or broad allowance was introduced;
- the added G3 range contains no `#[allow(...)]` addition;
- retired generic symbols occur only in guard literals or historical design/
  investigation records that describe their deletion;
- renderer execution consumes prepared G3 order and operation identity;
- the sidecar is private, payload-only, and keyed solely by prepared node identity;
- manifests, dependencies, lockfiles, workflows, packages, and external repositories
  are unchanged;
- no second validator, graph, compatibility alias, or forwarding path remains.

## Remaining risks and deliberate phase seams

### G4

- `current_runtime_gpu_capabilities()` still translates fixed legacy renderer facts;
  backend adapter/device admission must replace it.
- the crate-private `RenderFlowId`-derived resource-owner bridge remains until GPU
  context/work-scope identity exists.
- render shader, pipeline, interface, binding, and WGPU realization payload remains in
  the temporary adapter/sidecar.
- backend-specific copy/query alignment, format, feature, and limit admission remains
  G4 work.

### G5

- G3 preparation proves graph-time intent only; it does not prove upload, preserved
  prior-epoch state, synchronization, command encoding, submission, completion, or
  retirement.
- current renderer WGPU encoding consumes prepared G3 work, but GPU-owned ordinary
  prepare/submit, progress, pressure, cancellation, completion, readback delivery,
  query-resolution execution, and delayed last-lease retirement remain G5.
- current timing `map_async`/poll/readback behavior remains render-owned legacy
  execution evidence, not a reusable G5 progress contract.

### G7

- G3 `Present` is logical work intent only. Surface acquisition, actual presentation,
  surface/device generations, thread affinity, loss classification, and
  reconstruction reports remain G7.
- ordinary external imports and surface-owned facts remain unresolved rather than
  fabricated by G3.

No environment-dependent GPU execution result is claimed as new G3 proof. These are
accepted phase boundaries, not hidden implementation gaps or extraction readiness.

## Merge readiness and next safe action

The local implementation, migration, deletion, focused validation, documentation
reconciliation, and complete-diff audit are complete with no known local correctness
blocker once the final-candidate placeholders above are replaced by observed results.
PR `#181` owns final remote-head equality, exact-head Actions, Documentation Build,
comments, reviews, and unresolved-thread evidence.

The next safe action is independent review of the exact draft-PR head. Do not merge
until its required workflows pass and no unresolved correctness or ownership finding
remains. Do not begin G4, G5, G7, external extraction, or a new package as part of
this slice.
