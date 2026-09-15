---
title: Runenwerk Editor Current Architecture
description: Current architecture overview for the runnable Runenwerk editor app.
status: active
owner: editor
layer: app
canonical: true
last_reviewed: 2026-09-14
related_designs:
  - ../../design/accepted/app-neutral-ui-composition-design.md
  - ../../design/accepted/runenwerk-editor-coordination-semantic-model.md
  - ../../design/implemented/editor-tool-suite-registry-and-workbench-host-design.md
related_adrs:
  - ../../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md
  - ../../adr/accepted/0025-normalize-editor-coordination-and-semantic-ownership.md
---

# Runenwerk Editor Current Architecture

`apps/runenwerk_editor` is the runnable editor application. It composes editor
domain behavior, the app-neutral UI composition model, retained UI projection,
engine runtime systems, persistence, and viewport expression routing.

Code and tests own current behavior. ADR 0025 and the accepted editor coordination
semantic model define normalized target ownership for later editor-boundary repair;
they do not imply that current Rust already conforms to that target.

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
one-way importer, typed extension schema, diagnostics, and shell projection.
`apps/runenwerk_editor` owns providers, sessions, storage paths, profile
selection, target-to-presentation bindings, native-window policy, and command
execution.

Product-facing UI may still say “workspace” for a task-oriented editor profile.
`WorkspaceState` is not the live structural authority.

The current editor implementation still contains predecessor-shaped generic
`editor_core` document/session/mode/selection/history contracts. Those are
current implementation facts. They are not the normalized long-term semantic
ownership model accepted by ADR 0025.

## Structural Composition Runtime

`RunenwerkEditorShellState` stores one `EditorCompositionRuntime`, pairing a
ratified `CompositionState` with one validated `EditorCompositionExtensionV1`.
It also stores the derived `EditorCompositionProjectionArtifact`. Installing a
runtime validates the core state, extension coverage, compatibility identities,
target binding, and projection before replacing live state.

The editor extension contains only app/editor associations that do not belong
in the neutral graph: profile identity, panel and surface compatibility IDs,
stable content keys, tab-stack chrome IDs, floating bounds, and viewport restore
identity. Split topology, parentage, mounted-unit order, active units, targets,
and roots remain exclusively in `CompositionState`.

The current built-in editor profiles are imported through
`import_legacy_workspace`. The resulting `WorkspaceState` input is dropped.
Legacy workspace construction and reduction remain only as compatibility/test
inputs where current source still requires them; they are not a second live
structural authority.

Reusable shell projection DTOs and route assembly are owned by
`composition/structural/projection.rs`. The legacy
`workspace/projection.rs::project_workspace_for_shell` path exists for parity
coverage rather than as the normal structural owner.

## Structural Transactions

The earlier static cutover gate is no longer current behavior.

Current architecture guards prove that ordinary structural shell commands commit
through the `ui_composition` transaction path and advance composition revision.
The same guard explicitly rejects a return of the old
`editor_composition.static.mutation_deferred` behavior for that path.

Structural tab/stack/layout actions therefore use the composition gateway rather
than a writable `WorkspaceState` reducer authority. This statement describes
current tested behavior only; it does not claim the normalized ADR 0025 editor
coordination model is implemented.

## Provider And Content Liveness

Mounted provider requests are projected from core mounted units plus typed
editor extension records. Requests, surface sessions, viewport instances,
routes, and pruning use `MountedUnitId` as their structural key.
`ToolSurfaceInstanceId` remains compatibility/editor metadata where current code
still needs it; it is not structural authority.

Content resolution has seven explicit states: resolved, missing, loading,
suspended, denied, unsupported profile, and crashed. Unavailable content uses
this order:

1. app-provided unavailable-content projection;
2. neutral diagnostic placeholder;
3. hidden only when both the mounted content policy and host allow hiding.

Every rejection carries a stable `editor_composition.*` code, severity, stage,
typed subject, and actionable message. Editor records convert to the foundation
diagnostic contract.

## Presentation Targets And Native Windows

The composition graph owns `PresentationTargetId`; the app binds supported
targets to `EditorWindowPresentationBinding`. Native-window lifecycle, monitor
bounds, DPI, restore policy, and OS vetoes remain app/engine-owned.

The accepted native multi-window design owns the future/native presentation
mechanics. ADR 0025 separately governs semantic sharing: windows may explicitly
share editor bindings, selection contexts, history contexts, or persistence
contexts, while activation/focus/local presentation remain independently scoped.

## Persistence

`apps/runenwerk_editor/src/persistence/workspace_layout.rs` saves and loads
atomic composition bundle generations through `CompositionBundleRepository`.
Save explicitly promotes ratified state and snapshots the complete typed editor
extension. Load validates the linked core envelope, app compatibility,
extension schema, hashes, and editor extension before installation.

V1 through V5 workspace files are unsupported compatibility input. The app may
probe them to emit a diagnostic, but it does not make them live structural
authority. See [`composition-layouts.md`](./composition-layouts.md) for the
operator and developer contract.

Editor/domain semantic persistence is not implied by composition persistence.
Current project/scene/editor persistence contracts retain their concrete owners;
ADR 0025 defines the future coordination boundary through explicit
`PersistenceContext`s.

## Viewport Runtime

Viewport lifecycle, frame submission, input routing, and mounted-surface
registries read composition and editor extension bindings. The primary viewport
record map is keyed by `MountedUnitId`; compatibility indexes permit existing
viewport and surface consumers to resolve that record without becoming
structural authorities.

Viewport product targets, render jobs, picking, and retained
`ViewportSurfaceEmbed` projection remain app/runtime concerns. They do not write
composition structure during ordinary frame updates.

## Self-Authoring State

The app-owned self-authoring document lifecycle remains in
`apps/runenwerk_editor/src/shell/self_authoring`. It edits definition documents,
forms retained previews, validates them through owning domain crates, and keeps
applied snapshots for rollback. Applied workspace-layout definitions are
converted into candidate composition runtime state and installed through the
composition boundary rather than establishing a parallel workspace reducer
source of truth.

Current self-authoring product wording may still refer to documents, workspaces,
modes, and dirty state. Those terms describe existing product/implementation
behavior unless a concrete owner defines them; the normalized editor semantic
owner is ADR 0025 plus its companion model.

## Related Docs

- Normalized editor coordination: [`../../design/accepted/runenwerk-editor-coordination-semantic-model.md`](../../design/accepted/runenwerk-editor-coordination-semantic-model.md)
- Composition layout guide: [`composition-layouts.md`](./composition-layouts.md)
- Domain UI architecture: [`../../domain/ui/architecture.md`](../../domain/ui/architecture.md)
- Editor implementation grouping: [`../../domain/editor/README.md`](../../domain/editor/README.md)
- UI composition usage: [`../../domain/ui/ui-composition-usage.md`](../../domain/ui/ui-composition-usage.md)
