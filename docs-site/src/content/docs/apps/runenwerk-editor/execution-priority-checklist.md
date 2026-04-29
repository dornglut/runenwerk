---
title: Runenwerk Editor Execution Priority Checklist
description: Practical Now/Next/Later execution checklist aligned to MVP acceptance and post-MVP architecture boundaries.
status: active
owner: editor
layer: app
canonical: true
last_reviewed: 2026-04-29
related:
  - ./mvp/implementation-sequence.md
  - ./mvp/acceptance-criteria.md
  - ./mvp/first-3d-editor-mvp.md
  - ./roadmap.md
  - ../../design/active/editor-workspace-document-mode-panel-architecture.md
  - ../../design/active/engine-game-runtime-editor-ecs-scripting-hot-reload-design.md
---

# Runenwerk Editor Execution Priority Checklist

## Purpose

Provide one practical execution checklist without changing MVP scope.

Use this page as the short operational list.
Use the linked MVP and design docs as authoritative details.

## Rules

- Do not expand into post-MVP features until all `Now` items are complete.
- Mark a checkbox complete only when code and checks demonstrate it.
- Keep deferred boundary items deferred unless an owning design/roadmap is updated first.

## Now (MVP Critical Path)

- [ ] Readable editor shell and panel labels are usable.
- [ ] Engine-owned editor window/runtime integration is stable.
- [ ] Document-driven scene state is active.
- [ ] Projection into runtime/world state is active.
- [ ] Viewport renders at least one real scene entity.
- [ ] Viewport picking and hit detection are working.
- [ ] Outliner, inspector, and viewport selection are synchronized.
- [ ] Inspector transform editing works end-to-end.
- [ ] Translate gizmo interaction works end-to-end.
- [ ] Undo/redo works for core edit flows.
- [ ] Near-immediate scene persistence follow-up is complete.
- [ ] MVP acceptance criteria pass as written in `mvp/acceptance-criteria.md`.

## Next (Post-MVP, High Priority)

- [ ] Phase A: Introduce workspace profile abstraction without breaking MVP.
- [ ] Phase B: Decouple workspace layout persistence from scene path coupling.
- [ ] Phase C: Formalize document tabs and active document switching.
- [ ] Phase D: Replace adapter-only panel wiring with provider registry routing.
- [ ] Phase E: Expand global mode enum into scoped workspace/document mode sets.
- [ ] Add rotate and scale gizmos after translate workflow is stable.
- [ ] Add create/delete/duplicate flows for common scene-authoring actions.
- [ ] Improve inspector/component authoring breadth for common 3D flows.

## Later (Explicitly Deferred/Boundaried)

- [ ] Add language-neutral scripting boundary with Rhai as first adapter candidate.
- [ ] Add world-space/screen-projected UI attachment binding only via explicit authored binding contracts and runtime formation seams.
- [ ] Add constrained in-game editors only through capability-gated surfaces and explicit command/ratification boundaries.
- [ ] Continue Editor Design/self-authoring track as later-phase work (workspace/menu/theme/shortcut documents).
