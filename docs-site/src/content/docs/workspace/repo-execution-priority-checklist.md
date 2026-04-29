---
title: Repository Execution Priority Checklist
description: Cross-repo Now/Next/Later execution checklist that links to authoritative domain/app roadmaps.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-29
related:
  - ./roadmap-index.md
  - ../apps/runenwerk-editor/execution-priority-checklist.md
  - ../domain/ui/roadmap.md
  - ../engine/plugins/render/docs/roadmap.md
  - ../net/ecs-runtime-prioritized-roadmap.md
  - ../net/multiplayer-replication-implementation-roadmap.md
  - ../design/active/editor-workspace-document-mode-panel-architecture.md
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

- [ ] Complete the Runenwerk editor MVP critical path and acceptance gate (source: `apps/runenwerk-editor/execution-priority-checklist.md`).
- [ ] Complete UI roadmap current next steps for docking/tab behavior, non-viewport surface maturity, guard coverage, and cross-doc sync (source: `domain/ui/roadmap.md`).
- [ ] Execute render immediate remaining phases: `R4` binding model expansion, `R6` boids feature proof, `R7` SDF renderer rebuild on new path (source: `engine/plugins/render/docs/roadmap.md`).
- [ ] Continue ECS runtime convergence open foundation checklist items (`F1`-`F4`) in Priority 1 (source: `net/ecs-runtime-prioritized-roadmap.md`).
- [ ] Keep architecture guards and docs synchronized while these tracks land (source: `workspace/roadmap-index.md`).

## Next (After Now Is Stable)

- [ ] Execute editor workspace/document/mode/panel/provider rollout phases `A`-`E` (source: `design/active/editor-workspace-document-mode-panel-architecture.md`).
- [ ] Execute render usability/data-maturity phases `R5`, `R8`, `R9`, `R10` (source: `engine/plugins/render/docs/roadmap.md`).
- [ ] Drive multiplayer replication through Milestone A (authoritative replication core) with the existing phase plan (source: `net/multiplayer-replication-implementation-roadmap.md`).
- [ ] Continue ECS runtime multiplayer-enabling checklist items (`M1`-`M5`) in Priority 2 (source: `net/ecs-runtime-prioritized-roadmap.md`).
- [ ] Execute editor app post-MVP expansion areas only after MVP acceptance remains stable (source: `apps/runenwerk-editor/roadmap.md`).

## Later (Explicitly Deferred or Long-Horizon)

- [ ] Keep scripting boundary language-neutral with Rhai as first adapter candidate when scripting implementation starts (source: `design/active/engine-game-runtime-editor-ecs-scripting-hot-reload-design.md`).
- [ ] Add world-space/screen-projected UI attachment binding only through explicit authored binding contracts and runtime formation seams (source: `design/active/engine-game-runtime-editor-ecs-scripting-hot-reload-design.md`).
- [ ] Keep constrained in-game editors capability-gated with explicit command/ratification boundaries (source: `design/active/editor-workspace-document-mode-panel-architecture.md`).
- [ ] Continue Editor Design/self-authoring track as later-phase work (source: `design/active/editor-workspace-document-mode-panel-architecture.md` and `design/active/editor-ui-workspace-tool-surface-architecture.md`).
- [ ] Advance long-horizon geometry roadmap milestones when current higher-priority active tracks are stable (source: `domain/geometry/implementation-roadmap.md`).

## Completed Baselines (Do Not Reopen Without Reason)

- [x] Foundation SDF roadmap baseline is implemented (source: `domain/sdf/implementation-roadmap.md`).
- [x] Domain ECS Phase 6 closeout roadmap package is complete (source: `domain/ecs/roadmaps/phase6-closeout-roadmap.md`).
