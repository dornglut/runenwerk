---
title: Capability Workbench Clean Break
description: Superseded clean-break decision for Workbench surface identity and capability declarations.
status: superseded
owner: editor
layer: domain/app
canonical: false
last_reviewed: 2026-06-19
superseded_by:
  - ../accepted/0013-app-neutral-ui-composition-clean-cutover.md
related_adrs:
  - ../accepted/0001-use-domain-owned-commands.md
  - ../accepted/0004-separate-description-from-execution.md
  - ../accepted/0005-projections-are-derived-state.md
  - ./0006-editor-surface-provider-plugin-seam.md
  - ../accepted/0010-graph-substrate-canvas-boundary.md
---

# Capability Workbench Clean Break

## Supersession

ADR 0013 supersedes the stable surface/workbench identity and persistence
authority in this decision. Its durable clean-break principles remain:

- do not retain parallel legacy identity authorities;
- reject unsupported old schemas explicitly;
- keep provider and capability policy typed and fail-closed;
- do not auto-migrate the authority being removed.

The replacement authority is `MountedUnitId`, typed content/profile references,
app-owned provider bundles, and generic composition persistence.

## Historical Decision

Workbench identity was required to become registry-owned through typed suite
declarations, stable surface keys, profile declarations, provider declarations,
and host capability policy. `ToolSurfaceKind` was removed from Workbench
identity and legacy workspace layouts were rejected rather than migrated.

