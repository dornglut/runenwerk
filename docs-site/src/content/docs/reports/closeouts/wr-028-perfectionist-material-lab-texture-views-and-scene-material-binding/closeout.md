---
title: WR-028 Perfectionist Material Lab Texture Views And Scene Material Binding Closeout
description: "Closeout for the SDF-scoped WR-028 repair: source-backed material graph edits, graph canvas UX, live texture inspection, editor-scene SDF material assignments, generated SDF scene material selection, and GPU proof."
status: completed
owner: apps/runenwerk_editor
layer: domain / app-runtime / editor-ui / engine-render
canonical: true
last_reviewed: 2026-05-19
related_designs:
  - ../../../design/active/material-lab-and-material-preview-design.md
  - ../../../design/active/editor-rendered-world-and-multi-entity-viewport-design.md
  - ../../../design/active/editor-asset-pipeline-and-content-workflow-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
related_reports:
  - ../../../reports/implementation-plans/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/plan.md
  - ../../../reports/implementation-plans/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/phase-0-governance-note.md
  - ./proof-manifest.ron
---

# WR-028 Perfectionist Material Lab Texture Views And Scene Material Binding Closeout

## Status

Complete as of 2026-05-19 for the WR-028 SDF primitive scope after the
source-to-GPU, graph-canvas, and source-backed scene material table repair
passes.

WR-028 is deliberately not a model/mesh material-binding closeout. It closes the
SDF primitive path: source-backed Material Graph V2 editing, real retained graph
canvas projection and routed UX, catalog-backed live texture preview products,
editor-scene owned SDF material assignments, generated SDF scene WGSL
material-slot table dispatch, renderer bundle consumption, and GPU/manual proof
evidence. `WR-029` owns model/mesh renderable identity, submesh/material-region
assignment, and any future `renderable_index` ABI extension.

## Scope Closed

- `domain/material_graph/src/persistence.rs::MaterialGraphSourceFileV2` remains
  the material graph source file truth and now round-trips source-owned layout.
- `domain/material_graph/src/commands.rs` owns material graph source mutation
  commands for node move/connect/disconnect/property edits and texture refs.
- `domain/ui/ui_graph_editor`, `domain/ui/ui_tree`, `domain/ui/ui_runtime`, and
  `domain/ui/ui_render_data` provide the semantic-free graph canvas substrate.
- `domain/editor/editor_shell/src/composition/build_material_graph_surface.rs`
  composes a toolbar, real graph canvas, inspector, palette, node-picker popup,
  undo/redo controls, and diagnostic navigation buttons instead of label/list
  completion evidence.
- `apps/runenwerk_editor/src/material_lab/workflow.rs` routes graph edits,
  texture picks, node-picker confirmation, undo/redo, and diagnostic navigation
  through source-backed workflow state.
- `apps/runenwerk_editor/src/material_lab/state.rs` parses stable diagnostic
  subjects such as `material_graph.node:<id>` and
  `material_graph.port:<id>` into graph canvas overlay anchors.
- `domain/editor/editor_scene/src/model/material.rs` owns SDF material palette,
  assignment state, resolver behavior, diagnostics, and material table identity.
- `domain/editor/editor_persistence/src/scene_file.rs` persists
  `SceneFileV2` material assignments. Authored scene material slots persist
  stable slot/source references only; formed material artifact ids, shader
  artifact ids, cache keys, renderer table indices, and prior-valid runtime
  hints are not authored scene truth.
- `apps/runenwerk_editor/src/material_lab/state.rs` no longer owns authored SDF
  primitive material assignment truth.
- `apps/runenwerk_editor/src/runtime/systems/frame_submit.rs` extracts SDF
  primitive material slots from editor-scene source assignment state.
- `engine/src/plugins/render/material_compiler/wgsl/scene.rs` carries
  `sdf_primitive_index` and `material_slot_index` through raymarching and calls
  `evaluate_scene_material(march.material_slot_index, ...)`. The SDF material
  slot is carried in the typed `primitive_slot_flags.w` `u32` lane, not through
  `params_b.z` as an untyped `f32`.
- `engine/src/plugins/render/material_compiler::compile_scene_material_table_shader`
  compiles ordered source-backed scene material slots into one WGSL evaluator
  per material slot and dispatches with `switch material_slot_index`. This table
  path no longer treats `material_channel` branching as proof of source-backed
  material selection.
- `apps/runenwerk_editor/src/material_lab/renderer_handoff.rs` resolves
  editor-scene material slots to prepared source-backed material products before
  renderer handoff. Missing non-default slot products fail closed with a
  diagnostic instead of silently binding the active preview material.
- `apps/runenwerk_editor/src/runtime/systems/material_preview.rs` collects
  retained Material Lab preview products for scene material slots and registers
  their scene shaders before publishing the prepared material contribution.
- `apps/runenwerk_editor/src/texture_preview/mod.rs` prepares live Texture2D and
  Texture3D preview products from catalog-backed `TextureProduct` or
  `GeneratedTextureProduct` payloads.
- `engine/src/plugins/render/texture_upload.rs` shares the material KTX2 upload
  validator between material rendering and texture preview proof.
- `domain/ui/ui_tree/src/tree/node.rs::ProductSurfaceNode` is presentation-only:
  it displays a derived product-surface binding and owns no texture artifact,
  material source, or preview proof truth.

## Proof Summary

The proof manifest is
`docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/proof-manifest.ron`.

SDF assignment/render proof:

- `cargo test -p editor_scene material` proves material assignment contracts and
  persistence behavior.
- `cargo test -p engine scene_material_table_wgsl_dispatches_to_source_backed_slot_evaluators`
  proves scene material table WGSL emits distinct per-slot evaluators and
  dispatches from `material_slot_index` without rewriting `material_channel` as
  the proof mechanism.
- `cargo test -p runenwerk_editor material_binding_table_uses_resolved_source_backed_slot_products`
  proves two scene material slots produce distinct prepared material instances,
  shader/cache identities, and binding-table rows from resolved source-backed
  products.
- `cargo test -p runenwerk_editor unresolved_source_backed_scene_material_slot_fails_closed`
  proves a non-default source-backed slot with no prepared product is rejected
  with a diagnostic before renderer handoff.
- `cargo test -p runenwerk_editor sdf_two_primitives_render_different_material_slots`
  proves two authored SDF primitives reach viewport extraction with distinct
  material slots.
- `cargo test -p engine generated_scene_wgsl_selects_material_from_hit_sdf_slot`
  proves generated scene WGSL selects material output from the hit SDF slot and
  does not satisfy acceptance through a global-material path.
- `cargo test -p runenwerk_editor wr021_material_product_spine_runtime_boundaries_are_consumed`
  and render-flow provenance checks prove generated scene bundles are consumed by
  the scene pixel pass.
- The manual/windowed GPU smoke now records explicit `viewport.scene_color`
  samples for two SDF primitives in the same scene: entity `201` slot `0` at
  pixel `(382, 191)` renders `[255, 64, 42, 255]`, and entity `202` slot `1` at
  pixel `(553, 259)` renders `[40, 250, 124, 255]`. Background pixel `(16, 16)`
  remains `[85, 89, 97, 255]`.

Graph canvas proof:

- The Material Graph Canvas emits a real graph canvas surface with nodes, typed
  ports, edges, overlays, selection, toolbar, inspector, palette, diagnostics,
  source-owned layout, and epoch-guarded workflow actions.
- Focused graph-canvas shortcuts now route through the UI runtime and editor
  shell: `A` opens the node picker, `Delete` deletes selection, `Ctrl+Z` undo,
  `Ctrl+Y` redo, `Ctrl+B` builds the selected preview, and `F` focuses the
  material preview.
- Diagnostics are actionable buttons wired to
  `MaterialSurfaceAction::NavigateDiagnostic`; navigation selects the diagnostic
  subject, centers the source document viewport on the node or owning node, and
  marks the graph overlay active.
- Generic graph substrate crates do not depend on `material_graph`, and the
  substrate owns only view, gesture, input, layout, clipping, and primitive
  emission behavior.

Source-backed material graph GPU proof:

- The manual GPU smoke now exercises a durable
  `MaterialGraphSourceFileV2` source edit for Texture2D and Texture3D material
  graphs. The path is: source file fixture -> source reload ->
  `MaterialSurfaceAction::PickTextureResource` through
  `RunenwerkEditorApp::apply_material_surface_action` -> material graph
  lowering -> formed product -> generated preview/scene WGSL -> catalog
  resource resolution -> material table identity -> preview and scene captures.
- Texture2D source edit identity hashes:
  product `ba991b1688a7228e -> c24860ed3da84a7c`, preview shader
  `208378d70d56dd94 -> 13ff683d908c7c8c`, scene shader
  `fb9f73e30eda3462 -> c6694b7a91e54b7a`, material table
  `9afc2c6069cf05f5 -> 078caea1825fea4c`.
- Texture2D GPU pixels:
  scene capture
  `artifacts/captures/frame_18__flow_1__pass_2__stage_after__resource_runenwerk_editor_viewport_editor_viewport_2_1_primary.png`
  hash `blake3:ca33608b16beda75df6ed512096597402c152f60e947c5b7c8dca411e5b42849`
  has scene sample `[109, 211, 164, 255]` at `(385, 183)`;
  preview capture
  `artifacts/captures/frame_18__flow_2__pass_1__stage_after__resource_runenwerk_editor_viewport_editor_viewport_2_10070_primary.png`
  hash `blake3:e0d6b8d4442342f42e9a55237b6c900ba862387125f7b59cbaf7a3f34136a440`
  has preview sample `[105, 205, 158, 255]`.
- Texture3D source edit identity hashes:
  product `0bbeeaa40f72ec1d -> 9f785cec43905219`, preview shader
  `9d6a9f6dedbc46d9 -> 6ccdcd3ccade403d`, scene shader
  `47b3c690a184f4a9 -> 52553d6d3c57d32c`, material table
  `2ba67a7d2cea8cf7 -> dc1dbd754c7d1dbf`.
- Texture3D GPU pixels:
  scene capture
  `artifacts/captures/frame_31__flow_1__pass_2__stage_after__resource_runenwerk_editor_viewport_editor_viewport_2_1_primary.png`
  hash `blake3:5f3673fe358e4c2bc00d50f0842661ce797744ce85c9c2a825073277805dd0b4`
  has scene sample `[212, 146, 240, 255]` at `(381, 181)`;
  preview capture
  `artifacts/captures/frame_31__flow_2__pass_1__stage_after__resource_runenwerk_editor_viewport_editor_viewport_2_10071_primary.png`
  hash `blake3:d75912fe90fe7b9d278d97829c69e83a55f57d214d60c2a544b9f96d3394cad0`
  has preview sample `[207, 141, 233, 255]`.

Texture preview proof:

- Texture2D and Texture3D preview providers emit `ProductSurfaceNode` UI, not
  descriptor-only panels, and the GPU proof now exercises that provider/product
  surface path directly.
- The proof route is:
  `TextureViewerProvider` or `VolumeTextureViewerProvider` -> catalog
  `TextureProduct` or `GeneratedTextureProduct` -> `prepare_texture_preview` ->
  `ProductSurfaceNode` -> dynamic texture upload -> UI product-surface render
  target -> captured pixels.
- Preview preparation validates descriptor hash, artifact URI, artifact path,
  artifact validity, descriptor ratification, KTX2 byte layout, upload format,
  selected mip, selected slice, selected channel, sampler identity, bind group
  identity, and residency state.
- Invalid selected texture products fail with diagnostics and do not silently
  fall back to another catalog texture.
- Explicitly selected incompatible texture assets also fail closed with
  `MissingTextureProduct`; global catalog fallback is used only when no asset is
  selected.
- Nonresident mip requests are diagnostic-only; they do not fake preview pixels.
- The Material Graph Canvas texture picker lists catalog-backed texture
  products, filters by search text/resource kind, and routes selection through
  epoch-guarded `MaterialSurfaceAction::PickTextureResource` into the
  `domain/material_graph` source command. The remaining text entry is manual
  source-ref editing, not the evidence for picker completion.
- `proof-manifest.ron` records concrete texture proof metadata: texture product
  id, artifact id, descriptor hash, artifact URI, upload format, mip count,
  selected mip/slice/channel, sampler identity, bind group identity, residency
  state/class, product-surface key, preview target key, durable capture path, and
  capture hash.
- Durable proof artifacts live under
  `docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/artifacts/`.
  The closeout no longer depends on workspace temp paths for texture proof.

GPU/manual capture proof:

- Command run:
  `RUNENWERK_ENABLE_GPU_SMOKE=1 cargo test -p runenwerk_editor --test viewport_gpu_truth_smoke viewport_gpu_truth_smoke -- --ignored --exact --nocapture`
- Result: passed.
- Texture2D provider/product-surface proof:
  product `9028`, artifact `29028`, target
  `runenwerk.editor.texture_preview:texture2d.product9028.mip0.slice0.all`,
  center pixel `[108, 210, 162, 255]`, capture hash
  `blake3:483a40eff929a29193ca839ec96a069cc666764b2065c31a4986285bdec97eab`.
- Texture3D provider/product-surface proof:
  product `9029`, artifact `29029`, target
  `runenwerk.editor.texture_preview:texture3d.product9029.mip0.slice0.all`,
  center pixel `[153, 103, 173, 255]`, capture hash
  `blake3:917eb702a699db47d62821de87e240a75907e8470e8d5547966e983adcfe8dde`.
- SDF two-primitive scene capture pixels:
  left slot `0` `[255, 64, 42, 255]`, right slot `1`
  `[40, 250, 124, 255]`, unchanged background `[85, 89, 97, 255]`.
- SDF durable scene capture:
  `artifacts/captures/frame_5__flow_1__pass_2__stage_after__resource_runenwerk_editor_viewport_editor_viewport_2_1_primary.png`
  with capture hash
  `sha256:ec0abe32523b9af7ed8113484a38eec6b91d522dd05cd7df18a7217ed3464405`.
- The manual-only status is intentional because the repository marks this
  windowed GPU readback smoke test ignored unless `RUNENWERK_ENABLE_GPU_SMOKE`
  is set.

## Validation

- `cargo test -p material_graph material_graph_layout_round_trips_v2`
- `cargo test -p editor_shell material_graph`
- `cargo test -p ui_runtime graph_canvas`
- `cargo test -p ui_graph_editor graph`
- `cargo test -p runenwerk_editor material_graph`
- `cargo test -p runenwerk_editor material_graph_node_move_persists_source_layout`
- `cargo test -p runenwerk_editor material_graph_node_picker_confirm_adds_highlighted_node_to_source`
- `cargo test -p runenwerk_editor material_graph_navigate_diagnostic_selects_and_centers_subject`
- `cargo test -p runenwerk_editor material_graph_node_drag_is_one_undo_transaction`
- `cargo test -p runenwerk_editor material_graph_property_edit_groups_commit_correctly`
- `cargo test -p runenwerk_editor texture_preview_invalid_selected_asset_does_not_fallback`
- `cargo test -p runenwerk_editor texture_preview_selected_incompatible_asset_does_not_fallback`
- `cargo test -p runenwerk_editor texture_viewer_gpu_proof_uses_provider_product_surface_path`
- `cargo test -p runenwerk_editor volume_texture_viewer_gpu_proof_uses_provider_product_surface_path`
- `cargo test -p runenwerk_editor texture_viewer_gpu_proof_rejects_direct_temp_resource_bypass`
- `cargo test -p runenwerk_editor texture_preview_records_concrete_catalog_metadata`
- `cargo test -p runenwerk_editor texture_preview_proof_metadata_has_concrete_descriptor_hash`
- `cargo test -p runenwerk_editor texture_preview_proof_metadata_has_concrete_bind_group_identity`
- `cargo test -p runenwerk_editor wr028_proof_manifest_rejects_texture_metadata_placeholders`
- `cargo test -p runenwerk_editor wr028_proof_manifest_rejects_temp_artifact_paths`
- `cargo test -p runenwerk_editor wr028_proof_manifest_links_durable_texture_preview_artifacts`
- `cargo test -p runenwerk_editor material_graph_texture_picker_lists_catalog_texture_products`
- `cargo test -p runenwerk_editor material_graph_texture_picker_selection_updates_source_ref`
- `cargo test -p runenwerk_editor material_graph_texture_picker_rejects_missing_texture_product`
- `cargo test -p runenwerk_editor volume_texture_viewer_slice_changes_preview_request`
- `cargo test -p runenwerk_editor volume_texture_viewer_channel_changes_preview_request`
- `cargo test -p runenwerk_editor volume_texture_viewer_mip_request_is_diagnosed_when_unresident`
- `cargo test -p engine texture_preview_upload_residency_path`
- `cargo test -p engine texture_preview`
- `cargo test -p texture texture_preview_descriptor`
- `cargo test -p texture`
- `cargo test -p asset texture`
- `cargo test -p runenwerk_editor texture_viewer --no-fail-fast`
- `cargo test -p runenwerk_editor texture_preview --no-fail-fast`
- `cargo test -p runenwerk_editor volume_texture_viewer --no-fail-fast`
- `cargo test -p runenwerk_editor sdf_two_primitives_render_different_material_slots`
- `cargo test -p engine generated_scene_wgsl_selects_material_from_hit_sdf_slot`
- `cargo test -p engine scene_material_table_wgsl_dispatches_to_source_backed_slot_evaluators`
- `cargo test -p runenwerk_editor material_binding_table_uses_resolved_source_backed_slot_products`
- `cargo test -p runenwerk_editor unresolved_source_backed_scene_material_slot_fails_closed`
- `cargo test -p ui_graph_editor graph_canvas_pan_separates_incremental_updates_from_final_commit`
- `cargo test -p editor_scene sdf_assignment_state_revision_changes_only_when_assignment_changes`
- `cargo check -p runenwerk_editor`
- `cargo check -p runenwerk_editor -p engine`
- `RUNENWERK_ENABLE_GPU_SMOKE=1 cargo test -p runenwerk_editor --test viewport_gpu_truth_smoke viewport_gpu_truth_smoke -- --ignored --exact --nocapture`
- `task planning:validate`
- `task docs:validate`
- `task roadmap:validate`
- `task production:render`
- `task production:check`
- `task production:validate`
- `git diff --check`

## Remaining Scope

No WR-028 quality gaps remain for the SDF primitive scope:

```text
known_quality_gaps: []
```

Model/mesh material binding is not closed by this report. It remains open under
`WR-029`, including model/mesh source identity, submesh/material-region
assignment, renderer ABI extension, generated shader selection for model/mesh
material regions, and GPU pixel proof for that path.
