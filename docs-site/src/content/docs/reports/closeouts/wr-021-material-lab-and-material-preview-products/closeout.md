---
title: WR-021 Material Lab And Material Preview Products Closeout
description: "Current WR-021 closeout for the repaired source-backed Material Lab and GPU-proven material preview product spine."
status: completed
owner: apps/runenwerk_editor
layer: domain / app-runtime / editor-ui / engine-render
canonical: true
last_reviewed: 2026-05-17
related_designs:
  - ../../../design/active/material-lab-and-material-preview-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
related_reports:
  - ../../../reports/implementation-plans/wr-021-material-lab-and-material-preview-products/plan.md
---

# WR-021 Material Lab And Material Preview Products Closeout

## Status

This closeout supersedes the earlier drift-superseded WR-021 evidence. The
2026-05-17 repair now proves the material product chain that earlier passes only
described: source graph -> semantic ratification -> typed executable IR ->
generated WGSL -> validated engine shader -> catalog-backed material and shader
artifacts -> scene material slot indices -> KTX2-backed group-1 material
texture bindings -> material preview product -> viewport selection -> GPU pixel
evidence.

The repair keeps material source truth in `domain/material_graph`, keeps scene
material palette contracts in `domain/editor/editor_scene`, and keeps renderer
ownership limited to prepared material data, generated shader consumption, KTX2
GPU residency, and pass execution. The editor shell exposes a source-projected
Material Lab graph surface backed by `domain/ui/ui_graph_editor` contracts; the
current UI is accepted as the V1 product-spine editor surface, with visual node
editing polish left to later UX work rather than used as a hidden completion
blocker.

## Owning Scope

- `domain/material_graph/src/authored.rs::MaterialGraphDocument` owns material
  source identity and output target.
- `foundation/resource_ref/src/lib.rs::ResourceRef` owns portable external
  resource references for material texture/resource node values without adding an
  `asset` dependency to `domain/material_graph`.
- `domain/graph/src/model.rs::GraphMetadataEntry` and `GraphValue` own
  domain-neutral node/port metadata and values for authored graph contracts.
- `domain/material_graph/src/persistence.rs::MaterialGraphSourceFileV2` owns the
  current source-file round trip. `MaterialGraphSourceFileV1` is a hard break and
  returns a superseded-version diagnostic path.
- `domain/material_graph/src/lowering.rs::lower_material_graph` owns deterministic
  lowering into executable `MaterialIr`, cache keys, source maps, specialization
  fragments, and formed material products.
- `domain/material_graph/src/ir.rs::MaterialIr` owns the typed executable
  material graph contract consumed by render backends.
- `domain/material_graph/src/catalog.rs::MaterialNodeCatalog` owns stable material
  node catalog id/version identity for cache correctness. All first-slice nodes
  have explicit IR/compiler semantics; texture nodes require catalog-backed
  resource refs before lowering can publish.
- `apps/runenwerk_editor/src/material_lab/recipes.rs::ResolvedMaterialLoweringRecipe`
  owns the V1 typed runtime recipe boundary for serialized `"preview"` and
  `"render_material"` project policy.
- `apps/runenwerk_editor/src/material_lab/workflow.rs::rebuild_material_preview_for_asset`
  owns material source loading, recipe rejection before ledger allocation, lowering
  orchestration, ledger reuse/allocation, catalog artifact publication, and
  prior-valid preservation.
- `apps/runenwerk_editor/src/material_lab/publication.rs::publish_pending_material_preview_publications`
  owns ProductPublication barrier publication and active-preview state updates.
- `apps/runenwerk_editor/src/material_lab/renderer_handoff.rs::prepared_material_resource_for_preview`
  owns conversion from resolved material preview state into engine-owned typed
  prepared material parameter payloads, portable texture binding metadata, and
  generated scene shader bundle feature data.
- `engine/src/plugins/render/frame/contributions.rs::PreparedMaterialParameterPayloadV1`
  owns the V1 prepared material parameter payload contract, canonical
  length-prefixed encoding/decoding, profile/output-target enums, and payload
  version and parameter-count rejection.
- `engine/src/plugins/render/frame/contributions.rs::PreparedMaterialFeatureContribution`
  owns the fixed 64-slot material binding table, the fixed 128-slot material
  texture resource table guard, and the generated scene material bundle
  prepared-data contract.
- `engine/src/plugins/render/material_compiler/mod.rs::compile_material_shader`
  owns the engine material IR to WGSL compiler and WGSL validation boundary,
  including real texture/sampler declarations for material texture nodes.
- `domain/ui/ui_graph_editor` owns backend-neutral graph editor view models,
  selection, edit actions, shortcuts, and undo/redo grouping contracts.
- `domain/editor/editor_scene/src/model/material.rs` owns source-backed scene
  material palette slots and primitive slot reference contracts. Runtime scene
  packets consume compact material slot indices derived from those contracts.
- `engine/src/plugins/render/renderer/prepare.rs::hash_prepared_feature_contribution`
  hashes the encoded typed material payload, texture resource metadata, and
  generated scene bundle identity as renderer-prepared data.
- `engine/src/plugins/render/renderer/prepare.rs::prepare_material_gpu_resources`
  loads exact KTX2 artifact bytes, validates descriptor hash/revision/byte layout,
  creates GPU texture/sampler resources, and exposes prepared group-1 material
  bind groups.
- `engine/src/plugins/render/renderer/render_flow/execute_passes.rs` rejects
  material-feature shader fallback, consumes generated scene material bundle
  shaders, keeps group 0/group 1 pipeline layout indices stable, and binds
  prepared group-1 material resources for material passes.
- `apps/runenwerk_editor/src/runtime/systems/material_preview.rs::prepare_material_preview_render_resource_system`
  hands active material preview data to `PreparedMaterialFeatureResource` while
  preserving the prior ready bundle until exact generated shaders are loaded.
- `apps/runenwerk_editor/src/material_lab/default_material.rs::ensure_default_scene_material_preview`
  generates the default material through V2 material graph lowering and engine
  shader compilation so startup scene rendering does not use the old fallback
  shader path.
- `apps/runenwerk_editor/src/runtime/systems/material_preview.rs::produce_material_preview_dynamic_uploads_system`
  now owns material-preview render-frame requests for the dedicated preview flow;
  it no longer uploads CPU-generated preview pixels.
- `apps/runenwerk_editor/src/runtime/app.rs::register_editor_render_flow` registers
  a separate material-preview render flow that writes `MaterialPreview2D` targets.
- `apps/runenwerk_editor/src/runtime/viewport/product_registry.rs::material_preview_descriptor`
  and `apps/runenwerk_editor/src/runtime/viewport/product_targets.rs` own material
  preview viewport product descriptors and selectable primary target routing.
- `apps/runenwerk_editor/src/runtime/viewport/render_jobs.rs::sync_viewport_render_jobs_system`
  owns fixed scene/picking/overlay producer render-job alias bindings. Scene
  render jobs always write `viewport.scene_color`; selected material previews do
  not become scene pass color attachments.
- `apps/runenwerk_editor/src/runtime/viewport/presentation_resolver.rs::build_surface_binding_registry`
  and `apps/runenwerk_editor/src/runtime/viewport/render_product_selection.rs::prepare_viewport_render_product_selections`
  make selected-primary viewport presentation state the authority for the
  displayed primary surface and selected-product residency/query intent.
- `apps/runenwerk_editor/src/runtime/viewport/query_snapshots.rs::product_authority`
  classifies `ExpressionSourceRealityClass::DerivedMaterial` as deterministic
  derived authority.
- `domain/editor/editor_shell/src/surfaces/material.rs` owns typed Material Lab
  view models/actions; shell/providers remain IO-free projection and routing code.
- `domain/editor/editor_shell/src/commands/shell_command.rs::ShellCommand` owns
  epoch-carrying material workflow commands, and
  `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs::dispatch_shell_command_with_viewport_commands`
  rejects stale material commands before mutation.
- `engine/src/plugins/render/runtime/frame_prepare.rs` consumes
  `PreparedMaterialFeatureResource` into `MATERIAL_RENDER_FEATURE_ID` frame
  contributions.

## Completion Evidence

- Source-backed material graph documents round-trip through versioned RON source
  files and deterministic asset/source-derived document ids.
- Typed material recipes replace raw string runtime policy. The only V1 serialized
  material lowering targets are `"preview"` and `"render_material"`; empty,
  unknown, source-kind mismatched, or non-`AssetKind::Material` artifact targets
  produce blocking `ImportProfileRejected` diagnostics before ledger allocation.
- Material lowering uses `MaterialNodeCatalog::first_slice()` through the resolved
  recipe boundary, not through workflow-local string checks.
- Material recipe cache identity includes the import cache key, resolved recipe
  cache component, formed material cache key, source hash, source id, importer id,
  and expected artifact kind.
- Material lowering cache identity is canonical and catalog-versioned:
  `MaterialNodeCatalog` exposes stable id/version identity, `lower_material_graph`
  includes that identity, descriptor keys, and descriptor semantic versions in
  cache keys, display labels do not affect cache identity, and graph/node/port
  names are encoded through canonical length-prefixed fields instead of
  delimiter-joined strings or compact hash-only keys.
- Material node catalog construction is checked: invalid catalog identity,
  duplicate descriptor keys, empty node keys, zero catalog versions, and zero
  descriptor semantic versions are rejected rather than silently producing cache
  identity drift.
- Material source resolution rejects multiple material graph sources unless the
  asset has an explicit primary source, and this rejection happens before import
  ledger allocation or publication.
- `domain/asset::AssetKind::Material` is now a formed product kind, so material
  graph imports produce product jobs instead of bypassing runtime publication.
- Failed material source reads, document identity mismatches, document output
  target mismatches, and lowering failures preserve the prior valid material
  artifact without publishing invalid active preview or renderer data.
- Material preview publication goes through the ProductPublication barrier, records
  publication journals, updates active preview state only after accepted
  publication, and registers selectable viewport material preview products.
- Viewport presentation state now drives the displayed primary surface and
  selected-product residency/query intent. Scene color remains displayed while
  selected, and a material preview becomes the displayed primary product only
  when selected.
- Render producers now write fixed outputs. The editor scene pass writes
  `viewport.scene_color`, picking and overlay write their fixed support targets,
  and selected material previews cannot retarget the scene pass output.
- Material preview now has a real product producer path. The app consumes
  `PreparedMaterialFeatureResource`, registers the generated shader artifact with
  the shader registry, and `produce_material_preview_dynamic_uploads_system`
  contributes material-preview render-frame requests that write `MaterialPreview2D`
  dynamic texture targets through a dedicated render flow.
- Material preview preparation no longer registers generated scene shaders under
  the global `editor_viewport_scene_product` shader id. Scene shader artifacts
  are carried as `PreparedSceneMaterialBundle` feature data and are consumed by
  the stable scene material ABI. Missing generated scene bundles are treated as
  forbidden fallbacks.
- Material-feature render passes are forbidden from using builtin shader
  fallback. If an exact generated shader is missing, the pass fails closed or is
  skipped by the feature gate instead of drawing a success-shaped fallback.
- Generated shader artifact paths are content-addressed with fixed-length BLAKE3
  digests of canonical cache keys rather than raw cache-key filenames, so long or
  structured identities remain portable across Windows and other filesystems.
- Texture nodes compile to WGSL resource declarations and `textureSample(...)`
  calls using compiler-produced resource binding metadata. The old
  deterministic pseudo texture sampling helper is not production output.
- Texture resource refs resolve to exact catalog artifacts and ratified
  `domain/texture::TextureDescriptor` KTX2 metadata with artifact revision/cache
  residency identity. Prepared material handoff carries descriptor hashes,
  artifact paths, revisions, extents, pixel formats, supercompression state, and
  byte-layout metadata into renderer allocation. The renderer reads exact KTX2
  artifact bytes, validates descriptor/hash/byte-length agreement, creates
  group-1 GPU texture/sampler bind groups, and samples both Texture2D and
  Texture3D resources in the gated GPU proof. Unsupported KTX2 variants fail
  closed instead of producing fallback pixels.
- Executable material IR is topologically ordered by producer/consumer
  dependencies, so valid source graphs no longer depend on source node order for
  compiler success.
- Material Lab graph view models now project the active source
  `MaterialGraphDocument` directly, including source-owned layout, port ids,
  resource refs, groups, and viewport state. Formed IR is derived product state,
  not editable UI truth.
- Source-backed material edits, undo, and redo immediately refresh the active
  source projection and ratification/lowering diagnostics before preview rebuild.
- Renderer handoff publishes prepared material instances through
  `PreparedMaterialFeatureResource` and the existing `MATERIAL_RENDER_FEATURE_ID`
  frame contribution path. The prepared parameter payload is an engine-owned
  typed V1 payload with profile/output-target/parameter enums and deterministic
  encode/decode behavior rather than Rust debug-string-shaped bytes. Decode
  rejects unknown versions and oversized parameter counts before allocation.
  Renderer code consumes prepared data and does not own material graph semantics.
- Material Graph Canvas, Material Inspector, and Material Preview providers project
  typed source-backed view models/actions and route commands through app-owned
  workflow adapters. The canvas no longer uses the generic self-authoring control
  panel; it emits a Material Lab graph surface from `ui_graph_editor` contracts
  with source-owned layout, palette, property, texture picker, validation overlay,
  shortcut, fixture, preview-selection, and undo/redo action shape.
- Every material shell command carries `projection_epoch`; stale material commands
  return without mutating material selection, diagnostics, queued publications, or
  active preview state.

## Drift Findings

- No ADR is required. Material graph semantic ownership stayed in
  `domain/material_graph`, and renderer/app runtime did not become material source
  authority.
- The WR-021 implementation was reconciled into the prepared batch worktree
  `C:/Users/joshi/Projekte/Runenwerk-worktrees/2026-05-16-next-current-candidate-roadmap-batch-wr-021/WR-021`
  on branch `codex/2026-05-16-next-current-candidate-roadmap-batch-wr-021-wr-021`.
- Batch write scope was expanded for the necessary `Cargo.lock` and `domain/asset`
  changes because material source persistence adds a crate dependency and material
  artifacts must be recognized as formed products by the asset domain.
- Dynamic importer plugins and multiple material node catalogs remain deferred.
  WR-021 intentionally proves a typed recipe boundary first.
- Immediate post-closeout drift was corrected in the prepared WR-021 worktree:
  selected-primary product authority controls only the displayed primary surface,
  producer render-pass outputs stay fixed, material preview has a dedicated
  render producer, material prepared data uses a stable typed versioned payload,
  material cache identity is catalog-versioned, descriptor-semantic-versioned,
  label-insensitive, and delimiter-safe, material node catalog construction
  rejects invalid identity, ambiguous material sources fail closed, material
  source projection no longer depends on formed IR, generated scene shader
  identity no longer mutates the global scene shader id, generated scene bundle
  shaders are consumed by render flow provenance, group-1 KTX2 material
  texture/sampler bind groups are allocated and bound, runtime scene packets
  carry material slot indices, and the gated GPU smoke proves generated scene and
  preview material shaders sample Texture2D and Texture3D KTX2 resources through
  the prepared material path.

## Validation

Implementation validation completed on 2026-05-17:

- `cargo check -p runenwerk_editor` passed.
- `cargo fmt --check` passed.
- `cargo test -p resource_ref` passed: 2 tests.
- `cargo test -p graph` passed: 9 tests.
- `cargo test -p asset` passed: 26 tests.
- `cargo test -p material_graph` passed: 19 tests, including V1 hard-break,
  V2 round trip, executable IR, texture resource-ref rejection/carry-through, and
  cache identity coverage.
- `cargo test -p texture` passed: KTX2 descriptor validation and unsupported
  transcode rejection.
- `cargo test -p asset texture` passed: typed texture artifact payload coverage.
- `cargo test -p editor_shell material` passed: typed material view model/action
  and epoch command contract.
- `cargo test -p editor_viewport material` passed: material preview product
  descriptor classification.
- `cargo test -p runenwerk_editor material` passed: 36 material workflow/provider
  tests, including generated default material bootstrap and visible portable
  binding-limit diagnostics.
- `cargo test -p runenwerk_editor material_recipe` passed: recipe resolution,
  rejection, ledger non-allocation on rejected recipe, and changed-recipe cache
  split coverage.
- `cargo test -p runenwerk_editor material_epoch` passed: stale material commands
  produce no material workflow mutation.
- `cargo test -p runenwerk_editor material_handoff` passed: versioned prepared
  material resource formation.
- `cargo test -p engine material_parameter_payload` passed: typed prepared
  material payload round-trip, unknown-version rejection, and oversized
  parameter-count rejection.
- `cargo test -p engine material_contribution_hash` passed: renderer
  contribution hashing uses encoded typed material payload data.
- `cargo test -p engine material` passed: prepared material contribution,
  payload, hashing, engine WGSL compiler validation, and material compiler
  fixture identity coverage.
- `cargo test -p runenwerk_editor material_viewport` passed: material preview
  primary selection through viewport presentation state.
- `cargo test -p runenwerk_editor material_preview` passed: material preview
  render-frame request coverage and selected-preview render-job regression
  coverage.
- `cargo test -p runenwerk_editor viewport` passed: 104 unit tests plus viewport
  smoke/architecture tests.
- `RUNENWERK_ENABLE_GPU_SMOKE=1 RUNENWERK_ENABLE_MACOS_MAIN_THREAD_GPU_SMOKE=1 cargo test -p runenwerk_editor --test viewport_gpu_truth_smoke -- --ignored --nocapture` passed on 2026-05-17. The smoke captures the normal scene/UI product path and then proves generated scene and material-preview shaders sample prepared group-1 KTX2 texture bindings into visible pixels:
  - Texture2D proof: `C:/Users/joshi/AppData/Local/Temp/runenwerk-wr021-gpu-proof/19756/wr021-material-scene-2d.wgsl`,
    `C:/Users/joshi/AppData/Local/Temp/runenwerk-wr021-gpu-proof/19756/wr021-material-preview-2d.wgsl`,
    `C:/Users/joshi/AppData/Local/Temp/runenwerk-wr021-gpu-proof/19756/wr021-material-texture-2d.ktx2`,
    scene pixel `[108, 210, 162, 255]`, preview pixel `[108, 210, 162, 255]`.
  - Texture3D/triplanar-shape proof: `C:/Users/joshi/AppData/Local/Temp/runenwerk-wr021-gpu-proof/19756/wr021-material-scene-3d.wgsl`,
    `C:/Users/joshi/AppData/Local/Temp/runenwerk-wr021-gpu-proof/19756/wr021-material-preview-3d.wgsl`,
    `C:/Users/joshi/AppData/Local/Temp/runenwerk-wr021-gpu-proof/19756/wr021-material-texture-3d.ktx2`,
    scene pixel `[153, 103, 173, 255]`, preview pixel `[153, 103, 173, 255]`.
- `cargo test -p engine material_handoff` passed: prepared material data reaches
  `MATERIAL_RENDER_FEATURE_ID`.
- `task batch:scope-check -- --batch docs-site/src/content/docs/reports/batches/2026-05-16-next-current-candidate-roadmap-batch-wr-021/batch.toml` passed.

Closeout validation after roadmap/production evidence updates:

- `task docs:validate` passed on the official WR-021 worktree.
- `task roadmap:render` completed and refreshed generated roadmap docs.
- `task roadmap:validate` passed: 26 items, 34 edges.
- `task roadmap:check` passed: schema and rendered roadmap docs are in sync.
- `task production:validate` passed: 1 track, 9 milestones.
- `task production:check` passed: schema and generated production docs are in sync.

## Deferred Work

- A broader dynamic material importer/plugin framework remains out of scope until
  a later WR/ADR proves stable extension points over the typed recipe boundary.
- Multiple material node catalogs and recipe-selected node catalog versions remain
  future work; both V1 recipes intentionally use `MaterialNodeCatalog::first_slice()`.
- Rich visual polish beyond the V1 source-projected graph surface, shader
  authoring, material package distribution, and renderer-side specialization
  cache management remain separate production slices.
