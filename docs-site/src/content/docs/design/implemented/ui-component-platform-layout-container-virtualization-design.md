---
title: UI Component Platform Layout Container And Virtualization Design
description: Implemented owner-first layout vocabulary in ui_layout plus the Runenwerk control-facing layout bridge in ui_controls.
status: implemented
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-09-13
related_designs:
  - ../active/ui-component-platform-ownership-realignment-design.md
  - ./ui-component-platform-accessibility-focus-inspection-design.md
  - ./ui-component-platform-theme-state-style-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
---

# UI Component Platform Layout Container And Virtualization Design

## Status

Implemented in the owner-first shape established by the Phase 9 correction.

## Implemented owner split

`ui_layout` owns generic renderer-neutral layout vocabulary. Current contracts include layout roles, container kinds, size constraints, scroll requirements, content states, item and selection identity requirements, large-content budgets, virtualization requirements, and layout diagnostics.

`ui_controls` owns the per-control bridge through `ControlLayoutDescriptor` and derived capability/inspection projection. The bridge imports and references `ui_layout` types rather than redefining them.

Current implementation evidence includes:

```text
domain/ui/ui_layout/src/contracts.rs
domain/ui/ui_layout/tests/layout_contract.rs
domain/ui/ui_controls/src/layout.rs
domain/ui/ui_controls/src/base_control/lowering/layout.rs
domain/ui/ui_controls/tests/control_layout_contract.rs
domain/ui/ui_controls/tests/control_layout_catalog_contract.rs
```

## Implemented contract

The owner vocabulary covers panel/row/column/stack/split/scroll/list/table/tree and virtual collection roles; panel/viewport/section/group/collection/split-pane/scroll-region containers; min/max/preferred/fill/intrinsic size facts; scroll owner/axis facts; empty/loading/error/overflow/ready content states; stable item/selection identity; large-content budgets; and virtualization/overscan/windowing readiness.

`ControlLayoutDescriptor` records which of those owner-defined facts a control requires and projects read-only inspection evidence.

## Intentional landed differences

The proposal split work into `009B Layout Foundation` and `009C Control Layout Bridge`. Those labels were delivery decomposition, not runtime/public API names. The current implementation embodies both accepted responsibilities directly in `ui_layout` and `ui_controls`.

No separate compatibility vocabulary in `ui_controls` is required for the generic layout owner types.

## Boundary rules

- `ui_layout` contracts do not execute layout or own measured runtime geometry by virtue of this vocabulary.
- `ui_controls` does not become the generic layout source of truth.
- Runtime scroll position, product data, product selection, persistence, primitive output, and backend rendering retain separate owners.
- Virtualization requirements/budgets are semantic readiness facts, not an implicit virtualization algorithm.

## Evidence

Current owner-crate contracts and focused owner/bridge tests prove both halves of the intended Phase 9 boundary.
