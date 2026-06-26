---
title: UI Component Platform Accessibility Focus And Inspection Design
description: Phase 8 design for reusable accessibility roles, focus declarations, keyboard activation facts, and semantic inspection summaries without platform bridge ownership.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-06-26
related_designs:
  - ./runenwerk-ui-platform-capability-roadmap.md
  - ./runenwerk-ui-story-driven-golden-workflow-design.md
  - ./ui-component-platform-theme-state-style-design.md
  - ./ui-component-platform-state-binding-host-intent-design.md
  - ./ui-component-platform-input-gesture-device-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
---

# UI Component Platform Accessibility Focus And Inspection Design

## Status

This is the Phase 8 planning and acceptance design for `PT-UI-COMPONENT-PLATFORM-008`.

It follows Phase 1 `ControlPackage` / `ControlKernel` contracts, Phase 2 authoring ergonomics, Phase 3 story-proof envelopes, Phase 4 catalog/discovery/inspection, Phase 5 input/gesture/device declarations, Phase 6 state binding / host intent declarations, and Phase 7 theme/state/style declarations. It defines reusable accessibility roles, labels, descriptions, semantic hints, focus declarations, keyboard activation facts, value/range metadata, missing-requirement diagnostics, and catalog/inspection summaries. It does not authorize platform-native accessibility bridge implementation, OS integration, product copy workflows, runtime widget behavior, runtime mount eligibility, text editing implementation, canvas behavior, Gallery previews, Designer UX, Workbench behavior, renderer behavior, or ECS behavior.

## Existing Authority

`ui_controls` owns reusable control semantics and may declare accessibility roles, labels, descriptions, semantic hints, focus requirements, keyboard activation facts, state semantics, value/range facts, expected missing-requirement diagnostics, and inspection summaries.

Platform bridge, OS accessibility, renderer/output, apps, editor, game hosts, and product surfaces own concrete native accessibility integration, runtime focus routing, final copy, localization, user preference application, persistence, and final runtime behavior.

Gallery, Workbench, UI Designer, docs, and agents consume accessibility/focus/inspection declarations through catalog and inspection facts. They do not own reusable control semantics.

## Problem

Reusable controls can now declare packages, stories, catalog facts, input modes, state/host intent facts, and theme/style facts. They still need a shared way to describe accessibility and focus semantics before layout, rendering, base control packages, and interaction phases consume those facts.

Without a shared declaration, later controls could duplicate role, label, focus, keyboard activation, semantic state, and inspection vocabulary. That would make accessibility inconsistent and move product/runtime behavior into reusable control contracts.

## Decision

Add a reusable accessibility, focus, and semantic inspection declaration layer for the UI Component Platform.

The component layer owns declarations. It may describe semantic roles, label/description requirements, focusability, focus order, keyboard activation, semantic states, value/range metadata, missing-requirement diagnostics, and inspection summaries. It must not implement native accessibility bridges, product copy workflows, localization, runtime focus routing, or OS/platform behavior.

Correct ownership split:

```text
ui_controls
  owns accessibility declaration names, role requirements, label requirements,
  focus facts, keyboard activation facts, semantic state facts,
  value/range metadata shape, diagnostics, and inspection summaries.

platform bridge / OS integration
  owns native accessibility tree projection and platform API behavior.

apps / editor / game hosts
  own final user-facing copy, localization, user preferences, runtime focus policy,
  command execution, persistence, and product-specific behavior.

Gallery / Workbench / UI Designer / docs / agents
  consume declarations; they do not own reusable control semantics.
```

## Proposed Contract Shape

The first implementation should prefer one focused module:

```text
domain/ui/ui_controls/src/accessibility.rs
```

Split later only when stable responsibilities require it.

Candidate public concepts:

```text
ControlAccessibilityRole
ControlAccessibilityLabelRequirement
ControlAccessibilityDescriptionRequirement
ControlSemanticHint
ControlFocusRequirement
ControlFocusOrder
ControlKeyboardActivation
ControlSemanticState
ControlValueRangeMetadata
ControlAccessibilityDiagnostic
ControlAccessibilityDescriptor
ControlAccessibilityCapabilitySummary
ControlAccessibilityInspectionFact
```

The final names may differ after inspecting nearby conventions, but the responsibilities should remain:

- declare roles such as button, label, checkbox, slider, text, list, list item, tree, tree item, table, row, cell, menu, menu item, dialog, panel, canvas, and custom;
- declare label, description, and semantic hint requirements;
- declare focusable, focus order, focus trap, and focus return facts;
- declare keyboard activation requirements such as activate, cancel, commit, expand, collapse, increment, decrement, navigate next, and navigate previous;
- declare semantic states such as enabled, disabled, selected, pressed, expanded, collapsed, checked, unchecked, mixed, busy, invalid, readonly, and required;
- declare value/range metadata shape without owning product data;
- expose missing role/label/description/focus diagnostics as expected failure facts;
- expose summaries that catalog/inspection can consume;
- avoid platform-native bridge implementation, OS integration, product copy workflow, runtime focus routing, and runtime mutation.

## Minimum Phase 8 Scope

The first implementation pass should prove the contract with a small declaration model:

```text
ControlAccessibilityDescriptor
ControlAccessibilityRole
ControlAccessibilityLabelRequirement
ControlFocusRequirement
ControlKeyboardActivation
ControlSemanticState
ControlValueRangeMetadata
ControlAccessibilityDiagnostic
ControlAccessibilityCapabilitySummary
```

Minimum roles:

```text
button
label
checkbox
slider
text
list
list-item
tree
tree-item
table
row
cell
menu
menu-item
dialog
panel
canvas
custom
```

Minimum focus facts:

```text
focusable
focus-order
focus-trap
focus-return
```

Minimum keyboard activation facts:

```text
activate
cancel
commit
expand
collapse
increment
decrement
navigate-next
navigate-previous
```

Minimum semantic states:

```text
enabled
disabled
selected
pressed
expanded
collapsed
checked
unchecked
mixed
busy
invalid
readonly
required
```

Minimum diagnostics:

```text
missing-role
missing-label
missing-description
missing-focus-order
expected-failure
```

## Non-Goals

Do not implement:

- platform-native accessibility bridge;
- OS accessibility API integration;
- native accessibility tree projection;
- product copy workflow;
- localization;
- user preference persistence;
- runtime focus routing;
- runtime widget behavior;
- runtime mount eligibility;
- text editing implementation;
- canvas behavior;
- Gallery previews;
- Designer UX;
- Workbench behavior;
- renderer behavior;
- ECS behavior.

## Boundary Rules

- Accessibility roles are declarations, not native accessibility nodes.
- Labels and descriptions are requirements, not final product copy.
- Focus facts describe expected control behavior, not runtime routing.
- Keyboard activation facts describe semantic actions, not input event execution.
- Semantic states are reusable facts, not domain state truth.
- Value/range metadata describes shape, not live product data.
- Catalog/inspection may expose accessibility/focus/inspection declarations as read-only data.
- Story proof may reference accessibility requirements, but Phase 8 does not run stories.
- Runtime mount eligibility remains future-gated.

## Acceptance Criteria

Phase 8 is implementation-complete only when:

- reusable accessibility/focus/inspection declarations exist in `ui_controls`;
- declarations distinguish roles, labels, descriptions, semantic hints, focus facts, keyboard activation facts, semantic states, value/range metadata, and diagnostics;
- declarations can be summarized for catalog/inspection without platform bridge, OS, or product ownership;
- focused tests prove role requirements, label/description requirements, focus facts, keyboard activation facts, semantic states, diagnostics, inspection summaries, and no runtime focus behavior;
- no platform accessibility bridge, OS integration, native tree projection, product copy workflow, localization, runtime focus routing, runtime widget behavior, runtime mount, canvas behavior, Gallery, Designer, Workbench, renderer, or ECS behavior is implemented.

## Candidate Implementation Scope

The first implementation pass may touch:

```text
domain/ui/ui_controls/src/accessibility.rs
domain/ui/ui_controls/src/lib.rs
domain/ui/ui_controls/src/package.rs
domain/ui/ui_controls/src/catalog/inspection.rs
domain/ui/ui_controls/tests/control_accessibility_contract.rs
domain/ui/ui_controls/tests/control_accessibility_catalog_contract.rs
```

Use catalog inspection only to expose read-only accessibility/focus/inspection summaries. Do not add platform bridge, app, editor, game, Gallery, Designer, or Workbench code in Phase 8.

## Test Plan

Required focused tests for the future implementation pass:

```text
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
```

Required static checks:

```text
cargo fmt --all --check
cargo check -p ui_controls
git diff --check
```

Recommended test cases:

- accessibility descriptor records roles, label requirements, description requirements, and semantic hints;
- focus declarations distinguish focusable, focus-order, focus-trap, and focus-return facts;
- keyboard activation declarations distinguish activate, cancel, commit, expand, collapse, increment, decrement, navigate-next, and navigate-previous;
- semantic states distinguish enabled, disabled, selected, pressed, expanded, collapsed, checked, unchecked, mixed, busy, invalid, readonly, and required;
- diagnostics expose missing-role, missing-label, missing-description, missing-focus-order, and expected-failure facts;
- catalog/inspection summaries expose accessibility/focus/inspection declarations read-only;
- no declaration makes a control runtime-mount eligible.

## Phase 8 Implementation Gate

Before writing Rust code, confirm:

- this design is accepted;
- Phase 7 remains green on the branch base;
- existing `ui_controls` package, catalog, input, state, theme, story-proof, authoring, and validation contracts are still current;
- planning records name Phase 8 implementation as active;
- no new stop condition has been triggered.

## Handoff

Start implementation only after this design and planning state are accepted. The first implementation pass should add the smallest accessibility/focus/inspection declaration contract and focused tests. Do not implement platform-native accessibility bridges, OS integration, native tree projection, runtime focus routing, product copy workflow, localization, canvas behavior, Gallery previews, Designer UX, Workbench behavior, renderer behavior, ECS behavior, or runtime mount eligibility in Phase 8.
