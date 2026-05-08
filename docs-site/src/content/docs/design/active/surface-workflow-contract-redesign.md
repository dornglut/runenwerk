---
title: Surface Workflow Contract Redesign
description: Long-term provider-backed editor surface action, session mutation, and domain mutation contracts for outliner, entity table, inspector, viewport, and editor definition surfaces.
status: active
owner: editor
layer: domain
canonical: true
last_reviewed: 2026-05-08
related:
  - ../../domain/ui/roadmap.md
  - ../../apps/runenwerk-editor/roadmap.md
  - ../../reports/closeouts/surface-workflow-contract-redesign/closeout.md
  - ./ui-definition-formation-foundation-design.md
---

# Surface Workflow Contract Redesign

## Status

Implemented as the M3.7 follow-on surface workflow migration on 2026-05-08.

`domain/editor/editor_shell/src/surfaces/` owns the app-neutral contracts for active provider-backed surfaces. `ui_definition` remains behavior-free and only forms generic retained UI nodes from fixture data, slots, availability, collections, selections, routes, and embeds.

## Contract

Surface workflow is split into three explicit lanes:

- `SurfaceLocalAction` is the provider-entry action emitted by a formed UI route.
- `SurfaceSessionMutation` mutates per-tool-surface UI/session state.
- `EditorDomainMutation` requests domain/runtime state changes that must stay structurally targeted and provider validated.

The active surface wrappers are:

- `SurfaceLocalAction::{Outliner, EntityTable, Inspector, Viewport, EditorDefinition}`
- `SurfaceSessionMutation::{EntityTable, Inspector, Viewport}`
- `EditorDomainMutation::{Outliner, EntityTable, Viewport}`

`ShellCommand` no longer grows one variant per surface behavior. Surface proposals enter the shell through:

- `ApplySurfaceSessionMutation { target, projection_epoch, mutation }`
- `ApplyEditorDomainMutation { target, projection_epoch, mutation }`
- `DispatchSurfaceLocalAction { ... }` for provider-local action resolution

Normal shell commands such as workspace, toolbar, and editor-definition apply/rollback remain direct shell commands.

## App Dispatch

`apps/runenwerk_editor/src/shell/dispatch_shell_command.rs` is now the global facade. Surface-specific behavior lives in:

- `apps/runenwerk_editor/src/shell/dispatch/outliner.rs`
- `apps/runenwerk_editor/src/shell/dispatch/entity_table.rs`
- `apps/runenwerk_editor/src/shell/dispatch/inspector.rs`
- `apps/runenwerk_editor/src/shell/dispatch/viewport.rs`

This keeps provider behavior in `runenwerk_editor` and prevents `ui_definition` from learning editor semantics.

## Surface Workflows

Entity table sessions now store `EntityTableQuery`, including search text, selected-only filtering, hierarchy filtering, component filtering, and sort state. The entity-table fixture exposes search, clear, selected-only, roots-only, component filter, and sort controls while route handling stays provider-owned.

Inspector fields now carry `InspectorFieldControlKind`. Bool fields form toggles, integer/float fields form numeric inputs, text fields keep draft/commit/cancel text editing, enum fields form disabled selects, and read-only/group/unsupported fields form non-edit labels. `UiAvailability::Unavailable` omits inactive fixture alternatives during formation; `UiAvailability::Disabled` still forms a non-interactive node.

Viewport behavior is not redesigned again. Viewport product, camera, and debug/root background commands now use the same typed surface wrapper lane as other surfaces while preserving the existing viewport-scoped runtime binding validation.
