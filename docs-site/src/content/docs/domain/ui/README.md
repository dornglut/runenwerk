---
title: UI Domain
description: Documentation index for Runenwerk UI substrate and surface semantics.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-04-29
---

# UI Domain

`domain/ui/*` owns reusable, engine-agnostic UI substrate contracts and retained runtime layers.

## Current Docs

- [Current Architecture](./architecture.md)
- [Roadmap](./roadmap.md)

## Related Design Docs

- [Editor UI Workspace Tool Surface Architecture](../../design/active/editor-ui-workspace-tool-surface-architecture.md)
- [Workspace Identity Contract and Migration Map](../../design/active/workspace-identity-contract-and-migration-map.md)
- [Viewport Expression Upgrade Design](../../design/active/viewport-expression-upgrade-design.md)

## Scope Boundary

`domain/ui` owns substrate/runtime contracts (`ui_tree`, `ui_runtime`, `ui_widgets`, `ui_surface`).

`domain/ui` does not own editor-shell workspace semantics, app runtime wiring, or renderer execution policy.
