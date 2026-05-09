---
title: P1-A SDF Operation-Layer Closeout
description: Completion and drift-check record for the command-backed SDF operation-layer hardening slice.
status: completed
owner: editor
layer: app
canonical: true
last_reviewed: 2026-05-09
related_designs:
  - ../../../design/active/editor-procedural-content-and-simulation-workflow-plan.md
related_roadmaps:
  - ../../../apps/runenwerk-editor/roadmap.md
  - ../../../workspace/roadmap-index.md
related_domains:
  - ../../../domain/editor/README.md
  - ../../../domain/world-ops/README.md
  - ../../../domain/sdf/README.md
---

# P1-A SDF Operation-Layer Closeout

## Status

Complete as of 2026-05-09 for the command-backed SDF operation-layer slice.

This closeout does not complete full P1 field previews and does not start M6.2 procgen, gameplay graph, particles, physics, animation, simulation processes, scripting, renderer-private SDF preview execution, or material-channel overlay rendering.

Superseded for milestone status by the full P1 closeout in `docs-site/src/content/docs/reports/closeouts/p1-sdf-modeling-core/closeout.md`. This report remains the historical P1-A drift record.

## Completion Evidence

- `domain/editor/editor_scene/src/sdf_authoring/` owns authored SDF operation documents, layers, operation entries, command intents, semantic ratification, projection DTOs, and deterministic lowering to `world_ops::OperationRecord` windows.
- Supported Add/Subtract primitive operations lower to `world_ops` records with source refs, deterministic seeds, touched chunks, and replay-window metadata.
- Intersect and smooth boolean intents remain authored but emit commit-blocking diagnostics until accepted `world_ops` semantics exist.
- `domain/world_ops/src/invalidation.rs::dirty_reason_for_operation` classifies geometry, material-field, and structure operations before marking dirty chunks.
- `domain/editor/editor_shell/src/surfaces/sdf_operation.rs` defines typed SDF operation surface actions, session mutations, and domain mutations.
- `apps/runenwerk_editor/src/shell/providers/field_layer_stack.rs::FieldLayerStackProvider` projects layer order, enabled state, operation counts, ratification status, touched chunks, and commit eligibility.
- `apps/runenwerk_editor/src/shell/providers/sdf_graph_canvas.rs::SdfGraphCanvasProvider` projects descriptor-first SDF graph status and exposes no canvas-authoritative mutation routes.
- `apps/runenwerk_editor/src/shell/dispatch/sdf_operations.rs` dispatches typed SDF operation session/domain mutations into app-held SDF operation state.

## Drift Corrections

- The editor roadmap now records P1-A as complete and names P1-B field-preview formation as the next SDF phase before M6.2.
- The active procedural workflow design now distinguishes command-backed SDF operation-layer providers from deferred distance/normal/occupancy/material-channel field preview formation.
- Workspace status docs now reflect that P1-A SDF operation-layer providers exist while later field-preview and procedural/gameplay domains remain gated.
- `world_ops` docs now list operation-kind dirty-reason classification as part of the public invalidation surface.

## Deferred Work

- P1-B field-preview product formation for distance, normal/gradient, occupancy, and material-channel overlays.
- Source-backed SDF graph node editing as authoritative document content.
- Renderer or runtime SDF preview adapters beyond descriptor-first projection.
- P3 material/SDF rendered preview handoff.
- M6.2 procgen and later procedural/gameplay domains.

## Validation

Completed baseline validation before this slice:

- `python3 tools/docs/validate_docs.py`
- `cargo check -p runenwerk_editor`
- `cargo test -p editor_scene -p world_ops -p editor_shell`
- `./quiet_editor_gate.sh`
- `./quiet_full_gate.sh`

Completed focused validation during this slice:

- `cargo test -p editor_scene sdf`
- `cargo test -p world_ops`
- `cargo test -p editor_shell sdf`
- `cargo test -p runenwerk_editor sdf`

Milestone closeout validation:

- `cargo check -p runenwerk_editor`
- `./quiet_editor_gate.sh`
- `python3 tools/docs/validate_docs.py`
- `./quiet_full_gate.sh`
