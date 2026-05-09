---
title: P1 SDF Modeling Core Closeout
description: Completion and drift-check record for full P1 SDF operation, graph, boolean, invalidation, and CPU field-preview work.
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
  - ../../../domain/world-sdf/README.md
  - ../../../domain/sdf/README.md
---

# P1 SDF Modeling Core Closeout

## Status

Complete as of 2026-05-09 for the CPU/editor-surface P1 boundary.

This closeout completes P1-A plus the field-preview, graph-authoring, and accepted boolean work. It does not start M6.2 procgen, gameplay graph, particles, physics, animation, simulation processes, scripting, renderer-private SDF preview execution, GPU field textures, or P3 material/SDF rendered previews.

## Completion Evidence

- `domain/editor/editor_scene/src/sdf_authoring/` owns authored SDF operation documents, source-backed SDF graph documents over `domain/graph`, command intents, semantic ratification, projection DTOs, deterministic lowering to `world_ops::OperationRecord` windows, and CPU field-preview formation.
- `domain/world_ops/src/operations.rs::CsgBrushOperation` represents accepted P1 CSG brush semantics for Add, Subtract, Intersect, SmoothAdd, SmoothSubtract, and SmoothIntersect while preserving legacy Add/Subtract operation variants for existing logs.
- `domain/world_ops/src/invalidation.rs::dirty_reason_for_operation` classifies normalized P1 CSG brush operations as geometry invalidation.
- `domain/world_sdf/src/preview.rs` owns CPU field-preview payload DTOs for scalar distance, vector gradient, occupancy support, and material channel products, including descriptor/payload ratification.
- `apps/runenwerk_editor/src/editor_app/sdf_operations.rs::SdfOperationWorkspaceState` appends committed operation windows to an app-held `world_ops::OperationLog`, marks dirty chunks, forms CPU preview products, and tracks selected preview products without writing renderer or engine-private storage.
- `apps/runenwerk_editor/src/shell/providers/field_layer_stack.rs::FieldLayerStackProvider` shows committed vs draft operation status, dirty chunks, preview-product count, ratification, and commit eligibility.
- `apps/runenwerk_editor/src/shell/providers/sdf_graph_canvas.rs::SdfGraphCanvasProvider` is command-backed through typed graph commands while keeping canvas/session state non-authoritative.
- `apps/runenwerk_editor/src/shell/providers/field_product_viewer.rs::FieldProductViewerProvider` shows selected CPU field-preview product kind, sample grid, sample count, lineage revision, freshness, cache/rebuild policy, and reload diagnostics.

## Drift Corrections

- The active procedural workflow plan now records P1 as complete for CPU-formed field previews and command-backed SDF graph authoring, with rendered/GPU overlays deferred to P3.
- The editor roadmap now points to M6.2 procgen as the next M6 phase only after the procgen domain doc is accepted.
- `world_ops`, `world_sdf`, and editor domain docs now list the new public contracts and boundaries.
- The P1-A closeout remains as a historical operation-layer drift record and is explicitly superseded for milestone status by this report.

## Deferred Work

- Rendered viewport overlays and GPU field texture upload.
- P3 material/SDF rendered preview handoff.
- Texture3D GPU upload/runtime adapters.
- M6.2 procgen and later gameplay, particles, physics, animation, and simulation-process domains.
- M7 scripting/runtime gameplay execution.

## Validation

Baseline validation before implementation:

- `python3 tools/docs/validate_docs.py`
- `cargo check -p runenwerk_editor`
- `cargo test -p editor_scene -p world_ops -p world_sdf -p editor_shell`
- `./quiet_editor_gate.sh`
- `./quiet_full_gate.sh`

Focused validation during implementation:

- `cargo check -p editor_scene -p world_ops -p world_sdf`
- `cargo check -p runenwerk_editor`
- `cargo test -p editor_scene sdf`
- `cargo test -p sdf`
- `cargo test -p world_ops`
- `cargo test -p world_sdf`
- `cargo test -p editor_shell sdf`
- `cargo test -p runenwerk_editor sdf`

Milestone closeout validation:

- `cargo fmt --all -- --check`
- `cargo check -p runenwerk_editor`
- `./quiet_editor_gate.sh`
- `python3 tools/docs/validate_docs.py`
- `./quiet_full_gate.sh`
