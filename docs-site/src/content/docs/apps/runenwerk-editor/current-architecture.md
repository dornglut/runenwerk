---
title: Runenwerk Editor Current Architecture
description: Current architecture overview for the runnable Runenwerk editor app.
status: active
owner: editor
layer: app
canonical: true
last_reviewed: 2026-05-06
---

# Runenwerk Editor Current Architecture

`apps/runenwerk_editor` is the runnable editor application. It composes editor
domain crates, UI substrate crates, engine runtime systems, persistence, and
viewport expression routing into a concrete tool.

## Entry Points

- `apps/runenwerk_editor/src/main.rs`: binary entry point.
- `apps/runenwerk_editor/src/lib.rs`: public app crate surface.
- `apps/runenwerk_editor/src/runtime/app.rs`: app/runtime construction.
- `apps/runenwerk_editor/src/runtime/plugin.rs`: engine plugin integration.

## App-Owned Areas

- `editor_app`: high-level app state and facade.
- `editor_runtime`: scene state, command ratification, history, selection, and
  tool state.
- `editor_features`: editor feature actions and viewport tools.
- `editor_panels`: concrete panel/widget composition.
- `runtime`: engine-facing systems, resources, viewport routing, expression
  product registration, picking, and frame submission.
- `shell`: app-owned concrete editor surface providers, provider registry
  composition, shell controller wiring, and surface-session state.
- `persistence`: retained change storage, project files, runtime persistence,
  and workspace layout.

## Ownership Boundary

The app owns concrete wiring and host policy. It should not redefine editor
domain semantics, UI surface contracts, world edit contracts, or engine runtime
contracts that already live in owning crates.

## Surface Provider Architecture

The editor shell uses `EditorShellFrameModel`, where mounted surfaces are
resolved by `ToolSurfaceInstanceId`:

```text
Workspace/profile/document context
+ mounted ToolSurfaceInstanceId
+ ToolSurfaceKind / SurfaceDefinitionId
        -> app-owned provider registry
        -> provider-owned artifact + provider-local routes
        -> shell host chrome/docking/tabs
```

Concrete providers live in `apps/runenwerk_editor/src/shell/providers/`.
The M3.6 self-authoring workspace registers app-owned provider surfaces for
definition outliner, UI hierarchy, UI canvas/retained preview, style inspector,
bindings, dock/layout preview, theme editor, shortcut editor, menu editor,
definition validation, and command diff summaries. These surfaces inspect,
preview, and author editor/UI definition documents through retained control
routes that propose shell commands; they do not execute app commands directly
or move provider behavior into `domain/ui/ui_definition`.
Provider contracts that are app/runtime neutral live in
`domain/editor/editor_shell/src/surface_provider.rs`.

The registry is explicitly composed by the editor app/plugin host. It is not a
global mutable registry. Provider resolution is deterministic and fail-closed:
duplicate provider ids are rejected, equal-priority provider ambiguity produces
an ambiguous diagnostic artifact, unsupported surfaces render an unsupported
artifact, and diagnostic artifacts emit no provider-local routes.

Surface-local UI state is stored per `ToolSurfaceInstanceId` in
`apps/runenwerk_editor/src/shell/surface_session.rs`. Console lines, app
diagnostics, runtime/session state, and toolbar state remain app/global; console
view state, entity table filters, inspector draft/focus state, and viewport
interaction/details state are surface-session concerns.

## Shell Layout

The app toolbar is produced by
`apps/runenwerk_editor/src/shell/toolbar_adapter.rs::build_toolbar_observation_frame`
and rendered by
`domain/editor/editor_shell/src/composition/build_toolbar.rs::build_toolbar`.
It exposes File, Edit, and Window menu controls, followed by Scene and Modelling
workspace profile switches plus a disabled add-workspace placeholder. Menu
items whose workflows are not implemented are emitted as disabled toolbar
buttons so the retained UI renders them as unavailable instead of routing them
to app behavior.

Default workspace profiles are defined in
`domain/editor/editor_shell/src/workspace/profile.rs::default_workspace_profile_registry`.
The Scene and Modelling profiles are distinct workspace profiles; both currently
use the same structural shell template while retaining separate profile identity
and profile-addressed layout persistence. The Editor Design profile uses
`WorkspaceState::bootstrap_editor_design_layout` to expose self-authoring
definition, preview, validation, styling, binding, and diff surfaces.

The default structural layout is defined in
`domain/editor/editor_shell/src/workspace/state.rs::WorkspaceState::bootstrap_current_layout`.
It places the viewport in the expanding left/middle area, the scene hierarchy
above the inspector in the right sidebar, and the console/log surface in the
bottom band. The compatibility projection for that structure is maintained in
`domain/editor/editor_shell/src/workspace/projection.rs::project_fixed_layout`.

## Self-Authoring State

The app-owned self-authoring document lifecycle lives in
`apps/runenwerk_editor/src/shell/self_authoring.rs::SelfAuthoringWorkspaceState`.
It loads checked-in UI fixtures and editor-owned schema documents as editable
definition documents, validates them through `domain/editor/editor_definition`,
forms retained previews through `domain/ui/ui_definition`, edits draft UI node
text/theme color/workspace-layout data through explicit shell commands, and
keeps explicit applied snapshots for rollback. Apply and rollback are shell commands handled in
`apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`, preserving the
app/shell command boundary.

## Related Docs

- Domain editor contracts: [`../../domain/editor/README.md`](../../domain/editor/README.md)
- UI architecture: [`../../domain/ui/architecture.md`](../../domain/ui/architecture.md)
- Editor roadmap: [`roadmap.md`](roadmap.md)
