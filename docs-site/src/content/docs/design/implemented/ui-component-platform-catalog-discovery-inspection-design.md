---
title: UI Component Platform Catalog Discovery And Inspection Design
description: Implemented Runenwerk-local read-only catalog, query, discovery, and inspection contracts derived from control package authority.
status: implemented
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-09-13
related_designs:
  - ./ui-component-platform-control-kernel-design.md
  - ./ui-component-platform-authoring-kit-design.md
  - ./ui-component-platform-story-proof-envelope-design.md
  - ../active/runenwerk-ui-story-driven-golden-workflow-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
---

# UI Component Platform Catalog Discovery And Inspection Design

## Status

Implemented in the current Runenwerk-local UI stack.

## Implemented contract

`ui_controls` derives searchable, deterministic, read-only catalog and inspection views from validated package/control descriptors. Current catalog code includes the index/entry/query/filter/inspection model under `domain/ui/ui_controls/src/catalog/`.

The implemented query contract supports filtering by package id, control kind id, category, tag, target profile, capability, story requirement, mount eligibility, and diagnostic presence. Inspection projects normalized package/control facts such as identity, metadata, compatibility, schemas, kernels, routes, fixtures, stories/proof summaries, diagnostics, and mount explanation.

## Ownership

`ui_controls` owns catalog meaning and package-derived inspection DTOs. `ui_artifacts` may transport read-only snapshots. Gallery, Workbench, UI Designer, docs, and agents are consumers; they do not become reusable-control semantic owners.

Catalog state is rebuildable from owning descriptors/evidence and is not a new mutable source of truth.

## Intentional landed differences

The proposal listed candidate names including `ControlDiagnosticBadge`, `ControlCompatibilitySummary`, and `ControlStoryProofBadge`. The implementation converged on the current catalog entry/query/inspection types instead of preserving all candidate names as public wrappers. This is a naming/shape refinement, not loss of the required catalog semantics.

## Boundary rules

- Catalog entries are derived projections, not package authority.
- Queries and filters do not mutate package/runtime state.
- Inspection explains mount/proof status but cannot upgrade it.
- No hidden global package registry is introduced.
- Product-specific presentation, preview execution, story execution, and runtime widget behavior remain elsewhere.

## Evidence

Current catalog source plus focused `control_catalog*` and per-capability catalog/inspection tests prove deterministic derivation, filtering, and read-only projection from package contracts.
