---
title: WR-021 Material Lab And Material Preview Products Contract
description: Executed promotion and implementation-readiness contract for source-backed Material Lab and material preview product handoff.
status: completed
owner: apps/runenwerk_editor
layer: domain / app-runtime / editor-ui / engine-render
canonical: true
last_reviewed: 2026-05-17
related_designs:
  - ../../../design/active/material-lab-and-material-preview-design.md
  - ../../../design/active/editor-asset-pipeline-and-content-workflow-design.md
  - ../../../design/active/editor-rendered-world-and-multi-entity-viewport-design.md
  - ../../../design/implemented/render-product-surface-foundation-bundle-design.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-track-index.md
  - ../../../workspace/roadmap-index.md
  - ../../../engine/roadmaps/fully-featured-renderer-roadmap.md
related_reports:
  - ../../../reports/closeouts/wr-018-rendered-world-v1/closeout.md
  - ../../../reports/closeouts/wr-019-field-visualizer-product-workflow/closeout.md
  - ../../../reports/closeouts/wr-026-source-backed-asset-editor-adapters/closeout.md
---

# WR-021 Material Lab And Material Preview Products Contract

## Goal

Establish implementation readiness for `WR-021` under the
`PM-SDF-OW-001` production product spine. The slice must turn Material Lab from
descriptor/status provider stubs into a source-backed material product workflow:
material graph source persistence, deterministic lowering, catalog-backed
formed material artifacts, viewport product selection, prior-valid
preservation, and prepared renderer handoff.

`WR-021` is promotable only as a full bounded V1 material product spine. It is
not acceptable to close this slice with canvas-local material truth, provider
text panels, direct renderer mutation, or catalog bypasses. This contract does
not itself implement product code; it now records the executed contract and links
to the canonical completed closeout evidence.

This revision also records the immediate post-closeout product-surface repair:
render producers must write fixed producer targets, viewport presentation
selection must only choose which already-produced primary surface is shown,
material preview must have an actual product producer path, renderer prepared
material data must use a stable typed versioned payload, material lowering cache
identity must be canonical and material-node-catalog-versioned, material graph
source resolution must fail closed when ambiguous, and provider evidence must
prove a source-projected graph surface rather than status/control-panel output.

The 2026-05-17 perfectionist repair raises the completion bar again: `WR-021`
was accepted as the completion standard. Source-backed material graphs lower
into executable typed IR, the engine compiler generates and validates WGSL for
both preview and scene entrypoints, generated material programs drive scene
primitives through explicit material slot assignment, texture resource refs
resolve to KTX2 catalog artifacts and group-1 GPU-resident texture resources,
material preview has a real product producer, viewport presentation only selects
already-produced products, and Material Lab V1 exposes source-projected graph
editing through epoch-guarded commands. The completed evidence is recorded in
the canonical closeout.

## Source Of Truth

- `docs-site/src/content/docs/workspace/production-tracks.yaml` owns
  `PT-SDF-OW` and active milestone `PM-SDF-OW-001`. The milestone links
  `WR-019`, `WR-026`, and `WR-021`, and requires product workflows to avoid
  parallel viewers, parallel truth stores, and renderer-owned semantic sources.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` owns `WR-021`.
  The row is now `completed`, depends on `WR-018:completed`,
  `WR-019:completed`, and `WR-026:completed`, and links the completed closeout
  evidence for source-backed `MaterialGraphDocument`, lowering/ratification,
  preview product, KTX2 texture residency, diagnostics, scene material slots,
  GPU proof, and render handoff.
- `docs-site/src/content/docs/design/active/material-lab-and-material-preview-design.md`
  is the active owning design. It states that `MaterialGraphDocument` is source
  truth, canvas state is projection, V1 requires failed preview preserving the
  prior valid artifact, and providers fail closed until product handoff is
  available.
- `docs-site/src/content/docs/design/active/editor-asset-pipeline-and-content-workflow-design.md`
  owns source-backed asset/catalog policy. `WR-021` must consume the `WR-026`
  source-backed adapter path instead of adding a separate material truth store.
- `docs-site/src/content/docs/engine/roadmaps/fully-featured-renderer-roadmap.md`
  owns FR-4 sequencing. Material preview products must hand off renderer-ready
  data without moving material semantics into renderer code.

Readiness checks completed for this contract:

- `task production:plan -- --milestone PM-SDF-OW-001 --roadmap WR-021`
  classifies the current next action as `write_implementation_contract`.
- `task planning:validate` was required before product-code implementation and
  again before accepting closeout evidence.

## Readiness

Promotion verdict: `WR-021` honestly carried a bounded implementation contract
and is now completed by the superseding closeout evidence.

The dependencies are satisfied:

- `WR-018` completed rendered-world V1 and stabilized the viewport packet /
  product handoff path.
- `WR-019` completed the field visualizer product workflow and product-aware
  viewport routing evidence.
- `WR-026` completed source-backed asset editor adapters and the project
  catalog/import adapter spine.

The write-scope governance gate was expanded before execution. The completed
`WR-021` row includes `apps/runenwerk_editor`, `foundation/resource_ref`,
`domain/graph`, `domain/asset`, `domain/ui/ui_graph_editor`,
`domain/editor`, `domain/material_graph`, `engine/src/plugins/render`, the
implementation plan, and the closeout path.

ADR requirement: no ADR is required while `domain/material_graph` remains
material semantic truth and renderer/app runtime only consume formed products
and prepared data. Start architecture governance before implementation if
material graph ownership moves out of `domain/material_graph`, if renderer code
becomes material source authority, or if app runtime state becomes canonical
material truth.

## Current Drift And Reconciliation Gate

Earlier WR-021 review found drift in the product spine even after the first
closeout: material preview descriptors could overwrite scene color despite
scene color remaining selected; render product selection still required the
hardcoded scene color target; prepared material parameters were shaped as Rust
debug strings; material graph cache identity was delimiter-joined string data
without a stable node-catalog version; source fallback could pick an arbitrary
material graph source; and provider evidence risked overclaiming a full
Material Lab UX. This gate is now satisfied by the 2026-05-17 repair and the
canonical closeout supersedes the earlier drifted evidence.

The immediate correction was part of WR-021 completion quality, not a deferred
slice. The accepted fix satisfied these additional gates:

- Render pass producer aliases are fixed: the scene pass writes
  `viewport.scene_color`, picking writes `viewport.picking_ids`, and overlay
  writes `viewport.overlay`. `viewport.primary_color` may remain registered only
  as a compatibility/presentation alias, not as a render producer output.
- `ViewportPresentationState.selected_primary_product_id` is the only authority
  for the displayed primary product, not for scene pass attachment selection.
- `sync_viewport_product_targets_system` binds `PrimaryColor` only for the
  selected primary product; picking and overlay remain required support targets.
- `prepare_viewport_render_product_selections` records selected product
  residency/query intent and required selected-product targets, but render job
  alias bindings remain producer-specific.
- Material preview has a real producer path:
  `domain/material_graph` lowers source documents to executable `MaterialIr`,
  `engine/src/plugins/render/material_compiler` validates generated WGSL, the
  app publishes the generated shader as a derived catalog artifact, and
  `produce_material_preview_dynamic_uploads_system` contributes render-frame
  requests into a dedicated material-preview render flow that writes the
  selected `MaterialPreview2D` dynamic target. Selecting a material preview must
  never retarget the scene pass output.
- Material graph V1 source files are a hard break. `MaterialGraphSourceFileV2`
  is the current round-trip contract; V1 decode paths must return blocking,
  recoverable diagnostics rather than silently migrating source truth.
- Texture and triplanar catalog nodes remain active. Texture nodes require
  portable foundation `resource_ref::ResourceRef` values that the app resolves
  against catalog-backed texture artifacts before shader publication.
- `EditorMaterialPreviewProduct` carries the resolved
  `MaterialRendererParameterProfile` into runtime preview state.
- `material_parameter_payload` builds an engine-owned
  `PreparedMaterialParameterPayloadV1` with canonical length-prefixed encoding
  and does not depend on Rust debug formatting. Decoding must reject oversized
  V1 parameter counts before allocating.
- `MaterialNodeCatalog` exposes a stable id/version and material lowering cache
  keys include that identity plus descriptor semantic versions. Catalog
  construction must reject duplicate descriptor keys, empty keys, empty catalog
  ids, zero catalog versions, and zero descriptor semantic versions.
- Material graph cache keys use canonical length-prefixed structured encoding
  rather than delimiter-joined arbitrary names or compact hash-only keys.
- Material graph source resolution rejects multiple material graph sources
  unless the asset has an explicit primary source.
- Closeout evidence proves providers emit source-projected graph surfaces through
  `editor_shell` and `ui_graph_editor`, not generic self-authoring control
  panels.

## Perfectionist Repair Gate

`WR-021` remained open until these completion-quality requirements were
implemented and validated:

- `MaterialGraphSourceFileV2` is the only accepted source truth and owns graph
  semantics plus persistent editor layout state: node positions, groups, graph
  viewport, fixture defaults, preview selection defaults, and layout metadata.
  Workspace focus/selection may be transient; source layout cannot live only in
  provider/app runtime state.
- Material Lab graph edit commands must mutate the V2 source document through
  app-owned workflow adapters and then re-ratify/re-lower through
  `domain/material_graph`. Provider-created commands must carry
  `projection_epoch`; stale commands mutate no source graph, selected node,
  diagnostics, queued publication, shader artifact, material assignment, preview
  target, or renderer state.
- The rich V1 Material Lab UI is not satisfied by status lines. Required
  evidence includes graph pan/zoom/select, searchable node palette, add/delete
  nodes, connect/disconnect ports, typed property inspector, texture-resource
  picker, validation overlays, source-map diagnostic navigation, fixture
  selector, preview selector, undo/redo grouping, keyboard shortcuts, and layout
  persistence.
- Scene primitives carry explicit material asset assignments. Missing assignment
  uses an explicit default material asset generated through the same material
  graph path, not hardcoded shader color. Generated scene shader artifacts must
  shade assigned primitives from the generated material program.
- Texture sample nodes resolve to exact catalog artifacts and those artifacts
  must be renderer-bindable GPU resources with explicit format, dimension,
  sampler policy, cache identity, and residency status before shader/material
  publication.
- Shader registry integration is artifact-root aware and fail-closed. When a
  material preview changes the generated shader path, stale shader source is
  cleared; material handoff is not marked ready until the exact generated
  preview and scene shader files have loaded.
- Closeout needs behavior and visual evidence: source round trip, source edit
  persistence, ratification/lowering rejection, generated shader validation,
  material/shader catalog artifact publication, per-primitive scene material
  binding, separate scene/material-preview producer outputs, selected-primary
  viewport display, exact shader load gating, texture GPU residency, stale epoch
  rejection, prior-valid preservation, and captured pixel/GPU proof that
  generated material changes visible scene and preview pixels.

## 2026-05-17 Repair Completion

The 2026-05-17 repair closed the long-term material spine gaps that had been
left as contract-shaped pieces:

- `domain/ui/ui_graph_editor` exists as a backend-neutral graph editor contract
  crate. `domain/editor/editor_shell/src/surfaces/material.rs` adapts it into
  material-specific, epoch-guarded shell view models/actions.
- `domain/editor/editor_scene/src/model/material.rs` defines source-backed scene
  material palette slots and primitive slot references. Runtime scene packets
  consume compact material slot indices before render handoff.
- `apps/runenwerk_editor/src/material_lab/document_io.rs` probes source-file
  versions before full decode. V1 material graph files now hard-fail with a
  recoverable superseded-source outcome instead of being misclassified as
  arbitrary V2 decode failures.
- `apps/runenwerk_editor/src/material_lab/workflow.rs` writes generated shader
  artifacts to fixed-length BLAKE3 content-addressed paths derived from canonical
  cache keys. This prevents raw cache-key characters or long structured keys
  from becoming invalid platform filenames.
- `engine/src/plugins/render/material_compiler/mod.rs` no longer compiles texture
  nodes through pseudo texture sampling functions. Texture nodes require
  resource-binding metadata and generate WGSL `texture_*` declarations plus
  `textureSample(...)` calls.
- `engine/src/plugins/render/frame/contributions.rs` exposes a portable
  fixed-capacity material binding table prepared-data contract plus a portable
  128-slot material texture resource table guard. This is the public runtime
  shape for the later true-bindless backend.
- `domain/material_graph/src/ir.rs` now orders executable IR nodes
  topologically by material input dependencies. Source graph order no longer
  controls whether a valid producer-after-consumer document can compile.
- `apps/runenwerk_editor/src/material_lab/state.rs` now projects the active
  `MaterialGraphDocument` source directly into Material Lab graph view models,
  including source-owned port ids, edges, viewport state, groups, node layout,
  resource refs, and palette-backed typed properties. Formed IR remains derived
  diagnostic/product state instead of the editor source projection.
- `apps/runenwerk_editor/src/material_lab/workflow.rs` refreshes the active
  source projection and re-ratifies/re-lowers after source-backed edits, undo,
  and redo so validation overlays are not delayed until the next preview build.
- `apps/runenwerk_editor/src/runtime/systems/material_preview.rs` no longer
  registers generated scene shaders under the global
  `editor_viewport_scene_product` id. The generated scene shader identity is
  carried as `PreparedSceneMaterialBundle` feature data instead, so the scene
  renderer must consume it through the material ABI rather than global shader
  replacement.
- `engine/src/plugins/render/renderer/render_flow/execute_passes.rs` now
  forbids material-feature passes from using builtin shader fallback and binds
  group-1 GPU-resident material texture/sampler resources for material-feature
  passes.
- `engine/src/plugins/render/renderer/prepare.rs` now creates prepared
  material resource bind groups for group 1 texture/sampler bindings. The
  pipeline layout keeps group indices stable even when a material-preview pass
  has no group 0 resources, and prepared texture allocations use the
  catalog-derived descriptor extent carried by
  `PreparedMaterialTextureBinding`.
- `engine/src/plugins/render/resource/dynamic_target.rs` and the render-flow
  capture path now round-trip dynamic target labels through debug capture
  selectors, so viewport/material product GPU proof captures can target the
  actual dynamic product surface instead of hardcoded pass labels.
- `apps/runenwerk_editor/src/material_lab/resource_resolution.rs` now resolves
  texture resource refs into ratified `domain/texture::TextureDescriptor`
  metadata plus artifact revision/cache/residency identity. This removes the
  prior string-only resolver shape and feeds descriptor extent into prepared
  renderer texture allocation.
- The gated GPU smoke captures the normal viewport scene/UI path and then
  proves generated scene material shaders plus generated material-preview
  shaders sample prepared group-1 KTX2 Texture2D and Texture3D bindings into
  visible pixels:
  `RUNENWERK_ENABLE_GPU_SMOKE=1 RUNENWERK_ENABLE_MACOS_MAIN_THREAD_GPU_SMOKE=1 cargo test -p runenwerk_editor --test viewport_gpu_truth_smoke -- --ignored --nocapture`.
- The closeout records the required proof paths, pixel deltas, and validation
  commands. Later work can improve visual graph-editor polish and broaden
  material catalog UX, but those are not hidden blockers for the WR-021 product
  spine evidence.

## Implementation Scope

Owning domain and crate boundaries:

- `domain/material_graph` owns source material graph documents, material graph
  ids, semantic ratification, deterministic lowering, formed material products,
  source maps, material cache keys, specialization fragments, and material
  product descriptors.
- `domain/asset` owns project source/catalog/artifact identity, import settings,
  deterministic import plans, diagnostics, ratification, and prior-valid
  preservation contracts. Serialized `ImportSettings::MaterialGraph` may carry
  project policy input, but it must not become the runtime material lowering
  recipe or learn renderer internals.
- `domain/editor/editor_shell` owns typed material view models, local material
  surface actions, and epoch-carrying command proposals. It must not perform IO,
  lowering, catalog mutation, renderer mutation, or ratification.
- `domain/editor/editor_viewport` owns typed viewport expression product kinds,
  product descriptors, presentation selection, and product-target contracts.
- `apps/runenwerk_editor` owns material workflow host IO, typed material recipe
  resolution, asset/material id bridging, catalog publication, preview
  orchestration, diagnostics mapping, prior-valid preservation, provider
  routing, and viewport product registry adapters.
- `engine/src/plugins/render` owns prepared material feature consumption, the
  material IR to WGSL compiler, generated shader validation, and preview render
  execution. It must not load, save, ratify, or lower material graphs from source
  documents.

Executed V1 implementation steps:

1. Add versioned material graph source persistence owned outside renderer code.
   The source document must round-trip a `material_graph::MaterialGraphDocument`
   with explicit versioning and deterministic source identity. Host IO belongs
   in the editor app or persistence layer, not in `domain/material_graph`.
2. Define the asset/material identity bridge. A material graph asset must map
   deterministically from `asset::AssetId` / `asset::AssetSourceId` to
   `material_graph::MaterialGraphDocumentId`. A formed material product must map
   deterministically to `material_graph::MaterialProductId`,
   `asset::AssetArtifactId`, catalog artifact payload, source lineage, and
   prior-valid preservation identity.
3. Add `apps/runenwerk_editor/src/material_lab/recipes.rs` as the typed material
   recipe boundary before lowering. Define `MaterialLoweringRecipe` with exactly
   two V1 variants: `PreviewMaterial` and `RenderMaterial`.
4. Define `ResolvedMaterialLoweringRecipe` as the runtime lowering contract used
   by `rebuild_material_preview_for_asset`. It owns material output target,
   material node catalog selection, stable cache-key component, expected
   artifact kind, specialization policy, and renderer-prepared parameter
   profile. Both V1 recipes use `MaterialNodeCatalog::first_slice()` until a
   later contract introduces multiple node catalogs.
5. Treat `ImportSettings::MaterialGraph { lowering_target }` as serialized
   project policy only. It must resolve to `ResolvedMaterialLoweringRecipe`
   before import ledger allocation or material graph lowering:
   - `"preview"` maps to `MaterialLoweringRecipe::PreviewMaterial` with
     `MaterialOutputTarget::PbrPreview`.
   - `"render_material"` maps to `MaterialLoweringRecipe::RenderMaterial` with
     `MaterialOutputTarget::RenderMaterial`.
   - Empty targets, unknown targets, mismatched source kinds, and any expected
     artifact target other than `AssetKind::Material` emit blocking
     `ImportProfileRejected` diagnostics.
6. Unknown, incompatible, or ambiguous material recipe resolution must stop
   before import ledger allocation, material graph lowering, catalog artifact
   publication, or renderer prepared-data publication.
7. Add `apps/runenwerk_editor/src/material_lab/` for material document IO,
   source/catalog identity bridging, lowering orchestration, diagnostics
   mapping, preview publication, and prior-valid preservation. This module owns
   app workflow state only; `domain/material_graph` remains semantic truth.
8. Route Material Lab through the `WR-026` project session and catalog adapters.
   Valid material previews publish formed material artifacts through guarded
   catalog/runtime barriers. Invalid material graphs or failed preview
   publication must record diagnostics and preserve the previous valid material
   artifact.
9. Extend `domain/editor/editor_shell` with typed material surface view models
   and actions for Material Graph Canvas, Material Inspector, and Material
   Preview. Every material workflow shell command must carry
   `projection_epoch`, and stale commands must fail closed.
10. Update `apps/runenwerk_editor/src/shell/providers/material_graph_canvas.rs`,
   `material_inspector.rs`, and `material_preview.rs` to render typed material
   projections and route actions through app-owned material workflow adapters.
   Existing status lines may remain as diagnostics only; they are not acceptance
   evidence.
11. Extend `domain/editor/editor_viewport` and
   `apps/runenwerk_editor/src/runtime/viewport/product_registry.rs` with a
   material preview expression product kind/target, and update viewport query
   snapshot authority classification so `DerivedMaterial` is treated as
   deterministic derived product authority. Material preview must be selectable
   through the existing viewport product selection path rather than merely
   appended to descriptor lists or shown in a parallel viewer.
12. Integrate renderer handoff through existing prepared-data contracts:
   `engine::plugins::render::frame::PreparedMaterialFeatureContribution`,
   `PreparedMaterialInstanceInput`,
   `engine::plugins::render::features::PreparedMaterialFeatureResource`, and
   `MATERIAL_RENDER_FEATURE_ID`. Renderer code consumes prepared material data
   only; material graph lowering and source truth stay outside renderer code.

Write-scope update completed before implementation:

- Add `engine/src/plugins/render`
- Add `domain/editor/editor_viewport`
- Add `apps/runenwerk_editor/src/material_lab`
- Add `apps/runenwerk_editor/src/material_lab/recipes.rs`
- Add `apps/runenwerk_editor/src/shell/providers`
- Add `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`
- Add `apps/runenwerk_editor/src/editor_app/state.rs`
- Add `docs-site/src/content/docs/reports/closeouts/wr-021-material-lab-and-material-preview-products/closeout.md`

Implementation started only after the write-scope update was recorded for the
accepted WR-021 scope.

Historical non-goals for the contract-writing slice:

- The contract-writing slice itself did not change product code; product code
  changed only during the subsequent accepted implementation.
- No canvas-local material truth.
- No renderer-owned material graph source, lowering, ratification, or catalog
  authority.
- No direct renderer mutation from providers.
- No catalog or product publication bypass.
- No status-panel-only Material Lab closeout.
- No string-only material lowering policy as runtime recipe authority.
- No inactive descriptor-only material preview that cannot be selected through
  the existing viewport product-selection path.
- No compile-failing implementation or placeholder closeout as completion
  evidence.
- No prefab material binding before prefab V2 has source/catalog identity.
- No shader authoring language, texture generation system, marketplace
  importer, or external material plugin framework.
- No roadmap completion update for `WR-021` or `PM-SDF-OW-001` before
  implementation validation and closeout evidence.

## Acceptance Criteria

- A source-backed material graph document can round-trip from project-owned
  storage and is addressable from a material graph asset source descriptor.
- Invalid material graphs block lowering with diagnostics and do not publish
  catalog artifacts or renderer data.
- Valid material graphs lower to `material_graph::FormedMaterialProduct` with
  deterministic cache keys, source maps, specialization fragments, and product
  descriptors.
- Material graph project import policy resolves into
  `ResolvedMaterialLoweringRecipe` before lowering; `"preview"` and
  `"render_material"` are the only accepted serialized V1 targets.
- Unknown, empty, mismatched-source, or non-`AssetKind::Material` recipe targets
  produce blocking `ImportProfileRejected` diagnostics before ledger allocation.
- Formed material products publish as catalog artifacts and runtime material
  products through the guarded app publication barrier.
- Failed material lowering or preview publication preserves the prior valid
  material artifact, exposes that preserved state in provider projections, and
  does not publish invalid renderer data.
- Material preview is selectable through viewport product selection and is not
  implemented as a parallel viewer.
- Selected-primary viewport presentation state drives `PrimaryColor` UI surface
  binding and selected-product residency/query intent. It must not retarget
  producer render-pass outputs.
- The scene pass always writes the scene-color dynamic target, and selecting a
  material preview does not bind the material target as a scene pass color
  attachment.
- Material preview pixels are produced through a dedicated material-preview
  render producer that consumes typed prepared material data and writes the
  `MaterialPreview2D` target. CPU pseudo-preview uploads are not acceptance
  evidence.
- Material providers expose typed controls/actions and reject stale projection
  epochs before mutating material workflow state.
- Renderer handoff consumes prepared material data through
  `PreparedMaterialFeatureContribution` / `PreparedMaterialFeatureResource`
  without becoming material graph truth, and the prepared parameter payload is
  versioned/profile-driven rather than debug-string-shaped.
- Material lowering cache keys include the stable `MaterialNodeCatalog`
  id/version, descriptor keys, and descriptor semantic versions, and are
  produced with canonical length-prefixed structured encoding.
- Assets with multiple material graph sources and no primary source produce a
  blocking diagnostic before import ledger allocation or publication.
- `cargo check -p runenwerk_editor` passes before closeout evidence is accepted.

## Stop Conditions

Stop implementation and return to planning if any of these happen:

- Material graph/canvas/provider/session state becomes semantic material truth.
- Renderer code starts owning material graph lowering, ratification, source
  document IO, or catalog identity.
- Formed material products cannot be mapped deterministically to both material
  product ids and asset artifact ids.
- Material import policy remains raw string runtime behavior instead of
  resolving into a typed material recipe.
- Viewport product selection can append a material descriptor but cannot make the
  selected material preview drive the rendered primary surface.
- Material prepared-data handoff depends on Rust debug formatting or an
  unversioned parameter payload.
- Material prepared-data decoding can allocate from an unbounded parameter count.
- Material cache identity omits node catalog id/version or can collide because
  graph/node/port names contain delimiter-like characters.
- Material node catalog construction silently overwrites duplicate descriptor
  keys or accepts invalid semantic identity.
- Material source fallback can choose among multiple material graph sources
  without an explicit primary source.
- Preview publication needs to mutate catalog/runtime/renderer state outside
  the guarded publication barrier.
- Stale shell commands can mutate material workflow state.
- Failed material preview cannot preserve the prior valid artifact.
- `ExpressionSourceRealityClass::DerivedMaterial` cannot be integrated into
  query snapshot authority classification as deterministic derived authority.
- Engine renderer files are required but roadmap write scopes have not been
  expanded, rendered, and validated.
- The implementation cannot provide real viewport product selection and would
  close with status-panel-only provider behavior.
- The implementation is not reconciled with the accepted WR-021 batch
  branch/worktree.
- `task planning:validate` does not pass before product-code implementation
  resumes or before closeout.

## Validation

Contract validation:

- `task production:plan -- --milestone PM-SDF-OW-001 --roadmap WR-021`
- `task planning:validate`
- `task docs:validate`

Future implementation validation:

- `cargo test -p ui_graph_editor`
- `cargo check -p runenwerk_editor`
- `cargo test -p material_graph`
- `cargo test -p editor_shell material`
- `cargo test -p editor_viewport material`
- `cargo test -p runenwerk_editor material`
- `cargo test -p runenwerk_editor viewport`
- `cargo test -p engine material_compiler`
- `cargo test -p runenwerk_editor material_recipe`
- `cargo test -p runenwerk_editor material_epoch`
- `cargo test -p runenwerk_editor material_handoff`
- `cargo test -p engine material_parameter_payload`
- `cargo test -p engine material`
- `cargo test -p runenwerk_editor material_viewport`
- Renderer/product handoff test proving prepared material data reaches
  `MATERIAL_RENDER_FEATURE_ID`.
- Material shell/provider tests proving provider-created material commands carry
  `projection_epoch` and stale material commands do not mutate workflow state.
- Tests proving scene color remains bound while selected, material preview binds
  only when selected, material preview selection does not retarget the scene
  render pass, the material-preview producer writes the material target,
  prepared material payloads are profile-versioned, unknown payload versions and
  oversized parameter counts are rejected, material cache keys ignore display
  labels, change on node catalog and descriptor semantic version changes,
  delimiter-like names cannot collide, duplicate catalog descriptor keys are
  rejected, and ambiguous material sources block before ledger allocation.
- `task docs:validate`
- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task production:validate`
- `task production:check`

## Closeout Requirements

The implementation closeout must be written to
`docs-site/src/content/docs/reports/closeouts/wr-021-material-lab-and-material-preview-products/closeout.md`
and must include:

- evidence that the implementation was reconciled with the accepted WR-021 batch
  branch/worktree and that the batch TOML advanced from
  `integration_status = "not_started"` / `closeout_status = "not_started"`;
- evidence that `cargo check -p runenwerk_editor` passed before closeout;
- exact source documents, modules, and command paths changed;
- evidence that material project import policy resolves through
  `apps/runenwerk_editor/src/material_lab/recipes.rs` into
  `ResolvedMaterialLoweringRecipe`, accepts only `"preview"` and
  `"render_material"` as V1 targets, and blocks unknown/empty/incompatible
  recipes before ledger allocation;
- evidence that source-backed material graph round trip passed;
- evidence that invalid graphs block lowering and preserve prior valid state;
- evidence that formed material products publish through catalog/runtime
  barriers;
- evidence that material preview is selectable through the existing viewport
  product-selection path, producer render pass outputs remain fixed, material
  preview has a dedicated product producer path, and `DerivedMaterial`
  participates in query snapshot authority classification;
- evidence that renderer handoff uses prepared material data only and reaches
  `MATERIAL_RENDER_FEATURE_ID`;
- evidence that material provider actions carry projection epochs and stale
  shell commands fail closed;
- validation command output for all required gates;
- roadmap and production evidence updates, followed by render/validate/check
  gates.
- `task planning:validate` output passing before implementation resume and
  before closeout.

The 2026-05-17 closeout landed this evidence and moved `WR-021` from
`current_candidate` to `completed`.
