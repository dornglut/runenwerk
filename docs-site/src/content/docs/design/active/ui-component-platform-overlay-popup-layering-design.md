---
title: UI Component Platform Overlay / Popup / Layering Design
description: Complete owner-first overlay, popup, dropdown, tooltip, focus-containing, placement, package, catalog, inspection, runtime proof, static proof, and no-bypass design.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-07-01
related_docs:
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
  - ../../workspace/planning/completed-work.md
  - ../../workspace/planning/decision-register.md
  - ./ui-component-platform-generic-interaction-design.md
  - ./ui-component-platform-executable-interaction-story-design.md
  - ./ui-component-platform-executable-interaction-story-implementation-scope.md
---

# UI Component Platform Overlay / Popup / Layering Design

Lifecycle state: `active-implementation`.

Planning ID: `PT-UI-COMPONENT-PLATFORM-013`.

Phase 13 is complete only when overlay / popup / layering is package-backed, catalog-visible, inspection-visible, runtime-proven, static-mount-proven, and covered by no-bypass evidence. Overlay runtime belongs under `ui_runtime::overlay`, not under `ui_runtime::input`.

## Complete proof chain

```text
ui_controls overlay vocabulary
  -> base-control overlay lowering
  -> ControlPackageDescriptor.overlay_descriptors
  -> package validation
  -> catalog projection
  -> inspection projection
  -> ui_input normalized facts
  -> ui_runtime::overlay replay/report/stack/placement/focus/dismissal/suppression evidence
  -> OverlayLayeringVisualProof
  -> OverlayLayeringProofRenderFrame / UiFrame
  -> ui_static_mount validation
```

## Owner boundaries

`ui_controls` owns reusable overlay declarations, ergonomic builders, base-control lowering, package descriptors, package validation, catalog projection, and inspection projection. It must not own runtime stack state, raw input, product commands, product/editor/game mutation, app-specific modal state, authored editing, renderer backend behavior, or text editing.

`ui_input` owns normalized input facts only. It must not interpret overlay kind, placement, focus containment, dismissal, product behavior, story execution, or layer rendering.

`ui_runtime` owns renderer-neutral overlay proof under `domain/ui/ui_runtime/src/overlay/`. It may consume package-backed declarations and normalized facts to form open intents, stack entries, placement, layer, focus, dismissal, suppression, keyboard, pointer-capture, viewport, proof-frame, and no-bypass evidence. It must not execute product commands, mutate product/editor/game state, own authored editing, own app modal lifecycle, own text editing, own renderer backend behavior, or extract a plugin framework.

`ui_static_mount` validates renderer-neutral proof frames only.

Product/editor/game layers own command execution, state mutation, route authorization, persistence, authored editing, app-specific modal lifecycle, and product policy.

## Completion scope

Implementation must include:

- `ControlOverlayDescriptor`, requirements, kinds, triggers, placement, layer, dismissal, focus, and support summaries;
- base-control lowering for popup, menu/submenu, dropdown, tooltip hover/focus, picker popup, and focus-containing overlay support;
- package-level overlay descriptors for all base controls;
- package validation for duplicate and unresolved overlay descriptors;
- catalog projection of overlay kinds, triggers, layers, dismissal policies, focus policies, and no-command/no-mutation flags;
- inspection projection of overlay support facts;
- runtime package-backed fixture construction;
- runtime open intent, stack, placement, layer, focus, dismissal, suppression, keyboard, pointer-capture, viewport, and no-bypass evidence;
- deterministic replay evidence;
- runtime report to visual proof frame to static mount validation;
- tests for package, catalog, inspection, runtime package-backed consumption, runtime behavior, static mount, and no-bypass boundaries.

## Correct implementation files

```text
domain/ui/ui_controls/src/overlay.rs
domain/ui/ui_controls/src/package/descriptor.rs
domain/ui/ui_controls/src/package/validation.rs
domain/ui/ui_controls/src/package/overlay_validation.rs
domain/ui/ui_controls/src/base_control/lowering/layering_support.rs
domain/ui/ui_controls/src/base_control/lowering/mod.rs
domain/ui/ui_controls/src/base_control/compiler.rs
domain/ui/ui_controls/src/base_control/lowering/inspection.rs
domain/ui/ui_controls/src/catalog/entry.rs
domain/ui/ui_controls/src/catalog/inspection.rs
domain/ui/ui_controls/tests/base_controls_overlay_package.rs
domain/ui/ui_controls/tests/base_controls_overlay_catalog.rs
domain/ui/ui_controls/tests/base_controls_overlay_inspection.rs

domain/ui/ui_input/tests/overlay_normalized_facts.rs

domain/ui/ui_runtime/src/overlay/mod.rs
domain/ui/ui_runtime/src/overlay/report.rs
domain/ui/ui_runtime/src/overlay/fixture.rs
domain/ui/ui_runtime/src/overlay/placement.rs
domain/ui/ui_runtime/src/overlay/stack.rs
domain/ui/ui_runtime/src/overlay/layering.rs
domain/ui/ui_runtime/src/overlay/proof_frame.rs
domain/ui/ui_runtime/tests/overlay_layering_report.rs
domain/ui/ui_runtime/tests/executable_overlay_layering_story.rs
domain/ui/ui_runtime/tests/overlay_package_backed.rs

domain/ui/ui_static_mount/tests/base_controls_overlay_layering_static_mount.rs
```

Do not place overlay runtime semantics under `domain/ui/ui_runtime/src/input/overlay_*`.

## Required proof scenarios

Implementation must prove package-backed declarations, catalog projection, inspection projection, runtime package-backed consumption, Button popup, ActionPrompt menu/submenu, Dropdown, tooltip hover/focus, picker popup, focus-containing overlay, topmost Escape dismissal, outside pointer dismissal, inside-active-overlay no dismissal, pointer capture, keyboard navigation without product command execution, scroll and viewport placement recomputation, anchor invalidation, runtime report to static proof frame, and deterministic replay.

## No-bypass counters

`OverlayBoundaryAssertions` must prove zero host command execution, zero product mutation, zero text-edit transaction, zero app-specific modal operation, zero authored UI edit, and zero plugin-framework operation.

## Validation gate

```text
cargo fmt --all --check
cargo check -p ui_controls
cargo check -p ui_input
cargo check -p ui_runtime
cargo check -p ui_static_mount
cargo check -p ui_story
cargo check -p runenwerk_editor
cargo test -p ui_controls overlay
cargo test -p ui_controls --test base_controls_overlay_package
cargo test -p ui_controls --test base_controls_overlay_catalog
cargo test -p ui_controls --test base_controls_overlay_inspection
cargo test -p ui_input input
cargo test -p ui_runtime overlay_layering
cargo test -p ui_runtime --test overlay_layering_report
cargo test -p ui_runtime --test executable_overlay_layering_story
cargo test -p ui_runtime --test overlay_package_backed
cargo test -p ui_static_mount base_controls_overlay_layering
python tools/docs/validate_docs.py
git diff --check
```

## Stop conditions

Stop and redesign if implementation requires overlay runtime under `ui_runtime::input`, command execution in generic UI, product/editor/game mutation in generic UI, app-specific modal lifecycle in generic UI, runtime behavior in `ui_controls`, input semantics in `ui_input` beyond facts, story registry ownership in `ui_runtime`, editor shell registration, Workbench provider redesign, UI Gallery, UI Designer, full text editing, dynamic plugin framework, `foundation/meta`, shared plugin primitives, phase-shaped public API names, or compatibility-only aliases/shims.
