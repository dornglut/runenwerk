---
title: Editor Surface Provider Plugin Seam
description: Accepted decision for per-surface-instance editor provider composition.
status: accepted
owner: editor
layer: domain
canonical: true
last_reviewed: 2026-05-04
---

# Editor Surface Provider Plugin Seam

## Context

The editor shell needs to mount any registered surface type as one or more
runtime instances. Fixed panel-slot composition blocks multiple instances of the
same surface type and makes routes depend on hardcoded panel slots.

## Decision

Use `ToolSurfaceInstanceId` as the primary runtime surface identity. The shell
hosts chrome, docking, tabs, splits, and structural routes. Surface providers
own observation, presentation artifacts, provider-local routes, and mapping
local actions to typed command proposals.

Provider contracts that are app/runtime neutral belong in
`domain/editor/editor_shell`. Concrete providers and registry composition belong
in `apps/runenwerk_editor` or the editor plugin host. The registry is passed
explicitly; it is not global mutable state.

Provider resolution must be deterministic and fail-closed:

- duplicate provider ids are rejected;
- explicit descriptor priority resolves multiple supported providers;
- equal-priority ambiguity produces a diagnostic artifact;
- unsupported, ambiguous, and error artifacts emit zero provider-local routes.

Actual mutations remain behind existing command and ratification boundaries.
Provider id is not a command origin.

## Consequences

`EditorShellFrameModel.surfaces` is a resolved artifact lookup, not layout
authority. Workspace layout, tab order, split hierarchy, active panel, and active
mounted surface continue to come from `WorkspaceState` projection.

Surface-local UI state must be keyed by `ToolSurfaceInstanceId`, with explicit
cleanup/retention behavior when surfaces unmount or documents reload.
