---
title: Repository Execution Priority Checklist
description: Cross-repo Now/Next/Later execution checklist that links to authoritative domain/app roadmaps.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-13
related:
  - ./sdf-first-execution-roadmap.md
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
- For SDF-first cross-track sequencing, `workspace/sdf-first-execution-roadmap.md` wins over local roadmap summaries.
- If this checklist and an owning roadmap differ, the owning roadmap wins.

## Now (Current Cross-Repo Priorities)

- [ ] Execute the SDF-first open-world substrate phases before new product-domain implementation. Status: active; Batch 1 product contract alignment plus Phases 1 through 4 are complete, but procgen readiness and the accepted procgen domain doc still gate M6.2 procgen code (source: `workspace/sdf-first-execution-roadmap.md`).
- [ ] Continue ECS runtime convergence open foundation checklist items (`F1`-`F4`) as inputs to the SDF-first execution fabric. Status: remaining lifecycle/finalization, deterministic registration/plan reporting, and diagnostics work now feeds product jobs, query snapshots, and publication barriers (source: `workspace/sdf-first-execution-roadmap.md` and `net/ecs-runtime-prioritized-roadmap.md`).
- [ ] Execute render immediate remaining phases only through product-selection and derived GPU-residency contracts. Status: backend-neutral `RenderProductSelection` producers and logical derived GPU residency are in place; `R4` binding model expansion, `R6` boids proof, and `R7` SDF renderer rebuild must consume those contracts rather than bypassing them (source: `workspace/sdf-first-execution-roadmap.md` and `engine/plugins/render/docs/roadmap.md`).
- [ ] Preserve and extend UI/editor guard coverage for structural routing, capability gating, and seam ownership. Status: active; current guard suites pass (source: `domain/ui/roadmap.md` and `workspace/roadmap-index.md`).
- [ ] Keep editor/UI cross-doc sequencing aligned with shipped behavior. Status: active; docs validation currently passes (source: `domain/ui/roadmap.md` and `workspace/roadmap-index.md`).
- [ ] Keep architecture guards and docs synchronized while these tracks land (source: `workspace/roadmap-index.md`).

## Other Tracked And Gated Work

- [ ] Sequence procedural authoring domains for material/texturing, procgen, particles, physics, animation, and simulation. Status: gated by the SDF-first open-world substrate roadmap; M6.2 procgen code waits on the Phase 5 procgen readiness gate and accepted procgen domain doc now that Phases 1 through 4 have closed publication barriers, query snapshots/strict consumer policy, render product selection producers, and derived GPU residency (source: `workspace/sdf-first-execution-roadmap.md` and `design/active/editor-procedural-content-and-simulation-workflow-plan.md`).
- [ ] Keep scripting boundary language-neutral with Rhai as first adapter candidate when scripting implementation starts. Status: promoted for editor/runtime tracking, but not implementation-started (source: `design/active/engine-game-runtime-editor-ecs-scripting-hot-reload-design.md`).
- [ ] Add world-space/screen-projected UI attachment binding through explicit authored binding contracts and runtime formation seams. Status: tracking only; still boundary-gated by the owning design doc before implementation (source: `design/active/engine-game-runtime-editor-ecs-scripting-hot-reload-design.md`).
- [ ] Keep constrained in-game editors capability-gated with explicit command/ratification boundaries. Status: tracking only; still boundary-gated by the owning design doc before implementation (source: `design/active/editor-workspace-document-mode-panel-architecture.md`).

## Next (After Now Is Stable)

- [ ] Execute render usability/data-maturity phases `R5`, `R8`, `R9`, `R10` (source: `engine/plugins/render/docs/roadmap.md`).
- [ ] Resume M6.2 procgen as the first product-domain implementation only after the SDF-first open-world substrate phases and procgen readiness gate are satisfied (source: `workspace/sdf-first-execution-roadmap.md` and `apps/runenwerk-editor/roadmap.md`).
- [ ] Drive multiplayer replication through Milestone A (authoritative replication core) with the existing phase plan (source: `net/multiplayer-replication-implementation-roadmap.md`).
- [ ] Continue ECS runtime multiplayer-enabling checklist items (`M1`-`M5`) in Priority 2 (source: `net/ecs-runtime-prioritized-roadmap.md`).

## Later (Explicitly Deferred or Long-Horizon)

- [ ] Advance long-horizon geometry roadmap milestones when current higher-priority active tracks are stable (source: `domain/geometry/implementation-roadmap.md`).

## Completed Baselines (Do Not Reopen Without Reason)

- [x] Complete the Runenwerk editor MVP critical path and acceptance gate. Status: automated and manual/UI verified; the source checklist is complete (source: `apps/runenwerk-editor/execution-priority-checklist.md`).
- [x] Complete SDF-first execution Phase 4: derived GPU residency. Status: implemented and validated as of 2026-05-13; renderer-owned logical cache handles, deterministic residency allocation/preservation/invalidation/eviction/rejection, read-only residency inspection, editor viewport residency summaries, and typed world render-cache invalidation are in place while real GPU uploads, SDF renderer rebuilds, material/texture upload, and procgen remain deferred (source: `workspace/sdf-first-execution-roadmap.md` and `reports/closeouts/sdf-first-execution-phase-4/closeout.md`).
- [x] Complete SDF-first execution Phase 3: render product selection producers. Status: implemented and validated as of 2026-05-13; typed product-domain render selection state, stricter selection ratification, producer-scoped render selection contributions, prepared-frame inspection DTOs, and app-owned editor viewport selection production from accepted query snapshots are in place while real GPU uploads and procgen remain deferred (source: `workspace/sdf-first-execution-roadmap.md` and `reports/closeouts/sdf-first-execution-phase-3/closeout.md`).
- [x] Complete SDF-first execution Phase 2: query snapshots and strict consumer policy. Status: implemented and validated as of 2026-05-13; product strict-consumption decisions, query snapshot publication reports, deterministic `QuerySnapshotPublication` barriers, ECS source generation helpers, engine snapshot staging/publication/invalidation, render inspection DTOs, and app-owned editor viewport observation publication are in place as the query baseline for later render selection and residency phases (source: `workspace/sdf-first-execution-roadmap.md` and `reports/closeouts/sdf-first-execution-phase-2/closeout.md`).
- [x] Complete SDF-first execution Phase 1: serial product jobs and publication barriers. Status: implemented and validated as of 2026-05-12; product publication contracts, deterministic scheduler barriers, ECS generic barrier hooks, engine publication runtime support, and app-owned editor field-product publication are in place while render selection producers, GPU residency, and procgen remain deferred (source: `workspace/sdf-first-execution-roadmap.md` and `reports/closeouts/sdf-first-execution-phase-1/closeout.md`).
- [x] Editor Phase A: introduce workspace profile abstraction without breaking MVP. Status: implemented and test-covered (source: `apps/runenwerk-editor/execution-priority-checklist.md` and `design/active/editor-workspace-document-mode-panel-architecture.md`).
- [x] Editor Phase B: decouple workspace layout persistence from scene path coupling. Status: implemented and test-covered (source: `apps/runenwerk-editor/execution-priority-checklist.md` and `design/active/editor-workspace-document-mode-panel-architecture.md`).
- [x] Editor Phase C: formalize document tabs and active document switching. Status: implemented and test-covered; document taxonomy, ordered tabs, active switching, dirty/save/close transitions, compatibility validation, and generic app-local document-tab runtime state are in place (source: `apps/runenwerk-editor/execution-priority-checklist.md` and `design/active/editor-workspace-document-mode-panel-architecture.md`).
- [x] Editor Phase D: replace adapter-only panel wiring with provider registry routing. Status: implemented and test-covered; provider DTOs carry workspace profile, document context, surface definition, capabilities, and provider-local routes, and concrete providers are split into subdomain modules (source: `apps/runenwerk-editor/execution-priority-checklist.md` and `design/active/editor-workspace-document-mode-panel-architecture.md`).
- [x] Close editor app-domain operation migration seams for ratification, scene command execution, transaction orchestration, history, scene, and selection ownership. Status: implemented for the M1 seam; domain-owned scene operation functions handle command/transaction execution plus history insertion, while app-owned ECS/reflection, snapshots, retention, projection parity, selection sync, and recording remain app-local (source: `apps/runenwerk-editor/roadmap.md` and `reports/audits/editor-ui-priority-code-audit-2026-05-05.md`).
- [x] Editor Phase E: expand global mode enum into scoped workspace/document mode sets. Status: implemented and test-covered through mode ids, descriptors, a registry, and workspace/document compatibility validation (source: `apps/runenwerk-editor/execution-priority-checklist.md` and `design/active/editor-workspace-document-mode-panel-architecture.md`).
- [x] Complete UI docking/tab behavior on top of existing structural identity and binding contracts. Status: implemented and test-covered; tab reorder, rehome, floating host creation, split resizing, area split/duplicate/reset/close, dynamic split-area composition, and structural identity preservation have automated coverage (source: `apps/runenwerk-editor/roadmap.md` and `apps/runenwerk-editor/execution-priority-checklist.md`).
- [x] Expose editor-area/type switching as a reachable shell UI control. Status: implemented and test-covered; tab chrome renders an editor type selector and routes `SelectChanged` through `SwitchPanelToolSurfaceKind` (source: `apps/runenwerk-editor/execution-priority-checklist.md`).
- [x] Add plus/new-tab affordance for creating a new tab in a tab stack. Status: implemented and test-covered; tab chrome exposes a plus/new-tab control that allocates panel and tool-surface identities through `WorkspaceIdentityAllocator` after structural ratification (source: `apps/runenwerk-editor/execution-priority-checklist.md`).
- [x] Expand non-viewport surface maturity using existing surface contracts. Status: implemented as of 2026-05-08; typed surface action/session/domain wrappers are in place, entity-table query workflows cover search/selected-only/hierarchy/component filters plus sorting, and richer inspector controls render through generic `ui_definition` availability without provider behavior leakage (source: `domain/ui/roadmap.md` and `reports/closeouts/surface-workflow-contract-redesign/closeout.md`).
- [x] Complete the M3.5 UI definition formation framework before M3.6 and M4. Status: implemented and validated; `domain/ui/ui_definition`, `domain/editor/editor_definition`, checked-in RON fixtures, retained formation, app fixture validation, toolbar route-slot/menu-item integration, normal shell chrome formation, and common provider surface fixture formation exist (source: `apps/runenwerk-editor/roadmap.md` and `domain/ui/roadmap.md`).
- [x] Implement the M3.6 UI self-authoring workspace and styling before M4. Status: complete as of 2026-05-06; durable schemas, Editor Design workspace/profile, self-authoring provider surfaces, fixture document loading, validation, retained previews, command diff summaries, retained authoring control routes, UI node/theme/workspace-layout draft edits, and apply/rollback shell commands are implemented (source: `apps/runenwerk-editor/roadmap.md` and `design/active/editor-self-authoring-and-final-ui-design.md`).
- [x] Add rotate and scale gizmos after translate workflow is stable. Status: implemented and test-covered for M3; translate, rotate, and scale tool activation, preview, snap-aware interaction, commit, undo/redo, and scene-authoring smoke coverage exist (source: `apps/runenwerk-editor/execution-priority-checklist.md` and `apps/runenwerk-editor/roadmap.md`).
- [x] Add create/delete/duplicate flows for common scene-authoring actions. Status: implemented and test-covered for M3; outliner/app scene commands cover create child, rename, reparent, duplicate subtree, delete single entity, batch delete, and SDF primitive creation through viewport SDF tool routing (source: `apps/runenwerk-editor/execution-priority-checklist.md` and `apps/runenwerk-editor/roadmap.md`).
- [x] Improve inspector/component authoring breadth for common 3D flows. Status: implemented and test-covered for M3; common reflected primitive fields are editable through typed inspector edit values, and component add/remove remains routed through scene command intents (source: `apps/runenwerk-editor/execution-priority-checklist.md` and `apps/runenwerk-editor/roadmap.md`).
- [x] Build the editor SDF/field-first asset pipeline foundation and import workflow. Status: M4A-M4I complete as of 2026-05-09; `domain/asset`, asset taxonomy, import plans, field-product formation contracts, generic product invalidation, app-owned catalog/import jobs, asset browser/import inspector/field-product viewer/SDF brush browser providers, and failed-artifact preservation exist. Mesh/GLB import remains a foreign-reference path, not the primary world substrate (source: `apps/runenwerk-editor/roadmap.md` and `design/active/editor-asset-pipeline-and-content-workflow-design.md`).
- [x] Continue Editor Design/self-authoring polish for UI/editor layout design, creation, and management. Status: 2026-05-08 maturity pass complete; reusable field/control polish, no-payload ECS enum inspector mutation, versioned export packaging, non-theme live activation, active UI/editor catalogs, and panel/tool-surface replacement guards are implemented before M4 starts (source: `design/active/editor-self-authoring-and-final-ui-design.md`, `apps/runenwerk-editor/roadmap.md`, and `domain/ui/roadmap.md`).
- [x] Foundation SDF roadmap baseline is implemented (source: `domain/sdf/implementation-roadmap.md`).
- [x] Domain ECS Phase 6 closeout roadmap package is complete (source: `domain/ecs/roadmaps/phase6-closeout-roadmap.md`).
