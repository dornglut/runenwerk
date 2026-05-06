---
title: Repository Execution Priority Checklist
description: Cross-repo Now/Next/Later execution checklist that links to authoritative domain/app roadmaps.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-06
related:
  - ./roadmap-index.md
  - ../apps/runenwerk-editor/execution-priority-checklist.md
  - ../domain/ui/roadmap.md
  - ../engine/plugins/render/docs/roadmap.md
  - ../net/ecs-runtime-prioritized-roadmap.md
  - ../net/multiplayer-replication-implementation-roadmap.md
  - ../reports/audits/editor-ui-priority-code-audit-2026-05-05.md
  - ../design/active/editor-workspace-document-mode-panel-architecture.md
  - ../design/active/editor-self-authoring-and-final-ui-design.md
  - ../design/active/editor-asset-pipeline-and-content-workflow-design.md
  - ../design/active/editor-procedural-content-and-simulation-workflow-plan.md
  - ../design/active/engine-game-runtime-editor-ecs-scripting-hot-reload-design.md
---

# Repository Execution Priority Checklist

## Purpose

Provide one practical repository-wide execution checklist.

This page is an operational summary.
Owning domain/app roadmap docs remain the source of truth for detailed sequencing.

## Usage Rules

- Keep MVP scope unchanged unless owning MVP docs change first.
- Update checkbox status only when code and validation evidence exist.
- If this checklist and an owning roadmap differ, the owning roadmap wins.

## Now (Current Cross-Repo Priorities)

- [x] Complete the Runenwerk editor MVP critical path and acceptance gate. Status: automated and manual/UI verified; the source checklist is complete (source: `apps/runenwerk-editor/execution-priority-checklist.md`).
- [x] Editor Phase A: introduce workspace profile abstraction without breaking MVP. Status: implemented and test-covered (source: `apps/runenwerk-editor/execution-priority-checklist.md` and `design/active/editor-workspace-document-mode-panel-architecture.md`).
- [x] Editor Phase B: decouple workspace layout persistence from scene path coupling. Status: implemented and test-covered (source: `apps/runenwerk-editor/execution-priority-checklist.md` and `design/active/editor-workspace-document-mode-panel-architecture.md`).
- [x] Editor Phase C: formalize document tabs and active document switching. Status: implemented and test-covered; document taxonomy, ordered tabs, active switching, dirty/save/close transitions, compatibility validation, and generic app-local document-tab runtime state are in place (source: `apps/runenwerk-editor/execution-priority-checklist.md` and `design/active/editor-workspace-document-mode-panel-architecture.md`).
- [x] Editor Phase D: replace adapter-only panel wiring with provider registry routing. Status: implemented and test-covered; provider DTOs carry workspace profile, document context, surface definition, capabilities, and provider-local routes, and concrete providers are split into subdomain modules (source: `apps/runenwerk-editor/execution-priority-checklist.md` and `design/active/editor-workspace-document-mode-panel-architecture.md`).
- [x] Close editor app-domain operation migration seams for ratification, scene command execution, transaction orchestration, history, scene, and selection ownership. Status: implemented for the M1 seam; domain-owned scene operation functions handle command/transaction execution plus history insertion, while app-owned ECS/reflection, snapshots, retention, projection parity, selection sync, and recording remain app-local (source: `apps/runenwerk-editor/roadmap.md` and `reports/audits/editor-ui-priority-code-audit-2026-05-05.md`).
- [x] Editor Phase E: expand global mode enum into scoped workspace/document mode sets. Status: implemented and test-covered through mode ids, descriptors, a registry, and workspace/document compatibility validation (source: `apps/runenwerk-editor/execution-priority-checklist.md` and `design/active/editor-workspace-document-mode-panel-architecture.md`).
- [x] Complete UI docking/tab behavior on top of existing structural identity and binding contracts. Status: implemented and test-covered; tab reorder, rehome, floating host creation, split resizing, area split/duplicate/reset/close, dynamic split-area composition, and structural identity preservation have automated coverage (source: `apps/runenwerk-editor/roadmap.md` and `apps/runenwerk-editor/execution-priority-checklist.md`).
- [x] Expose editor-area/type switching as a reachable shell UI control. Status: implemented and test-covered; tab chrome renders an editor type selector and routes `SelectChanged` through `SwitchPanelToolSurfaceKind` (source: `apps/runenwerk-editor/execution-priority-checklist.md`).
- [x] Add plus/new-tab affordance for creating a new tab in a tab stack. Status: implemented and test-covered; tab chrome exposes a plus/new-tab control that allocates panel and tool-surface identities through `WorkspaceIdentityAllocator` after structural ratification (source: `apps/runenwerk-editor/execution-priority-checklist.md`).
- [ ] Expand non-viewport surface maturity using existing surface contracts. Status: active/partially implemented; entity-table, console, inspector, outliner, and independent surface-session tests exist, but richer surface coverage remains open (source: `domain/ui/roadmap.md`).
- [ ] Preserve and extend UI/editor guard coverage for structural routing, capability gating, and seam ownership. Status: active; current guard suites pass (source: `domain/ui/roadmap.md` and `workspace/roadmap-index.md`).
- [ ] Keep editor/UI cross-doc sequencing aligned with shipped behavior. Status: active; docs validation currently passes (source: `domain/ui/roadmap.md` and `workspace/roadmap-index.md`).
- [ ] Insert the M3.5 UI definition formation framework before M3.6 and M4. Status: planned; the owning roadmaps now place `domain/ui/ui_definition` and `domain/editor/editor_definition` before UI self-authoring and asset/procedural workspace expansion so menu, toolbar, shell chrome, provider surface, workspace, popover, theme, and unavailable-feature structure does not keep hard-coding into shell/app paths (source: `apps/runenwerk-editor/roadmap.md` and `domain/ui/roadmap.md`).
- [ ] Implement the M3.6 UI self-authoring workspace and styling track before M4. Status: planned; the former final self-authoring work is now promoted before asset/procedural/gameplay expansion so later editor, debug overlay, runtime overlay, and game UI templates can be authored through the same definition system (source: `apps/runenwerk-editor/roadmap.md` and `design/active/editor-self-authoring-and-final-ui-design.md`).
- [x] Add rotate and scale gizmos after translate workflow is stable. Status: implemented and test-covered for M3; translate, rotate, and scale tool activation, preview, snap-aware interaction, commit, undo/redo, and scene-authoring smoke coverage exist (source: `apps/runenwerk-editor/execution-priority-checklist.md` and `apps/runenwerk-editor/roadmap.md`).
- [x] Add create/delete/duplicate flows for common scene-authoring actions. Status: implemented and test-covered for M3; outliner/app scene commands cover create child, rename, reparent, duplicate subtree, delete single entity, batch delete, and SDF primitive creation through viewport SDF tool routing (source: `apps/runenwerk-editor/execution-priority-checklist.md` and `apps/runenwerk-editor/roadmap.md`).
- [x] Improve inspector/component authoring breadth for common 3D flows. Status: implemented and test-covered for M3; common reflected primitive fields are editable through typed inspector edit values, and component add/remove remains routed through scene command intents (source: `apps/runenwerk-editor/execution-priority-checklist.md` and `apps/runenwerk-editor/roadmap.md`).
- [ ] Build the editor SDF/field-first asset pipeline foundation and import workflow. Status: open; `ProjectFileV1`, scene RON migration/normalization/formation, `domain/sdf`, `domain/world_ops`, `domain/world_sdf`, loose scene manifests, shader reload helpers, render import descriptors, Blender config, and model files exist, but there is no asset domain/catalog/SDF-field asset taxonomy/import plan/field-product formation plan/artifact cache/asset browser/field-product viewer/import diagnostics pipeline yet. Mesh/GLB import remains a foreign-reference path, not the primary world substrate (source: `apps/runenwerk-editor/roadmap.md` and `design/active/editor-asset-pipeline-and-content-workflow-design.md`).
- [ ] Sequence procedural authoring domains for material/texturing, procgen, particles, physics, animation, and simulation. Status: open; target plan exists, but the domain crates and editor providers are not implemented yet (source: `design/active/editor-procedural-content-and-simulation-workflow-plan.md`).
- [ ] Keep scripting boundary language-neutral with Rhai as first adapter candidate when scripting implementation starts. Status: promoted to Now for editor/runtime tracking, but not implementation-started (source: `design/active/engine-game-runtime-editor-ecs-scripting-hot-reload-design.md`).
- [ ] Add world-space/screen-projected UI attachment binding through explicit authored binding contracts and runtime formation seams. Status: promoted to Now for tracking, but still boundary-gated by the owning design doc before implementation (source: `design/active/engine-game-runtime-editor-ecs-scripting-hot-reload-design.md`).
- [ ] Keep constrained in-game editors capability-gated with explicit command/ratification boundaries. Status: promoted to Now for tracking, but still boundary-gated by the owning design doc before implementation (source: `design/active/editor-workspace-document-mode-panel-architecture.md`).
- [ ] Continue Editor Design/self-authoring track for UI/editor layout design, creation, and management. Status: promoted to planned M3.6 after M3.5 and before M4 (source: `design/active/editor-self-authoring-and-final-ui-design.md`, `apps/runenwerk-editor/roadmap.md`, and `domain/ui/roadmap.md`).
- [ ] Execute render immediate remaining phases: `R4` binding model expansion, `R6` boids feature proof, `R7` SDF renderer rebuild on new path (source: `engine/plugins/render/docs/roadmap.md`).
- [ ] Continue ECS runtime convergence open foundation checklist items (`F1`-`F4`) in Priority 1 (source: `net/ecs-runtime-prioritized-roadmap.md`).
- [ ] Keep architecture guards and docs synchronized while these tracks land (source: `workspace/roadmap-index.md`).

## Next (After Now Is Stable)

- [ ] Execute render usability/data-maturity phases `R5`, `R8`, `R9`, `R10` (source: `engine/plugins/render/docs/roadmap.md`).
- [ ] Drive multiplayer replication through Milestone A (authoritative replication core) with the existing phase plan (source: `net/multiplayer-replication-implementation-roadmap.md`).
- [ ] Continue ECS runtime multiplayer-enabling checklist items (`M1`-`M5`) in Priority 2 (source: `net/ecs-runtime-prioritized-roadmap.md`).

## Later (Explicitly Deferred or Long-Horizon)

- [ ] Advance long-horizon geometry roadmap milestones when current higher-priority active tracks are stable (source: `domain/geometry/implementation-roadmap.md`).

## Completed Baselines (Do Not Reopen Without Reason)

- [x] Foundation SDF roadmap baseline is implemented (source: `domain/sdf/implementation-roadmap.md`).
- [x] Domain ECS Phase 6 closeout roadmap package is complete (source: `domain/ecs/roadmaps/phase6-closeout-roadmap.md`).
