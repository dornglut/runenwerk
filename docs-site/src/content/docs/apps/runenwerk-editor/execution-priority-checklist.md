---
title: Runenwerk Editor Execution Priority Checklist
description: Practical Now/Next/Later execution checklist aligned to MVP acceptance and post-MVP architecture boundaries.
status: active
owner: editor
layer: app
canonical: true
last_reviewed: 2026-05-04
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
- Use `Automated verified` only when current tests exercise the behavior.
- Use `Manual/UI verification remaining` when the code path exists but visual, windowed, or GPU-facing acceptance still needs a manual pass.
- Keep deferred boundary items deferred unless an owning design/roadmap is updated first.

## Reconciliation Snapshot

Last reconciled: 2026-05-04.

Manual confirmation: 2026-05-04. User confirmed the remaining visual/windowed acceptance items.

Evidence command:

```text
cargo test -p runenwerk_editor -p editor_shell -p editor_viewport -p ui_runtime
```

Result at reconciliation: passed. The windowed GPU truth smoke remains intentionally ignored unless run manually with:

```text
RUNENWERK_ENABLE_GPU_SMOKE=1 RUNENWERK_ENABLE_MACOS_MAIN_THREAD_GPU_SMOKE=1 cargo test -p runenwerk_editor --test viewport_gpu_truth_smoke -- --ignored
```

Representative evidence:

- `apps/runenwerk_editor/tests/scene_authoring_workflow_smoke.rs::scene_authoring_workflow_smoke_select_edit_translate_undo_redo`
- `apps/runenwerk_editor/tests/startup_render_smoke.rs::startup_render_smoke_publishes_editor_shell_submission`
- `apps/runenwerk_editor/tests/viewport_branch_truth_smoke.rs::viewport_branch_truth_smoke`
- `apps/runenwerk_editor/tests/viewport_architecture_guards.rs`
- `domain/editor/editor_shell/src/tests.rs`
- `domain/ui/ui_runtime/src/runtime/ui_runtime.rs`

## Now (MVP Critical Path)

- [x] Readable editor shell and panel labels are usable. Status: Manual/UI verified on 2026-05-04. Automated shell frame and panel composition tests exist.
- [x] Engine-owned editor window/runtime integration is stable. Status: Manual/UI verified on 2026-05-04. `startup_render_smoke_publishes_editor_shell_submission` covers editor shell submission.
- [x] Document-driven scene state is active. Status: Automated verified by scene authoring and persistence tests.
- [x] Projection into runtime/world state is active. Status: Automated verified by scene workflow, viewport branch truth, and projection/parity tests.
- [x] Viewport renders at least one real scene entity. Status: Manual/UI verified on 2026-05-04. Branch/startup smoke coverage exists.
- [x] Viewport picking and hit detection are working. Status: Automated verified by viewport interaction and picking tests.
- [x] Outliner, inspector, and viewport selection are synchronized. Status: Automated verified by scene authoring, shell, outliner, inspector, and viewport tests.
- [x] Inspector transform editing works end-to-end. Status: Automated verified by inspector edit and scene authoring workflow tests.
- [x] Translate gizmo interaction works end-to-end. Status: Automated verified by transform tool and viewport interaction tests.
- [x] Undo/redo works for core edit flows. Status: Automated verified by scene authoring and retained transaction replay tests.
- [x] Near-immediate scene persistence follow-up is complete. Status: Automated verified by scene file and retained change-log roundtrip tests.
- [x] MVP acceptance criteria pass as written in `mvp/acceptance-criteria.md`. Status: Automated and manual/UI verified on 2026-05-04.

## Next (Post-MVP, High Priority)

- [x] Phase A: Introduce workspace profile abstraction without breaking MVP.
- [x] Phase B: Decouple workspace layout persistence from scene path coupling.
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
