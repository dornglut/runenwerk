---
title: Phase 16 Surface2D Source Investigation
description: Source-level investigation for PT-UI-COMPONENT-PLATFORM-016 before implementation authorization.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ../../design/active/ui-component-platform-surface2d-design.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/complete-investigation-gate.md
  - ../../workspace/complete-design-gate.md
  - ../../workspace/evidence-quality-taxonomy.md
---

# Phase 16 Surface2D Source Investigation

## Status

This report records the source-level investigation for `PT-UI-COMPONENT-PLATFORM-016` Surface2D.

Lifecycle state: `active-planning`.

Implementation is not authorized by this report alone. It supplies the exact implementation contract input needed for planning promotion.

## Evidence classes

```text
E2 connector file inspection
E2 merged PR metadata / changed-file inspection
E8 accepted planning and design authority
```

Command validation was not run in the connector.

## Question

Which exact files should Phase 16 Surface2D implementation touch, which crates are confirmed owners, which crates are not required, and what tests/proofs must exist before merge?

## Current source facts

### `ui_controls`

`ui_controls` is the primary package-backed declaration owner.

Confirmed extension points:

```text
domain/ui/ui_controls/src/lib.rs
  public module/re-export surface for reusable control domains

domain/ui/ui_controls/src/package/descriptor.rs
  descriptor vector fields
  builder methods
  descriptor lookup methods

domain/ui/ui_controls/src/package.rs
  validation module registration

domain/ui/ui_controls/src/package/validation.rs
  validation reasons
  duplicate checks
  descriptor validation calls

domain/ui/ui_controls/src/catalog/entry.rs
  catalog entry fields and support-summary projection

domain/ui/ui_controls/src/catalog/inspection.rs
  inspection section enum
  inspection fact projection

domain/ui/ui_controls/src/base_control/compiler.rs
  lowering call chain for package-backed support descriptors

domain/ui/ui_controls/src/base_control/lowering/mod.rs
  lowering module registry

domain/ui/ui_controls/src/base_control/mod.rs
  base-control target kind list

domain/ui/ui_controls/src/base_control/preset.rs
  preset enum used by base controls

domain/ui/ui_controls/src/base_control/plugin.rs
  base-control contribution list
```

The existing Generic Text path proves the pattern: a dedicated declaration module, descriptor storage on `ControlPackageDescriptor`, catalog projection, inspection projection, validation module, base-control lowering, and focused package tests.

### `ui_runtime`

`ui_runtime` is the primary runtime proof/report/frame owner.

Confirmed extension points:

```text
domain/ui/ui_runtime/src/lib.rs
  public module/re-export surface for runtime proofs

domain/ui/ui_runtime/src/output/evidence.rs
  runtime-owned render output evidence helpers

domain/ui/ui_runtime/src/output/build_ui_frame.rs
  retained tree to UiFrame path; not the owner of Surface2D proof

domain/ui/ui_runtime/src/input/pointer/mod.rs
  pointer dispatch module registry

domain/ui/ui_runtime/src/input/pointer/graph_canvas.rs
  existing graph-canvas-specific pointer behavior to avoid reusing for Surface2D

domain/ui/ui_runtime/src/runtime/ui_runtime/graph_canvas.rs
  existing graph-canvas keyboard behavior to avoid reusing for Surface2D

domain/ui/ui_runtime/src/layout/engine/surface.rs
  existing surface/embed retained layout helpers; not the owner of reusable Surface2D semantics
```

Implementation should add a dedicated `surface2d` runtime proof module instead of expanding graph-canvas or product-surface behavior.

### `ui_static_mount`

`ui_static_mount` already accepts renderer-neutral `UiFrame` evidence directly.

Confirmed extension point:

```text
domain/ui/ui_static_mount/tests/base_controls_surface2d_static_mount.rs
```

No `ui_static_mount/src` change is required if the Surface2D proof frame contains at least one surface, at least one primitive, a rectangle/background primitive, a border/outline primitive, and stable draw order.

### `ui_render_data`

`ui_render_data` is not required for Phase 16 source changes if Surface2D proof uses existing primitives.

Existing primitive support is sufficient for Phase 16 proof:

```text
RectPrimitive      -> background, viewport, diagnostic fills
BorderPrimitive    -> viewport outline, selection rectangle, diagnostic outline
StrokePrimitive    -> grid lines, axes, pan/gesture traces
ClipPrimitive      -> viewport clipping proof
UiFrame/UiSurface  -> renderer-neutral proof frame envelope
```

Do not add a new primitive unless implementation proves the existing primitive set cannot express the delivered proof.

### `ui_render_primitives`

`ui_render_primitives` is not required for Phase 16 source changes.

It currently generates backend-neutral primitives from runtime-view reports and button data. Surface2D proof should use `ui_runtime` -> `ui_render_data::UiFrame` directly, matching the Generic Text proof style, instead of extending runtime-view primitive generation.

### `ui_input`

`ui_input` is not required for Phase 16 source changes unless implementation discovers a missing generic normalized input fact.

Existing exported input modules include pointer, keyboard, focus, routing, selection, semantic, shortcut, and text. Surface2D runtime proof can consume existing `PointerEvent`, `KeyboardEvent`, focus, modifier, and pointer packet vocabulary.

Trackpad pinch, touch pan/zoom, and controller navigation must be reported explicitly as delivered or not delivered by the Surface2D descriptor/report. Do not create new input vocabulary for them unless implementation proves the current input vocabulary cannot represent the required fact.

### `ui_surface`

`ui_surface` remains existing semantic surface vocabulary and is not a Phase 16 implementation owner.

Confirmed current semantic modules:

```text
capability
definition
diagnostics
intent
mount
observation
presentation
ratification
session
validation
```

Phase 16 must not rename, replace, remove, or absorb these contracts. `Surface2D` is lower-level coordinate/navigation vocabulary and has no direct dependency on `ui_surface` in the Phase 16 implementation contract.

## Exact implementation contract

### Required `ui_controls` files

```text
domain/ui/ui_controls/src/surface2d.rs
  new declarations, support summary, inspection facts, no-mutation flags

domain/ui/ui_controls/src/lib.rs
  add module and public re-export

domain/ui/ui_controls/src/package/descriptor.rs
  add surface2d_descriptors field
  add with_surface2d_descriptor()
  add surface2d_descriptor()

domain/ui/ui_controls/src/package.rs
  add surface2d_validation module

domain/ui/ui_controls/src/package/validation.rs
  add Surface2D validation reasons
  add duplicate descriptor check
  call surface2d validator

domain/ui/ui_controls/src/package/surface2d_validation.rs
  new validator for unresolved descriptor, invalid bounds, invalid transform, unsupported input/status rows, and backend/product mutation flags

domain/ui/ui_controls/src/catalog/entry.rs
  add Surface2D catalog fields and with_surface2d_summary()

domain/ui/ui_controls/src/catalog/inspection.rs
  add Surface2D inspection section and projected facts

domain/ui/ui_controls/src/base_control/compiler.rs
  lower Surface2D descriptor into package

domain/ui/ui_controls/src/base_control/lowering/mod.rs
  register surface2d_support lowering module

domain/ui/ui_controls/src/base_control/lowering/surface2d_support.rs
  new lowering from Surface2D control definition to descriptor

domain/ui/ui_controls/src/base_control/mod.rs
  add Surface2D control kind id to base-control target list

domain/ui/ui_controls/src/base_control/preset.rs
  add Surface2D preset

domain/ui/ui_controls/src/base_control/plugin.rs
  contribute Surface2D base control

domain/ui/ui_controls/src/surface2d_control.rs or domain/ui/ui_controls/src/surface2d.rs
  define SURFACE2D_CONTROL_KIND_ID and control_contribution()
```

Implementation should prefer a single `surface2d.rs` module unless size or clarity requires a submodule split.

### Required `ui_controls` tests

```text
domain/ui/ui_controls/tests/surface2d_package.rs
  descriptor validation reasons
  catalog projection facts
  inspection projection facts
  no host-command/product-mutation flags

domain/ui/ui_controls/tests/control_package_validation.rs
  update base-control counts
  add duplicate Surface2D descriptor validation check
```

### Required `ui_runtime` files

```text
domain/ui/ui_runtime/src/surface2d/mod.rs
  Surface2D proof report
  Surface2D proof frame
  transform helpers
  navigation state
  hover/selection/capture/gesture facts
  accessibility/input status rows
  budget evidence rows
  no-bypass boundary assertions

domain/ui/ui_runtime/src/lib.rs
  add public Surface2D re-export
```

The runtime proof must not expand graph-canvas-specific pointer/keyboard modules. Existing graph-canvas files remain evidence of what Surface2D must not become.

### Required `ui_runtime` tests

```text
domain/ui/ui_runtime/tests/surface2d_runtime_proof.rs
  report contains descriptor, transform, navigation, hover, selection, capture, accessibility/input, budget, catalog/inspection, static-mount expectation, and no-bypass evidence
  invalid transform emits expected-failure diagnostic
  no product/editor/game mutation is recorded
```

### Required `ui_static_mount` tests

```text
domain/ui/ui_static_mount/tests/base_controls_surface2d_static_mount.rs
  Surface2D proof frame mounts through UiStaticMountReport::from_frame
  frame has surface, rect/background, border/outline, stable draw order
  proof summary records expected Surface2D facts
```

### Conditional files

Do not touch these unless implementation proves the existing contracts cannot carry the required evidence:

```text
domain/ui/ui_render_data/src/**
domain/ui/ui_render_primitives/src/lib.rs
domain/ui/ui_input/src/**
domain/ui/ui_surface/src/**
```

## Validation envelope

Required focused validation after implementation:

```text
cargo test -p ui_controls surface2d
cargo test -p ui_controls control_package
cargo test -p ui_runtime surface2d
cargo test -p ui_static_mount surface2d
cargo test --workspace
python tools/docs/validate_docs.py
git diff --check
```

Conditional validation:

```text
cargo test -p ui_render_data        # only if ui_render_data changes
cargo test -p ui_render_primitives  # only if ui_render_primitives changes
cargo test -p ui_input              # only if ui_input changes
cargo test -p ui_surface            # only if ui_surface changes; should not be needed for Phase 16
```

## Implementation stop conditions

Stop and redesign if implementation requires:

```text
new crate creation
ui_surface source changes
ui_render_primitives source changes for backend generation
ui_input source changes without proven missing normalized fact
GraphCanvas or Timeline vocabulary in Surface2D public API
product/editor/game mutation
renderer backend handles
host command execution inside domain/ui
bypassing ControlPackageDescriptor/catalog/inspection projection
static mount proof that does not use the runtime proof frame
```

## Recommendation

Phase 16 can move from design intake to implementation planning after this source investigation is reviewed and docs validation passes.

The implementation should be a single complete Phase 16 vertical proof across `ui_controls`, `ui_runtime`, and `ui_static_mount`, with conditional crates untouched unless evidence proves they are required.
