---
title: Editor Composition Layouts
description: User and developer guide for Runenwerk editor composition profiles, persistence, content liveness, and the static cutover gate.
status: active
owner: editor
layer: app
canonical: true
last_reviewed: 2026-06-20
related_designs:
  - ../../design/accepted/app-neutral-ui-composition-design.md
related_adrs:
  - ../../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md
---

# Editor Composition Layouts

Runenwerk editor layouts use the app-neutral composition bundle format. A
layout is structural data plus a linked editor extension; it is not a serialized
editor runtime, native-window snapshot, provider payload, or workspace reducer
state.

## Current User Behavior

The editor still presents task-oriented profile names such as Scene, Modelling,
and Editor Design. Selecting a profile installs its validated composition. The
term “workspace” in the UI is product wording only.

The current clean-cutover checkpoint is static. Layout content renders and
normal editor tools work, but structural tab and docking edits are deferred
until the Region Compass runtime checkpoint. Attempted structural commands keep
the current layout unchanged and emit
`editor_composition.static.mutation_deferred`.

## What Is Saved

A save operation explicitly promotes the ratified composition state to a new
saved definition and snapshots one complete `runenwerk.editor.layout` extension.
The bundle links both documents with layout identity, definition revision,
schema versions, app compatibility, and content hashes.

The core document saves:

- presentation targets and structural roots;
- region topology and fixed-point split fractions;
- mounted-unit order and active mounted units;
- opaque mounted-content references and capability references.

The editor extension saves:

- editor profile identity;
- mounted-unit to panel/surface compatibility associations;
- stable editor content and panel-kind keys;
- region to tab-stack chrome associations and optional lock keys;
- floating bounds and viewport restore identity.

Labels never establish identity or ordering. Core and extension documents are
never written or loaded independently.

## Load And Failure Semantics

Loading is atomic. The editor validates the repository generation, linked core
and extension hashes, app compatibility, exact extension coverage, unique
compatibility IDs, stable content keys, target binding, and static projection.
Any failure leaves the current composition installed.

Legacy V1 through V5 `*.workspace.ron` files are unsupported. The editor may
read enough to classify the source and report that it is unsupported, but it
does not migrate, rewrite, delete, or accept the file.

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

Use `MountedUnitId` for provider requests, sessions, content liveness, routes,
viewport-instance records, and pruning. `ToolSurfaceInstanceId` is temporary
editor compatibility metadata and must not become a new map authority.

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

The main focused checks are:

```text
cargo test -p editor_shell composition
cargo test -p runenwerk_editor composition
cargo test -p runenwerk_editor --test composition_architecture_guards
cargo test -p runenwerk_editor --test startup_render_smoke
```

The architecture guard proves the live shell has no independent
`WorkspaceState` authority, active persistence has no legacy writer or reverse
loader, projection DTOs are composition-owned, and structural identities are
MountedUnitId-keyed.
