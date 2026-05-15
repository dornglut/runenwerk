---
title: UI Model Multiple Execution Strategies Design
description: Deferred design boundary for one shared UI model with possible compiled-reactive and ECS-driven execution strategies.
status: deferred
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-05-04
related:
  - ../active/editor-ui-workspace-tool-surface-architecture.md
  - ../active/editor-ui-runtime-v2-and-interaction-formation-design.md
  - ../../adr/accepted/0009-ui-interaction-formation-v2.md
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
---

# UI Model Multiple Execution Strategies Design

## Status

Deferred design. Do not implement a Svelte-like compiled reactive path or an ECS-driven UI path from this document until an active design or accepted ADR upgrades the decision.

## Purpose

Record the unresolved UI execution-strategy idea without making it current architecture.

Current code truth is narrower:

- `domain/ui/ui_tree` owns retained UI tree contracts.
- `domain/ui/ui_runtime` owns retained UI runtime orchestration, input routing, layout, and frame output.
- `domain/ui/ui_widgets` owns reusable widget constructors.
- `domain/ui/ui_render_data` owns renderer-facing UI frame and primitive contracts.
- `domain/ui/ui_surface` owns surface definition, mount, observation, session, presentation, intent, capability, and ratification contracts.

This document preserves a possible future direction:

```text
One shared UI domain model.
Multiple explicit execution strategies.
Common render-data/backend boundary.
```

## Non-Decision

Runenwerk has not accepted multiple UI execution strategies as implemented architecture.

Do not infer from this document that the repository should immediately add:

- a Svelte-inspired compiled reactive UI runtime;
- an ECS-driven UI runtime;
- automatic conversion between retained tree UI and ECS UI;
- reactive components that directly run ECS logic;
- ECS systems that depend on reactive internals.

## Current Accepted Working Model

The current editor/UI implementation should continue to grow through the
retained UI substrate, Interaction V2 formation, and tool-surface architecture.
Retained UI remains the production execution target:

```text
Authored UI / editor definitions
  -> validation / normalization
  -> formed interaction contracts
  -> formed retained UI product
  -> ui_runtime enforcement
  -> render/product-surface output
```

This path is already in production editor flows and is guard-tested.

## Deferred Strategic Idea

The future model may distinguish:

- shared UI vocabulary: state, style, layout concepts, events, focus, render descriptors, surface identity;
- execution strategy: how a surface updates and schedules UI state;
- render backend boundary: how resolved UI output becomes renderer-facing primitives or embedded surface products.

Possible execution strategies:

1. Compiled reactive strategy
   - analyzes declarative UI definitions;
   - emits direct update logic;
   - avoids runtime virtual-DOM style reconciliation;
   - may fit stable menus, HUDs, ordinary panels, and authored UI definitions.
2. ECS-driven UI strategy
   - represents UI nodes or world-attached UI as ECS entities/components;
   - updates through scheduled systems;
   - may fit world-bound labels, entity markers, selection outlines, simulation-linked UI, or debug overlays tied to runtime ECS state.
3. Retained tree strategy
   - keeps the current `ui_tree`/`ui_runtime` model;
   - remains the current implementation baseline for editor shell and tool panels.

## Boundary Rules If Reactivated

If this design becomes active, the hard rules should be:

- one surface chooses one execution strategy at a time;
- strategies meet at shared input/focus/layout/render-data contracts, not through hidden deep conversion;
- UI semantics do not move into renderer backends;
- editor workspace semantics do not become required for ordinary game HUD/menu UI;
- ECS-driven UI must not become a backdoor for editor/domain mutation outside command and ratification boundaries;
- compiled reactive UI must not directly own engine/runtime authority.
- compiled-reactive UI and ECS-driven UI must consume normalized UI definitions
  plus formed interaction contracts;
- no execution target may replace authored UI identity, source maps, command
  ratification, or the rule that renderer output is derived product data.

## Open Questions

- What exactly belongs in the shared UI domain model beyond current `domain/ui/*` crates?
- Is layout shared as data, shared as algorithms, or strategy-specific above common primitives?
- Which surfaces, if any, need compiled reactive execution before retained UI is exhausted?
- Which world-bound UI cases justify ECS-driven execution rather than ordinary retained UI plus runtime observation bridges?
- Should this become an ADR once a concrete implementation target exists?

## Reactivation Criteria

Move this document to `design/active/` only when there is a concrete implementation driver, such as:

- a game HUD or menu feature that retained UI cannot handle cleanly;
- world-bound UI that requires ECS scheduling and identity;
- authored UI definitions that need compile-time or formation-time reactive analysis;
- measured performance or ergonomics evidence that retained tree execution is insufficient.

Until then, continue improving the retained UI substrate, tool-surface framework, and editor workspace/provider architecture.
