---
title: WR-028 Phase 0 Governance Note
description: Governance lock for WR-028 implementation decisions before product-code work.
status: active
owner: apps/runenwerk_editor
layer: domain / app-runtime / editor-ui / engine-render
canonical: false
last_reviewed: 2026-05-17
related_reports:
  - ./plan.md
---

# WR-028 Phase 0 Governance Note

This note records the required Phase 0 decisions for WR-028. Product-code work may proceed only within these boundaries.

## Selected Identities

- Stable SDF primitive source identity: `editor_core::EntityId` is the current editor-scene SDF primitive identity used by `domain/editor/editor_scene/src/model/entity.rs`, scene commands, and viewport extraction. WR-028 may wrap it in an SDF-specific assignment type, but assignments must remain editor-scene source state and must not use ECS runtime entities or UI selection ids.
- Scene material source identity: add or confirm `SceneMaterialSourceRef` as a source reference over `AssetId` plus `AssetSourceId` plus source revision or an equivalent repo-proven material source identity. It must not use raw artifact ids, shader ids, renderer table indices, paths, display names, or UI ids.
- Runtime material table entry identity: derived from scene material slot id, material source ref, generated material/shader identity, texture residency identity, and assignment revision.

## Selected Graph Canvas Substrate Shape

- Use a retained graph canvas node or equivalent retained UI tree representation.
- `domain/ui/ui_graph_editor` owns backend-neutral graph interaction vocabulary and session view/gesture state.
- `domain/ui/ui_tree`, `domain/ui/ui_runtime`, and `domain/ui/ui_render_data` own retained representation, input ownership, layout, clipping, and primitive emission.
- `domain/editor/editor_shell` composes the material graph surface only.
- `apps/runenwerk_editor` maps graph actions to source-backed material workflow commands.
- `domain/material_graph` remains the only material graph semantic, validation, catalog, lowering, and source mutation owner.

## Selected GPU Proof Harness

- Primary proof harness: extend the existing windowed GPU capture path in `apps/runenwerk_editor/tests/viewport_gpu_truth_smoke.rs`.
- Capture method: render debug provenance and `RenderCapturedTextureState` readback for scene color and material/texture preview products.
- CI status: GPU proof may remain manual-only when a real GPU/window is required, but manual-only status and ignored-in-CI reason must be recorded in `proof-manifest.ron`.

## WR-029 Handoff

- Proposed follow-up id: WR-029.
- Proposed scope: model/mesh material binding, model/mesh source identity, submesh/material-region assignment, and any `renderable_index` ABI extension.
- WR-028 must preserve model/mesh gap language in design, roadmap, production-track, audit, and closeout docs until WR-029 is accepted and proven.

## Confirmed Packages And Tasks

Confirmed packages:

- `material_graph`
- `editor_scene`
- `ui_graph_editor`
- `ui_tree`
- `ui_runtime`
- `ui_render_data`
- `texture`
- `asset`
- `engine`
- `editor_shell`
- `runenwerk_editor`

Confirmed tasks:

- `task planning:validate`
- `task docs:validate`
- `task roadmap:validate`
- `task roadmap:render`
- `task roadmap:check`
- `task production:validate`
- `task production:render`
- `task production:check`
- `task ai:architecture-governance`

## Acceptance Test Inventory

Existing or broad current tests to reuse:

- `cargo test -p material_graph persistence`
- `cargo test -p editor_scene material`
- `cargo test -p texture texture_preview_descriptor`
- `cargo test -p asset texture`
- `cargo test -p engine material`
- `cargo check -p runenwerk_editor`

Required focused tests to create or confirm during implementation:

- `cargo test -p runenwerk_editor material_graph_source_edits_round_trip_v2`
- `cargo test -p ui_graph_editor graph_canvas_hit_testing`
- `cargo test -p ui_runtime graph_canvas_pointer_capture`
- `cargo test -p ui_render_data graph_canvas_emits_primitives`
- `cargo test -p editor_shell material_graph_surface_contains_graph_canvas`
- `cargo test -p ui_graph_editor graph_canvas_has_no_material_semantics`
- `cargo test -p editor_shell material_graph_actions_route_to_workflow_adapters`
- `cargo test -p runenwerk_editor material_assignment_mutates_editor_scene_not_material_lab_runtime`
- `cargo test -p runenwerk_editor texture_viewer_gpu_preview_uses_catalog_residency`
- `cargo test -p engine generated_scene_wgsl_reads_hit_material_slot`
- `cargo test -p runenwerk_editor sdf_two_primitives_render_different_material_slots`

## Required Design/Planning Updates

- Update `docs-site/src/content/docs/design/active/material-lab-and-material-preview-design.md` so WR-028 acceptance is SDF primitive only and WR-029 owns model/mesh material binding.
- Update roadmap, production-track, audit, and inherited closeout language that currently implies WR-028 closes model/mesh material binding.
- No ADR is required before starting WR-028 SDF-only implementation if the implementation keeps the ownership boundaries above. An ADR or accepted design update is required before implementing model/mesh material binding.
