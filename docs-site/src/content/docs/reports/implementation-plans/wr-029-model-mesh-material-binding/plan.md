---
title: WR-029 Model Mesh Material Binding Planning Stub
description: "Planning stub for model/mesh source identity, submesh/material-region assignment, renderable material-slot selection, and non-regression of the WR-028 SDF material path."
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
  - ../../../reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/closeout.md
  - ../../../reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/proof-manifest.ron
---

# WR-029 Model Mesh Material Binding Planning Stub

## Status

WR-029 is a planning stub, not an implementation contract. It exists only to
hold the model/mesh material-binding gap that WR-028 intentionally did not
close.

WR-028 is frozen as complete for the SDF primitive path. WR-029 must not reopen,
weaken, or rewrite the proven WR-028 SDF contracts unless a later accepted ADR
explicitly proves that a shared abstraction preserves the SDF behavior and proof
evidence.

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

## Required Governance Before Implementation

Before WR-029 product code starts, produce either an accepted implementation
contract or ADR that records these decisions against repository truth:

- the stable source identity for model/mesh renderables;
- the stable identity for imported mesh assets and foreign mesh artifacts;
- the stable identity for submesh/material-region assignments;
- whether assignments live in `domain/editor/editor_scene` directly or in a
  source-owned submodule with editor-scene persistence;
- how model/mesh assignments survive save/reload, asset rebuilds, and imported
  mesh reprocessing;
- how the renderer ABI carries `renderable_index` or an equivalent typed model
  surface identity without weakening the SDF `sdf_primitive_index` path;
- how runtime material table identity includes model/mesh assignment changes;
- how generated shaders or fixed mesh render passes select material output by
  the hit model/mesh material region;
- how prior-valid material/shader/mesh products are preserved with diagnostics;
- which tests already exist and which tests must be created.

Implementation must not proceed if any stable identity above is unknown or if
the selected identity is a transient runtime entity id, renderer table index,
palette vector index, generated artifact id, display name, or UI selection id.

## Source Truth Boundaries

- `domain/editor/editor_scene` must remain the source owner for scene material
  assignments.
- Imported mesh assets and generated mesh artifacts must remain catalog-backed
  products; UI providers and renderer residency state must not become authored
  mesh or material truth.
- Renderer state may compact model/mesh material bindings into runtime table
  indices only as derived product state with explicit invalidation inputs.
- WR-028 SDF material assignment and generated WGSL behavior must remain
  non-regressed and separately testable.

## Required Future Proof

The eventual WR-029 closeout must include:

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

## Initial Validation Targets

Exact package and test names must be confirmed during WR-029 Phase 0. The
minimum target set is:

- `cargo test -p editor_scene model_mesh_material_assignment_survives_save_reload`
- `cargo test -p runenwerk_editor model_mesh_assignment_survives_asset_rebuild`
- `cargo test -p engine model_mesh_renderable_index_material_selection`
- `cargo test -p runenwerk_editor model_mesh_renderable_uses_source_backed_material_slot`
- `cargo test -p runenwerk_editor sdf_two_primitives_render_different_material_slots`
- `cargo check -p runenwerk_editor`
- `task docs:validate`
- `task roadmap:validate`
- `task production:validate`
- `task planning:validate`
