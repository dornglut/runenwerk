---
title: Runenwerk Editor Current Architecture
description: Current architecture overview for the runnable Runenwerk editor app.
status: active
owner: editor
layer: app
canonical: true
last_reviewed: 2026-04-28
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
- `shell`: adapters from editor domain shell state to app view models.
- `persistence`: retained change storage, project files, runtime persistence,
  and workspace layout.

## Ownership Boundary

The app owns concrete wiring and host policy. It should not redefine editor
domain semantics, UI surface contracts, world edit contracts, or engine runtime
contracts that already live in owning crates.

## Related Docs

- Domain editor contracts: [`../../domain/editor/README.md`](../../domain/editor/README.md)
- UI architecture: [`../../domain/ui/architecture.md`](../../domain/ui/architecture.md)
- Editor roadmap: [`roadmap.md`](roadmap.md)
