---
title: Runenwerk Editor Current Architecture
description: Current architecture overview for the runnable Runenwerk editor app.
status: active
owner: editor
layer: app
canonical: true
last_reviewed: 2026-06-20
related_designs:
  - ../../design/accepted/app-neutral-ui-composition-design.md
  - ../../design/active/editor-ui-workspace-tool-surface-architecture.md
related_adrs:
  - ../../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md
related_roadmaps:
  - ../../workspace/production-tracks.yaml
---

# Runenwerk Editor Current Architecture

`apps/runenwerk_editor` is the runnable editor application. It composes editor
domain behavior, the app-neutral UI composition model, retained UI projection,
engine runtime systems, persistence, and viewport expression routing.

## Entry Points

- `apps/runenwerk_editor/src/main.rs`: editor binary entry point.
- `apps/runenwerk_editor/src/lib.rs`: public app crate surface.
- `apps/runenwerk_editor/src/runtime/app.rs`: headless and interactive app construction.
- `apps/runenwerk_editor/src/runtime/plugin.rs`: engine plugin and native-window integration.

## Ownership Boundary

`domain/ui/ui_composition` owns structural targets, roots, regions, mounted
units, transactions, history, promotion, fixtures, and persistence envelopes.
It does not depend on editor, engine, native-window, renderer, `UiProgram`, or
`ui_surface` contracts.

`domain/editor/editor_shell/src/composition/structural` owns the editor-specific
one-way importer, typed extension schema, diagnostics, and pure static shell
projection. `apps/runenwerk_editor` owns providers, sessions, storage paths,
profile selection, target-to-presentation bindings, native-window policy, and
command execution.

Product-facing UI may still say “workspace” for a task-oriented editor profile.
`WorkspaceState` is not the live structural authority.

## Structural Composition Runtime

`RunenwerkEditorShellState` stores one `EditorCompositionRuntime`, pairing a
ratified `CompositionState` with one validated `EditorCompositionExtensionV1`.
It also stores the derived `EditorCompositionProjectionArtifact`. Installing a
runtime validates the core state, extension coverage, compatibility identities,
static target binding, and projection before replacing any live state.

The editor extension contains only app/editor associations that do not belong
in the neutral graph: profile identity, panel and surface compatibility IDs,
stable content keys, tab-stack chrome IDs, floating bounds, and viewport restore
identity. Split topology, parentage, mounted-unit order, active units, targets,
and roots remain exclusively in `CompositionState`.

The current built-in editor profiles are imported once through
`import_legacy_workspace`. The resulting `WorkspaceState` input is dropped.
Legacy workspace construction and reduction remain available only to crate
tests as a temporary parity oracle while the clean cutover branch proceeds.

Reusable shell projection DTOs and route assembly are owned by
`composition/structural/projection.rs`. The legacy
`workspace/projection.rs::project_workspace_for_shell` path only produces that
composition-owned artifact for parity tests.

## Static Cutover Gate

This checkpoint intentionally projects a static composition. Profile selection,
provider content, viewport interaction, retained controls, focus, and product
actions remain operational. Structural tab, split, close, duplicate, reset,
lock, drag, and docking commands emit
`editor_composition.static.mutation_deferred` and do not mutate the graph.

Region Compass is the selected visual direction, but its chrome and adaptive
docking runtime belong to later governed checkpoints. This branch state is not
mergeable until those checkpoints, cleanup, accessibility acceptance, and the
final closeout pass.

## Provider And Content Liveness

Mounted provider requests are projected from core mounted units plus typed
editor extension records. Requests, surface sessions, viewport instances,
routes, and pruning use `MountedUnitId` as their structural key.
`ToolSurfaceInstanceId` remains temporary editor compatibility metadata, not
the authority.

Content resolution has seven explicit states: resolved, missing, loading,
suspended, denied, unsupported profile, and crashed. Unavailable content uses
this order:

1. app-provided unavailable-content projection;
2. neutral diagnostic placeholder;
3. hidden only when both the mounted content policy and host allow hiding.

Every rejection carries a stable `editor_composition.*` code, severity, stage,
mounted-unit or other typed subject, and an actionable message. Editor records
convert to the foundation diagnostic contract.

## Presentation Targets And Native Windows

The composition graph owns `PresentationTargetId`; the app binds supported
targets to `EditorWindowPresentationBinding`. The static editor checkpoint
accepts exactly one target and binds it to the primary app-owned native window
and render surface. Native-window lifecycle, monitor bounds, DPI, restore
policy, and OS vetoes remain app/engine-owned.

## Persistence

`apps/runenwerk_editor/src/persistence/workspace_layout.rs` saves and loads
atomic composition bundle generations through `CompositionBundleRepository`.
Save explicitly promotes ratified state and snapshots the complete typed editor
extension. Load validates the linked core envelope, app compatibility,
extension schema, hashes, and editor extension before installation.

V1 through V5 workspace files are unsupported compatibility input. The app may
probe them to emit a diagnostic, but it does not migrate, rewrite, delete, or
load them. See [`composition-layouts.md`](./composition-layouts.md) for the
operator and developer contract.

## Viewport Runtime

Viewport lifecycle, frame submission, input routing, and mounted-surface
registries read composition and editor extension bindings. The primary viewport
record map is keyed by `MountedUnitId`; compatibility indexes permit existing
viewport and surface consumers to resolve that record without becoming
structural authorities.

Viewport product targets, render jobs, picking, and retained
`ViewportSurfaceEmbed` projection remain app/runtime concerns. They do not write
composition structure during frame updates.

## Self-Authoring State

The app-owned self-authoring document lifecycle remains in
`apps/runenwerk_editor/src/shell/self_authoring`. It edits definition documents,
forms retained previews, validates them through owning domain crates, and keeps
applied snapshots for rollback. Applied workspace-layout definitions are
converted into a candidate composition runtime and installed atomically; they
do not replace live state through a workspace reducer.

## Related Docs

- Composition layout guide: [`composition-layouts.md`](./composition-layouts.md)
- Domain UI architecture: [`../../domain/ui/architecture.md`](../../domain/ui/architecture.md)
- Editor domain contracts: [`../../domain/editor/README.md`](../../domain/editor/README.md)
- UI composition usage: [`../../domain/ui/ui-composition-usage.md`](../../domain/ui/ui-composition-usage.md)
