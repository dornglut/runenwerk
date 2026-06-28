---
title: PT UI Component Platform 011 Base Control Packages Closeout
description: Historical closeout evidence for Phase 11 base control package hardening.
status: active
owner: workspace
layer: reports
canonical: true
last_reviewed: 2026-06-28
related_docs:
  - ../../workspace/planning/completed-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
  - ../../design/active/ui-component-platform-base-control-packages-design.md
---

# PT UI Component Platform 011 Base Control Packages Closeout

ID: `PT-UI-COMPONENT-PLATFORM-011`

Title: UI Component Platform Base Control Packages

Completed on: 2026-06-28 through merged PR #37 and user validation report

Owner: `ui_controls`

## Scope promised

Phase 11 had to harden the base controls as package-quality reusable inventory before full interaction work starts.

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

Phase 11 had to keep the work UI-local, preserve owner-crate vocabulary boundaries, keep runtime interaction out of scope, and avoid shared plugin-framework extraction.

## Scope delivered

PR #37 merged the Phase 11 base-control implementation into `main`.

Delivered shape:

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

Each control declares intent through its own module-level `control_contribution()` function. Central compiler/lowering code produces descriptors, layout summaries, render summaries, input summaries, state summaries, theme summaries, accessibility summaries, catalog output, and inspection output.

## Files changed

Primary implementation evidence lives under:

```text
domain/ui/ui_controls/src/base_control/
domain/ui/ui_controls/src/lib.rs
domain/ui/ui_controls/src/action_prompt/
domain/ui/ui_controls/src/button/
domain/ui/ui_controls/src/color_picker/
domain/ui/ui_controls/src/inspector_field/
domain/ui/ui_controls/src/label/
domain/ui/ui_controls/src/list_view/
domain/ui/ui_controls/src/table_view/
domain/ui/ui_controls/src/tree_view/
domain/ui/ui_controls/tests/base_control_contract.rs
domain/ui/ui_controls/tests/control_catalog_contract.rs
domain/ui/ui_controls/tests/control_layout_contract.rs
domain/ui/ui_controls/tests/control_render_contract.rs
```

Primary planning evidence is recorded in:

```text
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/completed-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
docs-site/src/content/docs/workspace/planning/decision-register.md
```

## Validation run

PR #37 reported the following commands green locally:

```text
cargo fmt --all --check
cargo check -p ui_controls
cargo test -p ui_controls control_package
cargo test -p ui_controls control_catalog
cargo test -p ui_controls control_layout
cargo test -p ui_controls control_render
cargo test -p ui_controls base_control
cargo test -p ui_controls
python3 tools/docs/validate_docs.py
git diff --check
```

## Validation unavailable

This closeout update was prepared through the GitHub connector, so local command execution was unavailable for the documentation patch itself. The expected docs validation for this patch remains:

```text
python3 tools/docs/validate_docs.py
git diff --check
```

## Known gaps

- Full runtime interaction remains Phase 12.
- Hover, pressed, selected, focus, pointer, and keyboard runtime state machines remain out of Phase 11.
- Overlay, popup, dropdown, tooltip, and layering behavior remain Phase 13.
- Text editing remains later.
- App/editor/game-specific command behavior and product state changes remain outside reusable base controls.
- No shared plugin framework extraction is authorized.
- No `foundation/meta` is authorized.

## Drift found

Planning records still described Phase 11 as review or pending merge after PR #37 merged. This closeout updates planning state from Phase 11 review to Phase 11 completed and opens Phase 12 as planning only.

No product-code drift was fixed in this closeout patch.

## Follow-up

Proceed to `PT-UI-COMPONENT-PLATFORM-012-PLANNING` Generic Interaction design intake.

Do not start implementation until Phase 12 records exact owner files, validation gate, evidence expectation, and stop conditions.

## Evidence links

- PR #37: `https://github.com/Crystonix/Runenwerk/pull/37`
- Completed work: `../../workspace/planning/completed-work.md`
- Roadmap: `../../workspace/planning/roadmap.md`
- Production track: `../../workspace/planning/production-tracks.md`
- Phase 11 design: `../../design/active/ui-component-platform-base-control-packages-design.md`
