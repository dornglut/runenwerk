---
title: Editor UI Priority Code Audit 2026-05-05
description: Code-grounded audit of editor and UI priority order, implementation state, and open gaps.
status: active
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-05
related:
  - ../../workspace/repo-execution-priority-checklist.md
  - ../../workspace/roadmap-index.md
  - ../../apps/runenwerk-editor/execution-priority-checklist.md
  - ../../apps/runenwerk-editor/roadmap.md
  - ../../domain/ui/roadmap.md
  - ../../domain/ui/architecture.md
  - ../../design/implemented/editor-self-authoring-and-final-ui-design.md
  - ../../design/implemented/editor-workspace-document-mode-panel-architecture.md
  - ../../design/active/editor-ui-workspace-tool-surface-architecture.md
  - ../../design/implemented/workspace-identity-contract-and-migration-map.md
---

# Editor UI Priority Code Audit 2026-05-05

## Purpose

Record the current code-grounded status of editor and UI Now work, with the priority order that should drive follow-up implementation.

This report audits the plan documents against current repository code. It does not replace the canonical checklists.

## Scope

Plan sources audited:

- `docs-site/src/content/docs/workspace/repo-execution-priority-checklist.md`
- `docs-site/src/content/docs/apps/runenwerk-editor/execution-priority-checklist.md`
- `docs-site/src/content/docs/apps/runenwerk-editor/roadmap.md`
- `docs-site/src/content/docs/domain/ui/roadmap.md`
- `docs-site/src/content/docs/domain/ui/architecture.md`
- `docs-site/src/content/docs/design/implemented/editor-workspace-document-mode-panel-architecture.md`
- `docs-site/src/content/docs/design/active/editor-ui-workspace-tool-surface-architecture.md`
- `docs-site/src/content/docs/design/implemented/workspace-identity-contract-and-migration-map.md`
- `docs-site/src/content/docs/design/active/engine-game-runtime-editor-ecs-scripting-hot-reload-design.md`

Code areas audited:

- `domain/editor/editor_core/src/document.rs`
- `domain/editor/editor_core/src/session.rs`
- `domain/editor/editor_core/src/tool.rs`
- `domain/editor/editor_shell/src/workspace/profile.rs`
- `domain/editor/editor_shell/src/workspace/state.rs`
- `domain/editor/editor_shell/src/workspace/reducer.rs`
- `domain/editor/editor_shell/src/workspace/projection.rs`
- `domain/editor/editor_shell/src/workspace/persisted.rs`
- `domain/editor/editor_shell/src/workspace/surface_contract.rs`
- `domain/editor/editor_shell/src/surface_provider.rs`
- `domain/editor/editor_shell/src/composition/build_editor_shell.rs`
- `domain/editor/editor_shell/src/composition/build_outliner_panel.rs`
- `domain/editor/editor_shell/src/composition/build_entity_table_panel.rs`
- `domain/editor/editor_shell/src/composition/build_inspector_panel.rs`
- `domain/editor/editor_shell/src/composition/build_viewport_panel.rs`
- `domain/ui/ui_surface/src/ratification.rs`
- `domain/ui/ui_surface/src/validation/mount_ratification.rs`
- `domain/ui/ui_runtime/src/runtime/ui_runtime.rs`
- `domain/ui/ui_runtime/src/input/pointer.rs`
- `domain/ui/ui_runtime/src/input/hit_test.rs`
- `domain/ui/ui_runtime/src/input/interaction_result.rs`
- `domain/ui/ui_runtime/src/output/build_ui_frame.rs`
- `domain/ui/ui_tree/src/tree/node.rs`
- `domain/ui/ui_widgets/src/lib.rs`
- `apps/runenwerk_editor/src/shell/state.rs`
- `apps/runenwerk_editor/src/shell/controller.rs`
- `apps/runenwerk_editor/src/shell/providers/mod.rs`
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`
- `apps/runenwerk_editor/src/shell/surface_session.rs`
- `apps/runenwerk_editor/src/editor_runtime/document/mod.rs`
- `apps/runenwerk_editor/src/editor_runtime/runtime.rs`
- `apps/runenwerk_editor/src/editor_runtime/commands/ratification.rs`
- `apps/runenwerk_editor/src/editor_runtime/commands/scene_commands.rs`
- `apps/runenwerk_editor/src/editor_runtime/commands/transactions.rs`
- `apps/runenwerk_editor/src/editor_features/tools.rs`
- `apps/runenwerk_editor/src/editor_features/viewport/tools.rs`
- `apps/runenwerk_editor/src/editor_features/viewport/interaction.rs`

## Priority Verdict

The current Now list is directionally right, but the implementation order should be dependency-driven:

1. Keep MVP, workspace profiles, and profile-addressed layout persistence closed.
2. Close document tabs and active-document switching before broader workspace/document mode work.
3. Close provider-registry migration and the editor app-domain operation migration together.
4. Replace global coarse editor modes with scoped workspace/document modes.
5. Productize docking/tab behavior on the existing structural identity contracts.
6. Expand non-viewport surface maturity and reusable control adoption.
7. Add scene-authoring breadth: rotate/scale, create/delete/duplicate, and richer inspector/component authoring.
8. Keep scripting, world-space UI attachment, constrained in-game editors, and self-authoring in Now for tracking only until their boundary contracts are made implementation-ready.

## Current Code State

### Completed Baselines

- Editor MVP acceptance is complete and represented as a completed baseline in `docs-site/src/content/docs/apps/runenwerk-editor/execution-priority-checklist.md`.
- Workspace profile abstraction is implemented in `domain/editor/editor_shell/src/workspace/profile.rs` and used by `apps/runenwerk_editor/src/shell/state.rs`.
- Profile-addressed layout persistence is implemented through `domain/editor/editor_shell/src/workspace/persisted.rs` and shell persistence dispatch in `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`.
- Workspace structural identity exists in `domain/editor/editor_shell/src/workspace/state.rs`, including `WorkspaceId`, `PanelHostId`, `TabStackId`, `PanelInstanceId`, and `ToolSurfaceInstanceId`.
- Per-tool-surface session state exists in `apps/runenwerk_editor/src/shell/surface_session.rs`.

### Document Model

- `domain/editor/editor_core/src/document.rs` defines `DocumentKind`, `DocumentId`, and `DocumentDescriptor`.
- `domain/editor/editor_core/src/session.rs` stores documents and an active document in `EditorSession`.
- `apps/runenwerk_editor/src/editor_runtime/document/mod.rs` currently contains `SceneDocumentState`, not a generic document-tab runtime.
- `apps/runenwerk_editor/src/editor_runtime/runtime.rs` seeds and maintains a default scene document.

Gap: the target document taxonomy and tab workflow are not implemented. The code does not yet expose generic document tab ordering, active document switch commands, workspace/document compatibility checks, or non-scene document state.

### Provider Routing

- `domain/editor/editor_shell/src/surface_provider.rs` contains app-neutral provider request/result/route DTOs.
- `apps/runenwerk_editor/src/shell/providers/mod.rs` contains concrete outliner, entity table, viewport, inspector, and console providers.
- `apps/runenwerk_editor/src/shell/controller.rs` builds `EditorShellFrameModel` and dispatches routed surface interactions through the retained UI runtime.
- Route guards and provider mismatch handling are present in shell tests and dispatch paths.

Gap: provider migration is active but not closed. Surface-local actions remain current-surface-specific, the app registry is concrete rather than an extensible editor registry, and legacy app presenters/adapters still own meaningful behavior.

### Editor Modes and Tools

- `domain/editor/editor_core/src/session.rs` still uses a global `EditorMode` enum with `Edit`, `Play`, and `Simulate`.
- `domain/editor/editor_shell/src/workspace/profile.rs` exposes default mode filters but only in the current coarse form.
- `apps/runenwerk_editor/src/editor_features/tools.rs`, `apps/runenwerk_editor/src/editor_features/viewport/tools.rs`, and `apps/runenwerk_editor/src/editor_features/viewport/interaction.rs` implement select/translate workflows.

Gap: scoped workspace/document mode sets are not implemented. Tool availability is not mode-registry-driven, and command validation does not yet use workspace/document-specific mode contracts.

### Docking and Tabs

- `domain/editor/editor_shell/src/workspace/reducer.rs` supports tab activation, move/reorder, rehome between stacks, floating host creation, and split fraction changes.
- `domain/editor/editor_shell/src/workspace/projection.rs` produces fixed-layout tab stack projections and routing maps.
- `domain/editor/editor_shell/src/composition/build_editor_shell.rs` renders tab strips, drop zones, explicit shelf columns, and split handles.
- `apps/runenwerk_editor/src/shell/controller.rs` and `apps/runenwerk_editor/src/shell/state.rs` track tab drag and split resize interactions.

Gap: docking/tab behavior is real but still fixed-editor-shell oriented. The projection keeps named slots for outliner, viewport, inspector, and console, and floating behavior is represented as a column rather than a full editor windowing/docking product.

### Editor Area Type and New Tab Affordances

- `domain/editor/editor_shell/src/commands/shell_command.rs` defines `ShellCommand::SwitchPanelToolSurfaceKind`.
- `apps/runenwerk_editor/src/shell/state.rs` implements `RunenwerkEditorShellState::switch_panel_tool_surface_kind`.
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs` dispatches the switch command.
- `apps/runenwerk_editor/src/shell/tests.rs::editor_type_switch_replaces_mounted_surface_without_changing_panel_identity` verifies the internal switch behavior.
- `domain/ui/ui_widgets/src/select.rs` exposes a retained select/dropdown constructor, and `domain/ui/ui_runtime/src/input/pointer.rs` emits `UiInteraction::SelectChanged`.

Gap: editor type switching is not exposed as a shell UI affordance. `domain/editor/editor_shell/src/commands/map_interactions.rs` currently ignores `UiInteraction::SelectChanged`, and `domain/editor/editor_shell/src/composition/build_editor_shell.rs` does not render a select/dropdown for switching the active panel's editor area type.

Gap: there is no plus/new-tab affordance. The workspace reducer can move existing panels between tab stacks and into floating hosts, but there is no user-facing command/control that allocates a new `PanelInstanceId` plus `ToolSurfaceInstanceId` and inserts it into a tab stack from a plus button.

### UI Substrate

- `domain/ui/ui_tree/src/tree/node.rs` defines retained node kinds for panel, label, button, text input, toggle, numeric input, tabs, select, table, tree, image, viewport embed, scroll, stack, and split.
- `domain/ui/ui_runtime/src/input/pointer.rs` handles pointer hover, press, activation, scroll wheels, middle-drag panning, scrollbar dragging, table rows, tree rows, tabs, select, toggle, and numeric stepping.
- `domain/ui/ui_runtime/src/output/build_ui_frame.rs` emits draw primitives for current retained node kinds.
- `domain/ui/ui_surface/src/ratification.rs` and `domain/ui/ui_surface/src/validation/mount_ratification.rs` enforce capability and mount guards.
- `domain/ui/ui_widgets/src/lib.rs` exposes reusable widget constructors.

Gap: low-level UI primitives are not the main blocker. The open work is adoption and product surface maturity: shell surfaces still use panel-specific composition, outliner rows are still button rows instead of the retained tree control, and inspector/component authoring breadth is limited.

### Scene Authoring Breadth

- Translate is implemented and verified through viewport/tool paths.
- Delete exists through outliner command flows.
- Create entity support exists in domain/runtime command layers and tests, but not as a common shell/editor action set.
- Duplicate was not found as a common editor action.
- Rotate/scale appear in transform preview/domain data, but not as implemented viewport tool commands.

Gap: rotate/scale, create/delete/duplicate as a consistent action family, and richer component authoring remain open after the structural editor/UI work.

### App-Domain Migration

- `apps/runenwerk_editor/src/editor_runtime/commands/ratification.rs` owns `ratify_scene_change`.
- `apps/runenwerk_editor/src/editor_runtime/commands/scene_commands.rs` owns scene command execution plus history/ratification orchestration.
- `apps/runenwerk_editor/src/editor_runtime/commands/transactions.rs` owns transaction execution plus history/ratification orchestration.
- `apps/runenwerk_editor/src/editor_runtime/history/*`, `apps/runenwerk_editor/src/editor_runtime/scene.rs`, and `apps/runenwerk_editor/src/editor_runtime/selection.rs` still contain domain-like editor behavior.

Gap: this migration should be tracked as a Now dependency with provider migration, because later document/mode/surface work will otherwise keep depending on app-owned orchestration seams.

### Boundary-Gated Tracks

- Scripting boundary work remains design-level. No language-neutral script runtime crate or adapter implementation was found.
- Runtime UI is still overlay/template-driven; world-space and screen-projected attachment bindings remain design-level.
- Constrained in-game editors and Editor Design/self-authoring are tracked but not implementation-started.
- UI/editor self-authoring is architecturally planned in `docs-site/src/content/docs/design/active/editor-ui-workspace-tool-surface-architecture.md` as authored editor-definition groundwork and in `docs-site/src/content/docs/design/implemented/editor-workspace-document-mode-panel-architecture.md` as the `Editor Design` workspace. The concrete target design now lives in `docs-site/src/content/docs/design/implemented/editor-self-authoring-and-final-ui-design.md`.

Gap: these should remain visible in Now for planning, but they should not be ordered ahead of document/provider/mode/docking closure. UI/editor self-authoring now has a target design, but it remains implementation-gated until the prerequisite editor/UI foundations are closed.

## Documented Gaps

- Document tabs are the first structural blocker: generic document identity exists, but document tab UX/runtime semantics do not.
- Provider migration has a working core but needs closeout: current app providers and surface-local actions are not yet the final registry-oriented model.
- App-domain migration is missing from the top-level Now order even though the app roadmap identifies it as active architectural debt.
- Scoped modes remain open and should follow document/provider closure.
- Docking/tab work is partially implemented and test-covered, but still fixed-layout productization rather than full docking.
- Editor type switching is internally implemented and tested, but there is no reachable dropdown/select UI for users to switch a panel's editor area type.
- New tab creation is not implemented as a plus button or shell command; current docking work only moves existing panels/tabs.
- Reusable UI controls exist, but adoption is uneven across editor surfaces.
- Scene-authoring breadth remains feature work after the structural seams close.
- Boundary-gated future tracks are correctly promoted for tracking, but they are not implementation-ready.
- UI/editor self-authoring now has a dedicated target design, but implementation should wait for the prerequisite document/provider/mode/docking work.

## Recommended Now Order

1. Phase C: formalize document tabs and active document switching.
2. Phase D: finish provider-registry routing and close the app-domain operation migration.
3. Phase E: introduce scoped workspace/document modes and mode-aware command validation.
4. Docking/tab product completion on current structural identity contracts.
5. Add reachable editor-area/type selection UI and plus/new-tab creation affordances.
6. Non-viewport surface maturity and reusable control adoption.
7. Guard and docs drift control for all editor/UI seams.
8. Scene-authoring breadth: rotate/scale, create/delete/duplicate, and richer inspector/component authoring.
9. Boundary-gated tracking: scripting, world-space UI attachment, constrained in-game editors, and Editor Design/self-authoring.
10. Keep `design/implemented/editor-self-authoring-and-final-ui-design.md` aligned as the target design while implementation remains gated behind the prerequisite editor/UI foundations.

## Validation

This audit was prepared from code inspection and roadmap reconciliation on 2026-05-05.
Run documentation validation after any checklist edit:

```text
python3 tools/docs/validate_docs.py
```
