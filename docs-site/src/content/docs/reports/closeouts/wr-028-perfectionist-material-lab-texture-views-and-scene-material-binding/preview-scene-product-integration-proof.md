---
title: PreviewSceneProduct Integration Proof
description: R2A-R2D closeout proof for the app-owned PreviewSceneProduct integration track.
status: completed
owner: apps/runenwerk_editor
layer: app-runtime / editor-ui / engine-render
canonical: false
last_reviewed: 2026-05-19
related_designs:
  - ../../../design/active/material-lab-and-material-preview-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
related_reports:
  - ./closeout.md
  - ./proof-manifest.ron
  - ../../../reports/implementation-plans/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/plan.md
---

# PreviewSceneProduct Integration Proof

## Status

Complete as of 2026-05-19 for the R2A-R2D PreviewSceneProduct integration
track.

This is a proof and closeout record only. It does not add runtime behavior,
renderer behavior, Workbench/tool-suite architecture, persistence, asset IO,
model/mesh binding, SDF/world renderer residency, dirty-region runtime, a second
texture binding model, or shader compilation in renderer prepare/submit.

## Track Evidence

| Slice | Commit | Owning files | Proof summary |
| --- | --- | --- | --- |
| Prerequisite | `29699d1a render: support texture-bearing scene material tables` | `engine/src/plugins/render/frame/contributions.rs`, `engine/src/plugins/render/material_compiler/**`, `apps/runenwerk_editor/src/material_lab/renderer_handoff.rs`, `apps/runenwerk_editor/src/runtime/systems/material_preview.rs` | Existing renderer contribution and material-table support accept texture-bearing scene material tables without making PreviewSceneProduct a renderer input. |
| R2A | `d6187bdd editor: introduce preview scene product model` | `apps/runenwerk_editor/src/material_lab/preview_scene_product.rs`, `apps/runenwerk_editor/src/material_lab/mod.rs` | Introduces the app-owned `PreviewSceneProduct` contract, stable product identity, slots, resources, shader product references, build status, and diagnostics. |
| R2B | `e066f8d7 editor: build preview scene product from material handoff` | `apps/runenwerk_editor/src/material_lab/preview_scene_product.rs`, `apps/runenwerk_editor/src/material_lab/renderer_handoff.rs` | Builds PreviewSceneProduct from the existing material handoff path for single-material and scene-material-table previews, then converts it back into `PreparedMaterialFeatureContribution` for renderer consumption. |
| R2C | `e6ff2052 editor: track preview scene product state` | `apps/runenwerk_editor/src/material_lab/preview_scene_product.rs`, `apps/runenwerk_editor/src/material_lab/state/runtime.rs`, `apps/runenwerk_editor/src/material_lab/state/mod.rs`, `apps/runenwerk_editor/src/material_lab/state/tests.rs`, `apps/runenwerk_editor/src/material_lab/renderer_handoff.rs`, `apps/runenwerk_editor/src/runtime/systems/material_preview.rs` | Stores current and last-valid product state only in `MaterialLabRuntime`, enforces request identity and stale-product rejection, records build failures, and fails closed before renderer handoff. |
| R2D | `6ba3614b editor: surface preview scene product lineage` | `domain/editor/editor_shell/src/surfaces/material.rs`, `apps/runenwerk_editor/src/material_lab/state/preview_status.rs`, `apps/runenwerk_editor/src/material_lab/state/diagnostics.rs`, `apps/runenwerk_editor/src/shell/providers/material_preview.rs`, `apps/runenwerk_editor/src/shell/providers/tests.rs` | Projects product identity, mode, material table, resource layout, shader labels, slot/resource counts, last-valid identity, and failure reason into presentation-only DTOs and provider status rows. |

## Ownership Proof

- `apps/runenwerk_editor/src/material_lab/preview_scene_product.rs` owns the
  app-side product contract. It records identities and lineage only; it does not
  own `material_graph` source truth and has a guard test proving it does not
  store `PreparedSceneMaterialBundle`.
- `apps/runenwerk_editor/src/material_lab/state/runtime.rs::MaterialLabRuntime`
  is the only owner of `current_preview_scene_product` and
  `last_valid_preview_scene_product`.
- `apps/runenwerk_editor/src/runtime/systems/material_preview.rs::build_and_record_preview_scene_product`
  builds through the R2B path and records success or failure through
  `MaterialLabRuntime`.
- `apps/runenwerk_editor/src/material_lab/renderer_handoff.rs::prepared_material_contribution_for_preview_scene_product`
  rebuilds the expected product for the active request and rejects stale product
  identities before producing `PreparedMaterialFeatureContribution`.
- `domain/editor/editor_shell/src/surfaces/material.rs::MaterialPreviewStatusViewModel`
  stores presentation labels, counts, and identity strings only. It does not
  store renderer structs or app workflow state.
- `apps/runenwerk_editor/src/shell/providers/material_preview.rs::material_preview_status_lines`
  renders the compact Preview Scene Product section from the status view model
  without adding mutation routes.

## Identity And Staleness Proof

`PreviewSceneProductRequestIdentity` is derived from stable request inputs:
product mode, viewport product id, active material product id, active material
artifact cache identity, ordered material slots, material table identity,
resource layout identity, slot-to-table-resource mapping, and shader identity
when available.

It deliberately excludes diagnostic text, display labels, provider labels,
UI-only names, and filesystem paths when artifact/cache identity exists.
Shaderless identity is used only to decide whether a same-scene transient bundle
availability failure may preserve last-valid state. Full handoff validation
rebuilds the product with the active bundle and compares product identity before
renderer contribution formation.

The R2C tests prove last-valid preservation is rejected for different resource
layout identity, material table identity, slot/resource mapping, and shader
identity where applicable. The explicit stale-product handoff tests prove a
same-scene product with a different shader artifact/cache identity fails before
`PreparedMaterialFeatureContribution` is produced.

## Failure Policy Proof

The integrated path fails closed for missing generated scene-table bundles,
stale generated bundles, resource layout mismatch, unresolved explicit scene
slots, table-wide resource-slot identity conflicts, and current product/request
identity mismatch.

The failure path does not silently fall back to active preview material, default
material, unrelated prior product, single-material shader, or unrelated texture
resources. Prior valid state is preserved only for the same request identity or
a same-scene transient shader/bundle availability issue.

## Presentation Proof

R2D keeps diagnostics and status display presentation-only:

- `apps/runenwerk_editor/src/material_lab/state/preview_status.rs::material_preview_status_view_model`
  reads current or last-valid product lineage and maps it into
  `MaterialPreviewStatusViewModel`.
- `apps/runenwerk_editor/src/material_lab/state/diagnostics.rs::preview_scene_product_diagnostic_rows`
  derives status rows from existing runtime state and diagnostics.
- `apps/runenwerk_editor/src/shell/providers/material_preview.rs::material_preview_status_lines`
  renders status, mode, product identity, material table, resource layout,
  shader identity, shader artifact, slot count, resource count, last-valid
  identity, and failure reason.

Diagnostics are informational. They do not decide command acceptance, resource
resolution, renderer fallback, or product adoption.

## Non-Goals Confirmed

- No renderer behavior change in R2C or R2D.
- No Workbench/tool-suite architecture change.
- No V5/workspace persistence.
- No asset save/load behavior.
- No material publication semantics change.
- No WR-029/model-mesh material binding.
- No SDF/world renderer residency.
- No world dirty-region runtime.
- No second texture binding model.
- No renderer prepare/submit shader compilation.
- No `PreviewSceneProduct` ownership of `material_graph` source truth.
- No `PreparedSceneMaterialBundle` stored inside `PreviewSceneProduct`.

## Test Coverage

- `cargo test -p runenwerk_editor preview_scene_product` covers product
  identity stability, diagnostic exclusion, material-graph source exclusion,
  engine prepared-bundle exclusion, build outcomes, scene-table modes, slot
  ordering, resource layout, and bundle freshness.
- `cargo test -p runenwerk_editor material_preview` covers runtime recording,
  current/last-valid preservation, failure semantics, status projection, provider
  rendering, bundle requirements, and single-material behavior.
- `cargo test -p runenwerk_editor scene_material_table_handoff` covers scene
  table handoff resource mapping, resource deduplication/conflict behavior, and
  texture-bearing table layout.
- `cargo test -p runenwerk_editor stale_scene_material_table_shader_bundle_fails_closed`
  covers stale generated bundle rejection.
- `cargo test -p runenwerk_editor renderer_handoff` covers stale product
  rejection before renderer contribution formation and proves renderer handoff
  still produces `PreparedMaterialFeatureContribution`.
- `cargo test -p runenwerk_editor material_inspector` and
  `cargo test -p runenwerk_editor material_graph_canvas` cover that related
  Material Lab presentation surfaces remain compatible after the lineage DTO
  extension.
- `cargo test -p runenwerk_editor material_viewport` covers viewport integration
  and non-regression around preview product presentation.

## R2E Validation

R2E validation on 2026-05-19 passed. `git diff --check` exited zero and
reported only existing LF-to-CRLF working-copy warnings, not whitespace errors.

The local command set was:

- `cargo test -p runenwerk_editor preview_scene_product`
- `cargo test -p runenwerk_editor material_preview`
- `cargo test -p runenwerk_editor material_inspector`
- `cargo test -p runenwerk_editor material_graph_canvas`
- `cargo test -p runenwerk_editor scene_material_table_handoff`
- `cargo test -p runenwerk_editor stale_scene_material_table_shader_bundle_fails_closed`
- `cargo test -p runenwerk_editor material_viewport`
- `cargo test -p runenwerk_editor renderer_handoff`
- `cargo check -p engine -p editor_shell -p runenwerk_editor`
- `task docs:validate`
- `task puml:validate`
- `git diff --check`
