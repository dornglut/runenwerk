---
title: UI Component Platform Overlay / Popup / Layering Design
description: Proposed owner-first design for reusable overlay, popup, dropdown, tooltip, modal-like, anchor, placement, focus, dismissal, layer, proof, and no-bypass semantics.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-06-30
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

Lifecycle state: `proposed-design`.

Planning ID: `PT-UI-COMPONENT-PLATFORM-013`.

This document is the Phase 13 design intake for reusable overlay, popup, dropdown, tooltip, modal-like, and layering semantics.

It does not authorize Rust implementation. Before implementation, the implementation-scope section below must be accepted and then reflected in `active-work.md` as active implementation.

Phase labels may appear in planning, history, and reports. Public APIs, stable ids, reusable fixture helpers, proof-host names, current test names, and implementation-scope files must use durable domain names such as `overlay_intent`, `overlay_layering`, or `base_controls_overlay_layering`, not `phase13_*` names.

## Decision summary

Reusable controls may declare overlay/open intent requirements. Runtime may form overlay intent, layer, focus, placement, dismissal, replay, report, and static proof evidence. Generic UI must not execute product commands, mutate product/editor/game state, create app-specific modal behavior, own authored UI editing, or become a plugin framework.

The reusable overlay proof path is:

```text
ui_controls overlay/open declarations
  -> ui_input normalized input facts
  -> ui_runtime overlay intent, layer, placement, focus, and dismissal evidence
  -> OverlayLayeringVisualProof
  -> OverlayLayeringProofRenderFrame / UiFrame
  -> ui_static_mount validation
```

This phase consumes Phase 12/12A generic interaction and executable story mechanics. It also extracts durable overlay lessons from Interaction V2 popup-stack work: stable anchors, layer order, outside dismissal, Escape dismissal, focus return, scroll/input ownership, and viewport-fallback rejection. It must not copy editor shell command behavior into generic UI.

## Goals

- Define reusable overlay declarations without turning product/editor/game behavior into generic UI behavior.
- Distinguish popup, dropdown, menu, tooltip, modal-like, picker, and diagnostic overlay semantics.
- Define stable anchor identity, preferred placement, collision policy, viewport clamping, and placement evidence.
- Define layer ordering without hard-coding app/editor/game z behavior.
- Define focus containment, focus return, outside pointer dismissal, Escape dismissal, pointer capture interaction, and keyboard navigation evidence.
- Define disabled/suppressed overlay-open behavior.
- Define inspection metadata for declared and observed overlay behavior.
- Define deterministic proof fixture shape, story proof shape, replay/report evidence, static proof frame, and no-bypass assertions.
- Define exact implementation gate before Rust changes.

## Non-goals

This phase does not implement or authorize:

- product-facing UI Gallery;
- full UI Designer;
- authored UI editing;
- product/editor/game command execution;
- product/editor/game state mutation;
- app-specific modal behavior;
- full text editing;
- caret, selection, text buffer, IME, clipboard, or undo-redo behavior;
- dynamic external plugin framework;
- `foundation/meta`;
- broad shared plugin primitives;
- broad Workbench/provider redesign;
- backend renderer behavior;
- world-space overlays;
- pixel-perfect screenshot parity.

## Owner boundaries

### `ui_controls`

May own reusable overlay/open declarations only:

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

Allowed responsibilities:

- declare that a Button, ActionPrompt, ColorPicker, Select-like future control, or menu-like control can request an overlay;
- declare overlay kind, trigger, placement preference, dismissal policy, focus policy, and inspection summary;
- expose read-only catalog/inspection metadata derived from descriptors.

Forbidden responsibilities:

- runtime layer stack state;
- raw input collection;
- product command execution;
- product/editor/game mutation;
- app-specific modal state;
- authored UI editing;
- renderer backend behavior;
- full text editing.

### `ui_input`

Owns normalized input facts only.

Allowed responsibilities:

```text
PointerInputFact
KeyboardInputFact
FocusInputFact
SemanticInputFact
NormalizedInputSample
```

Future-compatible input fields may include pointer id, device kind, scroll delta, pressure, modifiers, click count, and logical timestamp, but this phase must only add what the overlay proof needs.

Forbidden responsibilities:

- overlay kind interpretation;
- anchor placement policy;
- focus containment semantics;
- dismissal decisions;
- product behavior;
- story execution;
- layer rendering.

### `ui_runtime`

May own renderer-neutral overlay intent formation and proof evidence:

```text
OverlayAnchorId
OverlayRequestId
OverlayLayerId
OverlayDeclaration
OverlayOpenIntent
OverlayPlacementRequest
OverlayPlacementResolution
OverlayLayerAssignment
OverlayDismissalPolicy
OverlayDismissalEvidence
OverlayFocusPolicy
OverlayFocusEvidence
OverlayKeyboardNavigationEvidence
OverlayPointerCaptureEvidence
OverlaySuppressionEvidence
OverlayBoundaryAssertions
OverlayLayeringReport
OverlayLayeringVisualProof
OverlayLayeringProofRenderFrame
```

Allowed responsibilities:

- resolve overlay open intent from reusable declarations and normalized input facts;
- record anchor, requested placement, resolved placement, layer assignment, clamp/collision result, focus behavior, dismissal behavior, and keyboard navigation evidence;
- maintain proof/session state needed for deterministic replay and story proof;
- project renderer-neutral proof into `UiFrame` render data;
- record no-bypass counters.

Forbidden responsibilities:

- product/editor/game command execution;
- product/editor/game mutation;
- app-specific modal behavior;
- authored UI editing;
- full text editing;
- backend renderer behavior;
- story registry/discovery authority;
- broad plugin framework extraction.

### `ui_static_mount`

Owns static frame validation.

Allowed responsibilities:

- validate that overlay/layer proof frames have deterministic surfaces, primitive ordering, anchor evidence, inspector evidence, and report evidence;
- keep validation renderer-neutral.

Forbidden responsibilities:

- live interaction;
- overlay session execution;
- dismissal policy ownership;
- product behavior.

### `runenwerk_editor`

May contain only narrow proof/test adapters if required by a later accepted implementation scope.

Allowed responsibilities:

- adapt existing proof-host input events to normalized input samples;
- expose current proof/report/frame/static-mount evidence for focused tests.

Forbidden responsibilities:

- product-facing UI Gallery surface;
- UI Designer surface;
- Workbench/provider redesign;
- editor command execution for overlay outcomes;
- editor scene/workspace mutation;
- app-specific modal policy.

### Product/editor/game layers

Own command execution, state mutation, route authorization, persistence, app-specific modal behavior, authored UI editing, and product policy.

They consume generic overlay intents and evidence. They do not define reusable overlay semantics.

## Overlay vocabulary

```text
Overlay declaration
  A control-owned reusable declaration that says a control can request an overlay and what generic behavior it requires.

Overlay open intent
  Runtime evidence that normalized input matched a declaration and requested an overlay. It is not a product command.

Overlay request
  The renderer-neutral request record containing request id, source control id, anchor id, kind, trigger, placement preference, layer preference, dismissal policy, focus policy, and disabled/suppressed status.

Anchor identity
  Stable identity for the element or region an overlay is positioned against. It must survive replay and report comparison without relying on transient retained widget ids.

Placement preference
  Preferred side/alignment/offset/fallback policy before viewport collision resolution.

Placement resolution
  Runtime evidence for chosen side/alignment, clamp, flip, shift, size limit, and viewport constraint results.

Layer assignment
  Runtime evidence for overlay ordering relative to ordinary content, tooltips, menus, modal-like surfaces, diagnostic overlays, and persistent status/chrome overlays.

Dismissal policy
  Reusable policy for outside pointer, Escape, focus loss, selection, explicit close, none, or host-owned dismissal.

Dismissal evidence
  Runtime report row proving why an overlay remained open or closed.

Focus policy
  Reusable focus behavior: none, focus overlay, contain focus, restore focus on close, or modal-like containment.

No-bypass assertion
  Runtime counter proving overlay proof did not execute host commands, mutate product state, perform text edits, create app-specific modal behavior, or bypass normalized input.
```

## Overlay kind distinction

| Kind | Generic meaning | Required generic proof | Not owned by generic UI |
| --- | --- | --- | --- |
| `Popup` | Anchored transient surface with explicit content role. | anchor, placement, layer, dismissal policy, focus policy | product command result or product content mutation |
| `Dropdown` | Anchored popup for choosing from options. | open intent, anchor, placement, keyboard navigation, selection-intent evidence | committing selection to app state |
| `Menu` | Anchored command-choice surface. | menu scope, roving/linear keyboard navigation, submenu-ready anchor evidence, Escape/outside dismissal | executing menu commands |
| `Tooltip` | Non-command informational overlay from hover/focus trigger. | hover/focus trigger evidence, delayed/open intent seam, non-focus-stealing policy | app-specific help content source or mutation |
| `PickerPopup` | Anchored picker-like popup for future controls such as color pickers. | open intent, anchor, placement, focus policy, dismissal policy | value mutation, color management, app command |
| `ModalLike` | Focus-containing overlay-like proof substrate. | layer/focus containment evidence, Escape policy, focus return | app-specific modal lifecycle, blocking product workflow |
| `DiagnosticOverlay` | Renderer-neutral proof/report overlay. | layer classification and non-dismissable/persistent policy | developer-tool product surface behavior |

`ModalLike` is intentionally named as a substrate concept. It must not become app-specific modal behavior in this phase.

## Placement model

`OverlayPlacementPreference` should describe:

```text
side: top | right | bottom | left | center | cursor
alignment: start | center | end | stretch
main_axis_offset
cross_axis_offset
fallback_order
collision_policy: none | flip | shift | clamp | resize | hide
viewport_margin
max_size_policy
```

`OverlayPlacementResolution` should record:

```text
anchor_id
anchor_rect
requested_side
requested_alignment
resolved_side
resolved_alignment
resolved_rect
clamped: bool
flipped: bool
shifted: bool
resized: bool
hidden_or_suppressed: bool
viewport_rect
collision_notes
```

The proof must be deterministic. Fallback order and clamp behavior must not depend on unordered maps or backend renderer side effects.

## Layer ordering model

Layer policy must be renderer-neutral and reportable:

```text
BaseContent
FloatingPanel
AnchoredPopup
Menu
Submenu
Tooltip
ModalLike
DiagnosticOverlay
```

Rules:

- ordinary controls render below overlays;
- menu/submenu chains preserve parent-before-child evidence;
- tooltip overlays do not steal focus unless explicitly declared later;
- modal-like layers may contain focus in proof, but app-specific modal lifecycle remains host-owned;
- persistent chrome/status overlays can share placement mechanics but must be distinguishable from dismissible overlay stack entries;
- layer ids must be stable within proof and replay, not global application ids.

## Focus and dismissal model

The reusable substrate must support:

```text
focus none
focus overlay on open
contain focus while open
return focus to anchor on close
return focus to previous focus target on close
Escape dismiss
outside pointer dismiss
explicit close dismiss
selection-intent dismiss
non-dismissable persistent overlay
host-owned dismissal seam
```

Dismissal evidence must record:

```text
request_id
active_layer_id
reason
input_sample_id
focus_before
focus_after
focus_return_target
outside_pointer_target
escape_key_seen
closed: bool
suppressed: bool
```

Outside-click dismissal and Escape dismissal must be reported as reusable UI evidence only. They must not execute product commands.

## Pointer capture interaction

Overlay proof must define how pointer capture interacts with overlay open/dismiss:

- captured pointer release inside anchor may complete open intent for click-triggered overlays;
- captured pointer release outside anchor may suppress open intent or record cancelled click evidence;
- outside pointer dismissal must not fire while the same pointer sequence is still captured by the opening anchor unless the policy explicitly allows it;
- dismissal evidence must identify whether the event was handled by capture, outside dismissal, or no-target routing.

## Keyboard navigation

Overlay content keyboard navigation evidence should cover:

```text
open by Enter/Space
open by semantic action
Escape dismiss
ArrowDown/ArrowUp menu or dropdown movement
ArrowRight/ArrowLeft submenu-ready movement evidence
Home/End optional edge movement evidence
Tab containment or escape depending focus policy
```

The proof records navigation intent and focus movement evidence. It does not execute product commands, mutate selected product state, or edit text.

## Disabled and suppressed behavior

If an anchor or declaration is disabled, unavailable, read-only-for-open, or lacks required capability, runtime must record suppression evidence instead of opening an overlay.

Required suppression evidence:

```text
anchor_id
request_kind
trigger
reason
input_sample_id
opened: false
host_commands_executed: 0
product_mutations: 0
```

## Inspection metadata

Catalog and inspection projection should expose declaration and observed evidence separately.

Declaration metadata:

```text
overlay.kind
overlay.trigger
overlay.anchor_role
overlay.placement_preference
overlay.layer_preference
overlay.dismiss_policy
overlay.focus_policy
overlay.keyboard_navigation
```

Observed runtime metadata:

```text
overlay.request_id
overlay.anchor_id
overlay.placement_resolution
overlay.layer_assignment
overlay.focus_evidence
overlay.dismissal_evidence
overlay.suppression_evidence
overlay.boundary_assertions
```

Inspection remains read-only. It must not become a UI Designer or authored UI editing path.

## Static proof and story proof shape

The proof fixture should mount base controls and overlay-capable declarations without product behavior:

```text
BaseControlsOverlayLayeringFixture
  controls:
    Button opens Popup
    ActionPrompt opens Menu-like overlay intent
    ColorPicker requests PickerPopup
    Dropdown-like fixture requests Dropdown
    Tooltip anchor opens Tooltip from hover/focus evidence
    Disabled anchor suppresses open
    Outside target records outside dismissal
  overlay declarations:
    popup declaration
    dropdown declaration
    menu declaration
    tooltip declaration
    picker popup declaration
    modal-like declaration
  scripts:
    positive overlay open/dismiss scripts
    negative/suppression scripts
  proof:
    OverlayLayeringVisualProof
    OverlayLayeringReport
    OverlayLayeringProofRenderFrame
```

The static frame must expose three regions:

```text
main view
  mounted anchor controls, overlay surfaces, layer markers, placement markers

inspector view
  selected anchor/declaration, request id, anchor id, placement, layer, focus, dismissal, keyboard, suppression facts

report view
  replay steps, input samples, overlay requests, placement resolutions, layer assignments, focus transitions, dismissal rows, no-bypass counters
```

Story proof must support deterministic replay and recorded live-log replay using the Phase 12A executable interaction standard. Replay/live modes may differ only by input source after normalization.

## Positive proof scenarios

Required positive scenarios:

- Button emits reusable popup open intent without executing product behavior.
- ActionPrompt emits menu-like open intent without executing product behavior.
- Dropdown-like request records anchor, placement, layer, keyboard navigation policy, and dismissal policy.
- Tooltip-like request records hover trigger evidence.
- Tooltip-like request records focus trigger evidence.
- Picker-like request records open intent, anchor, placement, focus policy, and dismissal policy without value mutation.
- Escape dismissal records dismissal evidence and focus return.
- Outside pointer dismissal records dismissal evidence and focus return.
- Pointer capture blocks accidental outside dismissal during the opening pointer sequence.
- Keyboard navigation through menu/dropdown content records focus/navigation evidence without command execution.
- Static proof frame exposes main, inspector, and report evidence.

## Negative proof scenarios

Required negative scenarios:

- Disabled anchor suppresses overlay open.
- Missing anchor suppresses overlay placement.
- Invalid anchor identity is rejected or recorded as no-target evidence.
- Missing placement policy records diagnostic evidence.
- Viewport clamp failure suppresses or resizes according to declared collision policy.
- Escape with no open overlay records no dismissal.
- Outside pointer with no open overlay records no dismissal.
- Outside pointer inside the active overlay does not dismiss.
- Tooltip request does not steal focus.
- Modal-like proof does not execute app-specific modal behavior.
- Overlay request never executes host commands.
- Overlay request never mutates product/editor/game state.
- Overlay request never performs text editing.
- Overlay proof never creates UI Gallery, UI Designer, authored UI editing, plugin framework, `foundation/meta`, generic plugin primitives, or Workbench/provider redesign.

## No-bypass counters

`OverlayBoundaryAssertions` must include at least:

```text
host_commands_executed: 0
product_mutations: 0
text_edit_transactions: 0
app_specific_modal_operations: 0
authored_ui_edits: 0
plugin_framework_operations: 0
```

Overlay-specific counters should include:

```text
overlay_open_requests
overlay_opened
overlay_suppressed
overlay_dismissed_by_escape
overlay_dismissed_by_outside_pointer
overlay_dismissed_by_focus_policy
placement_clamped
placement_flipped
placement_shifted
focus_returned
```

No-bypass assertions must fail if overlay evidence appears without a declaration, without normalized input evidence, or without runtime report evidence.

## Public API vocabulary

Candidate durable public names:

```text
ControlOverlayDescriptor
ControlOverlayRequirement
ControlOverlayKind
ControlOverlayTrigger
ControlOverlayDismissPolicy
ControlOverlayPlacementPreference
ControlOverlayLayerPreference
ControlOverlaySupportSummary

OverlayAnchorId
OverlayRequestId
OverlayLayerId
OverlayDeclaration
OverlayOpenIntent
OverlayPlacementRequest
OverlayPlacementResolution
OverlayLayerAssignment
OverlayDismissalPolicy
OverlayDismissalEvidence
OverlayFocusPolicy
OverlayFocusEvidence
OverlayKeyboardNavigationEvidence
OverlayPointerCaptureEvidence
OverlaySuppressionEvidence
OverlayBoundaryAssertions
OverlayLayeringReport
OverlayLayeringVisualProof
OverlayLayeringProofRenderFrame

BASE_CONTROLS_OVERLAY_LAYERING_PROOF_ID
BASE_CONTROLS_EXECUTABLE_OVERLAY_LAYERING_STORY_ID
base_controls_overlay_layering_fixture
base_controls_overlay_layering_positive_script
base_controls_overlay_layering_negative_scripts
base_controls_overlay_layering_proof_frame
base_controls_executable_overlay_layering_story_session
base_controls_overlay_layering_expected_evidence
```

Names may change during implementation inspection, but they must remain durable and domain-shaped. Do not introduce `phase13_*` public names.

## Stable ids

Stable ids must be string-backed or typed ids that are deterministic in fixtures and reports:

```text
base-controls.overlay-layering.proof
base-controls.overlay-layering.story
anchor.button.popup
anchor.action-prompt.menu
anchor.dropdown.fixture
anchor.tooltip.hover
anchor.tooltip.focus
anchor.color-picker.picker-popup
layer.popup.primary
layer.menu.primary
layer.tooltip.primary
layer.modal-like.primary
step.open-popup.button
step.dismiss.escape
step.dismiss.outside-pointer
step.suppress.disabled-anchor
```

Implementation may choose typed wrappers, but report strings must remain readable and durable.

## Relationship to Phase 12 and 12A

Phase 12 provides the lower-tier substrate:

```text
control interaction descriptors
normalized input facts
descriptor-backed replay/report
InteractionVisualProof / InteractionProofFrame
UiFrame static mount validation
zero host-command/product-mutation/overlay/text-edit counters
```

Phase 12A provides the executable proof standard:

```text
InteractionStorySession
replay mode
live proof-host mode
normalized input log
semantic replay/live parity
static frame evidence
no-bypass assertions
```

Phase 13 must consume those mechanics. It must not create a parallel replay system, fake overlay state outside runtime evidence, or bypass normalized input.

## Relationship to later UI Designer / authored UI framework work

Later UI Designer and authored UI framework work may author overlay declarations, menu templates, tooltip content, modal-like flows, and app-specific command bindings. That later work must consume this reusable overlay substrate instead of redefining layer/focus/dismissal semantics.

This phase does not create UI Designer product surfaces, authored UI editing, command binding editors, or Gallery exposure. It only designs the reusable substrate and proof envelope they should later consume.

## Implementation-scope gate

Rust implementation is blocked until this section is accepted and copied into active implementation planning.

### Exact owner crates and files

Allowed implementation files:

```text
domain/ui/ui_controls/src/overlay.rs
domain/ui/ui_controls/src/package/descriptor.rs
domain/ui/ui_controls/src/package/validation.rs
domain/ui/ui_controls/src/authoring/mod.rs
domain/ui/ui_controls/src/base_control/compiler.rs
domain/ui/ui_controls/src/base_control/lowering/overlay.rs
domain/ui/ui_controls/src/base_control/lowering/inspection.rs
domain/ui/ui_controls/src/catalog/entry.rs
domain/ui/ui_controls/src/lib.rs

domain/ui/ui_input/src/facts.rs
domain/ui/ui_input/src/event.rs
domain/ui/ui_input/src/lib.rs

domain/ui/ui_runtime/Cargo.toml
domain/ui/ui_runtime/src/input/overlay_layering.rs
domain/ui/ui_runtime/src/input/overlay_layering_fixture.rs
domain/ui/ui_runtime/src/input/overlay_layering_visual_frame.rs
domain/ui/ui_runtime/src/input/overlay_layering_story_session.rs
domain/ui/ui_runtime/src/input/mod.rs
domain/ui/ui_runtime/tests/overlay_layering_report.rs
domain/ui/ui_runtime/tests/executable_overlay_layering_story.rs

domain/ui/ui_static_mount/Cargo.toml
domain/ui/ui_static_mount/src/lib.rs
domain/ui/ui_static_mount/tests/base_controls_overlay_layering_static_mount.rs

domain/ui/ui_story/src/workflow/builtin.rs
domain/ui/ui_story/src/workflow/mod.rs
domain/ui/ui_story/tests/executable_overlay_layering_workflow.rs

apps/runenwerk_editor/Cargo.toml
apps/runenwerk_editor/src/editor_features/mod.rs
apps/runenwerk_editor/src/editor_features/base_controls_overlay_layering_proof_host.rs
apps/runenwerk_editor/tests/base_controls_overlay_layering_proof_host.rs
```

Any required file outside this list is a stop condition unless the implementation scope is revised before code.

### Exact non-goals

Implementation must not add:

- product-facing UI Gallery;
- UI Designer;
- authored UI editing;
- product/editor/game command execution;
- product/editor/game mutation;
- app-specific modal behavior;
- full text editing;
- caret/selection/text buffer/IME/clipboard/undo-redo work;
- dynamic external plugin framework;
- `foundation/meta`;
- broad shared plugin primitives;
- broad Workbench/provider redesign;
- backend renderer behavior.

### Validation commands

Minimum validation gate:

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
cargo test -p ui_story executable_overlay_layering_workflow
cargo test -p runenwerk_editor base_controls_overlay_layering_proof_host
python tools/docs/validate_docs.py
git diff --check
```

If an editor proof adapter is not needed, the implementation PR must explicitly remove the editor check/test from the gate and explain why no adapter is required.

### Required proof scenarios

Implementation must prove:

- Button popup open intent;
- ActionPrompt menu-like open intent;
- Dropdown request anchor/placement/layer/dismissal policy;
- Tooltip hover trigger evidence;
- Tooltip focus trigger evidence;
- Picker popup open intent without value mutation;
- Escape dismissal with focus return;
- outside pointer dismissal with focus return;
- pointer capture interaction during opening sequence;
- keyboard navigation through overlay content;
- static frame main/inspector/report evidence;
- replay/report evidence;
- executable replay/live parity if live proof-host mode is included.

### Required negative scenarios

Implementation must prove:

- disabled anchor suppresses open;
- missing anchor prevents placement;
- invalid anchor records diagnostic/no-target evidence;
- outside pointer inside overlay does not dismiss;
- Escape without open overlay does not dismiss;
- tooltip does not steal focus;
- modal-like proof does not execute app-specific modal behavior;
- overlay request does not execute host commands;
- overlay request does not mutate product/editor/game state;
- overlay request does not perform text editing;
- overlay proof does not create Gallery, Designer, authored editing, plugin framework, `foundation/meta`, generic plugin primitives, or Workbench/provider redesign.

### Expected evidence contract

Reports must include:

```text
input_sample_id
source_control_id
anchor_id
request_id
overlay_kind
trigger
open_intent
placement_request
placement_resolution
layer_assignment
focus_policy
focus_evidence
dismissal_policy
dismissal_evidence
pointer_capture_evidence
keyboard_navigation_evidence
suppression_evidence
boundary_assertions
static_frame_mount_verdict
```

### No-bypass assertions

Implementation must assert:

```text
host_commands_executed == 0
product_mutations == 0
text_edit_transactions == 0
app_specific_modal_operations == 0
authored_ui_edits == 0
plugin_framework_operations == 0
```

It must also assert that every overlay open/dismiss/suppression report row has normalized input evidence and a declared reusable overlay requirement.

### Stop conditions

Stop and redesign if implementation requires:

- command execution in generic UI;
- product/editor/game mutation in generic UI;
- app-specific modal lifecycle in generic UI;
- overlay behavior in `ui_controls` beyond declarations;
- input semantics in `ui_input` beyond facts;
- story registry/discovery moved into `ui_runtime`;
- editor shell surface registration or Workbench provider redesign;
- UI Gallery or UI Designer product surface;
- full text editing;
- dynamic external plugin framework;
- `foundation/meta`;
- generic plugin primitives;
- phase-shaped public API names;
- compatibility-only aliases or shims.

## Acceptance criteria for this design

This design is accepted when it records:

- owner crates/modules;
- allowed files;
- forbidden files;
- public API vocabulary;
- stable ids;
- proof fixture shape;
- positive scenarios;
- negative scenarios;
- no-bypass counters;
- validation gate;
- stop conditions;
- relationship to Phase 12;
- relationship to later UI Designer / authored UI framework work.
