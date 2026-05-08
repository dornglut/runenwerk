---
title: Workspace Roadmap Index
description: Workspace-level roadmap index and sequencing links across active architecture tracks.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-08
---

# Workspace Roadmap Index

## Purpose

Provide one workspace-level index of active roadmap/design tracks without duplicating domain-level phase details.

This page is an index, not the source of truth for domain-specific execution steps.

Operational execution checklist:

- [workspace/repo-execution-priority-checklist.md](./repo-execution-priority-checklist.md)
- [apps/runenwerk-editor/roadmap.md](../apps/runenwerk-editor/roadmap.md)
- [reports/audits/editor-ui-priority-code-audit-2026-05-05.md](../reports/audits/editor-ui-priority-code-audit-2026-05-05.md)

## Source-of-Truth Tracks

- Editor final end-to-end implementation roadmap:
  - [apps/runenwerk-editor/roadmap.md](../apps/runenwerk-editor/roadmap.md)
- UI substrate and surface execution roadmap:
  - [domain/ui/roadmap.md](../domain/ui/roadmap.md)
- UI current-state architecture:
  - [domain/ui/architecture.md](../domain/ui/architecture.md)
- Editor/UI/workspace long-horizon architecture:
  - [design/active/editor-ui-workspace-tool-surface-architecture.md](../design/active/editor-ui-workspace-tool-surface-architecture.md)
- Editor self-authoring and UI workspace design:
  - [design/active/editor-self-authoring-and-final-ui-design.md](../design/active/editor-self-authoring-and-final-ui-design.md)
- Editor asset pipeline and content workflow design:
  - [design/active/editor-asset-pipeline-and-content-workflow-design.md](../design/active/editor-asset-pipeline-and-content-workflow-design.md)
- Editor procedural content and simulation workflow plan:
  - [design/active/editor-procedural-content-and-simulation-workflow-plan.md](../design/active/editor-procedural-content-and-simulation-workflow-plan.md)
- Gameplay graph ATR IR and ECS lowering design:
  - [design/active/gameplay-graph-atr-ir-and-ecs-lowering-design.md](../design/active/gameplay-graph-atr-ir-and-ecs-lowering-design.md)
- Workspace identity contract and migration map:
  - [design/active/workspace-identity-contract-and-migration-map.md](../design/active/workspace-identity-contract-and-migration-map.md)
- Viewport backend closeout evidence:
  - [reports/closeouts/viewport-backend-cleanup/phase-1-plan.md](../reports/closeouts/viewport-backend-cleanup/phase-1-plan.md)
- Surface workflow contract closeout evidence:
  - [reports/closeouts/surface-workflow-contract-redesign/closeout.md](../reports/closeouts/surface-workflow-contract-redesign/closeout.md)

## Current Cross-Track Status

- viewport backend cleanup is complete for its closeout scope;
- workspace structural identity and routing contracts are implemented and guard-tested;
- UI substrate crates and `ui_surface` contracts are implemented and integrated in production editor flows;
- editor MVP acceptance is complete and the active editor/UI work has moved to the repository Now list;
- current active editor/UI work has completed M3.6 self-authoring: durable schemas, Editor Design workspace/profile, provider surfaces, fixture document loading, validation, retained previews, command diff summaries, retained authoring control routes, UI node/theme/workspace-layout draft edits, and apply/rollback exist before SDF/field asset and procedural workspace expansion.
- the follow-on provider surface workflow redesign and self-authoring maturity pass are complete: outliner, entity-table, inspector, viewport, and editor-definition provider actions route through typed surface wrappers; entity-table query workflows, typed no-payload ECS enum inspector mutation, active UI/editor definition catalogs, and richer inspector controls are implemented while provider behavior stays outside `ui_definition`;
- remaining near-term work is now the integrated UI/editor/asset foundation: active menus, shortcuts, command bindings, panel registries, and tool-surface registries must be consumed by live shell/input/projection paths before new asset/import/field-product surfaces expand the editor.

## Recommended Near-Term Order

1. Add the active command-binding spine: authored route ids resolve to known app/domain commands, and invalid binding activation leaves the previous active catalog unchanged.
2. Consume active shortcuts in `apps/runenwerk_editor/src/runtime/systems/input_bridge.rs::dispatch_shortcuts` through that command-binding spine.
3. Consume active menus in `apps/runenwerk_editor/src/shell/toolbar_adapter.rs` and `domain/editor/editor_shell/src/composition/toolbar_definition.rs`, with checked-in fixtures only as fallback defaults.
4. Finish active panel/tool-surface registry projection in `domain/editor/editor_shell/src/composition/build_editor_shell.rs` so future create/switch choices reflect active definitions while existing workspace state remains unchanged unless a workspace layout is applied.
5. Broaden reusable-control adoption across editor surfaces before adding asset surfaces, keeping provider data, route proposals, and behavior outside `ui_definition`.
6. Add the SDF/field-first asset domain and project-file foundation: `domain/asset`, `ProjectFileV2`, `world_sdf` product descriptors, import plans, diagnostics, ratification, and dependency graph contracts.
7. Add first asset runtime/catalog surfaces through active provider/catalog seams: Asset Browser, Import Inspector, Field Product Viewer, and SDF Brush Browser.
8. Add app-owned import and field-product execution jobs, failed-artifact preservation, Blender/glTF foreign-reference import, and scene-manifest migration to catalog-backed queries.
9. Expand viewport product producers and history workflows for field, atlas, volume, brickmap, and asset-backed edits.
10. Add runtime preview/data hot reload only after catalog, import, and formed-product contracts exist.
11. Sequence procedural material/texturing, procgen, particles, physics, animation, and simulation domains after asset/catalog/product foundations exist.
12. Add gameplay graph ATR IR and ECS lowering only after narrower gameplay event/action/state/quest, authority, and source-map contracts are explicit.
13. Treat compiled-reactive UI, ECS-driven UI, world-space/runtime UI, in-game editors, native OS menu/shortcut mirroring, external marketplace packages, and payload ECS enum variants as design-first gates, not implementation tickets.
14. Preserve architecture guards and keep domain/workspace docs synchronized as each phase lands.

## Rule

When domain roadmaps and workspace index notes diverge, the owning domain roadmap is authoritative for implementation sequencing.
