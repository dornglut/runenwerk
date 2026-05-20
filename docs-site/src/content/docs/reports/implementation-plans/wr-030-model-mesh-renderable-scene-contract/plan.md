---
title: WR-030 Model Mesh Renderable Scene Contract
description: "Implementation contract for the source-backed model/mesh preview and renderable surface path required before WR-029 Phase 4 GPU pixel proof."
status: active
owner: apps/runenwerk_editor
layer: domain / app-runtime / engine-render
canonical: true
last_reviewed: 2026-05-20
related_designs:
  - ../../../design/active/material-lab-and-material-preview-design.md
  - ../../../design/active/editor-rendered-world-and-multi-entity-viewport-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
related_reports:
  - ../wr-029-model-mesh-material-binding/plan.md
---

# WR-030 Model Mesh Renderable Scene Contract

## Status

WR-030 is the prerequisite contract for the WR-029 Phase 4 model/mesh pixel
proof. It exists because the active rendered-world design intentionally renders
authored SDF primitives only, while WR-029 now has source-backed model/mesh
material assignments, prepared renderer transport, pass provenance, and a
material-region shader lane.

This contract does not complete WR-029 by itself. Product code may claim WR-029
Phase 4 only after a source-backed model/mesh surface produces visible pixels
from the selected material table entry and the proof names the exact consuming
renderer module.

## Goal

Provide a source-backed model/mesh surface contract that Material Lab can use in
the editor workspace and standalone Material Lab workbench to prove model/mesh
material assignment on rendered pixels.

The first implementation surface is a Mesh Preview product surface owned by
Material Lab workflow, not general editor-scene mesh extraction. This preserves
the existing SDF-first rendered-world contract while creating a real
model/mesh-oriented proof path.

## Architecture Governance Evidence

Governance review result, 2026-05-20: implement WR-030 through a Mesh Preview
product surface first. Do not add general mesh scene extraction, renderer-owned
world truth, or raw renderer-index material identity as a shortcut.

- DDD owner: `domain/editor/editor_scene/src/model/material.rs` remains the
  owner of authored material assignment identity. `domain/asset/src/foreign_mesh.rs`
  owns foreign mesh material-region descriptors. `apps/runenwerk_editor` owns
  catalog-to-workflow projection and preview extraction. `engine/src/plugins/render`
  owns prepared render execution and pass evidence only.
- Clean Architecture boundary: domain crates define stable identity and
  invariants; apps translate catalog/runtime state; engine consumes prepared
  products. Engine render code must not depend on editor app concepts, and
  domain code must not depend on renderer residency, draw order, or UI state.
- ADR decision: no ADR is required for the Mesh Preview product surface because
  it preserves existing source ownership and dependency direction. Write an ADR
  before adding general authored mesh scene extraction, changing renderer ABI
  semantics beyond prepared products, or making mesh products authoritative
  world truth.
- ATAM-lite: WR-030 trades completeness of general mesh scene rendering for
  honest Material Lab proofability. A Mesh Preview product gives source-backed
  model/mesh pixels with bounded blast radius; general scene extraction remains
  a later design.
- Strangler shape: add Mesh Preview beside the existing SDF scene path. Do not
  replace `EditorViewportSceneRenderPacket` or the WR-028 SDF material path.
- Ownership mode: stream-aligned Material Lab product workflow with enabling
  support from app runtime and engine render.

## Required Vocabulary

WR-030 reuses existing source-backed material vocabulary where possible:

- `SceneModelMeshSourceId`
- `SceneMeshMaterialRegionId`
- `SceneModelMeshMaterialRegionSourceId`
- `SceneModelMeshMaterialSlotAssignment`
- `PreparedModelMeshMaterialSelection`

New code may add Mesh Preview vocabulary only where it names preview product
responsibility, such as:

- model/mesh preview source selection;
- preview material-region packet;
- preview render target or product descriptor;
- pass-consumption evidence for model/mesh preview pixels.

Do not introduce `renderable_index`, draw order, residency slot, UI row index,
or display label as source truth.

## Implementation Phases

Phase 0, contract and roadmap readiness:

- Keep WR-029 Phase 4 open.
- Update roadmap evidence so WR-030 is the visible prerequisite for model/mesh
  pixel proof.
- Keep the active rendered-world design explicit that general mesh scene
  extraction is still excluded from V1.

Phase 1, Mesh Preview product surface contract:

- Add an app-owned Material Lab Mesh Preview projection that selects one
  catalog-backed model/mesh asset and its assignable material regions.
- The projection must resolve regions through
  `CatalogModelMeshMaterialRegion::scene_material_region`, not through artifact
  ids, renderer state, display labels, or UI selection ids.
- The standalone Material Lab workbench must be able to host the same projection
  as the full editor workspace.

Phase 2, prepared preview render packet:

- Extend `apps/runenwerk_editor/src/runtime/resources.rs` module
  `runtime::resources` with a bounded Mesh Preview packet that carries the
  source-backed material-region selections and preview layout.
- The packet must be derived from `SceneMaterialAssignmentState` and catalog
  descriptors, and must preserve prior-valid preview product state with
  diagnostics when model/mesh resolution fails.

Phase 3, renderer consumption and proof:

- Use a material-consuming renderer pass that resolves the selected
  `PreparedModelMeshMaterialSelection.material_table_index` through the scene
  material table.
- The proof must show changed pixels for at least one source-backed model/mesh
  material region when the assigned Material Lab slot changes.
- Pass provenance must name the material table identity, selected model/mesh
  region, shader bundle identity, and consuming pass.

Phase 4, WR-029 closeout handoff:

- Re-run WR-028 SDF material non-regression proof.
- Update WR-029 closeout evidence only after Mesh Preview GPU/manual proof is
  captured.
- Leave general authored mesh scene extraction outside WR-030 unless a later
  design or ADR accepts it.

## Implementation Evidence

2026-05-20, Phase 1/2 projection and packet readiness slice:

- `domain/editor/editor_shell/src/surfaces/material.rs` module
  `surfaces::material` now exposes a typed
  `MaterialModelMeshPreviewViewModel` under `MaterialPreviewViewModel`.
- `apps/runenwerk_editor/src/material_lab/state/model_mesh_preview_projection.rs`
  function `material_model_mesh_preview_view_model` projects catalog-backed
  model/mesh regions through stable source identity and scene material
  assignment resolution.
- `apps/runenwerk_editor/src/material_lab/state/preview_status.rs` method
  `MaterialLabRuntime::preview_view_model_with_scene_material_assignments`
  binds the projection into the existing Material Preview state contract.
- `apps/runenwerk_editor/src/shell/providers/material_preview.rs` function
  `MaterialPreviewProvider::build_frame` renders the Mesh Preview projection
  in the same provider used by the full editor and standalone Material Lab
  workbench.
- `apps/runenwerk_editor/src/runtime/resources.rs` module
  `runtime::resources` has focused packet coverage proving source-backed
  material-region identity reaches `PreparedModelMeshMaterialSelection` without
  transient `renderable_index` identity.

This evidence does not satisfy the Phase 3 visible-pixel proof. WR-029 Phase 4
must remain open until the Mesh Preview consuming render pass changes pixels
from a source-backed model/mesh material selection and names the pass evidence.

## Acceptance Criteria

- Material Lab full editor workspace exposes a model/mesh preview surface or
  equivalent provider-owned product view.
- Standalone Material Lab workbench exposes the same model/mesh preview surface.
- Preview source identity comes from catalog/source identity and stable material
  region keys.
- Assigning a Material Lab scene material slot to a model/mesh region changes
  visible model/mesh preview pixels.
- The consuming render pass evidence includes material table identity and the
  exact source-backed model/mesh material selection.
- WR-028 SDF two-primitive material-slot selection still passes.
- No authored or persisted model/mesh material truth uses renderer draw order,
  renderer indices, UI indices, or display names.

## Failure Conditions

- Fail if proof uses status text, descriptor rows, or SDF primitives instead of
  model/mesh preview pixels.
- Fail if Material Lab workbench cannot open the model/mesh preview path.
- Fail if renderer state becomes authored scene truth.
- Fail if general mesh scene extraction is added without updating the rendered
  world design and running architecture governance.
- Fail if WR-029 is marked complete without WR-030 proof evidence.

## Initial Validation Targets

- `cargo test -p runenwerk_editor material_preview_model_mesh_projection_uses_source_backed_regions`
- `cargo test -p runenwerk_editor material_preview_provider_renders_source_backed_model_mesh_preview_surface`
- `cargo test -p runenwerk_editor material_lab_workbench_preview_uses_model_mesh_preview_projection`
- `cargo test -p runenwerk_editor model_mesh_preview_packet_uses_source_backed_material_regions`
- `cargo test -p runenwerk_editor model_mesh`
- `cargo test -p runenwerk_editor material_lab_workbench`
- `cargo test -p runenwerk_editor model_mesh_material_selection_packet_serializes_scene_product_uniform_regions`
- `cargo test -p runenwerk_editor model_mesh_material_selection_packet_reaches_material_feature_payload`
- `cargo test -p engine generated_scene_wgsl_renders_model_mesh_regions_from_material_selection_lane`
- `cargo test -p engine material_pass_provenance_exposes_model_mesh_selection_table`
- `cargo test -p runenwerk_editor sdf_two_primitives_render_different_material_slots`
- `cargo check -p runenwerk_editor`
- `task docs:validate`
- `task roadmap:validate`
