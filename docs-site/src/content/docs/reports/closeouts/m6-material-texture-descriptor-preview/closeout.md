---
title: M6.1 Material And Texture Descriptor Preview Closeout
description: Completion and drift-check record for the descriptor-first M6.1 material and texture provider slice.
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
  - ../../../domain/material-graph/README.md
  - ../../../domain/texture/README.md
---

# M6.1 Material And Texture Descriptor Preview Closeout

## Status

Complete as of 2026-05-09 for the descriptor-first M6.1 slice.

This closeout does not start M6.2 procgen, gameplay graph, particles, physics, animation, simulation processes, scripting, graph execution, renderer-private material execution, or Texture3D GPU upload.

## Completion Evidence

- `domain/material_graph` owns authored material graph documents, first-slice catalog entries, semantic ratification, deterministic lowering, source maps, cache keys, and formed material product descriptors.
- `domain/texture` owns Texture2D and Texture3D/volume descriptors, sampler/color-space/compression metadata, generated texture lineage, typed preview descriptors, and ratification.
- `apps/runenwerk_editor/src/shell/providers/material_graph_canvas.rs::MaterialGraphCanvasProvider` projects catalog and material graph descriptor status without making canvas state authoritative.
- `apps/runenwerk_editor/src/shell/providers/material_inspector.rs::MaterialInspectorProvider` projects formed material product, cache, source, and reload diagnostics without exposing mutation routes.
- `apps/runenwerk_editor/src/shell/providers/material_preview.rs::MaterialPreviewProvider` exposes the descriptor-first preview boundary and fails closed until the P3 render adapter exists.
- `apps/runenwerk_editor/src/shell/providers/texture_viewer.rs::TextureViewerProvider` and `volume_texture_viewer.rs::VolumeTextureViewerProvider` project texture products and `TexturePreviewDescriptor` mip/slice/channel intent while keeping GPU upload adapter-owned.
- `apps/runenwerk_editor/src/shell/providers/m6_workspace.rs::M6WorkspaceProvider` remains the low-priority fail-closed fallback for unimplemented M6 surfaces.

## Drift Corrections

- The editor roadmap previously said concrete material/texture providers did not exist. It now records the descriptor-first provider closeout and names P1/SDF hardening as the next phase before M6.2.
- The active procedural workflow design previously marked texture viewers and material providers as missing. It now distinguishes descriptor-first provider surfaces from deferred rendered preview and GPU upload adapters.
- The workspace roadmap index and priority checklist now reflect that material/texture provider surfaces have landed while later procedural/gameplay domains remain gated.
- The material-graph and texture domain docs now state the editor provider boundary explicitly.

## Deferred Work

- P1/SDF hardening for operation layers, SDF graph authoring, field previews, and material-channel overlays.
- P3 material/render handoff, including the first PBR capability matrix, rendered material previews, and runtime adapter support.
- Texture3D GPU upload/runtime adapter, supported format set, and compression policy beyond descriptor-first inspection.
- Full source-backed material graph document persistence/import UX.
- M6.2 procgen and later M6 procedural/gameplay domains.

## Validation

Completed baseline validation before this slice:

- `python3 tools/docs/validate_docs.py`
- `cargo fmt --all -- --check`
- `cargo test -p material_graph -p texture -p asset -p editor_preview -p editor_shell`
- `cargo check -p runenwerk_editor`
- `./quiet_editor_gate.sh`
- `./quiet_full_gate.sh`

Completed closeout validation after this slice:

- `cargo check -p runenwerk_editor`
- `cargo test -p runenwerk_editor providers`
- `cargo test -p material_graph -p texture -p asset -p editor_preview -p editor_shell`
- `python3 tools/docs/validate_docs.py`
- `./quiet_full_gate.sh`
