---
title: Runenwerk Editor Roadmap
description: Post-MVP expansion roadmap for the Runenwerk editor application.
status: active
owner: editor
layer: app
canonical: true
last_reviewed: 2026-05-04
related:
  - ./mvp/first-3d-editor-mvp.md
  - ./mvp/acceptance-criteria.md
  - ./mvp/implementation-sequence.md
  - ./execution-priority-checklist.md
  - ../../domain/ui/roadmap.md
  - ../../design/active/editor-ui-workspace-tool-surface-architecture.md
  - ../../design/active/editor-workspace-document-mode-panel-architecture.md
---

# Runenwerk Editor Roadmap

## Purpose

This document tracks editor expansion after the first 3D scene authoring MVP is stable.

This page tracks app-level feature scope. UI substrate/surface architecture sequencing is owned by domain and design roadmap documents.

## Post-MVP Expansion

Likely post-MVP areas:

- rotate and scale gizmos;
- more component and shape editing;
- better viewport overlays;
- docking and layout persistence;
- create, delete, and duplicate flows;
- UI authoring mode;
- 2D mode if needed;
- asset workflows;
- deeper inspector and component authoring.

## Current Post-MVP Status

Completed baseline infrastructure:

- workspace profile abstraction is implemented in `domain/editor/editor_shell/src/workspace/profile.rs`;
- `RunenwerkEditorShellState` tracks active workspace profile identity separately from `WorkspaceState`;
- workspace layout persistence is profile-addressed through `apps/runenwerk_editor/src/persistence/workspace_layout.rs::default_workspace_layout_path_for_profile`;
- scene-derived workspace layout paths are retained only as a legacy load fallback.

## Rule

Post-MVP expansion should not obscure the first MVP acceptance criteria.

The first milestone remains a 3D SDF scene authoring MVP with readable text, fixed workspace layout, synced selection, transform editing, translate gizmo interaction, undo/redo, and near-immediate persistence.

## App-Domain Migration Track

This track folds the current app-thinning audit into the roadmap. It is not a new MVP feature. It is architecture cleanup required to keep `apps/runenwerk_editor` as a composition root instead of a second editor domain.

### Goal

Move scene mutation orchestration, ratification construction, retained transaction policy, and history semantics toward domain-owned operation contracts. Keep app code responsible for runtime integration, engine systems, IO, viewport resources, and host-specific adapters.

### Current App-Owned Domain Behavior

The current app still owns domain-like behavior in these exact locations:

- `apps/runenwerk_editor/src/editor_runtime/commands/ratification.rs::ratify_scene_change`
  - builds `RatifiedChange` metadata, affected domains/scopes, version progression, retention hints, reconciliation policy, and propagation structure.
- `apps/runenwerk_editor/src/editor_runtime/commands/scene_commands.rs::execute_scene_command_and_push_history_with_origin`
  - executes scene commands, pushes history, syncs selection, asserts projection parity, ratifies the change, and records retained transactions.
- `apps/runenwerk_editor/src/editor_runtime/commands/transactions.rs::execute_scene_transaction_and_push_history_with_origin_and_causality`
  - executes multi-command transactions, pushes one history entry, ratifies the transaction, and records retained transactions.
- `apps/runenwerk_editor/src/editor_runtime/history/*`
  - owns retained transaction replay/storage policy that should remain aligned with editor-domain retention semantics.
- `apps/runenwerk_editor/src/editor_runtime/scene.rs`
  - should be audited for domain operation ownership versus app runtime state ownership.
- `apps/runenwerk_editor/src/editor_runtime/selection.rs`
  - should be audited for selection mutation rules that belong in editor-domain contracts.
- `apps/runenwerk_editor/src/editor_features/*`
  - should become thin action/adaptor routing where behavior is already owned by domain/editor crates.
- `apps/runenwerk_editor/src/editor_panels/*`
  - should remain concrete app UI composition only; inspector/outliner semantics should flow through domain observation/provider contracts.

### Target Domain Locations

- `domain/editor/editor_core/src/ratification.rs`
  - add domain-owned scene-change ratification construction or a focused submodule when the existing file grows.
- `domain/editor/editor_scene/src/operations/execute_scene_command.rs`
  - own scene command operation orchestration against a narrow domain-owned context trait.
- `domain/editor/editor_scene/src/operations/execute_scene_transaction.rs`
  - own multi-command scene transaction orchestration against the same narrow contract family.
- `domain/editor/editor_scene/src/operations/mod.rs`
  - expose legal scene operations without introducing a generic `services`, `handlers`, or `use_cases` layer.

### Required Dependency Inversion

Domain operations must not take `apps/runenwerk_editor::editor_runtime::RunenwerkEditorRuntime`.

Instead, introduce narrow domain-owned context traits for the exact responsibilities needed by each operation, such as:

- scene command execution context;
- session/history access;
- scene snapshot capture;
- retained transaction recording;
- reality version allocation;
- selection repair after scene changes;
- projection parity verification hook, if it remains part of the operation contract.

`apps/runenwerk_editor/src/editor_runtime/runtime.rs::RunenwerkEditorRuntime` should implement those traits in the app crate.

### Sequencing

1. Extract ratification construction inputs into a domain-owned parameter object in `domain/editor/editor_core/src/ratification.rs`.
2. Move single scene command orchestration to `domain/editor/editor_scene/src/operations/execute_scene_command.rs`.
3. Move multi-command scene transaction orchestration to `domain/editor/editor_scene/src/operations/execute_scene_transaction.rs`.
4. Keep app wrappers temporarily in `apps/runenwerk_editor/src/editor_runtime/commands/*` while call sites migrate.
5. Remove app wrappers once shell, tool actions, inspector, outliner, and viewport flows call the domain operations through app-implemented context traits.
6. Preserve current tests during migration, especially scene authoring smoke, retained transaction replay, inspector edit undo/redo, and architecture guard tests.

## Related Documents

- [`mvp/first-3d-editor-mvp.md`](./mvp/first-3d-editor-mvp.md)
- [`mvp/acceptance-criteria.md`](./mvp/acceptance-criteria.md)
- [`mvp/implementation-sequence.md`](./mvp/implementation-sequence.md)
- [`execution-priority-checklist.md`](./execution-priority-checklist.md)
- [`../../domain/ui/roadmap.md`](../../domain/ui/roadmap.md)
- [`../../design/active/editor-ui-workspace-tool-surface-architecture.md`](../../design/active/editor-ui-workspace-tool-surface-architecture.md)
- [`../../design/active/editor-workspace-document-mode-panel-architecture.md`](../../design/active/editor-workspace-document-mode-panel-architecture.md)
