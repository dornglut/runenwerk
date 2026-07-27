---
title: PT-RUNENGPU-G2 Normalized Capabilities and Logical Resources Implementation Closeout
description: Implemented migration, deletion, validation, equality, adapter disposition, and remaining-boundary evidence for RunenGPU G2.
status: completed
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

RunenGPU G2 is implemented and reviewed through issue `#172` and pull request `#173`. This closeout becomes authoritative through the merge of PR `#173`; repository Git history and the closed issue/merged PR represent maintainer acceptance. This pre-merge branch artifact deliberately asserts no merge SHA.

The implementation creates the bounded future-transferable capability, logical-resource, kind-typed-handle, prepared-data, provenance, and structured-error contracts under `engine::plugins::gpu`. Current render consumers lower through exactly three temporary adapter responsibilities, and the replaced renderer capability, descriptor, import/lifetime, and generic-handle authority is deleted without aliases or forwarding modules.

G2 creates no external package and implements no G3 hazards/work graph, G4 backend realization, G5 execution/readback/retirement, G6 offscreen graphics, or G7 surface ownership.

## Revisions and publication state

| Evidence | Revision or state |
|---|---|
| Parent program | issue `#167` |
| Owning implementation issue | issue `#172` |
| Implementation pull request | `#173` |
| Branch | `codex/runengpu-g2-capabilities-resources` |
| Accepted implementation base | `48d3e26dd29e7a20b8a4e3978e8e8465d24e6f84` |
| Contract commit | `8753dc0ab75269b78c357df6392ed6b61ac33e50` |
| Final reviewed code candidate head | `e0ee749f7a9f1c6e24abaf48ff92a0abb814818d` |
| Merge base against `origin/main` at publication | exact accepted implementation base |
| Acceptance record | repository Git history plus closed issue `#172` and merged PR `#173`; no merge SHA asserted here |

At reviewed code candidate `e0ee749f7a9f1c6e24abaf48ff92a0abb814818d`, the accepted-base diff contained exactly 78 changed files with 7,804 additions and 2,022 deletions. The final PR body owns the later documentation-head SHA, final aggregate statistics, exact-head Actions URLs and conclusions because those publication facts occur after this self-contained report is committed.

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

### Semantic equality audit

Every G2 `PartialEq`, `Eq`, `Ord`, and `Hash` implementation in the five future-transferable GPU API files and the three temporary render adapters was audited against the diagnostic-authority rule.

Corrections made:

- `GpuExportRelationship` now compares exactly its kind-preserving resource reference, consumer-owned export key, and required final access intent. Stored provenance—including producer label, source generation, and source revision—remains inspectable but is excluded.
- `GpuCapabilityAdmission` excludes diagnostic message text while comparing granted required facts, preferred availability/degradation facts, and verified-disabled facts.
- `RenderGpuParamsLayout` excludes process-local `TypeId` and diagnostic type name while comparing the checked normalized data layout.
- Render texture, imported texture, imported buffer, and target-alias intent equality excludes diagnostic labels while retaining logical ID and the owned semantic fields.

Confirmed intentional contracts:

- `PreparedGpuData` already excludes provenance and diagnostic type name; `GpuResourceCommon` already excludes label and provenance; descriptors inherit those semantic implementations.
- Typed handle equality, ordering, and hashing use the private in-process lease ID and kind so distinct live resource references remain distinct. This is an explicitly bounded handle-reference contract, not persistence, replay, wire, cache, binding, or descriptor-semantic authority; descriptor and diagnostic payloads remain excluded.
- `GpuResourceLabel` and `GpuResourceProvenance` retain value equality for direct diagnostic inspection, and structured error types retain value equality as error DTOs. Neither equality is inherited by resource semantic contracts after this correction.

No other semantic inconsistency was found, and useful equality implementations were retained.

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
| `gpu_resources.rs` | Return one explicit `RenderGpuResourceLowering` outcome for checked normalized descriptors, unresolved imported texture/buffer intent, or target aliases while retaining render-owned target, history, and surface policy. | G4-G7 as resource realization, typed layout views, history execution, and surface acquisition move to their accepted owners. |

No adapter maps normalized GPU descriptors into WGPU types in G2. Existing renderer-owned WGPU realization remains the legacy runtime path until G4. A fixed sRGB surface validation format exists only inside the render adapter to complete pure descriptor validation; the actual WGPU surface format remains current runtime input and G7 owns its removal.

The optional `Result<Option<GpuResourceDescriptor>, _>` lowering contract is deleted. The replacement is:

```rust
pub enum RenderGpuResourceLowering {
    Normalized(Box<GpuResourceDescriptor>),
    ImportedTexture(RenderImportedTextureIntent),
    ImportedBuffer(RenderImportedBufferIntent),
    TargetAlias(RenderTargetAliasDeclaration),
}
```

`Normalized` is returned only for render uniform/storage declarations and owned sampled, storage, color, depth, and history textures whose checked G2 descriptor facts are complete. The boxed field is only storage for the already-authoritative `GpuResourceDescriptor`; it is not a second descriptor type or compatibility layer.

Imported declarations retain only logical ID, validated non-empty diagnostic label, and explicit render semantic. They do not invent extent, format, usage, size, memory layout, reconstruction source, surface acquisition, backend realization, or unsupported ownership facts. G4 resolves ordinary imported texture/buffer admission and realization. G7 resolves surface acquisition and presentation-owned facts. Render history stays render-owned policy while generic imported-resource realization moves to its owning phase. Target aliases remain explicit render-graph relationships.

The temporary legacy runtime exhaustively consumes this outcome into buffer, flow-texture, invocation-history, imported-texture, imported-buffer, or target-alias dispositions. It allocates only normalized buffers and owned textures, preserves invocation-scoped history and current surface-format behavior, leaves imports and aliases unallocated for their existing paths, and returns a structured unsupported-current-runtime error for normalized texture views, samplers, or query sets if they become reachable. No wildcard or optional result silently discards a future variant.

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

The following commands passed on the final correction tree:

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

Focused evidence includes 32 GPU unit and source-guard tests, 67 named render integration tests across the five required binaries, 9 focused runtime-disposition unit tests, 8 passing rustdoc examples, and 12 passing compile-fail rustdoc cases. Canonical `cargo validate` also passed workspace tests, workspace doctests, strict workspace Clippy, docs validation, and the repository audit.

Exact-head permanent Actions remain pull-request delivery evidence rather than self-referential branch-document content. Pull request `#173` records its final remote SHA, workflow URLs, and conclusions after this documentation commit is pushed and checked. Maintainer acceptance is represented by the eventual merge and issue closure; this report does not invent either an exact-head result before it exists or a merge SHA before merge.

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
- excluding provenance from `GpuExportRelationship` semantic equality and excluding remaining diagnostic-only admission, type, and label fields found by the complete equality audit;
- replacing ambiguous optional render lowering with the explicit `RenderGpuResourceLowering` outcome;
- retaining imported texture and buffer declarations as unresolved render-owned intent until sufficient G4/G7 facts exist instead of fabricating normalized descriptors;
- exhaustively consuming every lowering outcome in the current runtime, with explicit non-allocation for imports/aliases, invocation-scoped history, and structured unsupported-kind failures;
- scanning the entire GPU source subtree and rejecting retired authority names and paths.

No unresolved correctness finding remains in the locally reviewed candidate.

## Remaining risks and deliberate boundaries

- G2 generic buffer handles intentionally do not retain one universal compile-time Rust element/layout view. G4 owns typed binding and layout views. Current render adapters preserve `TypeId` checks for uniform projection and indirect arguments, but a GPU-primitive call using a same-size wrong element type cannot be diagnosed from the G2 generic handle alone.
- `current_runtime_gpu_capabilities()` is a fixed translation of legacy renderer assumptions, not backend-admission evidence. G4 must replace it with real normalized facts.
- Handle leases establish clone/drop ownership shape only. G5 must connect last drop to delayed backend retirement.
- Ordinary imported resource facts intentionally remain unresolved. G4 must supply admission and realization facts; the current external-import rejection and prepared-resource paths remain unchanged until then.
- Existing WGPU allocation, shader/pipeline, queue, and submission code remains renderer-owned. The explicit current-runtime disposition is a temporary realization seam, not new G2 authority; G4-G5 must migrate those owners rather than extending the adapter.
- Surface validation still uses a bounded render-adapter placeholder while actual runtime surface format remains legacy input. G7 owns surface acquisition and deletion of that seam.
- Render history remains invocation-scoped render policy; G5 owns execution behavior and G7 owns surface/presentation outcomes rather than G2.
- No environment-dependent GPU execution was run because G2 adds no execution behavior; deterministic execution proofs remain G5-G7 work.

These are accepted phase boundaries, not claims of extraction readiness.

## Merge readiness and next safe action

Local implementation validation, focused validation, canonical validation, documentation reconciliation, and complete-diff review are complete, with no known correctness blocker after the final independent-review corrections. Pull request `#173` owns final remote-head equality, exact-head Actions, and unresolved-review evidence. Maintainer acceptance is represented only by its eventual merge and issue `#172` closure; no merge SHA is claimed here.

The next safe action after merge is to create one bounded, decision-complete G3 planning issue and specification. Do not begin G3 implementation until that authority is accepted, and do not infer G4-G7 implementation from G2.
