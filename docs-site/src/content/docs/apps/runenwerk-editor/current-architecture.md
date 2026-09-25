---
title: Runenwerk Editor Current Architecture
description: Current architecture overview for the runnable Runenwerk editor app.
status: active
owner: editor
layer: app
canonical: true
last_reviewed: 2026-09-15
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
selection, target-to-presentation bindings, native-window policy, command
execution, and the current scene persistence context.

Product-facing UI may still say “workspace” for a task-oriented editor profile.
`WorkspaceState` is not the live structural authority.

The current editor implementation still contains predecessor-shaped generic
`editor_core` document/session/mode contracts. Scene selection, scene history,
and generic scene dirty/save authority have been removed from that generic
session authority and now use explicit scene-owned/integration-owned contexts.
Those current repairs do not mean the whole Rust implementation already conforms
to ADR 0025.

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

## Scene Selection And History

Scene selection is held in an explicit scene-scoped selection context rather
than generic `EditorSession` state. Scope rollover on scene reset prevents stale
selection addresses from silently retargeting reused editor entity ids.

Scene undo/redo is coordinated by one app-owned scene history context. Each
history item retains the originating ratified change together with the scene
before/after snapshots needed to restore the admitted effect. New scene edits
clear scene redo; undo/redo restore first and then move exactly one item, so a
failed restore does not consume history and a successful redo preserves any
remaining redo chain. Selection resynchronization, projection-parity checks, and
undo/redo ratification remain part of the scene-history transition.

The current E2 command admission rule is deliberately narrower than the future
ADR-0025 activation model: scene history is the shell Undo/Redo target only when
the active document kind is `Scene`. A non-scene or absent active document makes
scene Undo/Redo unavailable and direct shell/shortcut dispatch fails closed
without consuming the dormant scene stack. This is a bounded current-routing
rule, not a claim that `DocumentKind` is the future universal history-context
resolver.

Toolbar and command-route availability are observed from that admitted scene
history context, not from `SessionReality`. Material Lab, `ui_composition`, and
self-authoring histories remain independent owner-specific histories. Later
ActivationScope/InvocationContext work may replace the bounded document-kind
admission with explicit context resolution; E2 does not invent that later
coordination subsystem.

## Scene Persistence

`RunenwerkEditorApp` owns one explicit `ScenePersistenceContext`. This is an app
host integration context for the current scene persistence owner, not a generic
editor registry or a new `editor_core` document service.

The context stores either an unbound scene origin or, after successful scene IO,
a concrete persistence target plus the last successfully loaded/written
normalized `SceneFileV2` projection. It does not store a dirty boolean or use a
monotonic editor/runtime revision as cleanliness authority. Current cleanliness
is derived on observation by forming and normalizing the current persistence-owned
scene projection and comparing it with the context comparison projection. This
means saving content A, editing away from A, undoing back to A, and redoing away
from A yields clean/dirty/clean/dirty according to effective persisted content.

Scene save advances the persisted target/baseline only after the normalized
projection is successfully written. Scene load advances it only after decode,
normalization, formation, and scene apply succeed. Failed save/load leaves the
previous context knowledge intact. A successful real scene load does not run the
MVP empty-scene bootstrap afterward, because that bootstrap creates authored
entities represented by `SceneFileV2` and would immediately diverge from an
empty loaded file. Startup/demo bootstrap remains a separate new-session concern.

The comparison boundary is intentionally exactly the current `SceneFileV2`
projection. Reflected/runtime-only components and resources not represented by
that DTO do not silently become persistence-owned content in E3. Material Lab,
asset/project, editor-definition, retained-change, and structural-composition
persistence are not folded into this scene context.

Primary native-window close admission queries the scene persistence context. A
dirty scene vetoes close; a clean scene admits it; inability to form the current
persistence projection fails closed. The removed generic `SaveDocumentTab` and
`CloseDocumentTab` commands are not retained as aliases for scene persistence.

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

The native multi-window foundation already supports distinct logical Editor windows,
native Host windows, Render surfaces, composition targets, target-local UI runtimes,
input routing, and detach-to-new-target coordination. Persisted multi-target
compositions restore through the same window coordinator: structural
`PresentationTargetId` values are retained, while every non-primary target receives a
fresh Editor/native/Render presentation attachment before the candidate replaces the
live composition. Creation failure rolls back provisional presentations and leaves the
current composition active.

Ordinary `Window > New Window` still requires the separately owned fresh-target layout
formation decision, and secondary-window viewport/product projection remains a later
multi-window closure slice. ADR 0025 separately governs semantic sharing: windows may
explicitly share editor bindings, selection contexts, history contexts, or persistence
contexts, while activation/focus/local presentation remain independently scoped.

## Persistence

`apps/runenwerk_editor/src/persistence/workspace_layout.rs` saves and loads
atomic composition bundle generations through `CompositionBundleRepository`.
Save explicitly promotes ratified state and snapshots the complete typed editor
extension. Load validates the linked core envelope, app compatibility,
extension schema, hashes, and editor extension before installation. Multi-target load
does not persist or reuse native/render handles: it reconstructs fresh secondary
presentation attachments and installs the complete target-binding set only after every
requested native window reaches the created state.

V1 through V5 workspace files are unsupported compatibility input. The app may
probe them to emit a diagnostic, but it does not make them live structural
authority. See [`composition-layouts.md`](./composition-layouts.md) for the
operator and developer contract.

Editor/domain semantic persistence is not implied by composition persistence.
The current scene path uses the explicit app-owned scene context described above;
project, asset, editor-definition, Material Lab, retained-change, and other
persistence contracts retain their concrete owners. This is a bounded ADR-0025
ownership repair, not a universal persistence framework.

## Viewport Runtime

Viewport lifecycle, frame submission, input routing, and mounted-surface
registries read composition and editor extension bindings. The primary viewport
record map is keyed by `MountedUnitId`; compatibility indexes permit existing
viewport and surface consumers to resolve that record without becoming
structural authorities.

Viewport product targets, render jobs, picking, and retained
`ViewportSurfaceEmbed` projection remain app/runtime concerns. They do not write
composition structure during ordinary frame updates.

Viewport tool activation is session-local state owned by the mounted-unit
`SurfaceSessionStore`. Provider actions and shortcuts carry the exact mounted
viewport target and projection epoch; stale or structurally mismatched requests
fail closed. Picking and direct manipulation resolve the tool from that same
viewport session, so mounted viewports do not share an active-tool authority.

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
