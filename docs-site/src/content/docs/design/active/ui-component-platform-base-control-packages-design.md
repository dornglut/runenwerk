---
title: UI Component Platform Base Control Packages Design
status: active
owner: ui_controls
layer: domain
canonical: true
last_reviewed: 2026-06-28
related_docs:
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/completed-work.md
  - ../../reports/closeouts/pt-ui-component-platform-011-base-control-packages-closeout.md
  - ./ui-component-platform-generic-interaction-design.md
  - ./ui-component-platform-ownership-realignment-design.md
  - ./ui-component-platform-render-surface-output-design.md
---

# UI Component Platform Base Control Packages Design

## Status

This is the completed Phase 11 design for `PT-UI-COMPONENT-PLATFORM-011`.

Phase 11 completed through merged PR #37 on 2026-06-28. The design remains active as the owner-boundary reference for the base-control package proof.

## Purpose

Make the base controls credible reusable package inventory before full interaction work starts.

Target controls:

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

## Owner split

`ui_controls` owns:

- base control package descriptors;
- per-control package metadata;
- per-control story, layout, render, input, theme, state, and accessibility requirement summaries;
- read-only catalog and inspection summaries for base controls.

Owner crates still own their generic vocabulary:

- `ui_layout` owns layout roles, container facts, constraints, scroll facts, content facts, identity facts, budgets, and virtualization vocabulary;
- `ui_render_data` owns renderer-neutral frame, primitive, and output evidence vocabulary;
- `ui_input` owns input, focus, device, keyboard, pointer, stylus, and routing vocabulary;
- `ui_theme` owns theme tokens and visual vocabulary;
- `ui_runtime` owns retained runtime orchestration and emitted frame output;
- engine render owns backend submission and execution proof.

## Non-goals

Phase 11 does not implement full runtime interaction behavior.

Phase 11 does not implement popup, overlay, or layering behavior.

Phase 11 does not implement text editing.

Phase 11 does not make controls runtime-mountable without accepted proof gates.

Phase 11 does not move owner-crate vocabulary into `ui_controls`.

Phase 11 does not authorize shared plugin framework extraction or `foundation/meta`.

## Required result

Each base control should have a package-quality declaration that is inspectable without reading implementation code.

For each target control, Phase 11 should verify or add:

- stable control kind identity;
- display name, description, category, and tags;
- property, state, and event payload schemas;
- kernel declarations;
- story proof requirements;
- layout requirements that reference `ui_layout` vocabulary;
- render requirements that reference `ui_render_data` vocabulary;
- input/state/theme/accessibility declarations already available from earlier phases;
- catalog summaries and inspection facts;
- explicit non-mount eligibility until later proof gates authorize mount.

## Delivered shape

Phase 11 delivered the base controls as a UI-local contribution, preset, and lowering proof:

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

Each target control declares intent through its own `control_contribution()` module. Central lowering/compiler code produces package descriptors plus layout, render, input, state, theme, accessibility, catalog, and inspection outputs.

## Gallery readiness

Phase 11 makes Gallery able to list and inspect the base controls as reusable inventory.

The minimum Gallery-facing result is descriptor/catalog visibility, proof status, and static preview/evidence metadata where already supported by owner contracts.

Full pointer/keyboard interaction in Gallery is Phase 12.

## Acceptance criteria

- Base control package validation remains green.
- Every target control has complete descriptor metadata.
- Every target control exposes story, layout, render, accessibility/focus, input, state, and theme summaries where applicable.
- Catalog inspection can present a coherent per-control summary.
- Runtime mount eligibility remains blocked unless an explicit accepted proof gate says otherwise.
- No generic layout, render, input, theme, or runtime vocabulary is added to `ui_controls` when an owner crate already exists.

## Validation envelope

Expected validation:

```text
cargo fmt --all --check
cargo check -p ui_controls
cargo test -p ui_controls control_package
cargo test -p ui_controls control_catalog
cargo test -p ui_controls control_layout
cargo test -p ui_controls control_render
cargo test -p ui_controls base_control
git diff --check
```

Additional tests may be added as the implementation touches more focused contracts.

PR #37 also reported `cargo test -p ui_controls` and `python3 tools/docs/validate_docs.py` green locally.

## Stop conditions

Stop and redesign if Phase 11 requires any of the following:

- backend renderer behavior in `ui_controls`;
- runtime interaction behavior in the base control package;
- text editing behavior inside generic base package hardening;
- first-class mount eligibility without story/render/layout/accessibility evidence;
- duplicated generic vocabulary that belongs to an owner crate;
- shared plugin framework extraction;
- `foundation/meta`.

## Next step

Phase 11 is closed. Proceed to `PT-UI-COMPONENT-PLATFORM-012-PLANNING` Generic Interaction design intake without reopening Phase 11 base controls, starting overlay/layering work, or extracting shared plugin infrastructure.
