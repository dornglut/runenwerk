---
title: Workspace Roadmap Index
description: Workspace-level roadmap index and sequencing links across active architecture tracks.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-10
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
- M5 runtime preview closeout evidence:
  - [reports/closeouts/m5-runtime-preview/closeout.md](../reports/closeouts/m5-runtime-preview/closeout.md)
- M6.1 material/texture descriptor preview closeout evidence:
  - [reports/closeouts/m6-material-texture-descriptor-preview/closeout.md](../reports/closeouts/m6-material-texture-descriptor-preview/closeout.md)
- P1 SDF modeling core closeout evidence:
  - [reports/closeouts/p1-sdf-modeling-core/closeout.md](../reports/closeouts/p1-sdf-modeling-core/closeout.md)
- P1-A SDF operation-layer historical closeout evidence:
  - [reports/closeouts/p1-sdf-operation-layer/closeout.md](../reports/closeouts/p1-sdf-operation-layer/closeout.md)

## Current Cross-Track Status

- viewport backend cleanup is complete for its closeout scope;
- workspace structural identity and routing contracts are implemented and guard-tested;
- UI substrate crates and `ui_surface` contracts are implemented and integrated in production editor flows;
- editor MVP acceptance is complete and the active editor/UI work has moved to the repository Now list;
- current active editor/UI work has completed M3.6 self-authoring: durable schemas, Editor Design workspace/profile, provider surfaces, fixture document loading, validation, retained previews, command diff summaries, retained authoring control routes, UI node/theme/workspace-layout draft edits, and apply/rollback exist before SDF/field asset and procedural workspace expansion.
- the follow-on provider surface workflow redesign and self-authoring maturity pass are complete: outliner, entity-table, inspector, viewport, and editor-definition provider actions route through typed surface wrappers; entity-table query workflows, typed no-payload ECS enum inspector mutation, active UI/editor definition catalogs, and richer inspector controls are implemented while provider behavior stays outside `ui_definition`;
- M4A-M4I integrated UI/editor/asset foundation is complete as of 2026-05-09: active menus, shortcuts, command bindings, panel registries, tool-surface registries, reusable-control cleanup, `domain/asset`, `ProjectFileV2`, field-product descriptors, generic product invalidation, first app-owned import/field-product jobs, asset provider surfaces, scene-manifest catalog adapter, and displayable field/volume viewport debug products exist.
- M5 external runtime preview, project-owned data hot reload classification, reload diagnostics projection, world_sdf runtime intake, and restart boundaries are complete as of 2026-05-09 for the existing scene/asset/field-product/world_sdf/shader/UI-definition slice.
- M6 has started: shared workspace/profile/surface substrate, first material/texture domain-contract crates, descriptor-first material/texture providers, and full P1 SDF modeling core exist in the current worktree; M6.2 procgen is next only after the procgen domain doc is accepted, while gameplay and later procedural domains remain open.

## Recommended Near-Term Order

1. Start M6.2 procgen design/domain-doc gate, then implement its first slice only after `docs-site/src/content/docs/domain/procgen/README.md` is accepted.
2. Keep rendered SDF/GPU overlays and P3 material/SDF preview handoff deferred from the P1 closeout path.
3. Sequence procgen, particles, physics, animation, and simulation domains after each owning first-slice design and formed-product contract exists.
4. Add gameplay graph ATR IR and ECS lowering only after narrower gameplay event/action/state/quest, authority, and source-map contracts are explicit.
5. Treat compiled-reactive UI, ECS-driven UI, world-space/runtime UI, in-game editors, native OS menu/shortcut mirroring, external marketplace packages, and payload ECS enum variants as design-first gates, not implementation tickets.
6. Preserve architecture guards and keep domain/workspace docs synchronized as each phase lands.

## Rule

When domain roadmaps and workspace index notes diverge, the owning domain roadmap is authoritative for implementation sequencing.
