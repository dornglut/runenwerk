---
title: M5 Runtime Preview Closeout
description: Completion and drift-check record for the M5 external runtime preview, reload, and restart-boundary slice.
status: completed
owner: editor
layer: app
canonical: true
last_reviewed: 2026-05-09
related_designs:
  - ../../../design/active/engine-game-runtime-editor-ecs-scripting-hot-reload-design.md
related_adrs:
  - ../../../adr/accepted/0007-external-runtime-preview-process.md
related_roadmaps:
  - ../../../apps/runenwerk-editor/roadmap.md
  - ../../../workspace/roadmap-index.md
---

# M5 Runtime Preview Closeout

## Status

Complete as of 2026-05-09.

The completed phase is the concrete M5 external-runtime slice for runtime preview, data hot reload classification, and restart boundaries. It does not start M6 procedural, gameplay, scripting, physics, particle, animation, or material/product domain implementation.

## Owning Scope

- Roadmap: `docs-site/src/content/docs/apps/runenwerk-editor/roadmap.md`, M5 Runtime Preview, Data Hot Reload, And Restart Boundaries.
- Design: `docs-site/src/content/docs/design/active/engine-game-runtime-editor-ecs-scripting-hot-reload-design.md`, M5 External Runtime Preview Boundary.
- ADR: `docs-site/src/content/docs/adr/accepted/0007-external-runtime-preview-process.md`.
- Domain crate: `domain/editor/editor_preview`.
- Net crates: `net/engine_net`, `net/engine_net_quic`.
- App crates: `apps/runenwerk_editor`, `apps/runenwerk_runtime_preview`.
- Engine subsystem: `engine/src/plugins/world/build`.

## Completion Evidence

- `domain/editor/editor_preview` owns engine-agnostic preview protocol vocabulary, checked command/event payload metadata, postcard payload encoding, lower-hex helpers, and the serialized bootstrap stdout format.
- `net/engine_net` owns generic bidirectional typed payload messages; preview semantics do not appear in generic network protocol enums.
- `net/engine_net_quic` carries generic typed payload messages over loopback QUIC.
- `apps/runenwerk_runtime_preview` owns the child runtime preview host, bootstrap output, headless command/event loop, and command handling for start session, mode changes, heartbeat, reload status, product publish, and shutdown.
- `apps/runenwerk_editor/src/runtime/preview_process/` owns child spawn, bootstrap parsing, connection, bounded command queueing, event ingestion, heartbeat/mode/reload/shutdown state, graceful shutdown, and fallback cleanup.
- `apps/runenwerk_editor/src/asset_pipeline/catalog_runtime.rs` classifies the current M5 product families as live reload, preview-session restart, runtime-process restart, unsupported, failed-preserved, or rejected.
- `engine/src/plugins/world/build/integration.rs::enqueue_ratified_world_sdf_payload_package` is the narrow engine-owned intake path for ratified `world_sdf` payload packages.
- Existing editor surfaces receive status through the current console, asset/import, field-product, and viewport/product diagnostics paths; M5 did not add a separate runtime-preview diagnostics surface.

## Drift Findings

- `docs-site/src/content/docs/design/active/engine-game-runtime-editor-ecs-scripting-hot-reload-design.md`, frontmatter: `last_reviewed` still predated the accepted M5 implementation. Corrected to 2026-05-09 and linked this closeout report.
- `docs-site/src/content/docs/apps/runenwerk-editor/roadmap.md`, M5 status sections: M5 was marked complete but did not link dedicated closeout evidence. Corrected both active M5 status locations to link this report.
- `docs-site/src/content/docs/apps/runenwerk-editor/roadmap.md`, M5 validation lists: the focused validation text omitted `python3 tools/docs/validate_docs.py` even though docs validation is part of the phase closeout. Corrected the validation lists.
- `docs-site/src/content/docs/workspace/roadmap-index.md`, closeout evidence list: M5 did not have a closeout evidence entry. Added this report.

No code drift was found that requires changing the implemented M5 boundary. The implementation preserves the accepted dependency direction: `editor_preview` has no engine, network, or app dependency; networking remains preview-agnostic; the editor does not mutate runtime ECS, shader registry, renderer resource, or `SdfChunkStore` internals directly.

## Deferred Work

The closeout intentionally leaves these items for later phases:

- M6 procedural authoring domains and formed-product contracts.
- Material graph, texture, procgen, particle, physics, animation, gameplay graph, graph execution, scripting, and broader simulation product semantics.
- Replacing central reload-kind matching with a domain capability registry after M6 domains exist.
- Making shader reload status fully typed instead of app-boundary string matching.
- Migrating remaining direct `SdfChunkStore` field access to methods and making internals private after all call sites use the intake API.
- A dedicated runtime preview diagnostics surface; M5 continues projecting status to existing surfaces.

## Next Phase

The next phase is M6 Procedural Authoring Workspaces Beyond Scene Editing.

M6 must start from the owning first-slice designs and domain contracts. It must not be implemented by expanding the M5 preview protocol into missing material, texture, procgen, particles, physics, animation, gameplay, scripting, or graph-execution semantics.

## Validation

Completed validation:

- `cargo metadata --no-deps --format-version 1`
- `cargo test -p editor_preview -p engine_net -p engine_net_quic`
- `cargo test -p runenwerk_runtime_preview -p runenwerk_editor`
- `cargo test -p engine -p asset -p world_sdf`
- `python3 tools/docs/validate_docs.py`
- `./quiet_editor_gate.sh`
- `./quiet_full_gate.sh`

This closeout pass reruns docs validation and the full gate after the documentation drift corrections.
