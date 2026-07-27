---
title: PT-RUNENGPU-G2 Normalized Capabilities and Logical Resources Implementation Closeout
description: Draft-review implementation, migration, deletion, validation, adapter disposition, and remaining-boundary evidence for RunenGPU G2.
status: active
owner: workspace
layer: reports
canonical: true
last_reviewed: 2026-07-27
related_docs:
  - ../../workspace/specs/pt-runengpu-g2-capabilities-resource-descriptors.ron
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/completed-work.md
  - ../../workspace/planning/roadmap.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../investigations/runengpu-g2-capabilities-resources-investigation.md
---

# PT-RUNENGPU-G2 Normalized Capabilities and Logical Resources Implementation Closeout

## Outcome

RunenGPU G2 is code-complete and locally validated on draft pull request `#173`. It is not merged or accepted yet.

The review candidate creates the bounded future-transferable capability, logical-resource, kind-typed-handle, prepared-data, provenance, and structured-error contracts under `engine::plugins::gpu`. Current render consumers lower through exactly three temporary adapter responsibilities, and the replaced renderer capability, descriptor, import/lifetime, and generic-handle authority is deleted without aliases or forwarding modules.

G2 creates no external package and implements no G3 hazards/work graph, G4 backend realization, G5 execution/readback/retirement, G6 offscreen graphics, or G7 surface ownership.

## Revisions and publication state

| Evidence | Revision or state |
|---|---|
| Parent program | issue `#167` |
| Owning implementation issue | issue `#172` |
| Draft implementation pull request | `#173` |
| Branch | `codex/runengpu-g2-capabilities-resources` |
| Accepted implementation base | `48d3e26dd29e7a20b8a4e3978e8e8465d24e6f84` |
| Contract commit | `8753dc0ab75269b78c357df6392ed6b61ac33e50` |
| Reviewed code implementation head | `5640242bdcba845e58a401b224113de5fe7c8c82` |
| Merge base against `origin/main` at publication | exact accepted implementation base |
| Merge state | draft and unmerged |

The code implementation head changed exactly 70 files with 6,475 additions and 1,846 deletions. The final PR also carries the eight documentation paths listed below. The PR body owns the final remote-head SHA, final aggregate statistics, exact-head Actions run, and conclusion because those publication facts occur after this self-contained report is committed.

## Delivered public authority

### Normalized capability facts and admission

`engine::plugins::gpu` now owns:

- `GpuCapabilityFeature`, `GpuCapabilityRequirement`, `GpuPreferredFallback`, and `GpuCapabilityRequirements`;
- `GpuCapabilityProfile` as named requirement composition, not a backend profile key;
- `GpuTextureFormat`, `GpuTextureFormatCapabilities`, `GpuLimits`, and `GpuCapabilities`;
- `GpuPreferredAvailability` and `GpuCapabilityAdmission`;
- structured requirement and admission errors with human operation, cause, and correction text.

Requirements distinguish required, preferred-with-explicit-fallback, and disabled features. Composition is deterministic, fallible, order-independent for compatible inputs, and rejects ambiguous preferred fallbacks. Admission distinguishes advertised availability from enabled features.

### Logical resources

The new backend-neutral resource authority includes:

- diagnostic-only `GpuResourceLabel` and `GpuResourceProvenance`;
- independent `GpuResourceLifetime`, `GpuResourceOwnership`, `GpuReconstruction`, and `GpuMemoryIntent` dimensions;
- `GpuBufferUsage`, `GpuBufferUsages`, `GpuTextureUsage`, and `GpuTextureUsages`;
- checked `GpuTextureDimension`, `GpuTextureExtent`, `GpuTextureAspect`, and `GpuTextureSubresourceRange`;
- sampler state through `GpuAddressMode`, `GpuFilterMode`, `GpuCompareFunction`, and `GpuSamplerDescriptor`;
- `GpuBufferInitialization`, `GpuPreparedTextureData`, and `GpuTextureInitialization` as separate initialization contracts;
- `GpuBufferDescriptor`, `GpuTextureDescriptor`, `GpuTextureViewDescriptor`, `GpuQueryKind`, and `GpuQuerySetDescriptor`;
- `GpuResourceDescriptor`, `GpuResourceAccessIntent`, and `GpuExportRelationship`.

Constructors validate dimensions, overflow, usages, memory and ownership combinations, format roles, subresources, row layout, sampler ranges, timestamp query constraints, initialization bounds, and export relationships. Pure descriptor fields are immutable behind validated constructors.

### Kind-typed logical handles

The logical handle authority now consists of:

- `GpuBufferHandle`;
- `GpuTextureHandle`;
- `GpuTextureViewHandle`;
- `GpuSamplerHandle`;
- `GpuQuerySetHandle`;
- kind-preserving `GpuResourceRef` for diagnostics and export relationships.

Handles are safe cloneable, non-`Copy` logical leases with private construction. They expose no raw-ID constructor, destroy-by-ID method, cross-kind reinterpretation, backend handle, or unchecked public escape hatch. Texture-view descriptors retain the parent texture lease and checked subresource range.

### Prepared-data boundary

The new data contract includes:

- sealed purpose markers `UniformData`, `StorageData`, `VertexData`, `IndirectData`, and `TransferData`;
- checked `GpuDataLayout` and `PreparedGpuData<Purpose>`;
- `GpuDataEncoder` and `prepare_gpu_data` for explicit source-to-purpose preparation;
- Pod-only transfer preparation;
- `GpuReadbackBytes` and `GpuReadbackDecoder` as decoding boundaries, without implementing readback execution.

Rustdoc compile-pass and compile-fail cases prove purpose separation, non-Pod transfer rejection, non-`Copy` handles, private construction, absent destroy-by-ID, and absent cross-kind reinterpretation.

## Render migration and temporary adapters

The current render facade returns `GpuBufferHandle` for uniform and storage declarations and uses render-owned `RenderDoubleBuffer` only for the semantic pair. Uniform projection and indirect argument paths retain transitional `TypeId` diagnostics where current render behavior requires them; normalized GPU descriptors do not treat `TypeId`, type names, or labels as layout, identity, binding, persistence, replay, wire, or cache authority.

Exactly three temporary adapter responsibilities exist under `engine/src/plugins/render/adapters`:

| Adapter | Current bounded purpose | Required deletion phase |
|---|---|---|
| `gpu_capabilities.rs` | Return the fixed normalized facts already assumed by the legacy renderer and validate compiled render requirements. It performs no adapter/device query. | G4, when backend admission constructs facts. |
| `gpu_data.rs` | Lower current `GpuParams` values and layout diagnostics into purpose-specific prepared data. | G4, when admitted shader interfaces and purpose-specific encoders replace transitional traits. |
| `gpu_resources.rs` | Lower current render declarations into normalized descriptors while retaining render-owned target, history, alias, and surface intent. | G4-G7 as resource realization, typed layout views, history execution, and surface acquisition move to their accepted owners. |

No adapter maps normalized GPU descriptors into WGPU types in G2. Existing renderer-owned WGPU realization remains the legacy runtime path until G4. A fixed sRGB surface validation format exists only inside the render adapter to complete pure descriptor validation; the actual WGPU surface format remains current runtime input and G7 owns its removal.

## Replaced authority deleted

The implementation deletes these files:

- `engine/src/plugins/render/graph/capabilities.rs`;
- `engine/src/plugins/render/resource/descriptors.rs`;
- `engine/src/plugins/render/resource/import.rs`;
- `engine/src/plugins/render/resource/lifetime.rs`;
- `engine/src/plugins/render/api/resources.rs`.

The deleted or replaced public authority includes `RenderBackendCapabilityProfile`, `RenderBackendCapabilityInspection`, the generic `RenderResourceDescriptor` family, `ImportedTextureSemantic`, `ImportedBufferSemantic`, `ImportedResourceDescriptor`, `ResourceLifetime`, `UniformHandle`, `StorageArrayHandle`, and `DoubleBufferHandle`. Render-specific texture policy and import semantics remain render-owned under explicit `Render*` names; generic GPU facts live only under `engine::plugins::gpu`.

Repository guards scan the complete G2 GPU source subtree for forbidden renderer, ECS, WGPU, Winit, application, product, UI, SDF, codec, shader-asset, `TypeId`, and `Any` reach-through. Separate guards reject retired names and deleted paths. No compatibility alias, deprecated parallel name, forwarding module, source mirror, duplicate descriptor path, manifest change, dependency change, lockfile change, or workflow change remains.

## Exact changed-file inventory

### Added code authority and adapters

- `engine/src/plugins/gpu/api/capability.rs`
- `engine/src/plugins/gpu/api/data.rs`
- `engine/src/plugins/gpu/api/errors.rs`
- `engine/src/plugins/gpu/api/handles.rs`
- `engine/src/plugins/gpu/api/resource.rs`
- `engine/src/plugins/render/adapters/gpu_capabilities.rs`
- `engine/src/plugins/render/adapters/gpu_data.rs`
- `engine/src/plugins/render/adapters/gpu_resources.rs`
- `engine/src/plugins/render/adapters/mod.rs`

### Deleted code authority

- `engine/src/plugins/render/api/resources.rs`
- `engine/src/plugins/render/graph/capabilities.rs`
- `engine/src/plugins/render/resource/descriptors.rs`
- `engine/src/plugins/render/resource/import.rs`
- `engine/src/plugins/render/resource/lifetime.rs`

### Modified code consumers and guards

- `apps/runenwerk_draw/src/runtime/gpu_ink.rs`
- `apps/runenwerk_editor/tests/startup_render_smoke.rs`
- `apps/runenwerk_editor/tests/viewport_architecture_guards.rs`
- `engine/benches/render_flow_planning.rs`
- `engine/examples/boids_render_flow/rendering/evidence.rs`
- `engine/examples/boids_render_flow/rendering/graph.rs`
- `engine/examples/render_fragment_compositor.rs`
- `engine/examples/sdf_render_flow/rendering/evidence.rs`
- `engine/src/plugins/gpu/api/mod.rs`
- `engine/src/plugins/gpu/mod.rs`
- `engine/src/plugins/render/api/bindings.rs`
- `engine/src/plugins/render/api/errors.rs`
- `engine/src/plugins/render/api/flow.rs`
- `engine/src/plugins/render/api/handles.rs`
- `engine/src/plugins/render/api/mod.rs`
- `engine/src/plugins/render/api/passes.rs`
- `engine/src/plugins/render/composition/fragment_registry.rs`
- `engine/src/plugins/render/composition/integration.rs`
- `engine/src/plugins/render/gpu_primitives/compaction.rs`
- `engine/src/plugins/render/gpu_primitives/counters.rs`
- `engine/src/plugins/render/gpu_primitives/draw_args.rs`
- `engine/src/plugins/render/gpu_primitives/plan.rs`
- `engine/src/plugins/render/gpu_primitives/scan.rs`
- `engine/src/plugins/render/graph/execution_plan.rs`
- `engine/src/plugins/render/graph/flow_graph.rs`
- `engine/src/plugins/render/graph/merge.rs`
- `engine/src/plugins/render/graph/mod.rs`
- `engine/src/plugins/render/graph/planning.rs`
- `engine/src/plugins/render/graph/prepared_validation.rs`
- `engine/src/plugins/render/graph/resource_graph.rs`
- `engine/src/plugins/render/graph/resource_lifetimes.rs`
- `engine/src/plugins/render/graph/validation.rs`
- `engine/src/plugins/render/inspect/graph_dump.rs`
- `engine/src/plugins/render/inspect/plan.rs`
- `engine/src/plugins/render/inspect/resource_inspector.rs`
- `engine/src/plugins/render/inspect/texture_view.rs`
- `engine/src/plugins/render/mod.rs`
- `engine/src/plugins/render/procedural/authoring.rs`
- `engine/src/plugins/render/procedural/descriptors.rs`
- `engine/src/plugins/render/procedural/population/uniform_grid.rs`
- `engine/src/plugins/render/renderer/render_flow/execute.rs`
- `engine/src/plugins/render/renderer/render_flow/mod.rs`
- `engine/src/plugins/render/renderer/render_flow/preflight_cache.rs`
- `engine/src/plugins/render/renderer/render_flow/runtime_resources.rs`
- `engine/src/plugins/render/renderer/render_flow/runtime_resources/inspect.rs`
- `engine/src/plugins/render/renderer/render_flow/runtime_resources/realize.rs`
- `engine/src/plugins/render/renderer/render_flow/runtime_resources/resolve.rs`
- `engine/src/plugins/render/resource/mod.rs`
- `engine/src/plugins/render/resource/usages.rs`
- `engine/tests/procedural_instance.rs`
- `engine/tests/render_dynamic_targets.rs`
- `engine/tests/render_flow_fragments.rs`
- `engine/tests/render_flow_v2.rs`
- `engine/tests/render_import_contract.rs`
- `engine/tests/render_resource_model.rs`
- `engine/tests/render_runtime_inspect.rs`

### Documentation in the final pull request

- `docs-site/src/content/docs/design/accepted/render-execution-graph-compiler-maturity-design.md`
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`
- `docs-site/src/content/docs/engine/reference/plugins/render/render-flow-usage-guide.md`
- `docs-site/src/content/docs/reports/closeouts/pt-runengpu-g2-implementation-closeout.md`
- `docs-site/src/content/docs/workspace/planning/active-work.md`
- `docs-site/src/content/docs/workspace/planning/completed-work.md`
- `docs-site/src/content/docs/workspace/planning/roadmap.md`
- `docs-site/src/content/docs/workspace/specs/pt-runengpu-g2-capabilities-resource-descriptors.ron`

## Validation evidence

The following commands passed on the reviewed code implementation tree:

```text
cargo test -p engine gpu:: --locked
cargo test -p engine --test render_resource_model --locked
cargo test -p engine --test render_import_contract --locked
cargo test -p engine --test render_flow_v2 --locked
cargo test -p engine --test render_dynamic_targets --locked
cargo test -p engine --test render_runtime_inspect --locked
cargo test -p engine --doc --locked
cargo clippy -p engine --lib --tests --benches --examples --locked -- -D warnings
cargo test -p runenwerk_draw --locked
cargo test -p runenwerk_editor --locked
cargo validate
git diff --check
```

Focused evidence includes 26 GPU unit tests, 59 named render integration tests across the five required binaries, 8 passing rustdoc examples, and 12 passing compile-fail rustdoc cases. Canonical `cargo validate` also passed workspace tests, workspace doctests, strict workspace Clippy, docs validation, and the repository audit.

Exact-head permanent Actions are required before merge. The live check surface is pull request `#173`; its final remote SHA, workflow URL, and conclusion are recorded in the PR body after the final documentation commit is pushed and checked. This report deliberately does not claim a CI conclusion that did not exist when its own commit was authored.

## Critical complete-diff review

The complete code diff was reviewed against the accepted investigation and design gates. Corrections made during review include:

- carrying complete normalized capability facts in the preflight cache key instead of inventing a collision-prone fingerprint or profile key;
- making compatible capability requirement composition deterministic and commutative, with explicit ambiguous-fallback rejection;
- separating advertised availability from enabled-feature admission;
- requiring both storage-read and storage-write facts for current storage-texture use;
- diagnosing absent indirect-draw capability;
- validating uniform projection type identity and rejecting uniform buffers used as storage;
- validating normalized vertex, index, and indirect usage roles;
- keeping target aliases retained and render-owned rather than falsely imported;
- removing normalized-GPU-to-WGPU and WGPU-to-normalized mapping authority from G2 adapters;
- scanning the entire GPU source subtree and rejecting retired authority names and paths.

No unresolved correctness finding remains in the locally reviewed candidate.

## Remaining risks and deliberate boundaries

- G2 generic buffer handles intentionally do not retain one universal compile-time Rust element/layout view. G4 owns typed binding and layout views. Current render adapters preserve `TypeId` checks for uniform projection and indirect arguments, but a GPU-primitive call using a same-size wrong element type cannot be diagnosed from the G2 generic handle alone.
- `current_runtime_gpu_capabilities()` is a fixed translation of legacy renderer assumptions, not backend-admission evidence. G4 must replace it with real normalized facts.
- Handle leases establish clone/drop ownership shape only. G5 must connect last drop to delayed backend retirement.
- Existing WGPU allocation, shader/pipeline, queue, and submission code remains renderer-owned. G4-G5 must migrate those owners rather than extending the temporary adapters.
- Surface validation still uses a bounded render-adapter placeholder while actual runtime surface format remains legacy input. G7 owns surface acquisition and deletion of that seam.
- No environment-dependent GPU execution was run because G2 adds no execution behavior; deterministic execution proofs remain G5-G7 work.

These are accepted phase boundaries, not claims of extraction readiness.

## Merge readiness and next safe action

Local implementation, focused validation, canonical validation, documentation reconciliation, and critical diff review are complete. Merge readiness remains pending the final documentation commit, exact-head Actions, remote-head equality, unresolved-review audit, and maintainer acceptance of draft PR `#173`.

Do not merge automatically. Review and accept or correct PR `#173`. Only after its merge and issue `#172` closeout may one decision-complete G3 planning slice be opened; do not begin G3 implementation from G2 authority.
