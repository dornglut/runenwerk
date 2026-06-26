---
title: UI Component Platform Theme State And Style Design
description: Phase 7 design for reusable control theme tokens, visual state declarations, and style contracts without renderer or product ownership.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-06-26
related_designs:
  - ./runenwerk-ui-platform-capability-roadmap.md
  - ./runenwerk-ui-story-driven-golden-workflow-design.md
  - ./ui-component-platform-state-binding-host-intent-design.md
  - ./ui-component-platform-input-gesture-device-design.md
  - ./ui-component-platform-catalog-discovery-inspection-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
---

# UI Component Platform Theme State And Style Design

## Status

This is the Phase 7 planning and acceptance design for `PT-UI-COMPONENT-PLATFORM-007`.

It follows Phase 1 `ControlPackage` / `ControlKernel` contracts, Phase 2 authoring ergonomics, Phase 3 story-proof envelopes, Phase 4 catalog/discovery/inspection, Phase 5 input/gesture/device declarations, and Phase 6 state binding / host intent declarations. It defines reusable control theme token requirements, visual state declarations, style role declarations, fallback diagnostics, and catalog/inspection summaries. It does not authorize renderer-owned styling semantics, product theme systems, animation tooling, runtime widget behavior, runtime mount eligibility, text editing implementation, canvas behavior, Gallery previews, Designer UX, Workbench behavior, renderer behavior, or ECS behavior.

## Existing Authority

`ui_controls` owns reusable control semantics and may declare theme token requirements, visual state names, style roles, fallback expectations, and missing-token diagnostics.

Renderer crates own backend-neutral render output and materialization. They consume style facts but do not define reusable control semantics.

Apps, editor, game hosts, and product surfaces own concrete theme packs, brand decisions, product presentation policy, user customization, persistence, and final runtime application.

Gallery, Workbench, UI Designer, docs, and agents consume theme/state/style declarations through catalog and inspection facts. They do not own reusable control semantics.

## Problem

Reusable controls can now declare packages, stories, catalog facts, input modes, state buckets, bindings, and host intent proposals. They still need a shared way to describe which visual states and theme/style tokens are required to present a control consistently.

Without a shared declaration, later controls could hardcode colors, spacing, typography, radius, elevation, or visual-state behavior. That would duplicate style vocabulary and blur ownership between reusable control semantics, renderer output, and product theme policy.

## Decision

Add a reusable theme/state/style declaration layer for the UI Component Platform.

The component layer owns declarations. It may describe token requirements, visual states, style roles, fallback behavior, missing-token diagnostics, and inspection summaries. It must not own concrete product themes, renderer materialization, animation tooling, product styling policy, or runtime application.

Correct ownership split:

```text
ui_controls
  owns required token names, token roles, visual state declarations,
  fallback requirements, style diagnostics, and inspection summaries.

renderer / output layers
  own backend-neutral render output and materialization details.

apps / editor / game hosts
  own concrete theme packs, brand choices, user customization, persistence,
  runtime application policy, and product presentation decisions.

Gallery / Workbench / UI Designer / docs / agents
  consume declarations; they do not own reusable control semantics.
```

## Proposed Contract Shape

The first implementation should prefer one focused module:

```text
domain/ui/ui_controls/src/theme.rs
```

Split later only when stable responsibilities require it.

Candidate public concepts:

```text
ControlThemeTokenKind
ControlThemeTokenRequirement
ControlThemeTokenRole
ControlVisualState
ControlVisualStateRequirement
ControlStyleRole
ControlStyleRequirement
ControlStyleFallback
ControlStyleDiagnostic
ControlThemeDescriptor
ControlThemeCapabilitySummary
ControlThemeInspectionFact
```

The final names may differ after inspecting nearby conventions, but the responsibilities should remain:

- declare semantic color, spacing, typography, radius, border, opacity, and elevation token requirements;
- declare normal, hover, pressed, focused, selected, disabled, error, warning, info, active, loading, and read-only visual states;
- declare style roles such as container, label, icon, value, background, foreground, border, accent, focus-ring, and overlay;
- declare fallback behavior without selecting concrete product values;
- expose missing-token and fallback diagnostics as reusable facts;
- expose summaries that catalog/inspection can consume;
- avoid renderer materialization, product theme application, animation execution, and runtime style mutation.

## Minimum Phase 7 Scope

The first implementation pass should prove the contract with a small declaration model:

```text
ControlThemeDescriptor
ControlThemeTokenRequirement
ControlVisualStateRequirement
ControlStyleRequirement
ControlStyleFallback
ControlThemeCapabilitySummary
```

Minimum token kinds:

```text
color
spacing
typography
radius
border
opacity
elevation
```

Minimum visual states:

```text
normal
hover
pressed
focused
selected
disabled
error
warning
info
active
loading
read-only
```

Minimum style roles:

```text
container
label
icon
value
background
foreground
border
accent
focus-ring
overlay
```

Minimum fallback facts:

```text
fallback-token
missing-token-diagnostic
expected-failure
```

## Non-Goals

Do not implement:

- renderer-owned styling semantics;
- product theme systems;
- concrete brand/theme values;
- user customization persistence;
- runtime style application;
- animation tooling;
- transition execution;
- runtime widget behavior;
- runtime mount eligibility;
- text editing implementation;
- canvas behavior;
- Gallery previews;
- Designer UX;
- Workbench behavior;
- Surface2D;
- SpatialCanvas;
- NodeCanvas;
- PortGraphCanvas;
- ProgressionTreeView;
- TrackSurface;
- Timeline;
- renderer behavior;
- ECS behavior.

## Boundary Rules

- Theme tokens are requirements, not concrete product values.
- Visual states are reusable declaration names, not renderer state machines.
- Style roles describe semantic placement, not backend materials.
- Fallbacks are declared expectations, not product theme resolution logic.
- Missing-token diagnostics are reusable facts, not final UI messages.
- Catalog/inspection may expose theme/state/style declarations as read-only data.
- Story proof may reference theme requirements, but Phase 7 does not run stories.
- Runtime mount eligibility remains future-gated.

## Acceptance Criteria

Phase 7 is implementation-complete only when:

- reusable theme/state/style declarations exist in `ui_controls`;
- declarations distinguish token kinds, visual states, style roles, fallbacks, and diagnostics;
- declarations can be summarized for catalog/inspection without renderer or product ownership;
- focused tests prove token requirements, visual state declarations, style roles, fallback diagnostics, inspection summaries, and no runtime style behavior;
- no renderer-owned styling semantics, product theme system, runtime style application, animation tooling, runtime widget behavior, runtime mount, canvas behavior, Gallery, Designer, Workbench, renderer, or ECS behavior is implemented.

## Candidate Implementation Scope

The first implementation pass may touch:

```text
domain/ui/ui_controls/src/theme.rs
domain/ui/ui_controls/src/lib.rs
domain/ui/ui_controls/src/package.rs
domain/ui/ui_controls/src/catalog/inspection.rs
domain/ui/ui_controls/tests/control_theme_contract.rs
domain/ui/ui_controls/tests/control_theme_catalog_contract.rs
```

Use catalog inspection only to expose read-only theme/state/style summaries. Do not add renderer, app, editor, game, Gallery, Designer, or Workbench code in Phase 7.

## Test Plan

Required focused tests for the future implementation pass:

```text
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

- theme descriptor records color, spacing, typography, radius, border, opacity, and elevation token requirements;
- visual state declarations distinguish normal, hover, pressed, focused, selected, disabled, error, warning, info, active, loading, and read-only;
- style roles distinguish container, label, icon, value, background, foreground, border, accent, focus-ring, and overlay;
- fallback diagnostics expose fallback-token, missing-token-diagnostic, and expected-failure facts;
- catalog/inspection summaries expose theme/state/style declarations read-only;
- no declaration makes a control runtime-mount eligible.

## Phase 7 Implementation Gate

Before writing Rust code, confirm:

- this design is accepted;
- Phase 6 remains green on the branch base;
- existing `ui_controls` package, catalog, input, state, story-proof, authoring, and validation contracts are still current;
- planning records name Phase 7 implementation as active;
- no new stop condition has been triggered.

## Handoff

Start implementation only after this design and planning state are accepted. The first implementation pass should add the smallest theme/state/style declaration contract and focused tests. Do not implement renderer materialization, product theme systems, runtime style application, animation tooling, canvas behavior, Gallery previews, Designer UX, Workbench behavior, renderer behavior, ECS behavior, or runtime mount eligibility in Phase 7.
