---
title: WR-028 Perfectionist Material Lab Texture Views And Scene Material Binding Contract
description: Corrective implementation-readiness contract for rich Material Lab graph canvas, live texture inspection, editor-scene-owned SDF material assignments, SDF per-hit material selection, and GPU proof.
status: active
owner: apps/runenwerk_editor
layer: domain / app-runtime / editor-ui / engine-render
canonical: true
last_reviewed: 2026-05-17
related_designs:
  - ../../../design/active/material-lab-and-material-preview-design.md
  - ../../../design/active/editor-rendered-world-and-multi-entity-viewport-design.md
  - ../../../design/active/editor-asset-pipeline-and-content-workflow-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
related_reports:
  - ../../../reports/closeouts/wr-021-material-lab-and-material-preview-products/closeout.md
  - ../../../reports/closeouts/wr-027-completion-quality-and-perfectionist-audit-gate/closeout.md
  - ../../../reports/audits/roadmap-perfectionist-audit-2026-05-17.md
---

# WR-028 Perfectionist Material Lab Texture Views And Scene Material Binding Contract

## 1. Problem Statement

WR-028 is not a greenfield material system. It is a completion and repair contract for existing material graph source files, texture artifact products, editor-scene SDF material assignment contracts, generated scene WGSL, and renderer material bundle consumption.

The contract must prevent fake graph-canvas evidence, descriptor-only texture previews, app-runtime-authored material assignments, prepared-but-unused scene material bundles, material slot indices that are ignored by generated WGSL, and global-material scene rendering.

WR-028 acceptance is SDF primitive only. WR-028 must not block WR-029 model/mesh material binding, but WR-028 must not close or erase model/mesh material gaps.

## 2. Non-Negotiable Source Truth

- `domain/material_graph/src/persistence.rs` `MaterialGraphSourceFileV2` remains material graph source truth.
- `domain/asset/src/artifact.rs` `ArtifactPayloadKind::TextureProduct` and `GeneratedTextureProduct`, with `TextureDescriptor`, `descriptor_hash`, and `artifact_uri`, remain texture artifact truth.
- `domain/editor/editor_scene` must own scene material palette and SDF primitive material assignments.
- `MaterialLabRuntime`, UI providers, renderer bind groups, generated WGSL, material IR, shader artifacts, preview targets, viewport packets, prepared scene bundles, and material table indices are derived state, projections, or products.
- Renderer state must never become authored material or authored scene truth.
- UI state may persist only source-owned graph layout/editor affordance state, not semantic material truth.

## 3. Data Model Contract

WR-028 must define or confirm these source-side identities before implementation:

- `SceneMaterialSlotId`, already present in `domain/editor/editor_scene/src/model/material.rs`.
- `SceneMaterialPaletteEntryId` or an equivalent stable palette entry identity if `SceneMaterialSlotId` alone is not enough to track palette entry lifecycle and diagnostics.
- `SceneMaterialSourceRef`.
- Stable SDF primitive source identity. Use an existing source-stable identity only if Phase 0 proves it survives save/reload and scene rebuild; otherwise add one in `editor_scene`.
- SDF primitive material assignment identity, defined as stable SDF primitive source identity plus `SceneMaterialSlotId`.
- Broken material binding diagnostic subject, carrying the SDF primitive source identity, material slot id, and material source ref when available.
- Runtime material table entry identity, derived from source slot id, material source/product identity, shader identity, texture residency identity, and assignment revision.

`SceneMaterialSourceRef` must reference stable material source identity such as `AssetId` plus `AssetSourceId` plus `AssetSourceRevisionId` or source revision, or an existing material document/source identity proven to survive save/reload and rebuild. It may include `MaterialGraphDocumentId` only if Phase 0 proves that id is tied to stable source identity. It must not be a raw path, display name, generated material artifact id, shader artifact id, renderer table index, or UI selection id.

Durable authored assignment state must not use:

- raw `AssetArtifactId`;
- generated shader artifact id;
- renderer table index;
- transient runtime entity id, unless `editor_scene` proves it is source-stable;
- UI selection id;
- palette vector index.

Runtime compaction into material table indices is allowed only as derived product state with explicit identity and invalidation inputs.

## 4. Current Repo Truth Summary

| Area | Current repo truth | Gap | Required WR-028 correction |
|---|---|---|---|
| `MaterialGraphSourceFileV2` | `domain/material_graph/src/persistence.rs` defines V2 with graph and editor state; app IO reads/writes V2. | No new material source format is justified. | Preserve V2; extend only for proven missing source-owned layout/editor affordances. |
| Material lowering/IR | `domain/material_graph/src/lib.rs`, `lowering.rs`, and `ir.rs` expose `lower_material_graph`, `MaterialIr`, `MaterialResourceBinding`, and `FormedMaterialProduct`. | Pipeline exists but scene-pixel consumption/proof is incomplete. | Consume and prove the existing pipeline; do not restart material lowering. |
| Texture products | `domain/asset/src/artifact.rs` has `TextureProduct`, `GeneratedTextureProduct`, `descriptor_hash`, `artifact_uri`, and prior-valid preservation. | Current viewers are descriptor/status-first. | Texture preview proof must sample GPU textures prepared from catalog-backed artifacts. |
| Scene material state | `domain/editor/editor_scene/src/model/material.rs` has `SceneMaterialPalette` and `PrimitiveMaterialSlotAssignment`. | No aggregate assignment state/resolver/persistence contract was found. | Add/strengthen editor-scene-owned SDF assignment state and resolver. |
| `MaterialLabRuntime` assignment map | `apps/runenwerk_editor/src/material_lab/state.rs` owns `primitive_material_slots`. | App runtime is currently authoritative for authored assignments. | Move authority into `editor_scene`; runtime becomes projection/cache/workflow state. |
| SDF viewport slot packet | `apps/runenwerk_editor/src/runtime/resources.rs` carries `material_slot_index`, currently packed into shader params as `f32`. | Generated scene WGSL does not use it for material selection. | Carry typed SDF primitive material slot identity and consume it in generated WGSL. |
| Generated scene WGSL | `engine/src/plugins/render/material_compiler/wgsl/scene.rs` returns distance-only SDF samples and evaluates one material. | Global material path can pass weak evidence. | Raymarch returns SDF primitive/material slot identity and selects material by hit slot. |
| Graph editor contracts | `domain/ui/ui_graph_editor/src/lib.rs` has graph view/action/hit-test contracts. | Hit targets are only background/node/port; no full retained canvas substrate. | Add generic graph canvas substrate before material-specific canvas work. |
| UI substrate | `ui_tree`, `ui_runtime`, `ui_render_data`, and `ui_input` exist. | No graph canvas node/emitter/session state was found. | Add graph canvas retained node or equivalent, input ownership, hit testing, and primitive emission. |
| Material graph surface | `domain/editor/editor_shell/src/composition/build_material_graph_surface.rs` renders labels/buttons/panels. | Not a real graph canvas. | Replace with toolbar plus graph canvas plus inspector/palette/diagnostics. |
| Texture viewers | `apps/runenwerk_editor/src/shell/providers/texture_viewer.rs` and `volume_texture_viewer.rs` expose descriptor-first text. | Descriptor text is not preview proof. | Add rendered Texture2D and Texture3D/volume preview products. |
| Prepared scene bundle | `engine/src/plugins/render/frame/contributions.rs` and render flow already have material feature bundle infrastructure. | Needs per-hit SDF slot selection and pass-consumption proof. | Build on existing bundle/table infrastructure; no second renderer material system. |

## 5. Domain-by-Domain Work Plan

### `material_graph`

Change locations: `domain/material_graph/src/persistence.rs`, `domain/material_graph/src/lowering.rs`, `domain/material_graph/src/ir.rs`, and adjacent command modules if added.

- Preserve `MaterialGraphSourceFileV2`.
- `domain/material_graph` remains the only owner of material graph semantics, validation, material node catalogs, lowering behavior, and source graph mutation contracts.
- Add or confirm source-backed commands for node move, connect, disconnect, property edit, and texture ref edit.
- Diagnostics must map to material graph nodes and ports.
- No renderer state, UI runtime state, graph canvas substrate state, or scene assignment truth may live in this crate.

### `editor_scene`

Change location: `domain/editor/editor_scene/src/model/material.rs`, or a new `domain/editor/editor_scene/src/model/material/` submodule if the contract grows.

- Add or strengthen `SceneMaterialAssignmentState`.
- Move authoritative SDF primitive/material assignment state out of `MaterialLabRuntime`.
- Define `resolve_material_slot_for_sdf_primitive`.
- Define `resolve_material_binding_for_sdf_scene_packet` or exact equivalent.
- Define default material fallback, deleted slot behavior, missing source behavior, invalid graph behavior, failed generation/upload behavior, and broken-binding diagnostics.
- Add persistence and ratification tests proving save/reload and broken binding behavior.

### `apps/runenwerk_editor/material_lab`

Change locations: `apps/runenwerk_editor/src/material_lab/state.rs`, `workflow.rs`, `renderer_handoff.rs`, and `document_io.rs`.

- Remove authoritative `primitive_material_slots` ownership from `MaterialLabRuntime`.
- Allow runtime projection/cache reads from editor-scene source assignment state.
- Assignment commands must mutate `editor_scene` source state through workflow adapters.
- Graph actions must map to source-backed workflow commands; apps code must not become source graph mutation authority.
- Provider-created actions must remain projection-epoch guarded.
- Material graph edits and scene assignment edits must invalidate generated products, shader identities, and scene material bundles.

### Generic Graph Canvas Substrate

Change locations: `domain/ui/ui_graph_editor`, `domain/ui/ui_tree`, `domain/ui/ui_runtime`, `domain/ui/ui_render_data`, `domain/ui/ui_input`, `domain/editor/editor_shell`, and `apps/runenwerk_editor`.

Ownership boundaries:

- `domain/ui/ui_graph_editor` owns backend-neutral graph interaction vocabulary and view/gesture state only.
- `domain/ui/ui_tree`, `domain/ui/ui_runtime`, and `domain/ui/ui_render_data` own retained UI representation, input ownership, layout, clipping, and primitive emission only.
- `domain/editor/editor_shell` composes the material graph surface but does not own material graph truth.
- `apps/runenwerk_editor` maps graph actions to source-backed workflow commands.
- `domain/material_graph` remains the only owner of material graph semantics, validation, material node catalogs, lowering behavior, and source graph mutation contracts.

Required substrate responsibilities:

- Graph view model.
- Graph canvas retained node or equivalent UI tree representation.
- Graph hit-test scene.
- Session-scoped graph gesture state.
- Node drag state.
- Connection drag/preview state.
- Marquee selection.
- Pan/zoom state.
- Keyboard shortcut handling.
- Focus ownership.
- Pointer capture until release.
- Wheel zoom ownership that does not leak to viewport or scroll owners.
- Clipping/layout ownership.
- Primitive emission for node boxes, ports, edges, labels, selection outlines, connection previews, and overlays.

Required evidence:

- Graph canvas owns pointer drag until release.
- Graph canvas owns wheel zoom.
- Hit testing distinguishes canvas background, node body, port, edge, selection, and empty space.
- Graph canvas emits `ui_render_data` primitives, not label-list fake evidence.
- Graph interaction state is session-scoped unless explicitly persisted as source-owned layout.

### Material Graph Canvas Surface

Change locations: `domain/editor/editor_shell/src/composition/build_material_graph_surface.rs`, `domain/editor/editor_shell/src/surfaces/material.rs`, and `apps/runenwerk_editor/src/shell/providers/material_graph_canvas.rs`.

- Convert `build_material_graph_surface` from label/list projection to toolbar plus graph canvas plus inspector/palette/diagnostics layout.
- Map graph actions to source-backed material graph commands through provider/workflow adapters.
- Persist only source-owned layout/editor affordance state.
- Keep semantic graph truth in `material_graph`.

### Texture / Asset / Texture Viewers

Change locations: `apps/runenwerk_editor/src/shell/providers/texture_viewer.rs`, `apps/runenwerk_editor/src/shell/providers/volume_texture_viewer.rs`, `apps/runenwerk_editor/src/asset_pipeline/catalog_runtime.rs`, and owning texture/renderer upload modules.

- Texture viewer GPU proof is valid only if the preview samples a GPU texture prepared from the catalog-backed `TextureProduct` or `GeneratedTextureProduct` through the same residency/upload/sampling class used by material rendering, or through an explicitly named diagnostic preview path that is not accepted as material-rendering proof.
- Add rendered Texture2D preview.
- Add rendered Texture3D/volume preview with slice, mip, and channel controls.
- Proof/diagnostics must include texture product id, descriptor hash, artifact URI, upload format, mip count, selected mip, selected slice, selected channel, sampler identity, bind group identity, residency state, and failure diagnostic if any.
- Descriptor text remains diagnostics only.

### Engine Render / Material Compiler / WGSL ABI

Change locations: `engine/src/plugins/render/material_compiler/wgsl/scene.rs`, `engine/src/plugins/render/frame/contributions.rs`, and `engine/src/plugins/render/renderer/render_flow/execute_passes.rs`.

- Fix generated scene WGSL so SDF raymarching returns hit SDF primitive identity and material slot identity.
- Replace global material evaluation with per-hit SDF material slot selection.
- Do not keep material slot identity as untyped `f32` long term when a `u32` path can be used.
- Generated scene bundle must be consumed by the pass producing scene pixels.
- Material table identity must change when SDF slot assignments or material bindings change.
- Shader identity must change when generated WGSL changes.
- Prior-valid scene material bundle may be preserved on generation/upload failure only with diagnostics.

### Model/Mesh Material Binding

Policy: WR-028 is SDF primitive only.

WR-029 owns model/mesh material binding, including model/mesh source identity, submesh/material-region assignment, and any future `renderable_index` ABI extension. WR-028 closeout must preserve or move model/mesh material binding gap language to WR-029 and must not claim model/mesh material completion.

## 6. Phased Implementation Plan

### Phase 0 - Governance and Repo-Truth Lock

Run architecture governance and `task planning:validate` before product-code implementation.

Phase 0 must produce either a short governance note or a plan update recording:

- selected stable SDF primitive source identity;
- selected graph canvas substrate shape, such as retained node versus equivalent representation;
- selected GPU proof harness and capture method;
- WR-029 file path, roadmap id, or proposed id for model/mesh material binding;
- confirmed package names;
- acceptance test names that already exist versus tests that must be created;
- any ADR/design update required before implementation.

Implementation may not proceed beyond Phase 0 if these named artifacts are missing.

Stop if source-truth ownership is ambiguous.

### Phase 1 - `editor_scene`-Owned SDF Assignment Truth

- Add/strengthen source schema for SDF material assignments.
- Move app runtime assignment authority out of `MaterialLabRuntime`.
- Add resolver, persistence, ratification, and diagnostics.
- Tests must prove save/reload, default fallback, deleted slot, missing source, invalid graph, and broken binding behavior.

### Phase 2A - Generic Graph Canvas Substrate

- Add graph canvas retained node or equivalent UI tree representation.
- Add session-scoped gesture state for drag, connection preview, marquee selection, pan/zoom, focus, and shortcut ownership.
- Add graph-specific hit testing and pointer/wheel ownership.
- Add primitive emission for graph visuals.
- Tests must prove capture, wheel ownership, hit target distinction, clipping, and primitive emission.
- Tests must prove the substrate has no material semantics, no material node catalog, no graph semantic validation, no material lowering, and no source graph mutation authority.

### Phase 2B - Material Graph Canvas Surface

- Replace material graph label/list projection.
- Use toolbar plus graph canvas plus inspector/palette/diagnostics layout.
- Map graph actions to source-backed material graph workflow commands.
- Persist only source-owned layout/editor affordance state.
- Tests must prove add/delete/connect/disconnect/move/property edit/texture edit/undo/redo.

### Phase 3 - Live Texture Inspection

- Add rendered Texture2D preview.
- Add rendered Texture3D/volume preview with slice/mip/channel controls.
- Use catalog-backed KTX2 artifacts only.
- Prove upload/residency/sampling path and bind group identity.
- Descriptor text is not acceptance evidence.

### Phase 4 - SDF Per-Hit Material Selection

- Extend SDF scene packet and generated WGSL ABI.
- SDF raymarch returns SDF primitive/material slot identity.
- Fragment shader selects material output by hit material slot.
- Two SDF primitives in one scene must render different material outputs.

### Phase 5 - WR-029 Handoff

- Create or update the follow-up WR/ADR for model/mesh material binding.
- Preserve model/mesh material gap language outside WR-028 completion.
- Do not add ambiguous implementation-time stop language to WR-028.

### Phase 6 - GPU Proof, Audit, Closeout

- Add `proof-manifest.ron`.
- Record GPU captures.
- Record sample points and pixel delta summaries.
- Run validation.
- `known_quality_gaps` must be `[]` before `perfectionist_verified`.

## 7. Renderer ABI Contract

WR-028 renderer ABI is SDF-specific. Exact names may differ, but semantics must match:

```wgsl
struct SceneDistanceSample {
    distance: f32,
    sdf_primitive_index: u32,
    material_slot_index: u32,
};

struct RaymarchResult {
    hit: bool,
    distance: f32,
    sdf_primitive_index: u32,
    material_slot_index: u32,
};
```

Required semantics:

- SDF scene evaluation returns hit SDF primitive identity and material slot identity.
- Fragment material evaluation selects from runtime material table by hit material slot.
- Missing, invalid, or failed material output uses visible fallback/error material with diagnostics.
- Runtime material table identity changes when graph products, SDF slot assignments, or bindings change.
- Prior-valid generated shader/material bundle may remain active only with an explicit diagnostic.
- No global material evaluation path may satisfy WR-028 acceptance.

## 8. Material Binding Resolver Contract

- Explicit SDF primitive assignment wins.
- Missing assignment uses scene default material slot.
- Deleted or missing material source produces a broken-binding diagnostic.
- Invalid material graph uses a visible error material.
- Failed generation/upload preserves prior-valid active product where possible.
- UI selection state never affects material resolution.
- Raw artifact ids are not assigned directly by users.
- Assignments survive save/reload and asset rebuilds through stable source/material references.

## 9. Anti-Drift Guards

Hard failure conditions:

- Fail if the generic graph canvas substrate introduces material semantics, graph semantic validation, material node catalogs, material lowering behavior, source graph mutation authority, or material-specific truth outside `domain/material_graph` and the owning material workflow adapters.
- Fail if Material Graph Canvas completion evidence is labels/status/control panels.
- Fail if graph canvas does not own pointer drag until release.
- Fail if graph canvas wheel zoom leaks to viewport camera or scroll owners.
- Fail if texture preview completion evidence is descriptor text only.
- Fail if texture preview proof does not name texture product id, descriptor hash, artifact URI, sampler identity, bind group identity, and residency state.
- Fail if scene material assignments remain authored in `MaterialLabRuntime`.
- Fail if material slot indices are written into packets/uniforms but generated scene WGSL does not select material output from the hit SDF primitive slot.
- Fail if generated scene bundle is prepared/hashed but not consumed by the pass producing scene pixels.
- Fail if material table identity ignores assignment changes.
- Fail if failed material generation silently falls back without diagnostics.
- Fail if WR-028 claims model/mesh material support.
- Fail if closeout claims `perfectionist_verified` with non-empty `known_quality_gaps`.

## 10. Acceptance Criteria

- Material graph source edits save to `MaterialGraphSourceFileV2` and restore correctly.
- Generic graph canvas substrate owns hit testing, pointer capture, wheel zoom, focus, clipping, session gesture state, and primitive emission without material semantics.
- Material Lab exposes an actual graph canvas with nodes, typed ports, edges, pan/zoom, selection, editing, validation overlays, diagnostics navigation, and undo/redo.
- Texture2D preview renders real pixels from a catalog-backed KTX2 `TextureProduct` or `GeneratedTextureProduct`.
- Texture3D/volume preview renders real pixels with slice/mip/channel controls.
- Scene material assignments are owned by `editor_scene` source state, persisted, ratified, and resolved deterministically for SDF primitives.
- Two SDF primitives in one scene render different material outputs through different material slots.
- Generated scene WGSL selects material output from the hit SDF primitive material slot.
- Material, shader, and table identity changes are observable when material graph or SDF assignment changes.
- Prior-valid preservation is tested for material, shader, and texture preparation failure.
- Model/mesh material binding is explicitly split to WR-029.
- Closeout has GPU captures, sample-point evidence, changed/unchanged region hashes, validation output, and empty `known_quality_gaps`.

## 11. Validation Matrix

Verified package/task names exist; exact test filters are acceptance-target names that must be created or confirmed during Phase 0.

| Acceptance | Unit test | Integration test | GPU/proof artifact | Owner |
|---|---|---|---|---|
| V2 graph source edits | `cargo test -p material_graph persistence` | `cargo test -p runenwerk_editor material_graph_source_edits_round_trip_v2` | None | `material_graph`, `runenwerk_editor` |
| Generic graph substrate | `cargo test -p ui_graph_editor graph_canvas_hit_testing`; `cargo test -p ui_runtime graph_canvas_pointer_capture`; `cargo test -p ui_render_data graph_canvas_emits_primitives` | `cargo test -p editor_shell material_graph_surface_contains_graph_canvas` | Graph canvas primitive signature | `ui_graph_editor`, `ui_tree`, `ui_runtime`, `ui_render_data`, `editor_shell` |
| Substrate ownership boundary | `cargo test -p ui_graph_editor graph_canvas_has_no_material_semantics` | `cargo test -p editor_shell material_graph_actions_route_to_workflow_adapters` | None | `ui_graph_editor`, `editor_shell`, `runenwerk_editor`, `material_graph` |
| Editor-scene assignment truth | `cargo test -p editor_scene material` | `cargo test -p runenwerk_editor material_assignment_mutates_editor_scene_not_material_lab_runtime` | Assignment identity entry in proof manifest | `editor_scene`, `runenwerk_editor` |
| Texture previews | `cargo test -p texture texture_preview_descriptor`; `cargo test -p asset texture` | `cargo test -p runenwerk_editor texture_viewer_gpu_preview_uses_catalog_residency` | Texture2D and volume GPU captures with bind group identity | `texture`, `asset`, `runenwerk_editor`, `engine` |
| SDF WGSL ABI | `cargo test -p engine generated_scene_wgsl_reads_hit_material_slot` | `cargo test -p runenwerk_editor sdf_two_primitives_render_different_material_slots` | Two-SDF-primitives pixel delta summary | `engine`, `runenwerk_editor` |
| Renderer consumption/prior-valid | `cargo test -p engine material` | `cargo check -p runenwerk_editor` | Pass provenance and prior-valid failure capture | `engine`, `runenwerk_editor` |
| Docs/governance | `task planning:validate`; `task docs:validate` | `task roadmap:validate`; `task production:validate` | Closeout links and proof manifest | docs/workspace |

When roadmap or production-track evidence changes, also run `task roadmap:render`, `task roadmap:check`, `task production:render`, and `task production:check`.

## 12. Proof Manifest

Closeout must create:

`docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/proof-manifest.ron`

Required shape:

```ron
(
  proof_id: "wr-028",
  scene_fixture: "...",
  material_sources_before: [...],
  material_sources_after: [...],
  texture_artifacts: [...],
  generated_shader_artifacts: [...],
  generated_wgsl_hash_before: "...",
  generated_wgsl_hash_after: "...",
  material_table_identity_before: "...",
  material_table_identity_after: "...",
  material_table_identity_inputs: [...],
  texture_preview_bind_group_identity: "...",
  gpu_backend: "...",
  capture_method: "...",
  execution_mode: "headless|windowed|manual",
  ci_status: "ci|manual_only|ignored_in_ci",
  manual_only_reason: "...",
  ignored_in_ci_reason: "...",
  gpu_captures: [...],
  sample_points: [
    (
      label: "left_sdf_primitive",
      target: "viewport.scene_color",
      pixel: (x: 0, y: 0),
      before_rgba: [0, 0, 0, 255],
      after_rgba: [0, 0, 0, 255],
      before_hash: "...",
      after_hash: "...",
    ),
  ],
  pixel_delta_summaries: [
    (
      target: "viewport.scene_color",
      expected_changed_regions: [...],
      expected_unchanged_regions: [...],
      changed_region_hash_before: "...",
      changed_region_hash_after: "...",
      unchanged_region_hash_before: "...",
      unchanged_region_hash_after: "...",
    ),
  ],
  validation_commands: [...],
  known_quality_gaps: [],
)
```

## 13. Closeout Requirements

Closeout must:

- Create or update `docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/closeout.md`.
- Include `proof-manifest.ron`.
- Link GPU captures.
- Include sample-point evidence and pixel delta summaries.
- Show validation output.
- Update `docs-site/src/content/docs/design/active/material-lab-and-material-preview-design.md` so WR-028-related acceptance language distinguishes SDF primitive completion from WR-029 model/mesh material binding.
- Update any roadmap, production-track, audit, or closeout language that currently implies WR-028 closes model/mesh material binding. WR-028 may close only the SDF primitive path; model/mesh material binding must remain open or be moved to WR-029.
- Update WR-021/WR-027 inherited gap language only for gaps actually closed by WR-028.
- Preserve or move model/mesh material binding gap language to WR-029.
- Move WR-028 to completed/perfectionist_verified only with `known_quality_gaps: []`.
