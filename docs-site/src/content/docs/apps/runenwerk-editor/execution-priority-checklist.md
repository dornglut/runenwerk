---
title: Runenwerk Editor Execution Priority Checklist
description: Practical Now/Next/Later execution checklist aligned to MVP acceptance and post-MVP architecture boundaries.
status: active
owner: editor
layer: app
canonical: true
last_reviewed: 2026-05-05
related:
  - ./mvp/implementation-sequence.md
  - ./mvp/acceptance-criteria.md
  - ./mvp/first-3d-editor-mvp.md
  - ./roadmap.md
  - ../../reports/audits/editor-ui-priority-code-audit-2026-05-05.md
  - ../../domain/ui/roadmap.md
  - ../../design/active/editor-self-authoring-and-final-ui-design.md
  - ../../design/active/editor-workspace-document-mode-panel-architecture.md
  - ../../design/active/editor-asset-pipeline-and-content-workflow-design.md
  - ../../design/active/editor-procedural-content-and-simulation-workflow-plan.md
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
- Boundary-gated items may be tracked in `Now`, but implementation still follows the owning design/roadmap contract.

## Reconciliation Snapshot

Last reconciled: 2026-05-05.

Manual confirmation: 2026-05-04. User confirmed the remaining visual/windowed acceptance items.

Evidence command:

```text
cargo test -p runenwerk_editor -p editor_shell -p editor_viewport -p ui_runtime
```

Result at reconciliation: passed on 2026-05-05. The windowed GPU truth smoke remains intentionally ignored unless run manually with:

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

## Completed MVP Baseline

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

## Now (Editor/UI Active Work)

- [x] Phase A: Introduce workspace profile abstraction without breaking MVP. Status: Automated verified by shell state and workspace profile tests.
- [x] Phase B: Decouple workspace layout persistence from scene path coupling. Status: Automated verified by workspace layout persistence tests.
- [ ] Phase C: Formalize document tabs and active document switching. Status: Open; scene load/reset keeps an active scene document, but generic document-tab management is not complete.
- [ ] Phase D: Replace adapter-only panel wiring with provider registry routing. Status: Active implementation; provider DTOs, concrete providers, deterministic registry composition, controller frame build path, and fail-closed route guards exist, but legacy presenter/adapter migration remains open.
- [ ] Close editor app-domain operation migration seams for ratification, scene command execution, transaction orchestration, history, scene, and selection ownership. Status: Open/active architectural debt; keep paired with provider migration closeout.
- [ ] Phase E: Expand global mode enum into scoped workspace/document mode sets. Status: Open.
- [ ] Complete UI docking/tab behavior on top of existing structural identity and binding contracts. Status: Active/partially implemented; tab reorder, rehome, floating host creation, split resizing, and structural identity preservation have automated coverage, but fixed-layout productization remains open.
- [ ] Expose editor-area/type switching as a reachable shell UI control. Status: Partially implemented internally; `SwitchPanelToolSurfaceKind` and state mutation paths exist and are tested, but no select/dropdown UI route exposes them to users.
- [ ] Add plus/new-tab affordance for creating a new tab in a tab stack. Status: Open; current docking work moves existing panels/tabs but does not expose a control/command that allocates a new panel and tool surface.
- [ ] Expand non-viewport surface maturity using existing surface contracts. Status: Active/partially implemented; entity-table, console, inspector, outliner, provider routing, and independent surface-session coverage exist, but richer common workflows remain open.
- [ ] Preserve and extend UI/editor guard coverage for structural routing, capability gating, and seam ownership. Status: Active.
- [ ] Keep editor/UI cross-doc sequencing aligned with shipped behavior. Status: Active.
- [ ] Add rotate and scale gizmos after translate workflow is stable. Status: Open; translate workflow is automated verified, but rotate/scale tool commands are not implemented.
- [ ] Add create/delete/duplicate flows for common scene-authoring actions. Status: Partially complete; delete exists through outliner flows, create exists in lower-level command/test paths, and duplicate remains open as a common action.
- [ ] Improve inspector/component authoring breadth for common 3D flows. Status: Open; current transform/property editing is automated verified, but reusable control adoption and component breadth remain uneven.
- [ ] Build the SDF/field-first asset pipeline foundation and import workflow. Status: Open; current repo has scene/project RON persistence, `domain/sdf`, `domain/world_ops`, `domain/world_sdf`, loose scene manifest discovery, shader reload helpers, render resource import descriptors, Blender config, and model files, but no asset catalog, asset ids, SDF/field asset taxonomy, import plan, field-product formation plan, artifact cache, asset browser, field-product viewer, import diagnostics surface, or project-owned reload stream. Mesh/GLB import is a foreign-reference path, not the primary world substrate.
- [ ] Plan and sequence procedural authoring features through explicit domains. Status: Open; target design exists in `design/active/editor-procedural-content-and-simulation-workflow-plan.md`, but no material graph, texture, procgen, particles, physics, or animation domain crates/providers exist yet.
- [ ] Add language-neutral scripting boundary with Rhai as first adapter candidate. Status: Boundary-gated/open; no implementation started in the editor checklist.
- [ ] Add world-space/screen-projected UI attachment binding only via explicit authored binding contracts and runtime formation seams. Status: Boundary-gated/open.
- [ ] Add constrained in-game editors only through capability-gated surfaces and explicit command/ratification boundaries. Status: Boundary-gated/open.
- [ ] Continue Editor Design/self-authoring track for UI/editor layout design, creation, and management. Status: Open/boundary-gated; target design now exists in `design/active/editor-self-authoring-and-final-ui-design.md`, but implementation remains gated behind document/provider/mode/docking closure.
