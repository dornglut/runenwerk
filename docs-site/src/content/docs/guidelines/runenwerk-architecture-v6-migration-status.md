---
title: Runenwerk Architecture v6 Migration Status
description: Executed status of the v6 architecture migration program.
---

# Runenwerk Architecture v6 Migration Status
_Last updated: 2026-04-17_

## Status

The v6 migration roadmap for the current editor/runtime architecture program is implemented.

## Executed Phases

### 1) Governing Ratification Spine

Implemented:
- `domain/editor/editor_core/src/ratification.rs`
- `domain/editor/editor_core/src/error.rs`
- `apps/runenwerk_editor/src/editor_runtime/commands/ratification.rs`
- `apps/runenwerk_editor/src/editor_runtime/history/ratified_change_log.rs`
- `apps/runenwerk_editor/src/editor_runtime/history/undo_redo.rs`
- `apps/runenwerk_editor/src/editor_runtime/commands/scene_commands.rs`
- `apps/runenwerk_editor/src/editor_runtime/commands/transactions.rs`

Outcome:
- Mutating scene commands and undo/redo now emit typed `RatifiedChange` artifacts with origin, causality, version, reconciliation policy, and propagation structure.

### 2) Runtime Reality Boundary Split

Implemented:
- `apps/runenwerk_editor/src/editor_runtime/runtime.rs` (`SceneRealityStore`)

Outcome:
- Authored, instantiated, and identity realities are no longer flattened as a single anonymous mutable interior.

### 3) Observation and Expression Contracts

Implemented:
- `domain/editor/editor_shell/src/observation/frame.rs`
- `domain/editor/editor_shell/src/observation/outliner.rs`
- `domain/editor/editor_shell/src/observation/inspector.rs`
- `domain/editor/editor_shell/src/expression/mod.rs`
- `apps/runenwerk_editor/src/shell/outliner_adapter.rs`
- `apps/runenwerk_editor/src/shell/inspector_adapter.rs`
- `apps/runenwerk_editor/src/shell/controller.rs`
- `apps/runenwerk_editor/src/runtime/systems/frame_submit.rs`

Outcome:
- Outliner/inspector consumption moved behind observation-frame metadata.
- Renderer-facing shell output moved behind `ShellExpressionFrame`.

### 4) Session and Workflow Reality Contracts

Implemented:
- `domain/editor/editor_core/src/session_change.rs`
- `domain/editor/editor_core/src/workflow.rs`
- `apps/runenwerk_editor/src/editor_runtime/selection.rs`
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`

Outcome:
- Session changes (tool/selection) and workflow events (dispatch/save/load/ratification/reconciliation) are first-class, typed logs.

### 5) Sharing and Reconciliation

Implemented:
- `domain/editor/editor_core/src/sharing.rs`
- `domain/editor/editor_core/src/reconciliation.rs`
- `apps/runenwerk_editor/src/editor_runtime/runtime.rs` (`propagate_shared_changes`, `reconcile_shared_change`)
- `apps/runenwerk_editor/src/editor_app/facade.rs` (app boundary for shared ingress/egress)
- `apps/runenwerk_editor/src/editor_runtime/tests/sharing.rs`

Outcome:
- Shared propagation is explicit via `SharedChangeOutbox` and sink contracts.
- Reconciliation policy is enforced through typed decisions and reject reasons.

### 6) Authored -> Normalized -> Formed -> Instantiated Path

Implemented:
- `domain/editor/editor_persistence/src/scene_migration.rs`
- `domain/editor/editor_persistence/src/scene_normalization.rs`
- `domain/editor/editor_persistence/src/scene_formation.rs`
- `apps/runenwerk_editor/src/persistence/files.rs`
- `apps/runenwerk_editor/src/persistence/runtime.rs`

Outcome:
- Scene loading now explicitly executes migration, normalization, formation, and instantiation stages.
- Invalid authored structures (duplicate IDs, missing/cyclic parents) are rejected before instantiation.

### 7) Retained Reality for Ratified Change History

Implemented:
- `apps/runenwerk_editor/src/persistence/retained_changes.rs`
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`

Outcome:
- Save flow now persists retained ratified-change logs beside scene files.
- Load flow reads retained logs for workflow visibility.

### 8) Boundary Hardening and Ingress Cuts

Implemented:
- `apps/runenwerk_editor/src/runtime/systems/input_bridge.rs` (undo/redo via shell command path)
- `apps/runenwerk_editor/src/editor_runtime/runtime.rs` (reduced broad mutable surfaces, tighter visibility)
- `apps/runenwerk_editor/src/editor_features/*` and `apps/runenwerk_editor/src/editor_panels/viewport_panel.rs` updated to use origin-aware boundary functions

Outcome:
- Input/runtime/editor/shell mutation ingress is cleaner and less ad hoc.

## Verification

Validated with:

```bash
cargo fmt --all
cargo test -p editor_core -p editor_persistence -p editor_shell -p runenwerk_editor
```

Result:
- Pass (all targeted tests green; GPU smoke remains intentionally ignored unless explicitly enabled).

