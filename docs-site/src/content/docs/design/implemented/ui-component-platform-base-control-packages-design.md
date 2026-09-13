---
title: UI Component Platform Base Control Packages Design
status: implemented
owner: ui_controls
layer: domain
canonical: true
last_reviewed: 2026-09-13
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../reports/closeouts/pt-ui-component-platform-011-base-control-packages-closeout.md
  - ./ui-component-platform-generic-interaction-design.md
  - ../superseded/ui-component-platform-ownership-realignment-design.md
  - ./ui-component-platform-render-surface-output-design.md
---

# UI Component Platform Base Control Packages Design

## Status

Implemented. Phase 11 completed through merged PR #37; current source/tests remain the behavior authority. This document is the current owner-boundary reference for the Runenwerk-local base-control package proof.

## Implemented inventory

The base-control package contains reusable declarations for:

```text
Label
Button
InspectorField
ColorPicker
ActionPrompt
ListView
TreeView
TableView
```

Current implementation is contribution/lowering based rather than one monolithic descriptor fixture:

```text
BaseControlsPlugin
UiControls
ControlContribution
ControlDef builder
control presets
field groups
theme groups
ControlCompiler
ControlCatalog
ControlInspection
```

Each control contributes stable identity, schemas, kernel declarations, story/evidence requirements, layout/render/input/state/theme/accessibility facts, catalog metadata, and inspection projection through the current `ui_controls` package model.

## Ownership

`ui_controls` owns base-control package declarations, per-control lowering, validation, and catalog/inspection projection. Generic vocabulary remains with its owning UI crates where already separated; runtime interaction remains in `ui_runtime`; renderer execution remains in engine render; product data and mutation remain with app/editor/game owners.

## Implemented boundaries

- Base-control descriptors are package/source facts, not runtime widget instances.
- Runtime interaction is supplied by the later implemented generic-interaction proof, not by Phase 11 package declaration alone.
- Overlay/layering, editable text, generic text, layout, render evidence, and Surface2D are additive implemented contracts with their own owner boundaries.
- Runtime mount eligibility remains evidence-gated rather than inferred from package presence.
- No shared plugin/meta-framework extraction follows merely from the local contribution shape.

## Evidence

The current package descriptor, base-control contribution/compiler/lowering code, catalog/inspection projection, and focused `ui_controls` tests are implementation truth. The retained Phase 11 closeout remains historical delivery evidence.
