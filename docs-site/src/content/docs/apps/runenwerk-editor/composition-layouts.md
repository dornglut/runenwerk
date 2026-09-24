---
title: Editor Composition Layouts
description: User and developer guide for Runenwerk editor composition profiles, persistence, content liveness, and structural transactions.
status: active
owner: editor
layer: app
canonical: true
last_reviewed: 2026-09-14
related_designs:
  - ../../design/accepted/app-neutral-ui-composition-design.md
  - ../../design/accepted/runenwerk-editor-coordination-semantic-model.md
related_adrs:
  - ../../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md
  - ../../adr/accepted/0025-normalize-editor-coordination-and-semantic-ownership.md
---

# Editor Composition Layouts

Runenwerk editor layouts use the app-neutral composition bundle format. A
layout is structural data plus a linked editor extension; it is not a serialized
editor runtime, native-window snapshot, provider payload, foreign semantic
state, or generic editor session.

## Current User Behavior

The editor presents task-oriented profile names such as Scene, Modelling, and
Editor Design. Selecting a profile installs its validated composition. The term
“workspace” in the UI is product wording; structural authority is
`ui_composition`.

Ordinary structural shell commands now commit through `ui_composition`
transactions. The old static-cutover behavior that rejected normal structural
commands with `editor_composition.static.mutation_deferred` is historical and
must not be used as the description of current behavior.

This current structural behavior is separate from ADR 0025. The accepted editor
coordination model normalizes future semantic ownership for bindings, sessions,
activation, projection publication, selection, history, and persistence; it
does not claim current Rust already implements that semantic model.

## What Is Saved

A save operation explicitly promotes ratified composition state to a new saved
definition and snapshots one complete `runenwerk.editor.layout` extension. The
bundle links both documents with layout identity, definition revision, schema
versions, app compatibility, and content hashes.

The core document saves:

- presentation targets and structural roots;
- region topology and fixed-point split fractions;
- mounted-unit order and active mounted units;
- opaque mounted-content references and capability references.

The editor extension saves:

- editor profile identity;
- mounted-unit to panel/surface compatibility associations where current code
  still needs them;
- stable editor content and panel-kind keys;
- region to tab-stack chrome associations and optional lock keys;
- floating bounds and viewport restore identity.

Labels never establish identity or ordering. Core and extension documents are
never written or loaded independently.

Composition persistence does not define the persistence unit for edited semantic
state. Project, scene, asset, UI-definition, drawing, graph, and future owners
retain their own persistence semantics. ADR 0025 models editor coordination to
those owners through explicit `PersistenceContext`s rather than one universal
layout/document dirty state.

## Load And Failure Semantics

Loading a composition bundle is atomic. The editor validates the repository
generation, linked core and extension hashes, app compatibility, exact extension
coverage, compatibility identities, stable content keys, target binding, and
projection. Failure leaves the current composition installed.

Legacy V1 through V5 `*.workspace.ron` files are unsupported compatibility
input. The editor may read enough to classify the source and report that it is
unsupported, but those files do not become a second live structural authority.

## Structural Transactions And History

Structural tab, split, move, close, activation, lock, and related layout changes
enter the composition transaction path. Structural history is owned by
`ui_composition` and is limited to structural composition state.

Do not reinterpret composition undo/redo as owner/domain history. Editor-facing
Undo/Redo for semantic content resolves through explicit history ownership; the
normalized target contract is `HistoryContext` in ADR 0025.

## Unavailable Content

The structural composition remains valid when mounted content is unresolved.
Each mounted unit may be resolved, missing, loading, suspended, denied,
unsupported by the current profile, or crashed.

Projection chooses the first available fallback:

1. an app-provided unavailable-content view;
2. a neutral diagnostic placeholder;
3. hidden content only when the mounted profile explicitly permits hiding and
   the host accepts it.

Content liveness never rewrites region topology, mounted-unit identity, or the
saved definition.

## Identity Rules For Developers

Use `MountedUnitId` for structural provider attachment, sessions, content
liveness, routes, viewport-instance records, and pruning where current
composition integration requires a structural key. `ToolSurfaceInstanceId` is
compatibility/editor metadata where still present and must not become a new
structural authority.

Do not infer semantic identity from `MountedUnitId`. A mounted unit locates
content in structural composition; edited semantic authority remains with the
owner exposed through editor binding/provider contracts.

Use `PresentationTargetId` to refer to a structural presentation target. Bind
it to app-owned window/render presentation state outside `ui_composition`.
Never place native handles, monitor data, DPI, or OS lifecycle state in core or
editor extension documents.

New extension records must be deterministic, exactly cover their core
identities, and pass `EditorCompositionExtensionV1::validate_against`. Do not
add a generic payload map or duplicate topology in the extension.

## Diagnostics

Editor layout diagnostics use the `editor_composition.*` namespace and include
severity, stage, typed subject, and an actionable message. Persistence uses
`composition_persistence.*`; neutral core validation uses `ui_composition.*`.
Do not derive behavior from diagnostic display text.

## Verification Entry Points

The main focused checks include:

```text
cargo test -p editor_shell composition
cargo test -p runenwerk_editor composition
cargo test -p runenwerk_editor --test composition_architecture_guards
cargo test -p runenwerk_editor --test startup_render_smoke
```

The architecture guard proves the live shell has no independent
`WorkspaceState` structural authority, active persistence has no legacy writer
or reverse loader, projection DTOs are composition-owned, structural identities
are `MountedUnitId`-keyed, and normal structural commands commit through the
composition gateway rather than the retired static mutation-deferred path.
