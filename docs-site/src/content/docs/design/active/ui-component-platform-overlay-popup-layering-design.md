---
title: UI Component Platform Overlay / Popup / Layering Design
description: Owner-first reusable overlay, popup, dropdown, tooltip, focus-containing, anchor, placement, focus, dismissal, stack, proof, and no-bypass design.
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
  - ./editor-ui-runtime-v2-and-interaction-formation-design.md
  - ../implemented/editor-self-authoring-and-final-ui-design.md
---

# UI Component Platform Overlay / Popup / Layering Design

## Status

Lifecycle state: `active-implementation`.

Planning ID: `PT-UI-COMPONENT-PLATFORM-013`.

This document is the canonical Phase 13 owner-first design and implementation scope. It replaces the earlier design-intake wording and folds in the implementation placement correction: overlay runtime belongs under `ui_runtime::overlay`, not under `ui_runtime::input`.

## Decision summary

Reusable controls may declare overlay/open requirements. Runtime may form overlay intent, stack entries, layer assignments, placement, focus, dismissal, replay, report, visual proof, proof frame, and no-bypass evidence. Generic UI must not execute product commands, mutate product/editor/game state, create app-specific modal lifecycle behavior, own authored UI editing, own full text editing, or become a plugin framework.

The proof path is:

```text
ui_controls overlay/open declarations
  -> ui_input normalized input facts
  -> ui_runtime overlay intent, stack, layer, placement, focus, and dismissal evidence
  -> OverlayLayeringVisualProof
  -> OverlayLayeringProofRenderFrame / UiFrame
  -> ui_static_mount validation
```

Phase 13 consumes Phase 12/12A generic interaction and executable story mechanics, but this implementation slice does not claim a real overlay-specific live proof-host or story-workflow integration. It proves deterministic replay/report/static-frame evidence only unless a later accepted scope adds live proof-host work.

## Owner boundaries

### `ui_controls`

Owns reusable overlay declarations and ergonomic descriptor builders only:

```text
ControlOverlayDescriptor
ControlOverlayRequirement
ControlOverlayKind
ControlOverlayTrigger
ControlOverlayDismissPolicy
ControlOverlayPlacementPreference
ControlOverlayLayerPreference
ControlOverlaySupportSummary
```

Allowed:

- declare that a reusable control can request an overlay;
- declare kind, trigger, placement preference, dismissal policy, focus policy, keyboard policy, and inspection summary;
- provide ergonomic defaults for popup, menu, dropdown, tooltip, picker popup, and focus-containing overlays;
- expose read-only summary/inspection facts.

Forbidden:

- runtime overlay stack state;
- raw input collection;
- product command execution;
- product/editor/game mutation;
- app-specific modal state;
- authored UI editing;
- renderer backend behavior;
- full text editing.

### `ui_input`

Owns normalized input facts only:

```text
PointerInputFact
KeyboardInputFact
FocusInputFact
SemanticInputFact
ScrollInputFact
ViewportInputFact
NormalizedInputSample
```

Forbidden:

- overlay kind interpretation;
- anchor placement policy;
- focus containment semantics;
- dismissal decisions;
- product behavior;
- story execution;
- layer rendering.

### `ui_runtime`

Owns renderer-neutral overlay runtime proof through a runtime-level overlay module:

```text
domain/ui/ui_runtime/src/overlay/
```

Allowed:

- resolve overlay open intent from reusable declarations and normalized input facts;
- maintain renderer-neutral overlay session/stack proof state;
- record anchor, requested placement, resolved placement, layer assignment, clamp/collision evidence, focus behavior, dismissal behavior, pointer capture behavior, keyboard navigation evidence, and suppression evidence;
- project proof evidence into renderer-neutral `UiFrame` data;
- record no-bypass counters.

Forbidden:

- product/editor/game command execution;
- product/editor/game mutation;
- app-specific modal lifecycle behavior;
- authored UI editing;
- full text editing;
- backend renderer behavior;
- story registry/discovery ownership;
- broad plugin framework extraction.

### `ui_static_mount`

Owns renderer-neutral static frame validation only. It does not execute live interaction, own overlay session execution, own dismissal policy, or own product behavior.

### Product/editor/game layers

Own command execution, state mutation, route authorization, persistence, authored editing, app-specific modal lifecycle behavior, and product policy. They may consume generic overlay evidence, but they do not define reusable overlay semantics.

## Overlay vocabulary

```text
Overlay declaration
  A control-owned reusable declaration that says a control can request an overlay and what generic behavior it requires.

Overlay open intent
  Runtime evidence that normalized input matched a declaration and requested an overlay. It is not a product command.

Overlay stack
  Ordered runtime-owned list of open overlay entries. It determines topmost dismissal and parent/child relationships.

Overlay stack entry
  Runtime-owned open overlay record with request id, optional parent request id, scope, anchor id, layer class, placement, focus policy, dismissal state, and hit regions.

Overlay scope
  Logical grouping for dismissal and focus. Menu/submenu share a menu scope; tooltip and focus-containing overlays have separate scopes.

Placement resolution
  Runtime evidence for requested side/alignment, resolved side/alignment, clamp/shift result, and viewport constraints.

Dismissal evidence
  Runtime report row proving why an overlay stayed open, closed, or was suppressed.

No-bypass assertion
  Runtime counter proof that overlay behavior did not execute host commands, mutate product state, edit text, create app-modal lifecycle behavior, or bypass normalized input.
```

## Current implementation scope

This Phase 13 implementation slice is intentionally narrow:

- `ui_controls` overlay declaration vocabulary and ergonomic builders;
- `ui_input` fact-only compatibility test for overlay consumption;
- `ui_runtime::overlay` deterministic replay/report/stack/placement/focus/dismissal/suppression proof;
- `OverlayLayeringVisualProof` / `OverlayLayeringProofRenderFrame` projection;
- `ui_static_mount` validation of runtime-generated overlay proof frames;
- focused runtime tests for exact evidence rows and no-bypass counters.

It does not implement:

- package/catalog/lowering integration of overlay declarations into every control package descriptor;
- real overlay-specific `ui_story` workflow profile;
- editor proof-host or product-facing UI Gallery exposure;
- UI Designer;
- authored UI editing;
- command execution or product mutation;
- full text editing;
- app-specific modal lifecycle;
- dynamic plugin framework;
- `foundation/meta`;
- shared plugin primitives;
- Workbench/provider redesign.

## Correct implementation files

```text
domain/ui/ui_controls/src/overlay.rs
domain/ui/ui_controls/src/lib.rs

domain/ui/ui_input/tests/overlay_normalized_facts.rs

domain/ui/ui_runtime/src/lib.rs
domain/ui/ui_runtime/src/overlay/mod.rs
domain/ui/ui_runtime/src/overlay/report.rs
domain/ui/ui_runtime/src/overlay/fixture.rs
domain/ui/ui_runtime/src/overlay/placement.rs
domain/ui/ui_runtime/src/overlay/stack.rs
domain/ui/ui_runtime/src/overlay/layering.rs
domain/ui/ui_runtime/src/overlay/proof_frame.rs
domain/ui/ui_runtime/tests/overlay_layering_report.rs
domain/ui/ui_runtime/tests/executable_overlay_layering_story.rs

domain/ui/ui_static_mount/tests/base_controls_overlay_layering_static_mount.rs
```

Do not place overlay runtime semantics under:

```text
domain/ui/ui_runtime/src/input/overlay_layering.rs
domain/ui/ui_runtime/src/input/overlay_layering_fixture.rs
domain/ui/ui_runtime/src/input/overlay_layering_visual_frame.rs
domain/ui/ui_runtime/src/input/overlay_layering_story_session.rs
```

## Required proof scenarios

Implementation must prove:

- Button popup open intent;
- ActionPrompt menu-like open intent;
- Dropdown request anchor/placement/layer/dismissal policy;
- Tooltip hover trigger evidence;
- Tooltip focus trigger evidence;
- Picker popup open intent without value mutation;
- focus-containing overlay evidence without app-specific modal lifecycle behavior;
- menu-to-submenu parent/child stack evidence;
- Escape dismisses topmost eligible overlay only;
- outside pointer dismissal;
- outside pointer inside the active overlay does not dismiss;
- pointer capture evidence during opening;
- keyboard navigation evidence without product command execution;
- scroll-driven placement recomputation;
- viewport-resize placement recomputation;
- anchor removal invalidation evidence;
- runtime report to static proof frame to static mount validation;
- deterministic replay evidence.

## Negative scenarios

Implementation must prove:

- disabled anchor suppresses open;
- outside pointer inside active overlay does not dismiss;
- overlay request does not execute host commands;
- overlay request does not mutate product/editor/game state;
- overlay request does not perform text editing;
- overlay proof does not create Gallery, Designer, authored editing, plugin framework, `foundation/meta`, shared plugin primitives, or Workbench/provider redesign.

## No-bypass counters

`OverlayBoundaryAssertions` must assert:

```text
host_commands_executed == 0
product_mutations == 0
text_edit_transactions == 0
app_specific_modal_operations == 0
authored_ui_edits == 0
plugin_framework_operations == 0
```

Overlay-specific counters should include open requests, opened overlays, suppressed overlays, Escape dismissals, outside-pointer dismissals, stack entries opened/closed, placement recomputation after scroll/viewport resize, anchor invalidation suppression, and focus return.

Every overlay open, dismissal, and suppression row must have normalized input evidence. Every open intent must link back to a declared reusable overlay requirement.

## Stable ids and durable names

Public APIs, stable ids, fixture helpers, proof names, and test names must use durable domain-shaped names such as `overlay_layering` and `base_controls_overlay_layering`, never `phase13_*` names.

Stable ids include:

```text
base-controls.overlay-layering.proof
base-controls.overlay-layering.story
base-controls.overlay-layering.fixture
anchor.button.popup
anchor.action-prompt.menu
anchor.action-prompt.submenu
anchor.dropdown.fixture
anchor.tooltip.hover
anchor.tooltip.focus
anchor.color-picker.picker-popup
anchor.focus-containing.fixture
anchor.disabled.fixture
```

## Validation gate

Run before merge:

```text
cargo fmt --all --check
cargo check -p ui_controls
cargo check -p ui_input
cargo check -p ui_runtime
cargo check -p ui_static_mount
cargo check -p ui_story
cargo check -p runenwerk_editor
cargo test -p ui_controls overlay
cargo test -p ui_input input
cargo test -p ui_runtime overlay_layering
cargo test -p ui_runtime --test overlay_layering_report
cargo test -p ui_runtime --test executable_overlay_layering_story
cargo test -p ui_static_mount base_controls_overlay_layering
python tools/docs/validate_docs.py
git diff --check
```

No editor proof-host command is required unless a later accepted scope adds a real editor adapter.

## Stop conditions

Stop and redesign if implementation requires:

- overlay runtime behavior under `ui_runtime::input`;
- command execution in generic UI;
- product/editor/game mutation in generic UI;
- app-specific modal lifecycle in generic UI;
- overlay behavior in `ui_controls` beyond declarations/builders;
- input semantics in `ui_input` beyond facts;
- story registry/discovery moved into `ui_runtime`;
- editor shell surface registration or Workbench provider redesign;
- UI Gallery or UI Designer product surface;
- full text editing;
- dynamic external plugin framework;
- `foundation/meta`;
- shared plugin primitives;
- phase-shaped public API names;
- compatibility-only aliases or shims.
