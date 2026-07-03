---
title: UI Domain Split Pass Closeout
description: Behavior-preserving closeout report for the pre-Surface2D UI module split pass.
status: completed
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-07-03
related:
  - ../../../domain/ui/README.md
  - ../../../domain/ui/architecture.md
  - ../../../domain/ui/dependency-boundaries.md
  - ../../../domain/ui/crate-ownership.md
  - ../../../design/active/ui-component-platform-surface2d-design.md
  - ../../../workspace/planning/active-work.md
---

# UI Domain Split Pass Closeout

## Scope

This report records the behavior-preserving UI domain split pass completed
before Phase 16 Surface2D implementation.

The pass did not implement Surface2D, did not create a shared plugin framework,
did not create foundation/meta, did not extract crates, and did not change
renderer, editor, game, graph, timeline, product, diagnostics, proof fact,
report-row, or snapshot behavior.

## Split Slices

| Slice | Branch | Status | Primary change |
| --- | --- | --- | --- |
| Slice A | `ui/split-runtime-layout-engine` | Merged via PR #57 | Split runtime layout engine into owner-aligned modules. |
| Slice B | `ui/split-runtime-generic-interaction` | Merged via PR #57 | Split runtime generic interaction proof module. |
| Slice C | `ui/split-definition-formation` | Merged via PR #57 | Split retained UI definition formation. |
| Slice D | `ui/move-ui-test-support` | Merged via PR #57 | Moved noisy tests/support out of public API files. |
| Slice E | `docs/ui-domain-split-pass-report` | Merged via PR #57 | Recorded residual structural debt and non-goals. |
| Batch 2 | `ui/split-remaining-large-ui-files` | Merged via PR #57 | Split the remaining listed large UI files before stack merge. |

## Before And After Counts

| Area | Before | After |
| --- | ---: | ---: |
| `ui_runtime/src/layout/engine.rs` | 1906 | removed |
| `layout/engine/mod.rs` | n/a | 510 |
| `layout/engine/popup.rs` | n/a | 308 |
| `layout/engine/measure.rs` | n/a | 288 |
| `layout/engine/controls.rs` | n/a | 281 |
| `layout/engine/containers.rs` | n/a | 253 |
| `layout/engine/overlay.rs` | n/a | 140 |
| `layout/engine/scroll.rs` | n/a | 78 |
| `layout/engine/surface.rs` | n/a | 62 |
| `layout/engine/dispatch.rs` | n/a | 56 |
| `ui_runtime/src/input/generic_interaction.rs` | 1733 | removed |
| `input/generic_interaction/replay.rs` | n/a | 776 |
| `input/generic_interaction/visual.rs` | n/a | 276 |
| `input/generic_interaction/fixture.rs` | n/a | 230 |
| `input/generic_interaction/inspector.rs` | n/a | 190 |
| `input/generic_interaction/report.rs` | n/a | 170 |
| `input/generic_interaction/state_mapping.rs` | n/a | 93 |
| `input/generic_interaction/formatting.rs` | n/a | 51 |
| `input/generic_interaction/boundary.rs` | n/a | 27 |
| `input/generic_interaction/mod.rs` | n/a | 26 |
| `ui_definition/src/form.rs` | 1186 | removed |
| `ui_definition/src/form/controls.rs` | n/a | 421 |
| `ui_definition/src/form/mod.rs` | n/a | 333 |
| `ui_definition/src/form/collections.rs` | n/a | 191 |
| `ui_definition/src/form/dispatch.rs` | n/a | 169 |
| `ui_definition/src/form/containers.rs` | n/a | 102 |
| `ui_definition/src/form/resolve.rs` | n/a | 79 |
| `ui_definition/src/form/slots.rs` | n/a | 73 |
| `ui_definition/src/form/context.rs` | n/a | 58 |
| `ui_definition/src/form/scroll.rs` | n/a | 58 |
| `ui_definition/src/form/state.rs` | n/a | 51 |
| `ui_runtime/src/output/build_ui_frame.rs` | 1446 | 1276 |
| `ui_runtime/src/output/test_support.rs` | n/a | 174 |
| `ui_controls/src/lib.rs` | 221 | 58 |
| `ui_controls/tests/control_package_validation.rs` | n/a | 174 |
| `ui_definition/src/lib.rs` | 80 | 51 |
| `ui_definition/tests/checked_in_fixtures.rs` | n/a | 25 |

## Batch 2 Counts

| Area | Before | After |
| --- | ---: | ---: |
| `ui_runtime/src/input/pointer.rs` | 1449 | removed |
| `input/pointer/mod.rs` | n/a | 15 |
| `input/pointer/dispatch.rs` | n/a | 490 |
| `input/pointer/graph_canvas.rs` | n/a | 275 |
| `input/pointer/scroll.rs` | n/a | 228 |
| `input/pointer/scrollbar.rs` | n/a | 126 |
| `input/pointer/popup.rs` | n/a | 114 |
| `input/pointer/press.rs` | n/a | 106 |
| `input/pointer/helpers.rs` | n/a | 105 |
| `input/pointer/middle_pan.rs` | n/a | 68 |
| `input/pointer/numeric.rs` | n/a | 29 |
| `input/pointer/hover.rs` | n/a | 14 |
| `ui_runtime/src/runtime/ui_runtime.rs` | 3423 | removed |
| `runtime/ui_runtime/mod.rs` | n/a | 14 |
| `runtime/ui_runtime/entry.rs` | n/a | 373 |
| `runtime/ui_runtime/focus.rs` | n/a | 107 |
| `runtime/ui_runtime/scroll_metrics.rs` | n/a | 67 |
| `runtime/ui_runtime/graph_canvas.rs` | n/a | 37 |
| `runtime/ui_runtime/popup.rs` | n/a | 28 |
| `runtime/ui_runtime/helpers.rs` | n/a | 20 |
| `ui_theme/src/token/mod.rs` | 1357 | 18 |
| `ui_theme/src/token/resolve.rs` | n/a | 312 |
| `ui_theme/src/token/declaration.rs` | n/a | 290 |
| `ui_theme/src/token/diagnostics.rs` | n/a | 111 |
| `ui_theme/src/token/packet.rs` | n/a | 74 |
| `ui_theme/src/token/activation.rs` | n/a | 46 |
| `ui_theme/src/token/graph.rs` | n/a | 14 |
| `ui_definition/src/persistence_activation/mod.rs` | 1332 | 18 |
| `persistence_activation/request.rs` | n/a | 307 |
| `persistence_activation/tests.rs` | n/a | 272 |
| `persistence_activation/migration.rs` | n/a | 184 |
| `persistence_activation/document.rs` | n/a | 169 |
| `persistence_activation/validation.rs` | n/a | 163 |
| `persistence_activation/diagnostics.rs` | n/a | 155 |
| `persistence_activation/diff.rs` | n/a | 151 |
| `ui_runtime/src/output/build_ui_frame.rs` | 1276 | 51 |
| `output/traversal.rs` | n/a | 292 |
| `output/build_ui_frame/tests/layering.rs` | n/a | 265 |
| `output/build_ui_frame/tests/scrollbars.rs` | n/a | 246 |
| `output/build_ui_frame/tests/visual_states.rs` | n/a | 231 |
| `output/build_ui_frame/tests/snapshot.rs` | n/a | 177 |
| `output/interaction_visual.rs` | n/a | 18 |
| `output/layer.rs` | n/a | 6 |
| `ui_definition/src/visual_layout/apply.rs` | 1218 | removed |
| `visual_layout/apply/mod.rs` | n/a | 113 |
| `visual_layout/apply/tests.rs` | n/a | 342 |
| `visual_layout/apply/containers.rs` | n/a | 331 |
| `visual_layout/apply/context.rs` | n/a | 285 |
| `visual_layout/apply/controls.rs` | n/a | 72 |
| `visual_layout/apply/dispatch.rs` | n/a | 60 |
| `visual_layout/apply/diagnostics.rs` | n/a | 58 |
| `visual_layout/apply/collections.rs` | n/a | 40 |
| `ui_definition/src/preview_fixture/mod.rs` | 1166 | 32 |
| `preview_fixture/tests.rs` | n/a | 299 |
| `preview_fixture/validation/matrices.rs` | n/a | 186 |
| `preview_fixture/validation/fixtures.rs` | n/a | 152 |
| `preview_fixture/validation/diagnostics.rs` | n/a | 139 |
| `preview_fixture/validation/scenarios.rs` | n/a | 125 |
| `preview_fixture/validation/mod.rs` | n/a | 110 |
| `preview_fixture/builders.rs` | n/a | 73 |
| `preview_fixture/catalog.rs` | n/a | 53 |
| `preview_fixture/routes.rs` | n/a | 46 |
| `preview_fixture/surfaces.rs` | n/a | 42 |
| `preview_fixture/controls.rs` | n/a | 30 |

## Module Trees Created

```text
domain/ui/ui_runtime/src/layout/engine/
  mod.rs
  dispatch.rs
  measure.rs
  controls.rs
  containers.rs
  scroll.rs
  popup.rs
  overlay.rs
  surface.rs

domain/ui/ui_runtime/src/input/generic_interaction/
  mod.rs
  fixture.rs
  replay.rs
  report.rs
  boundary.rs
  visual.rs
  inspector.rs
  formatting.rs
  state_mapping.rs

domain/ui/ui_definition/src/form/
  mod.rs
  context.rs
  state.rs
  dispatch.rs
  containers.rs
  controls.rs
  collections.rs
  slots.rs
  scroll.rs
  resolve.rs
```

Batch 2 added these module trees:

```text
domain/ui/ui_runtime/src/input/pointer/
  mod.rs
  dispatch.rs
  hover.rs
  press.rs
  scroll.rs
  scrollbar.rs
  middle_pan.rs
  graph_canvas.rs
  popup.rs
  numeric.rs
  helpers.rs

domain/ui/ui_runtime/src/runtime/ui_runtime/
  mod.rs
  entry.rs
  focus.rs
  graph_canvas.rs
  helpers.rs
  popup.rs
  scroll_metrics.rs
  tests/
    mod.rs
    console_scroll_policy.rs
    controls.rs
    graph_canvas_keyboard.rs
    graph_canvas_pointer.rs
    keyboard_focus.rs
    middle_pan.rs
    popup.rs
    scroll_overflow.rs
    scroll_wheel.rs
    scrollbar.rs

domain/ui/ui_theme/src/token/
  mod.rs
  declaration.rs
  graph.rs
  resolve.rs
  packet.rs
  diagnostics.rs
  activation.rs
  tests/
    mod.rs
    activation.rs
    alias.rs
    diagnostics.rs
    packet.rs
    precedence.rs
    selector.rs

domain/ui/ui_definition/src/persistence_activation/
  mod.rs
  document.rs
  migration.rs
  diff.rs
  request.rs
  validation.rs
  diagnostics.rs
  tests.rs

domain/ui/ui_runtime/src/output/
  build_ui_frame.rs
  traversal.rs
  layer.rs
  interaction_visual.rs
  build_ui_frame/tests/
    mod.rs
    layering.rs
    scrollbars.rs
    snapshot.rs
    visual_states.rs

domain/ui/ui_definition/src/visual_layout/apply/
  mod.rs
  context.rs
  dispatch.rs
  containers.rs
  controls.rs
  collections.rs
  diagnostics.rs
  tests.rs

domain/ui/ui_definition/src/preview_fixture/
  mod.rs
  builders.rs
  catalog.rs
  routes.rs
  controls.rs
  surfaces.rs
  tests.rs
  validation/
    mod.rs
    diagnostics.rs
    fixtures.rs
    scenarios.rs
    matrices.rs
```

## Preserved Public APIs

The split kept compatibility re-exports in the existing public module
locations:

- `ui_runtime::compute_tree_layout`
- `ui_runtime::layout::*`
- `ui_runtime::input::generic_interaction::*`
- `ui_runtime::*` re-exports for generic interaction proof types
- `ui_definition::form_retained_ui`
- `ui_definition::FormedRetainedUiProduct`
- `ui_definition::FormedUiRoute`
- `ui_definition::UiDefinitionContext`
- `ui_definition::WidgetIdScope`
- `ui_definition::*` re-exports from `lib.rs`
- `ui_controls::runenwerk_control_package`
- `ui_runtime::input::pointer::dispatch_pointer_event`
- `ui_runtime::runtime::ui_runtime::UiRuntime`
- `ui_theme::token::*`
- `ui_definition::persistence_activation::*`
- `ui_runtime::output::{build_ui_frame, InteractionVisualState}`
- `ui_runtime::output::evidence::*`
- `ui_definition::apply_visual_layout_operation`
- `ui_definition::preview_fixture::*`

## Final Largest UI Files

| Lines | File |
| ---: | --- |
| 1117 | `domain/ui/ui_definition/src/view_binding/mod.rs` |
| 1115 | `domain/ui/ui_definition/src/production_readiness/mod.rs` |
| 1096 | `domain/ui/ui_graph_editor/src/lib.rs` |
| 1069 | `domain/ui/ui_definition/src/component_recipe/mod.rs` |
| 932 | `domain/ui/ui_runtime/src/output/emit/controls.rs` |
| 866 | `domain/ui/ui_runtime_view/src/lib.rs` |
| 832 | `domain/ui/ui_controls/src/package/validation.rs` |
| 776 | `domain/ui/ui_runtime/src/input/generic_interaction/replay.rs` |
| 755 | `domain/ui/ui_render_primitives/src/lib.rs` |
| 730 | `domain/ui/ui_runtime/src/text_editing/replay.rs` |
| 692 | `domain/ui/ui_text/src/proof_layout.rs` |
| 689 | `domain/ui/ui_composition/src/transaction/apply.rs` |
| 656 | `domain/ui/ui_composition/tests/transaction_atomicity.rs` |
| 650 | `domain/ui/ui_controls/src/overlay.rs` |
| 641 | `domain/ui/ui_binding/src/lib.rs` |

## Deferred Refactors

The seven Batch 2 target files were split or reduced to compatibility
facades. Remaining large files are outside the requested Batch 2 list or are
existing proof/runtime modules whose split should be separately scoped:

- `ui_definition/src/view_binding/mod.rs`, `production_readiness/mod.rs`, and
  `component_recipe/mod.rs` are validation/evidence domains and should be
  split by declaration, diagnostics, report, target-profile gates, and tests.
- `ui_graph_editor/src/lib.rs` and `ui_runtime_view/src/lib.rs` are crate
  entrypoint candidates and need owner decisions before public API movement.
- `ui_runtime/src/output/emit/controls.rs` is render emission code; split only
  with snapshot evidence preserved.
- `ui_controls/src/package/validation.rs`, `overlay.rs`, `editable_text.rs`,
  `accessibility.rs`, `theme.rs`, and `state.rs` are package/control domains
  and should be split by package validation, authored declarations, and tests.
- `ui_runtime/src/input/generic_interaction/replay.rs` and
  `ui_runtime/src/text_editing/replay.rs` are proof replay orchestrators; split
  only when report row and proof evidence signatures remain exact.
- `ui_runtime/src/input/pointer/dispatch.rs` remains as the pointer facade
  dispatcher after Batch 2. It is no longer the original god module, but a
  follow-up can split event-kind handlers if the team wants every new file
  under the 400-line review threshold.

These are deferred because Batch 2 was intentionally constrained to the seven
listed large files and behavior-preserving movement. No remaining file is left
large because a listed Batch 2 target was skipped.

## Validation

Completed validation during the split pass:

- `cargo fmt -p ui_runtime`
- `cargo test -p ui_runtime layout`
- `cargo test -p ui_runtime generic_interaction`
- `cargo test -p ui_runtime`
- `cargo test -p ui_static_mount`
- `cargo fmt -p ui_definition`
- `cargo test -p ui_definition`
- `cargo test --workspace`
- `cargo fmt -p ui_controls -p ui_definition -p ui_runtime`
- `cargo test -p ui_controls`
- `cargo test -p ui_definition`
- `cargo test -p ui_runtime`
- `git diff --check`
- `git diff --cached --check`

Batch 2 focused validation was run after each grouped split:

- `cargo fmt --all --check`
- `cargo test -p ui_runtime`
- `cargo test -p ui_theme`
- `cargo test -p ui_definition`
- `git diff --check`

Batch 2 final validation:

- `cargo fmt --all --check`
- `cargo test --workspace` was attempted first and hit a rustc out-of-memory
  failure in parallel engine/app compilation.
- `$env:CARGO_INCREMENTAL='0'; cargo test --workspace -j 1` passed.
- `python tools/docs/validate_docs.py`
- `git diff --check`

## Next Step

Phase 16 Surface2D implementation can proceed through its separate planning and
authorization workflow after this closeout is present on `main`. The split pass
preserved current behavior and did not settle the separate Surface2D owner-split
decision.
