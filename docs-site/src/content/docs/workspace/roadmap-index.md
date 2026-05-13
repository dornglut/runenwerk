---
title: Workspace Roadmap Index
description: Workspace-level roadmap index and sequencing links across active architecture tracks.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-13
---

# Workspace Roadmap Index

## Purpose

Provide one workspace-level index of active roadmap/design tracks without duplicating domain-level phase details.

This page is an index, not the source of truth for domain-specific execution steps.

Operational execution checklist:

- [workspace/sdf-first-execution-roadmap.md](./sdf-first-execution-roadmap.md)
- [workspace/repo-execution-priority-checklist.md](./repo-execution-priority-checklist.md)
- [apps/runenwerk-editor/roadmap.md](../apps/runenwerk-editor/roadmap.md)
- [reports/audits/editor-ui-priority-code-audit-2026-05-05.md](../reports/audits/editor-ui-priority-code-audit-2026-05-05.md)

## Source-of-Truth Tracks

- SDF-first cross-track execution order:
  - [workspace/sdf-first-execution-roadmap.md](./sdf-first-execution-roadmap.md)
- SDF-first execution Phase 1 closeout evidence:
  - [reports/closeouts/sdf-first-execution-phase-1/closeout.md](../reports/closeouts/sdf-first-execution-phase-1/closeout.md)
- SDF-first execution Phase 2 closeout evidence:
  - [reports/closeouts/sdf-first-execution-phase-2/closeout.md](../reports/closeouts/sdf-first-execution-phase-2/closeout.md)
- SDF-first execution Phase 3 closeout evidence:
  - [reports/closeouts/sdf-first-execution-phase-3/closeout.md](../reports/closeouts/sdf-first-execution-phase-3/closeout.md)
- SDF-first execution Phase 4 closeout evidence:
  - [reports/closeouts/sdf-first-execution-phase-4/closeout.md](../reports/closeouts/sdf-first-execution-phase-4/closeout.md)
- SDF-first execution Phase 5 closeout evidence:
  - [reports/closeouts/sdf-first-execution-phase-5/closeout.md](../reports/closeouts/sdf-first-execution-phase-5/closeout.md)
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
- SDF-first field-world platform architecture:
  - [design/accepted/sdf-first-field-world-platform-design.md](../design/accepted/sdf-first-field-world-platform-design.md)
- SDF-first production capability map:
  - [design/accepted/sdf-first-production-capability-map.md](../design/accepted/sdf-first-production-capability-map.md)
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

## Current Focus

- Plan and execute the remaining SDF-first Phase 6B / M6.2 editor-runtime
  procgen proof from `workspace/sdf-first-execution-roadmap.md`. Phase 6A has
  created `domain/procgen` for graph-backed deterministic documents,
  terrain/material node semantics, ratification, lowering, and product
  job/publication descriptors.
- Keep remaining M6.2 procgen code scoped to bounded region terrain/material
  generation through `domain/procgen`, product publication barriers, query
  snapshots, render selection, and derived GPU residency.
- Keep editor/UI cross-doc sequencing aligned with shipped behavior while the
  execution substrate lands.
- Keep render SDF/GPU work on the completed product-selection and derived
  GPU-residency contracts.

## Other Tracked Work

- Sequence gameplay graph, particles, physics, animation, and simulation domains
  after each owning design can consume product jobs, query snapshots,
  publication barriers, and diagnostics.
- Treat compiled-reactive UI, ECS-driven UI, world-space/runtime UI, in-game
  editors, native OS menu/shortcut mirroring, external marketplace packages,
  and payload ECS enum variants as design-first gates, not implementation
  tickets.
- Preserve architecture guards and keep domain/workspace docs synchronized as
  each phase lands.

## Rule

When domain roadmaps and workspace index notes diverge, the owning domain roadmap is authoritative for implementation sequencing.

## Finished Cross-Track Baselines

- viewport backend cleanup is complete for its closeout scope;
- SDF-first execution Phase 1 is complete: serial product publication outcomes,
  deterministic publication barriers, ECS generic barrier hooks, engine runtime
  publication staging, and app-owned editor field-product barrier publication
  are implemented and validated;
- SDF-first execution Phase 2 is complete: runtime query snapshots, strict
  product consumption decisions, query-snapshot publication barriers,
  product-agnostic ECS source generation, engine snapshot staging/invalidation,
  render inspection DTOs, and app-owned editor viewport observation snapshot
  publication are implemented and validated;
- SDF-first execution Phase 3 is complete: typed render product selections,
  producer-scoped prepared render selection contributions, prepared-frame
  inspection DTOs, and app-owned editor viewport render-selection production
  from accepted query snapshots are implemented and validated;
- SDF-first execution Phase 4 is complete: renderer-owned logical GPU cache
  handles, deterministic derived residency allocation, preservation,
  invalidation, eviction, and rejection, read-only inspection, editor viewport
  residency summaries, and typed world render-cache invalidation are
  implemented and validated;
- SDF-first execution Phase 5 is complete: the accepted procgen domain contract
  defines `domain/procgen` ownership, graph-backed generator documents,
  planning lifecycle metadata, reservation boundaries, seed/scope/version
  policy, cache lineage, authored overlay preservation, runtime/offline policy,
  server-validated multiplayer authority, and product output paths;
- SDF-first execution Phase 6A is complete: `domain/procgen` implements the
  domain-first procgen product track for graph-backed deterministic documents,
  terrain/material node semantics, ratification, lowering to world operation
  windows, planning metadata, and product job/publication descriptors;
- workspace structural identity and routing contracts are implemented and
  guard-tested;
- UI substrate crates and `ui_surface` contracts are implemented and integrated
  in production editor flows;
- editor MVP acceptance is complete;
- M3.6 self-authoring is complete: durable schemas, Editor Design
  workspace/profile, provider surfaces, fixture document loading, validation,
  retained previews, command diff summaries, retained authoring control routes,
  UI node/theme/workspace-layout draft edits, and apply/rollback exist before
  SDF/field asset and procedural workspace expansion;
- the provider surface workflow redesign and self-authoring maturity pass are
  complete: outliner, entity-table, inspector, viewport, and editor-definition
  provider actions route through typed surface wrappers while provider behavior
  stays outside `ui_definition`;
- M4A-M4I integrated UI/editor/asset foundation is complete as of 2026-05-09;
- M5 external runtime preview, project-owned data hot reload classification,
  reload diagnostics projection, world_sdf runtime intake, and restart
  boundaries are complete as of 2026-05-09 for the existing
  scene/asset/field-product/world_sdf/shader/UI-definition slice;
- M6 has started with shared workspace/profile/surface substrate, first
  material/texture domain-contract crates, descriptor-first material/texture
  providers, full P1 SDF modeling core, and Batch 1 SDF-first contract
  alignment.
