---
title: Workspace Roadmap Index
description: Workspace-level roadmap index and sequencing links across active architecture tracks.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-29
---

# Workspace Roadmap Index

## Purpose

Provide one workspace-level index of active roadmap/design tracks without duplicating domain-level phase details.

This page is an index, not the source of truth for domain-specific execution steps.

## Source-of-Truth Tracks

- UI substrate and surface execution roadmap:
  - [domain/ui/roadmap.md](../domain/ui/roadmap.md)
- UI current-state architecture:
  - [domain/ui/architecture.md](../domain/ui/architecture.md)
- Editor/UI/workspace long-horizon architecture:
  - [design/active/editor-ui-workspace-tool-surface-architecture.md](../design/active/editor-ui-workspace-tool-surface-architecture.md)
- Workspace identity contract and migration map:
  - [design/active/workspace-identity-contract-and-migration-map.md](../design/active/workspace-identity-contract-and-migration-map.md)
- Viewport backend closeout evidence:
  - [reports/closeouts/viewport-backend-cleanup/phase-1-plan.md](../reports/closeouts/viewport-backend-cleanup/phase-1-plan.md)

## Current Cross-Track Status

- viewport backend cleanup is complete for its closeout scope;
- workspace structural identity and routing contracts are implemented and guard-tested;
- UI substrate crates and `ui_surface` contracts are implemented and integrated in production editor flows;
- current active work is primarily docking/tab behavior, broader non-viewport surface maturity, and continued guard/doc drift control.

## Recommended Near-Term Order

1. Complete docking/tab behavior on existing structural identity contracts.
2. Expand non-viewport surface maturity through existing `ui_surface` contracts.
3. Preserve and extend architecture guards while these features land.
4. Keep domain and workspace docs synchronized with shipped behavior.

## Rule

When domain roadmaps and workspace index notes diverge, the owning domain roadmap is authoritative for implementation sequencing.
