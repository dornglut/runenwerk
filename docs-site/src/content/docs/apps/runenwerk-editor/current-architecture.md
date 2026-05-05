---
title: Runenwerk Editor Current Architecture
description: Current architecture overview for the runnable Runenwerk editor app.
status: active
owner: editor
layer: app
canonical: true
last_reviewed: 2026-05-04
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

## Related Docs

- Domain editor contracts: [`../../domain/editor/README.md`](../../domain/editor/README.md)
- UI architecture: [`../../domain/ui/architecture.md`](../../domain/ui/architecture.md)
- Editor roadmap: [`roadmap.md`](roadmap.md)
