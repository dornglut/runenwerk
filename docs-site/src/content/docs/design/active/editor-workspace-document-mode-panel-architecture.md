---
title: Runenwerk Editor Workspace-Document-Mode-Panel Architecture
description: Repository-grounded architecture for task workspaces, document tabs, interaction modes, reusable panels, and context providers.
status: active
owner: editor
layer: domain
canonical: true
last_reviewed: 2026-05-04
related_designs:
  - ./editor-ui-workspace-tool-surface-architecture.md
  - ./workspace-identity-contract-and-migration-map.md
  - ./engine-game-runtime-editor-ecs-scripting-hot-reload-design.md
related:
  - ../../apps/runenwerk-editor/current-architecture.md
  - ../../apps/runenwerk-editor/roadmap.md
  - ../../apps/runenwerk-editor/mvp/first-3d-editor-mvp.md
---

# Runenwerk Editor Workspace-Document-Mode-Panel Architecture

## Status
Draft for implementation.

## Purpose

Define the concrete Runenwerk model for Blender-style task workspaces without collapsing document semantics into one global outliner model.

This design refines existing editor shell architecture into a strict separation:

```text
Workspace = task-focused layout preset
Document tab = specific authored or inspected target
Mode = interaction behavior inside current workspace/document context
Panel = reusable tool-surface view
Provider = context adapter that feeds panel observation + command routing
```

## Repository Grounding (Current State)

This design builds on contracts already implemented:

- Workspace structural identity and graph:
  `domain/editor/editor_shell/src/workspace/identity.rs`,
  `domain/editor/editor_shell/src/workspace/state.rs`,
  `domain/editor/editor_shell/src/workspace/reducer.rs`
- Workspace projection and routing:
  `domain/editor/editor_shell/src/workspace/projection.rs`,
  `domain/editor/editor_shell/src/workspace/projection_ratification.rs`
- Tool-surface capability and retention contracts:
  `domain/editor/editor_shell/src/workspace/surface_contract.rs`
- Editor session/document/tool baseline:
  `domain/editor/editor_core/src/session.rs`,
  `domain/editor/editor_core/src/document.rs`,
  `domain/editor/editor_core/src/tool.rs`
- Current app shell/runtime integration:
  `apps/runenwerk_editor/src/shell/controller.rs`,
  `apps/runenwerk_editor/src/shell/state.rs`,
  `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`

Current state and remaining gaps:

- Workspace profile contracts are now implemented in
  `domain/editor/editor_shell/src/workspace/profile.rs`.
- Workspace layout persistence is now profile-addressed via
  `apps/runenwerk_editor/src/persistence/workspace_layout.rs::default_workspace_layout_path_for_profile`.
- `EditorMode` is global and coarse (`Edit`, `Play`, `Simulate`) in
  `domain/editor/editor_core/src/session.rs`.
- Panel semantics are mostly hardcoded to fixed kinds in
  `domain/editor/editor_shell/src/workspace/state.rs` and
  `domain/editor/editor_shell/src/composition/build_editor_shell.rs`.
- Context-sensitive panel providers are implicit adapter functions in
  `apps/runenwerk_editor/src/shell/*_adapter.rs`, not a formal provider registry contract.

## Core Doctrine

### 1) Workspace owns tool layout, not document data

A workspace chooses panel arrangement, default tool surfaces, and default mode set.

It does not own scene/UI/graph/script document payloads.

### 2) Document tab owns edited target

The active document tab determines which provider family feeds hierarchy, inspector, graph/canvas, and actions.

### 3) Mode is scoped behavior

Mode is interaction policy. It is not panel identity and not document identity.

Modes can be constrained by workspace + document kind.

### 4) Panel is reusable view/tool surface

Panel instances remain reusable shells around tool surfaces.
Panel meaning comes from provider + active document context.

### 5) Provider is the semantic adapter

Providers bind document/runtime context into panel observation frames and command handling.
This keeps shell composition generic and keeps domain semantics out of layout code.

### 6) Blender-style editor type switching maps to tool-surface selection

Blender's editor type selector changes which editor is hosted in an area. In Runenwerk, that concept maps to selecting or swapping the hosted tool surface for a panel/tab host.

Use this mapping:

```text
Blender area                  -> Runenwerk panel host / tab stack / panel instance
Blender editor type selector  -> ToolSurfaceKind / SurfaceDefinitionId selection
Blender editor instance       -> ToolSurfaceInstanceId mounted into PanelInstanceId
```

This is not a document-kind switch and not a mode switch.

- `DocumentKind` identifies the authored or inspected target, such as scene, UI, graph, script, runtime-debug, or editor-design.
- mode identifies interaction behavior within the active workspace/document context.
- `ToolSurfaceKind` identifies the hosted surface type, such as outliner, viewport, inspector, console, entity table, graph editor, or timeline.

Near-term implementation should model an editor-type menu as a workspace command that changes a panel instance's active or mounted tool surface while preserving structural identity rules in:

- `domain/editor/editor_shell/src/workspace/state.rs::ToolSurfaceKind`
- `domain/editor/editor_shell/src/workspace/state.rs::ToolSurfaceState`
- `domain/editor/editor_shell/src/workspace/surface_contract.rs::editor_surface_definitions`
- `domain/editor/editor_shell/src/workspace/reducer.rs`

The provider registry then decides what the selected surface observes and which commands it can emit for the active `(workspace_profile, document_kind)` context.

## Architecture Contract

### Workspace Contract (editor_shell)

Keep `WorkspaceState` as canonical structural graph in
`domain/editor/editor_shell/src/workspace/state.rs`.

Add a workspace profile contract under
`domain/editor/editor_shell/src/workspace/` to represent task presets independent of currently open documents.

Target module:

- `domain/editor/editor_shell/src/workspace/profile.rs` (new module)

Target responsibilities:

- stable workspace preset identity and label;
- default root layout template (structural hosts/tab stacks/panels);
- default panel/tool surface set;
- default mode set for that workspace;
- optional default document-kind filters.

### Document Contract (editor_core)

Extend document model in:

- `domain/editor/editor_core/src/document.rs`
- `domain/editor/editor_core/src/session.rs`

Required evolution:

- keep `DocumentId`/`DocumentDescriptor`, but make document-kind taxonomy first-class for editor workflows (scene, ui, graph, script, runtime-debug, editor-design);
- session supports multiple open documents and active document switching without re-owning workspace graph state.

### Mode Contract (editor_core + app adapters)

Evolve mode ownership in:

- `domain/editor/editor_core/src/session.rs` (`EditorMode`)
- `apps/runenwerk_editor/src/editor_features/tools.rs`
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`

Required evolution:

- mode registry should support workspace-specific mode sets;
- mode activation should validate `(workspace, document_kind)` compatibility;
- mode behavior remains command-driven and ratifiable.

### Panel + Provider Contract (editor_shell + app)

Preserve panel identity and hosting in:

- `domain/editor/editor_shell/src/workspace/state.rs`
- `domain/editor/editor_shell/src/workspace/projection.rs`

Formalize providers in app/editor integration:

- `apps/runenwerk_editor/src/shell/` (new provider modules)
- `apps/runenwerk_editor/src/shell/build_view_model.rs`

Provider responsibilities:

- declare supported document kinds and workspace scopes;
- build observation frames/view models for panel type;
- map panel-local interactions into shell/domain commands.

This turns existing adapters like
`apps/runenwerk_editor/src/shell/outliner_adapter.rs`
and
`apps/runenwerk_editor/src/shell/inspector_adapter.rs`
into explicit provider implementations.

## Default Workspace Set (Target)

`Layout` (near-term)

- Documents: scene, prefab-like scene assets.
- Panels: outliner, entity-table, viewport, inspector, console.
- Modes: select, place, transform, paint, lighting, nav, physics-debug.

`UI` (post-MVP expansion)

- Documents: `*.ui.ron` style UI assets.
- Panels: ui hierarchy, canvas preview, style inspector, bindings, actions, diagnostics.
- Modes: layout, style, binding, animation, preview.

`Graphs` (post-MVP expansion)

- Documents: ability/material/behavior/dialogue graphs.
- Panels: graph canvas, node palette, graph outliner, params, preview, diagnostics.
- Modes: edit, connect, simulate, debug.

`Scripting` (post-MVP expansion)

- Documents: script assets.
- Panels: script editor, API browser, command/event refs, diagnostics, console.
- Modes: edit, run test, reload, trace.

Scripting remains language-neutral at the contract level; Rhai is the first concrete adapter candidate.

`Debug` (runtime inspection)

- Documents: optional runtime query tabs; primarily live runtime views.
- Panels: runtime outliner, ECS query viewer, component inspector, event log, profiler, render/physics debug.
- Modes: inspect, capture, replay, profiling.

`Editor Design` (self-authoring)

- Documents: workspace/menu/theme/shortcut definitions.
- Panels: workspace outliner, dock preview, panel registry, command palette, menu editor, shortcut editor, theme editor, inspector.
- Modes: layout-edit, command-bind, style-edit, preview.

This remains a later-phase track and aligns with authored editor-definition groundwork in
`docs-site/src/content/docs/design/active/editor-ui-workspace-tool-surface-architecture.md` (Phase E).

## Persistence and Ownership Rules

### Rule 1: Workspace presets are not scene-coupled

Legacy scene-coupled path helper retained for load fallback:

- `apps/runenwerk_editor/src/persistence/workspace_layout.rs::legacy_workspace_layout_path_for_scene`

Target:

- workspace profile/preset identity owned by editor shell domain contracts;
- workspace layout files addressed by explicit workspace profile identity;
- per-document layout overrides optional and explicit;
- session-only transient layout state remains app-local.

### Rule 2: Persisted structure keeps typed identities

Continue versioned persisted workspace DTOs in:

- `domain/editor/editor_shell/src/workspace/persisted.rs`

Do not infer host/panel/tab identities from index order.

### Rule 3: Document state and workspace state are separate stores

Document dirty/open/save state belongs in `editor_core` session documents.

Workspace graph/layout belongs in `editor_shell` workspace state.

## Context-Sensitive Panels: Provider Model

Runenwerk should keep one reusable outliner panel framework and switch semantics via providers.

Examples:

- Layout workspace + scene document -> outliner provider returns entity hierarchy.
- UI workspace + UI document -> outliner provider returns widget tree.
- Editor Design workspace + editor-layout document -> outliner provider returns dock host graph.

The same rule applies to inspector and canvas panels.

## Plugin Extension Direction

The extension seam should grow from existing tool-surface definition contracts:

- `domain/editor/editor_shell/src/workspace/surface_contract.rs::editor_surface_definitions`

Future plugin capability:

- register additional `ToolSurfaceKind`-equivalent definitions;
- attach provider implementations for supported document kinds;
- add workspace profile contributions (new tab + default layout template).

Constrained in-game editors should reuse these shared primitives but remain permissioned:

- surface interactions must stay capability-gated;
- all mutations must flow through explicit command + ratification boundaries.

This remains app-layer/plugin-layer composition, not domain invariant ownership.

## Migration Plan (Exact File Targets)

### Phase A - Introduce workspace profile abstraction without breaking current MVP

Status: complete for baseline scope.

- `domain/editor/editor_shell/src/workspace/mod.rs`
  - add `profile` module export.
- `domain/editor/editor_shell/src/workspace/profile.rs`
  - define workspace profile model and default profile registry.
- `apps/runenwerk_editor/src/shell/state.rs::RunenwerkEditorShellState`
  - track active workspace profile id separately from `WorkspaceState`.

### Phase B - Decouple layout persistence from scene path

Status: complete for baseline scope.

- `apps/runenwerk_editor/src/persistence/workspace_layout.rs`
  - replace scene-path-derived save/load coupling with explicit workspace-profile layout persistence APIs.
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs::save_scene_to_default_path`
  and `load_scene_from_default_path`
  - save and load layout through active workspace profile identity, with legacy scene-derived layout path used only as a read fallback.

### Phase C - Formalize document tabs and active document switching

- `domain/editor/editor_core/src/document.rs`
  - extend document taxonomy and metadata needed for workspace/document matching.
- `domain/editor/editor_core/src/session.rs`
  - enforce active document semantics independent from workspace state.
- `apps/runenwerk_editor/src/editor_runtime/document/` (module family)
  - split scene-specific runtime document state from generic document-tab management.

### Phase D - Replace adapter-only panel wiring with provider registry

- `apps/runenwerk_editor/src/shell/build_view_model.rs`
  - resolve panel view models via provider registry keyed by `(workspace, document_kind, panel_kind)`.
- `apps/runenwerk_editor/src/shell/outliner_adapter.rs`
  - migrate into outliner provider implementation.
- `apps/runenwerk_editor/src/shell/inspector_adapter.rs`
  - migrate into inspector provider implementation.
- `apps/runenwerk_editor/src/shell/viewport_adapter.rs`
  - migrate into viewport provider implementation.

### Phase E - Expand mode system from global enum to scoped mode sets

- `domain/editor/editor_core/src/session.rs::EditorMode`
  - evolve into mode ids + mode registry compatible with workspace/document constraints.
- `apps/runenwerk_editor/src/editor_features/tools.rs`
  - map user actions to scoped mode activation.
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`
  - validate mode commands against active workspace/document.

## Validation Plan

Add/expand tests in:

- `domain/editor/editor_shell/src/tests.rs`
  - workspace profile + workspace graph coexistence invariants.
- `apps/runenwerk_editor/src/editor_runtime/tests/architecture_guards.rs`
  - workspace/document separation guards.
- `apps/runenwerk_editor/src/shell/tests.rs`
  - provider routing correctness for context-sensitive panels.

Required checks:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `python3 tools/docs/validate_docs.py`

## Decision Summary

Runenwerk should adopt the model:

```text
Workspaces organize tools.
Documents own edited data.
Modes define interaction behavior.
Panels host reusable views.
Providers bind context to panels.
```

This keeps the current workspace identity architecture, resolves the scene-coupled layout issue, and provides a direct path from current MVP shell to Blender-style task workspaces with Runenwerk-native document semantics.
