---
title: Editor Surface Provider Plugin Seam
description: Superseded decision for per-surface-instance editor provider composition.
status: superseded
owner: editor
layer: domain
canonical: false
last_reviewed: 2026-06-19
superseded_by:
  - ../accepted/0013-app-neutral-ui-composition-clean-cutover.md
---

# Editor Surface Provider Plugin Seam

## Supersession

ADR 0013 supersedes this structural vocabulary and identity model. Its durable
provider-ownership rule remains: provider contracts stay app/editor-owned and
actual mutation stays behind typed command and ratification boundaries.

`ToolSurfaceInstanceId`, `WorkspaceState`, and surface-oriented structural
authority are replaced by app-neutral composition and `MountedUnitId`.

## Historical Decision

Use `ToolSurfaceInstanceId` as the primary runtime surface identity. The shell
hosts chrome, docking, tabs, splits, and structural routes. Surface providers
own observation, presentation artifacts, provider-local routes, and mapping
local actions to typed command proposals.

Provider contracts that are app/runtime neutral belong in
`domain/editor/editor_shell`. Concrete providers and registry composition belong
in `apps/runenwerk_editor` or the editor plugin host. The registry is passed
explicitly; it is not global mutable state.

Provider resolution must be deterministic and fail-closed. Actual mutations
remain behind existing command and ratification boundaries. Provider ID is not
a command origin.

