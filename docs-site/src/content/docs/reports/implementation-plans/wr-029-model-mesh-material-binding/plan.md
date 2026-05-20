---
title: WR-029 Model Mesh Material Binding Implementation Contract
description: "Accepted implementation contract for model/mesh source identity, submesh/material-region assignment, renderable material-slot selection, and non-regression of the WR-028 SDF material path."
status: active
owner: apps/runenwerk_editor
layer: domain / app-runtime / editor-ui / engine-render
canonical: true
last_reviewed: 2026-05-20
related_designs:
  - ../../../design/active/material-lab-and-material-preview-design.md
  - ../../../design/active/editor-rendered-world-and-multi-entity-viewport-design.md
  - ../../../design/active/editor-asset-pipeline-and-content-workflow-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
related_reports:
  - ../wr-030-model-mesh-renderable-scene-contract/plan.md
  - ../../../reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/closeout.md
  - ../../../reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/proof-manifest.ron
---

# WR-029 Model Mesh Material Binding Implementation Contract

## Status

WR-029 is the accepted implementation contract for the model/mesh
material-binding gap that WR-028 intentionally did not close. Product code may
start only after the WR-029 roadmap row is promoted to an implementation-legal
state and the roadmap gates pass.

WR-028 is frozen as complete for the SDF primitive path. WR-029 must not reopen,
weaken, or rewrite the proven WR-028 SDF contracts unless a later accepted ADR
explicitly proves that a shared abstraction preserves the SDF behavior and proof
evidence.

## Goal

Make model/mesh renderables consume authored Material Lab assignments through
stable scene and asset source identity, with persistence, renderer handoff, and
pixel evidence. The completed slice must prove that a model or imported mesh
surface can resolve a source-backed material slot without using transient
runtime entity ids, renderer table indices, display names, generated artifact
ids, or UI selection state as authored truth.

## Scope

WR-029 owns:

- model/mesh source identity;
- imported mesh and foreign mesh artifact relationship;
- submesh/material-region identity;
- scene material assignment extension from SDF primitives to model/mesh
  renderables;
- renderer ABI extension for `renderable_index` or an equivalent typed model
  surface identity;
- material table lookup for model/mesh surfaces;
- proof fixture with at least one model/mesh renderable using a source-backed
  material slot assignment;
- explicit non-regression of the WR-028 SDF material path.

WR-029 does not own WR-028 SDF primitive acceptance, graph canvas completion,
Texture2D/Texture3D preview completion, or WR-028 proof manifest edits except
for documenting non-regression evidence.

## Source Of Truth

- `domain/editor/editor_scene` must remain the source owner for scene material
  assignments.
- Imported mesh assets and generated mesh artifacts must remain catalog-backed
  products; UI providers and renderer residency state must not become authored
  mesh or material truth.
- Renderer state may compact model/mesh material bindings into runtime table
  indices only as derived product state with explicit invalidation inputs.
- WR-028 SDF material assignment and generated WGSL behavior must remain
  non-regressed and separately testable.

Repository truth inspected for this contract:

- `domain/editor/editor_scene/src/model/material.rs` module
  `model::material`: owns `SceneMaterialAssignmentState`,
  `assign_sdf_primitive_material_slot`,
  `resolve_material_binding_for_sdf_scene_packet`, and
  `material_table_identity`; it currently has only SDF assignment identity.
- `domain/editor/editor_persistence/src/scene_file.rs` module `scene_file`:
  owns `SceneMaterialAssignmentsRecord` and
  `SdfPrimitiveMaterialSlotAssignmentRecord`; it currently persists only SDF
  primitive assignments.
- `domain/asset/src/kind.rs` module `kind`: owns `AssetKind`, including
  `ForeignMeshReferenceSource` and `ForeignMeshReferenceArtifact`; these are
  reference/import kinds, not primary world-authoring truth.
- `apps/runenwerk_editor/src/asset_pipeline/import_jobs.rs` module
  `asset_pipeline::import_jobs` and
  `apps/runenwerk_editor/src/asset_pipeline/import_profiles.rs` module
  `asset_pipeline::import_profiles`: own current foreign mesh import planning
  and profile defaults.
- `apps/runenwerk_editor/src/runtime/resources.rs` module `runtime::resources`
  and `apps/runenwerk_editor/src/runtime/systems/frame_submit.rs` module
  `runtime::systems::frame_submit`: currently prepare SDF scene packet material
  slot indices from app runtime state.
- `engine/src/plugins/render/material_compiler/wgsl/scene.rs` module
  `material_compiler::wgsl::scene`: currently emits the SDF scene material table
  selection path by `material_slot_index`.

## Architecture Decisions

## Architecture Governance Evidence

Governance review result, 2026-05-20: implement WR-029 through the accepted
contract and start with Phase 1 domain/persistence contracts before app UI or
renderer consumption.

- DDD owner: `domain/editor/editor_scene/src/model/material.rs` owns authored
  scene material assignments, including the new model/mesh source and region
  vocabulary.
- Clean Architecture boundary: `domain/editor/editor_scene` may define stable
  assignment identity and invariants; `domain/editor/editor_persistence` may
  serialize that source truth; `apps/runenwerk_editor` may translate workflow
  and catalog state; `engine/src/plugins/render` may consume derived typed
  transport only.
- ADR decision: no ADR is required for Phase 1 because the active design and
  this accepted implementation contract preserve the existing source-of-truth
  owner. Write an ADR before implementation only if source ownership,
  dependency direction, or renderer ABI semantics diverge from this contract.
- Strangler shape: add the model/mesh path beside the proven SDF assignment
  path, keep SDF tests as non-regression guards, and defer any generic material
  target abstraction until model/mesh parity is runtime-proven.
- Fitness functions: Phase 1 must add domain and persistence tests for stable
  model/mesh source identity, region assignment, save/reload, and material table
  identity before renderer or UI work proceeds.
- Ownership mode: stream-aligned Material Lab product workflow over
  `editor_scene` source truth, with enabling support from app runtime and the
  complicated engine render subsystem.

### DDD Owner And Vocabulary

The bounded context owner for authored material assignments is
`domain/editor/editor_scene/src/model/material.rs` module `model::material`.
WR-029 must add a model/mesh assignment vocabulary beside the existing SDF
vocabulary instead of generalizing SDF prematurely.

Required public vocabulary:

- `SceneModelMeshSourceId`: stable scene-side identity for a model/mesh
  source reference. It must be derived from source/catalog identity, never from
  an ECS entity, renderer residency handle, or generated artifact path alone.
- `SceneMeshMaterialRegionId`: stable identity for one assignable submesh or
  material region inside a model/mesh source. It must come from source metadata
  or importer-authored stable region keys.
- `SceneModelMeshMaterialRegionSourceId`: compound authored assignment target
  combining `SceneModelMeshSourceId` and `SceneMeshMaterialRegionId`.
- `SceneModelMeshMaterialSlotAssignment`: assignment from
  `SceneModelMeshMaterialRegionSourceId` to `SceneMaterialSlotId`.

These names may change during implementation only if the owning module records
an equivalent explicit vocabulary with the same invariants.

### Assignment Ownership

Model/mesh material assignments live in `SceneMaterialAssignmentState` in
`domain/editor/editor_scene/src/model/material.rs`, beside
`sdf_primitive_slots`. They are not owned by `apps/runenwerk_editor`, the
renderer, material graph compilation, or asset import artifacts.

The implementation should add a subdomain folder under
`domain/editor/editor_scene/src/model/material/` only if the material assignment
module becomes large enough to need `mod.rs` boundaries. A catch-all helper file
or `_internal` suffix is not allowed.

### Imported Mesh Identity

Foreign mesh import products remain catalog-backed references. WR-029 may use
`AssetKind::ForeignMeshReferenceSource` and
`AssetKind::ForeignMeshReferenceArtifact` as part of source lineage, but the
authored assignment target must resolve through a stable source reference and a
stable region key.

Acceptable region-key sources, in order:

- explicit source material slot id/name from the imported model when stable;
- importer-authored region key persisted in the source/reference metadata;
- deterministic importer fallback derived from source topology plus material
  slot ordinal, only when diagnostics mark it as weaker than source-authored
  identity.

Forbidden assignment truth:

- Bevy or editor ECS entity ids;
- renderer draw order, mesh table index, `renderable_index`, or residency slot;
- generated artifact id without source lineage;
- palette vector index;
- display label or UI selection id.

### Renderer ABI

The renderer ABI must carry a typed model/mesh surface material identity or a
derived compact index that is explicitly built from
`SceneModelMeshMaterialRegionSourceId`. A raw `renderable_index` may exist only
as a derived transport detail and must not be stored as authored truth.

SDF must keep the existing `sdf_primitive_index` and `material_slot_index` path
until model/mesh parity is proven. Any later unification requires an ADR or
accepted design update.

### Material Table Identity

`SceneMaterialAssignmentState::material_table_identity` in
`domain/editor/editor_scene/src/model/material.rs` must include model/mesh
assignment changes. The identity must change when the same model/mesh region is
assigned to a different `SceneMaterialSlotId`, and must stay stable across
save/reload when the source and region keys are unchanged.

### Prior-Valid Preservation

`apps/runenwerk_editor/src/runtime/systems/material_preview.rs` module
`runtime::systems::material_preview` and the relevant viewport preparation path
must preserve the prior-valid material/shader/mesh product when model/mesh
assignment resolution fails. Failures must emit diagnostics tied to the source
and region ids; they must not silently fall back to the default material as a
success-shaped outcome.

## ATAM-Lite Review

Quality attributes under tension:

- Source stability vs import compatibility: assignments must survive asset
  rebuilds, but imported formats may expose weaker material-region metadata.
- Renderer efficiency vs source clarity: runtime render packets may need compact
  indices, but authored truth must stay in scene/source ids.
- SDF non-regression vs shared abstraction: model/mesh support should avoid
  duplicate drift, but premature unification could weaken the proven WR-028 SDF
  path.
- UX immediacy vs diagnostic honesty: the Material Lab should feel direct, but
  missing region identity must block assignment with actionable diagnostics
  instead of pretending a default material is correct.

Candidate options:

- Option A, renderer-owned `renderable_index`: fastest to wire, rejected because
  it makes transient render order authored truth.
- Option B, generic scene material target abstraction now: attractive for code
  reuse, deferred because SDF and model/mesh identity have different source
  semantics and WR-028 must stay independently testable.
- Option C, model/mesh assignment path beside SDF with typed source and region
  ids: accepted for this contract because it preserves proven SDF behavior while
  adding the missing model/mesh chain.

Sensitivity points:

- Imported mesh region-key stability determines whether save/reload and asset
  rebuild proof can be honest.
- The renderer ABI must expose enough typed material-surface information to
  prove consumption without storing renderer-derived truth.
- Material table identity must include both SDF and model/mesh assignments or
  shader/material cache invalidation can become stale.

Risks:

- Existing foreign mesh reference metadata may not yet expose a region key; if
  so, Phase 2 must add source/reference metadata before any user assignment UI.
- Fixed mesh render passes may live outside the SDF material compiler path; the
  implementation must name and test the real consuming pass in closeout.
- A broad generic material-target abstraction could hide SDF regressions unless
  introduced after model/mesh parity evidence exists.

Non-risks:

- Material graph authoring, Texture2D/Texture3D preview, and WR-028 SDF
  material assignment are already closed for WR-028 scope and are not reopened
  by this contract.
- Renderer compact tables are acceptable as derived runtime products when their
  invalidation inputs include scene-owned model/mesh assignments.

Decision:

Adopt Option C. Add model/mesh source and material-region assignments beside
the existing SDF assignment path, use typed source/region ids as authored truth,
derive any renderer transport index from those ids, and keep SDF tests as
mandatory non-regression guards.

## Implementation Scope

Phase 0, acceptance and guard setup:

- Promote this contract through the roadmap gate; replace it with an accepted
  ADR only if later review changes source-truth ownership, dependency
  direction, or renderer ABI beyond this contract.
- Add failing guard tests that name the accepted identity and forbid raw
  `renderable_index` authored truth.

Phase 1, domain and persistence contracts:

Status 2026-05-20: landed. `editor_scene` owns typed model/mesh source and
material-region assignment vocabulary, `editor_persistence` persists and
normalizes it, and `runenwerk_editor` scene-file roundtrip code bridges it into
runtime state. Focused tests cover save/reload, table identity changes, and
transient identity rejection.

- Extend `domain/editor/editor_scene/src/model/material.rs` module
  `model::material` with model/mesh source, region, compound target, and
  assignment types.
- Extend `SceneMaterialAssignmentState` with model/mesh assignment storage,
  assignment, resolution, diagnostics, and material table identity.
- Extend `domain/editor/editor_persistence/src/scene_file.rs` module
  `scene_file` with `model_mesh_assignments` records beside
  `sdf_primitive_assignments`.
- Add save/reload and normalization tests proving stable region ids survive
  persistence and reject transient ids.

Phase 2, asset and editor workflow translation:

Status 2026-05-20: landed for the app workflow boundary. `domain/asset` now
attaches foreign mesh material-region descriptors to foreign reference
artifacts, `apps/runenwerk_editor/src/asset_pipeline/import_jobs.rs` publishes a
stable source material-slot region for foreign mesh artifacts, and
`apps/runenwerk_editor/src/material_lab` resolves catalog regions into
`SceneModelMeshMaterialRegionSourceId` before updating scene-owned assignments.
`domain/editor/editor_shell/src/surfaces/material.rs` and
`domain/editor/editor_shell/src/composition/build_material_graph_surface.rs`
expose the typed Material Lab action and route from model/mesh regions to scene
material slots. This phase still does not claim renderer/GPU completion.

- Add an asset-domain foreign mesh material-region descriptor contract before
  app import adapters expose assignable regions. Source-authored and
  importer-authored keys are strong identity; deterministic topology fallback
  keys are allowed only when the descriptor marks that a weak-identity
  diagnostic is required.
- Teach `apps/runenwerk_editor/src/asset_pipeline/import_jobs.rs` module
  `asset_pipeline::import_jobs` and `import_profiles.rs` module
  `asset_pipeline::import_profiles` to expose stable imported mesh region keys
  or to surface a blocking diagnostic when a source cannot provide them.
- Add Material Lab assignment workflow tests under
  `apps/runenwerk_editor/src/material_lab` for selecting a model/mesh material
  region and assigning an existing material slot.

Phase 3, runtime and renderer consumption:

Status 2026-05-20: landed for typed prepared transport and cache
invalidation. `engine/src/plugins/render/frame/contributions.rs` module
`frame::contributions` owns `PreparedModelMeshMaterialSourceIdentity`,
`PreparedModelMeshMaterialRegionIdentity`, and
`PreparedModelMeshMaterialSelection`, validates portable limits, rejects
transient renderer identity, and rejects duplicate source-backed regions.
`engine/src/plugins/render/renderer/prepare.rs` module `renderer::prepare`
includes those selections in the prepared feature contribution hash.
`apps/runenwerk_editor/src/runtime/resources.rs` module `runtime::resources`
derives `EditorViewportModelMeshMaterialSelectionPacket` from
`SceneMaterialAssignmentState`.
`apps/runenwerk_editor/src/runtime/systems/frame_submit.rs` module
`runtime::systems::frame_submit` records the packet on
`EditorViewportRenderState`, and
`apps/runenwerk_editor/src/runtime/systems/material_preview.rs` module
`runtime::systems::material_preview` applies the packet to
`PreparedMaterialFeatureResource.payload.model_mesh_material_selections`.
This phase still does not claim visible model/mesh GPU pixel completion; that
remains Phase 4.

- Extend `apps/runenwerk_editor/src/runtime/resources.rs` module
  `runtime::resources` and
  `apps/runenwerk_editor/src/runtime/systems/frame_submit.rs` module
  `runtime::systems::frame_submit` so prepared model/mesh render packets carry
  the derived material slot selection from scene-owned source identity.
- Extend the engine render path under `engine/src/plugins/render` with a typed
  model/mesh material selection transport. If the final implementation must
  touch fixed mesh passes rather than `material_compiler::wgsl::scene`, record
  the exact module in the closeout.

Phase 4, proof and closeout:

Status 2026-05-20: open until the renderer's model/mesh pass consumes the
prepared model/mesh selections for visible pixels and GPU/manual proof captures
the selected material on a model/mesh surface.

Preflight evidence 2026-05-20: `engine/src/plugins/render/inspect/pass_provenance.rs`
module `inspect::pass_provenance` now exposes
`RenderPassMaterialBindingEvidence` on each `RenderPassProvenanceRecord`, and
`engine/src/plugins/render/renderer/render_flow/provenance.rs` module
`renderer::render_flow::provenance` fills that evidence through
`collect_pass_material_binding_evidence`. Material-consuming passes now report
the generated scene shader identity, material table identity, material binding
slot count, and exact source-backed model/mesh selections available to that
pass. This is pass-level transport/inspection evidence only; it does not close
Phase 4 until a real model/mesh pass produces changed pixels from the selected
table entry.

Material-region render packet evidence 2026-05-20:
`apps/runenwerk_editor/src/runtime/resources.rs` module `runtime::resources`,
method `EditorViewportRenderState::compose_scene_product_uniform`, appends
bounded `model_mesh_flags`, `model_mesh_region_rects`, and
`model_mesh_region_flags` lanes derived from
`EditorViewportModelMeshMaterialSelectionPacket`. The helper
`write_model_mesh_material_region_slots` maps each prepared source-backed
selection to a deterministic viewport material-region surface without using
authored renderer indices. `engine/src/plugins/render/material_compiler/wgsl/scene.rs`
module `material_compiler::wgsl::scene` emits `model_mesh_region_at` and
`shade_model_mesh_region`, and generated scene material-table shaders call
`evaluate_scene_material(hit.material_slot_index)` for those source-backed
regions before falling through to the existing SDF path. Static viewport shaders
under `assets/shaders` keep the uniform ABI aligned; the fallback scene shader
draws deterministic material-region colors when the generated material table is
not active. Focused tests cover the CPU packet, generated shader path, generated
scene material-table retention, static shader ABI, and WGSL validation. This is
selected-material pixel preflight for the prepared model/mesh material-selection
lane only. It does not prove imported mesh geometry extraction, a real mesh draw
pass, model/mesh picking, or a source-backed model/mesh entity contract.

Blocking review 2026-05-20: the active rendered-world design still excludes
general mesh scene extraction, and `domain/editor/editor_persistence/src/scene_file.rs`
module `scene_file` persists authored scene entities as SDF primitive records.
The next long-term move is the `WR-030` draft contract for a real source-backed
model/mesh renderable scene contract or Mesh Preview product surface. WR-029
Phase 4 must not use the current SDF pass, a status panel, or a descriptor-only
proof as a substitute for model/mesh pixels.

- Add a model/mesh fixture with at least two material regions or one
  source-backed material slot assignment whose material change alters visible
  pixels.
- Capture runtime/GPU proof that the model/mesh pass consumes the selected
  material table entry.
- Re-run WR-028 SDF non-regression proof so model/mesh support does not claim
  success by weakening the already-proven SDF path.

## Acceptance Criteria

The WR-029 closeout must include:

- save/reload proof for model/mesh material assignments;
- asset rebuild proof showing assignments survive through stable source
  references;
- renderer ABI proof for `renderable_index` or the accepted equivalent;
- one model/mesh renderable fixture with at least two material regions or one
  source-backed material slot assignment that changes visible pixels;
- material table identity proof when model/mesh assignments change;
- pass-consumption proof that the renderer pass producing model/mesh pixels uses
  the selected material table entry;
- non-regression proof that WR-028 two-SDF-primitives material-slot selection
  still passes.

The implementation is not complete if the user-facing Material Lab can assign
materials only in descriptor/status panels without a rendered model/mesh pixel
change.

## Anti-Drift Guards

- Fail if WR-029 claims completion without a stable model/mesh source identity.
- Fail if submesh/material-region assignments use raw renderer indices as
  authored truth.
- Fail if imported mesh artifact ids are assigned directly by users without a
  stable source reference.
- Fail if `renderable_index` is added as an untyped renderer-only escape hatch.
- Fail if model/mesh material table identity ignores assignment changes.
- Fail if model/mesh proof uses descriptor/status text instead of scene pixels.
- Fail if WR-029 changes WR-028 SDF material behavior without explicit
  non-regression proof.

## Stop Conditions

Stop and return to architecture review if:

- imported mesh sources cannot expose a stable region key;
- the only available renderer handoff is a raw renderer index with no typed
  source-to-derived mapping;
- model/mesh products cannot preserve prior-valid output with diagnostics when
  assignment resolution fails;
- implementation requires moving material assignment ownership out of
  `domain/editor/editor_scene`;
- SDF material assignment tests or generated scene WGSL material slot tests
  regress.

## Closeout Requirements

Closeout must update:

- `docs-site/src/content/docs/reports/closeouts/wr-029-model-mesh-material-binding/closeout.md`
  with the implemented source-to-GPU chain, exact test results, and any known
  quality gaps.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` and generated
  roadmap docs after evidence changes.
- `docs-site/src/content/docs/workspace/production-tracks.yaml` and generated
  production docs only if production milestone quality gaps or acceptance
  criteria change.
- `docs-site/src/content/docs/design/active/material-lab-and-material-preview-design.md`
  only if the accepted ownership or ABI differs from this contract.

Expected completion quality is `runtime_proven` unless the closeout includes a
completed perfectionist audit with no known UI, architecture, documentation,
renderer, or proof gaps. `perfectionist_verified` requires a named audit path,
model/mesh GPU pixel proof, SDF non-regression proof, and no remaining quality
gaps.

## Critical Review Gate

Before code changes begin, review this contract against the easy failure mode:
adding a model/mesh dropdown that stores a renderer index and never proves
render consumption. The implementation must instead prove this chain:

`catalog/source identity -> stable mesh region key -> editor_scene assignment -> persistence -> runtime prepared packet -> renderer material table lookup -> visible model/mesh pixels`

If any link is still a design choice rather than a decision, keep WR-029 in
planning and update this contract or write an ADR.

## Initial Validation Targets

Exact package and test names must be confirmed during WR-029 Phase 0. The
minimum target set is:

- `cargo test -p editor_scene model_mesh_material_assignment_survives_save_reload`
- `cargo test -p editor_scene model_mesh_material_table_identity_changes_when_assignment_changes`
- `cargo test -p asset foreign_mesh`
- `cargo test -p editor_persistence model_mesh_material_assignment_rejects_transient_identity`
- `cargo test -p runenwerk_editor model_mesh_assignment_survives_asset_rebuild`
- `cargo test -p engine model_mesh_surface_material_selection`
- `cargo test -p engine material_pass_provenance_exposes_model_mesh_selection_table`
- `cargo test -p engine non_material_pass_provenance_keeps_model_mesh_selection_out_of_pass_scope`
- `cargo test -p engine generated_scene_wgsl_renders_model_mesh_regions_from_material_selection_lane`
- `cargo test -p engine scene_material_table_wgsl_dispatches_to_source_backed_slot_evaluators`
- `cargo test -p runenwerk_editor model_mesh_renderable_uses_source_backed_material_slot`
- `cargo test -p runenwerk_editor model_mesh_material_selection_packet_serializes_scene_product_uniform_regions`
- `cargo test -p runenwerk_editor model_mesh_material_selection_packet_reaches_material_feature_payload`
- `cargo test -p runenwerk_editor viewport_wgsl_shaders_parse_and_validate`
- `cargo test -p runenwerk_editor sdf_two_primitives_render_different_material_slots`
- `cargo check -p runenwerk_editor`
- `task docs:validate`
- `task roadmap:validate`
- `task production:validate`
- `task planning:validate`
