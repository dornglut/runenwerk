---
title: UI Domain Split Pass Closeout
description: Behavior-preserving closeout report for the pre-Surface2D UI module split pass.
status: completed
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-07-02
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
| PR A | `ui/split-runtime-layout-engine` | Local validated commit | Split runtime layout engine into owner-aligned modules. |
| PR B | `ui/split-runtime-generic-interaction` | Local validated commit | Split runtime generic interaction proof module. |
| PR C | `ui/split-definition-formation` | Local validated commit | Split retained UI definition formation. |
| PR D | `ui/move-ui-test-support` | Local validated commit | Moved noisy tests/support out of public API files. |
| PR E | `docs/ui-domain-split-pass-report` | This docs-only report | Recorded residual structural debt and non-goals. |

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

## Final Largest UI Files

| Lines | File |
| ---: | --- |
| 3423 | `domain/ui/ui_runtime/src/runtime/ui_runtime.rs` |
| 1449 | `domain/ui/ui_runtime/src/input/pointer.rs` |
| 1357 | `domain/ui/ui_theme/src/token/mod.rs` |
| 1332 | `domain/ui/ui_definition/src/persistence_activation/mod.rs` |
| 1276 | `domain/ui/ui_runtime/src/output/build_ui_frame.rs` |
| 1218 | `domain/ui/ui_definition/src/visual_layout/apply.rs` |
| 1166 | `domain/ui/ui_definition/src/preview_fixture/mod.rs` |
| 1117 | `domain/ui/ui_definition/src/view_binding/mod.rs` |
| 1115 | `domain/ui/ui_definition/src/production_readiness/mod.rs` |
| 1069 | `domain/ui/ui_definition/src/component_recipe/mod.rs` |
| 1068 | `domain/ui/ui_graph_editor/src/lib.rs` |
| 932 | `domain/ui/ui_runtime/src/output/emit/controls.rs` |
| 866 | `domain/ui/ui_runtime_view/src/lib.rs` |
| 832 | `domain/ui/ui_controls/src/package/validation.rs` |
| 776 | `domain/ui/ui_runtime/src/input/generic_interaction/replay.rs` |

## Deferred Refactors

The next structural cleanup candidates are:

- split `ui_runtime/src/runtime/ui_runtime.rs` by event routing, focus,
  pointer capture, scroll ownership, graph canvas, popup dismissal, and state
  mutation coordination;
- split `ui_runtime/src/input/pointer.rs` by pointer routing, capture,
  scrollbar drag, graph canvas gestures, viewport routing, and suppression;
- split `ui_runtime/src/output/build_ui_frame.rs` further only after emission
  behavior tests are isolated enough to keep snapshot evidence local;
- split `ui_definition` proof/readiness modules by validation report,
  evidence collection, and target-profile gates;
- split `ui_theme/src/token/mod.rs` by token identity, selector matching,
  resolution, diagnostics, and tests;
- split `ui_runtime/src/input/generic_interaction/replay.rs` if replay
  orchestration grows further after Surface2D planning resolves.

These are deferred because this pass intentionally stayed limited to the
requested highest-risk UI god modules and test-support noise.

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

## Next Step

Phase 16 Surface2D implementation can begin only after the split branches are
reviewed and merged in order. The split pass preserved current behavior and
did not settle the separate Surface2D owner-split decision.
