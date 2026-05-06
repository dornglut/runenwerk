---
title: Workspace Roadmap Index
description: Workspace-level roadmap index and sequencing links across active architecture tracks.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-05
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

## Current Cross-Track Status

- viewport backend cleanup is complete for its closeout scope;
- workspace structural identity and routing contracts are implemented and guard-tested;
- UI substrate crates and `ui_surface` contracts are implemented and integrated in production editor flows;
- editor MVP acceptance is complete and the active editor/UI work has moved to the repository Now list;
- current active editor/UI work is primarily the promoted M3.6 UI self-authoring workspace/styling track after the validated M3.5 UI definition formation closeout candidate, broader non-viewport surface maturity through templates, and continued guard/doc drift control before SDF/field asset and procedural workspace expansion.

## Recommended Near-Term Order

1. Implement M3.6 UI self-authoring workspace, styling, validation, preview, and apply/rollback.
2. Continue richer provider surface template migrations only when retained behavior parity and provider-boundary preservation are explicit.
3. Use the M3.6 workspace to author later editor, debug overlay, runtime overlay, and game UI templates instead of adding new hard-coded shell/app UI.
4. Build the SDF/field-first asset pipeline and field-product foundation after the UI authoring substrate is available.
5. Sequence procedural material/texturing, procgen, particles, physics, animation, and simulation domains after asset/catalog/product foundations exist.
6. Add gameplay graph ATR IR contracts after semantic graph and ECS/runtime boundaries are explicit, then lower first-slice gameplay rules into ECS query/event/schedule products.
7. Preserve and extend architecture guards while these features land.
8. Keep domain and workspace docs synchronized with shipped behavior.

## Rule

When domain roadmaps and workspace index notes diverge, the owning domain roadmap is authoritative for implementation sequencing.
