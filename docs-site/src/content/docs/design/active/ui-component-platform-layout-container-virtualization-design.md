---
title: UI Component Platform Layout Container And Virtualization Design
description: Phase 9 owner-first design for layout, container, scroll, overflow, identity, and virtualization vocabulary.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-06-26
related_designs:
  - ./ui-component-platform-ownership-realignment-design.md
  - ./runenwerk-ui-platform-capability-roadmap.md
  - ./ui-component-platform-accessibility-focus-inspection-design.md
  - ./ui-component-platform-theme-state-style-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
---

# UI Component Platform Layout Container And Virtualization Design

## Status

This design replaces the older activation-era Phase 9 wording.

Phase 9 must be owner-first. Generic layout vocabulary belongs in `ui_layout`. `ui_controls` may only expose per-control layout requirements and catalog inspection summaries that reference `ui_layout` contracts.

## Decision

Split Phase 9 into two implementation slices after the ownership realignment pass:

```text
PT-UI-COMPONENT-PLATFORM-009B Layout Foundation
  Owner: ui_layout
  Adds generic layout/container/scroll/virtualization vocabulary.

PT-UI-COMPONENT-PLATFORM-009C Control Layout Bridge
  Owner: ui_controls
  Adds per-control layout requirements and catalog inspection summaries that reference ui_layout types.
```

## Ownership

```text
ui_layout
  owns layout roles, container kinds, size constraints, scroll facts,
  overflow/content states, item identity requirements,
  selection identity requirements, large-content budgets,
  virtualization readiness, layout diagnostics, and reusable layout facts.

ui_controls
  owns ControlLayoutDescriptor, ControlLayoutRequirement,
  ControlLayoutCapabilitySummary, and ControlLayoutInspectionFact as
  per-control wrappers over ui_layout vocabulary.

renderer / layout runtime / output layers
  own measured geometry, layout execution, clipping, primitive output,
  render data, and backend materialization.

apps / editor / game hosts
  own product data, persistence, runtime scroll state,
  table settings, domain selection truth, and product behavior.
```

## Generic vocabulary owned by ui_layout

Minimum layout roles:

```text
panel
row
column
stack
split
scroll
list
table
tree
virtual-list
virtual-table
```

Minimum container kinds:

```text
panel
viewport
section
group
collection
split-pane
scroll-region
```

Minimum size constraints:

```text
min-size
max-size
preferred-size
fill-width
fill-height
intrinsic-size
```

Minimum scroll facts:

```text
scrollable
scroll-owner
scroll-axis-x
scroll-axis-y
scroll-position-host-owned
```

Minimum content states:

```text
empty
loading
error
overflow
ready
```

Minimum virtualization facts:

```text
virtualization-ready
estimated-item-size
stable-item-identity
windowed-rendering
overscan-budget
```

Minimum diagnostics:

```text
missing-item-identity
missing-selection-identity
missing-scroll-owner
missing-large-content-budget
expected-failure
```

## Control bridge owned by ui_controls

`ui_controls` may add a bridge only after the `ui_layout` vocabulary exists.

Candidate bridge concepts:

```text
ControlLayoutDescriptor
ControlLayoutRequirement
ControlLayoutCapabilitySummary
ControlLayoutInspectionFact
```

The bridge records what a control requires. It must not define source-of-truth layout vocabulary.

## Non-goals

Do not implement:

```text
layout execution
measured geometry
primitive output
renderer behavior
product data ownership
app-specific table persistence
runtime scroll position ownership
runtime selection mutation
virtualization algorithm execution
sticky-header layout execution
runtime widget behavior
runtime mount eligibility
canvas behavior
Gallery previews
Designer UX
Workbench behavior
ECS behavior
```

## Acceptance criteria

Phase 9 is implementation-complete only after both owner-first slices exist:

```text
009B:
  ui_layout exposes generic layout/container/scroll/virtualization vocabulary and tests.

009C:
  ui_controls exposes per-control layout requirements that reference ui_layout types.
  catalog/inspection exposes read-only summaries.
  focused tests prove the bridge without runtime layout behavior.
```

## Validation gate

```text
cargo fmt --all --check
cargo check -p ui_layout
cargo check -p ui_controls
cargo test -p ui_layout layout
cargo test -p ui_controls control_layout
cargo test -p ui_controls control_accessibility
cargo test -p ui_controls control_theme
cargo test -p ui_controls control_state
cargo test -p ui_controls control_input
cargo test -p ui_controls control_catalog
cargo test -p ui_controls control_package
cargo test -p ui_controls control_registry
cargo test -p ui_controls control_story_proof
cargo test -p ui_controls control_authoring
cargo test -p ui_artifacts control_package
cargo test -p ui_program route
git diff --check
```

## Handoff

After this design is accepted, implement `009B Layout Foundation` first in `ui_layout`. Only after that should `009C Control Layout Bridge` add `ui_controls` declarations that reference the `ui_layout` vocabulary.
