---
title: UI Component Platform Generic Interaction Design
description: Completed Phase 12 reference for reusable generic interaction semantics across ui_controls, ui_input, ui_runtime, host-owned policy, visible proof, and later text-editing readiness.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-06-29
related_docs:
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
  - ./ui-component-platform-input-gesture-device-design.md
  - ./editor-ui-runtime-v2-and-interaction-formation-design.md
  - ./ui-component-platform-base-control-packages-design.md
---

# UI Component Platform Generic Interaction Design

## Status

This is the completed Phase 12 design for `PT-UI-COMPONENT-PLATFORM-012`.

Lifecycle state: `completed`.

PR #43 is the implementation PR for generic reusable interaction behavior for descriptor-backed controls. The design remains the owner-boundary reference for package-backed declarations, normalized input replay, renderer-neutral visible proof, negative proof cases, and no-bypass assertions.

## Decision summary

Phase 12 should define generic reusable interaction semantics for descriptor-backed controls without moving host policy, product behavior, OS input collection, game/world input policy, or app/editor/game state changes into `ui_controls`.

Phase 12 should make later text-editing controls possible by proving focus, keyboard, text-intent, and interaction-state seams. It should not implement a full editable text control yet.

Phase 12 must have a runtime-visible proof. The phase should not close with type definitions only. The implementation proof should replay deterministic input against mounted base controls, visibly show interaction state changes in a gallery/story scenario, and emit an auditable interaction report.

This Phase 12 design owns the component-platform boundary between reusable control declarations, normalized input substrate facts, runtime interaction formation, and host-owned product behavior. The existing Phase 5 input/gesture/device design covers declarative input capability facts, and the existing editor Interaction V2 design covers editor-retained runtime interaction formation.

## Problem

The UI track now has descriptor-backed, catalog-visible, package-quality base controls. Generic reusable interaction behavior is the purpose of Phase 12, and PR #43 records the first completed reusable interaction implementation path.

Phase 12 must define how reusable controls declare interaction needs and how runtime layers produce interaction facts without making `ui_controls` collect input, execute host commands, mutate app/editor/game state, or own product policy.

Text editing is a downstream consumer of the same interaction substrate, but it is larger than the first reusable interaction proof. It adds caret geometry, selection, text input composition, clipboard, undo/redo, validation, scrolling, and text layout requirements that should not be forced into the first interaction implementation.

The runtime proof also needs a no-bypass rule. A gallery demo that hard-codes hover, pressed, focus, or activation state outside the descriptor/catalog/runtime path would not prove the component platform boundary.

## Goals

- Define reusable control interaction declarations.
- Define semantic interaction states needed by reusable controls.
- Define control-level interaction requirements as descriptor/catalog/inspection facts.
- Define the seam between normalized input facts and runtime interaction formation.
- Define how runtime interaction facts/events are produced for reusable controls.
- Define focus, keyboard, and text-intent seams that later editable text controls can consume.
- Define a deterministic gallery/story interaction proof.
- Define an auditable `InteractionFormationReport`-style evidence shape.
- Keep app/editor/game command handling and product state changes outside reusable controls.
- Keep the implementation path narrow enough for one later PR.

## Non-goals

Phase 12 does not implement:

- overlay, popup, dropdown, tooltip, or layering behavior;
- full text editing behavior, including caret geometry, selection, IME/composition, clipboard, undo/redo, validation, scrolling, or text layout;
- app/editor/game-specific command behavior;
- product state changes;
- backend renderer behavior;
- broad shared plugin framework extraction;
- `foundation/meta`;
- generic input framework extraction beyond owner-local needs;
- Phase 11 base-control rewrites;
- Phase 13 work.

## Owner map

### `ui_controls`

`ui_controls` may own:

- reusable control interaction declarations;
- semantic interaction states needed by reusable controls;
- control-level interaction requirements;
- descriptor/catalog/inspection facts for interaction support;
- control kernel hooks only as declarations/contracts, not host behavior.

`ui_controls` must not own OS input collection, runtime hit testing, routing policy, command handling, app/editor/game state changes, backend rendering, overlay/layer policy, or full text editing behavior.

### `ui_input` / input substrate

`ui_input` or the existing input substrate may own:

- normalized input packet vocabulary;
- device and gesture facts;
- pointer, key, focus, and text-intent facts as reusable input data;
- runtime input sample formation.

It must not own reusable control semantics, app/editor/game command handling, product behavior, or full text editing behavior.

### `ui_runtime`

`ui_runtime` may own:

- runtime interaction formation over emitted or mounted UI;
- resolving normalized input facts against runtime UI structure;
- producing interaction facts/events for reusable controls;
- runtime frame/session evidence for interaction behavior.

It must not own static control package truth, app/editor/game command behavior, product state changes, or full text editing behavior.

### Hosts, apps, editor, and game layers

Hosts, apps, editor, and game layers own:

- OS/window input collection;
- routing policy;
- command handling;
- app/editor/game state changes;
- game/world input policy;
- product-specific behavior.

They may consume reusable interaction facts and choose product behavior, but they must not redefine reusable control interaction semantics.

## Text editing boundary

Text editing should be designed as a later consumer of Phase 12, not as part of the first Phase 12 implementation proof.

Phase 12 should provide the reusable substrate that text editing will need:

- focus ownership facts;
- keyboard input facts;
- text-intent input facts;
- enabled, hovered, pressed, active, focused, and disabled interaction states;
- runtime interaction events tied to concrete emitted or mounted UI structure;
- descriptor/catalog/inspection visibility for interaction support.

A later text-editing phase should add the specialized editor contract:

- editable text control declarations;
- caret geometry and movement;
- selection ranges;
- text insertion/deletion transactions;
- IME/composition;
- clipboard behavior;
- undo/redo integration;
- single-line and multi-line behavior;
- password/secret masking behavior;
- validation and commit/cancel semantics;
- scrolling and text layout integration.

This keeps Phase 12 useful for Button, ActionPrompt, ListView, TreeView, TableView, and later editable controls without collapsing the first generic interaction proof into a full text editor.

## Target architecture

```text
ControlContribution / ControlDef
  -> reusable interaction declarations in ui_controls
  -> descriptor/catalog/inspection facts

Normalized input packets and device facts in ui_input
  -> runtime input samples
  -> ui_runtime resolution against emitted or mounted UI structure
  -> reusable interaction facts/events
  -> host/app/editor/game routing and product behavior outside ui_controls
```

The control package says what interaction support a control requires. The input substrate says what input data exists. The runtime says what happened against a concrete emitted or mounted UI structure. The host decides what product behavior follows.

## Phase 12 proof contract

Phase 12 must prove reusable interaction through a deterministic mounted UI scenario, not through type definitions alone.

The proof should provide:

- a gallery/story scenario that mounts descriptor-backed base controls;
- a deterministic replay script of normalized input packets;
- visible state changes for reusable control states;
- an inspector or report panel that shows declarations, resolved targets, state transitions, facts, events, semantic outcomes, and suppressed input;
- tests that compare the replay result against expected interaction evidence;
- boundary assertions that host commands, product mutations, overlay behavior, and full text-editing transactions did not run.

The minimum proof pipeline is:

```text
ControlContribution / ControlDef
  -> catalog-visible interaction declarations
  -> mounted story/runtime UI structure
  -> deterministic input replay
  -> runtime target/focus resolution
  -> reusable interaction state transitions
  -> runtime facts/events/outcomes
  -> gallery-visible state/report
  -> host-owned behavior remains untouched
```

A proof that bypasses the catalog, descriptor, or runtime interaction path is invalid.

## Runtime-visible gallery result

The gallery/story result should show three layers at once:

```text
main view:
  mounted base controls with visible state markers

inspector view:
  selected control descriptor
  declared interaction requirements
  current interaction state set

report/event view:
  replay steps
  target resolution
  state transitions
  runtime interaction facts
  runtime interaction events
  semantic outcomes
  suppressed or ignored input
  boundary assertions
```

The gallery must not execute product commands. A Button activation may emit an activation outcome, but it must not invoke app/editor/game behavior as part of the reusable control proof.

PR #43 adds a renderer-neutral visible proof path in `domain/ui/ui_runtime/src/input/generic_interaction.rs` through `InteractionVisualProof`, `InteractionVisualMainView`, `InteractionVisualControl`, `InteractionVisualMarker`, `InteractionInspectorView`, `InteractionReportView`, `InteractionVisibleState`, and `InteractionProofFrame`.

That proof path is formed from compiled base-control package descriptors plus deterministic replay reports. It exposes a main view, inspector view, and report/event view without creating a product UI, overlay layer, popup, or broad gallery framework. Existing gallery/static-mount infrastructure can render the proof model later, but Phase 12 completion does not depend on a replay-only substitute.

## Base-control interaction matrix

Phase 12 should assign each Phase 11 base control a proof role.

| Control | Interaction role | Proof expectation |
| --- | --- | --- |
| Label | inert/read-only display | Pointer and keyboard input do not produce activation, mutation, or focus behavior unless explicitly declared later. |
| Button | hover, press, focus, keyboard activation | Pointer click and Enter/Space produce reusable activation outcome without host command execution. |
| ActionPrompt | focus and action intent | Keyboard activation produces action intent without executing product behavior. |
| InspectorField | focus and text-intent probe | Focus and text-intent facts are visible; no caret, selection, text buffer mutation, clipboard, or undo/redo behavior is introduced. |
| ColorPicker | focus and activation intent only | Interaction can request open/change behavior, but actual picker overlays, popups, and value mutation remain out of scope. |
| ListView | focused/active item intent | Keyboard navigation produces active-item interaction facts or outcomes, not product selection mutation. |
| TreeView | focused node and expand/collapse intent | Expand/collapse intent can be emitted; product tree data mutation remains host-owned. |
| TableView | focused row/cell and active item intent | Row/cell navigation facts are visible; product data selection and edits remain host-owned. |
| Disabled control fixture | suppressed interaction | Input against disabled controls emits ignored or suppressed interaction evidence. |

## Interaction state semantics

The first implementation pass should define a small canonical state set before adding control-specific behavior.

Required candidate states:

```text
enabled
disabled
read_only
hovered
pressed
active
focused
focus_visible
captured
suppressed
```

Required precedence and interpretation rules:

- `disabled` suppresses activation and mutable interaction outcomes.
- `read_only` may still allow focus, inspection, and copy-like future behavior, but not mutation.
- `focused` does not imply `active`.
- `active` does not imply product mutation.
- `focus_visible` is a keyboard/navigation-visible focus state, not merely any focus.
- `pressed` requires pointer ownership, capture, or an equivalent runtime rule.
- `suppressed` means input was understood and intentionally ignored because of the control or runtime state.

## Facts, events, outcomes, and commands

Phase 12 should keep these concepts distinct:

```text
InteractionFact:
  current or sampled runtime truth, such as ButtonA is hovered.

InteractionEvent:
  an edge or change produced by input resolution, such as ButtonA pointer_press began.

InteractionOutcome:
  reusable semantic intent a host may consume, such as activation_requested.

HostCommand:
  app/editor/game behavior, such as delete entity, open panel, mutate world, or edit product data.
```

Phase 12 may produce facts, events, and outcomes. Phase 12 must not execute host commands from `ui_controls`.

## Deterministic replay scenario

The implementation proof should include a deterministic replay scenario equivalent to:

```text
1. move pointer to Button
2. press primary pointer button
3. release primary pointer button
4. tab focus to ActionPrompt
5. press Enter or Space
6. move active item in ListView
7. move focus or active node in TreeView
8. move focus or active row/cell in TableView
9. send keyboard/text-intent facts to InspectorField text-intent probe
10. click or keyboard-activate a disabled control fixture
11. send input outside all controls
```

The expected report should include:

```text
hovered: Button
pressed: Button
activation_requested: Button
focused: ActionPrompt
activation_requested: ActionPrompt
active_item_intent: ListView
node_intent: TreeView
cell_or_row_intent: TableView
text_intent_seen: InspectorField
suppressed: DisabledControl
no_target: OutsideInput
host_commands_executed: 0
product_mutations: 0
overlay_events: 0
text_edit_transactions: 0
```

## Focus and keyboard scope

Phase 12 needs a minimal reusable focus and keyboard scope.

It should cover:

- focused control identity;
- focus traversal order for the proof fixture;
- keyboard activation target;
- focus loss or no-target behavior;
- disabled-control skip or suppression behavior;
- `focus_visible` state for keyboard-driven focus;
- arrow-key navigation intent for list, tree, and table controls.

It should not cover full accessibility, OS/window input collection, app/editor/game command routing, or text editing.

## Pointer capture and cancellation scope

Button-like controls need explicit pointer-state rules.

The first implementation proof should cover:

- press inside and release inside produces activation outcome;
- press inside and release outside cancels or suppresses activation;
- pointer cancel clears pressed/captured state;
- disabled-while-pressed cancels or suppresses activation;
- input outside all controls produces no target or ignored evidence.

## Text-intent probe

Phase 12 should include a text-intent probe, not an editable text control.

The proof should show:

- InspectorField or a dedicated fixture receives focus;
- keyboard facts and text-intent facts are represented at the input/runtime seam;
- the report shows `text_intent_seen` or equivalent evidence.

The proof must not add:

- caret geometry;
- selection ranges;
- text insertion/deletion transactions;
- IME/composition behavior;
- clipboard behavior;
- undo/redo integration;
- mutable text buffer ownership;
- text layout or scrolling integration.

## Negative proof scenarios

The implementation proof should include negative cases, not only happy paths.

Required negative scenarios:

- clicking a disabled control produces suppressed evidence;
- keyboard activation without focus is ignored or has no target;
- pointer leaves or cancels a pressed control without producing activation;
- input outside all controls produces no target evidence;
- text intent against a non-text control is ignored or suppressed;
- activation outcomes do not execute host commands;
- list/tree/table navigation does not mutate product data;
- overlay, popup, dropdown, tooltip, and layering behavior remain absent;
- full text editing remains absent.

## Interaction formation report

The proof should produce an auditable report shape. The implementation may refine names, but it should preserve these responsibilities:

```text
InteractionFormationReport
  replay_id
  mounted_story_id
  control_descriptors
  input_steps
  target_resolution
  focus_resolution
  state_transitions
  runtime_facts
  runtime_events
  semantic_outcomes
  suppressed_events
  no_target_events
  boundary_assertions
```

Boundary assertions should include:

```text
host_commands_executed: 0
product_mutations: 0
overlay_events: 0
text_edit_transactions: 0
```

## Candidate vocabulary

The implementation pass may refine names, but the responsibilities should remain stable:

```text
ControlInteractionDescriptor
ControlInteractionRequirement
ControlInteractionState
ControlInteractionStateSet
ControlInteractionTrigger
ControlInteractionOutcome
ControlInteractionSupportSummary
ControlInteractionInspectionFact
RuntimeInteractionFact
RuntimeControlInteractionEvent
InteractionReplayScript
InteractionReplayStep
InteractionReplayReport
InteractionFormationReport
```

Do not introduce a universal interaction framework or shared plugin primitive during Phase 12.

## Implemented write scope

PR #43 touched the minimum owner files needed to prove the contract:

```text
domain/ui/ui_controls/src/interaction.rs
domain/ui/ui_controls/src/package/descriptor.rs
domain/ui/ui_controls/src/package/validation.rs
domain/ui/ui_controls/src/authoring/mod.rs
domain/ui/ui_controls/src/base_control/compiler.rs
domain/ui/ui_controls/src/base_control/lowering/interaction.rs
domain/ui/ui_controls/src/base_control/lowering/inspection.rs
domain/ui/ui_controls/src/catalog/entry.rs
domain/ui/ui_controls/src/lib.rs
domain/ui/ui_controls/tests/control_interaction_contract.rs
domain/ui/ui_controls/tests/control_interaction_catalog_contract.rs
domain/ui/ui_input/src/facts.rs
domain/ui/ui_runtime/src/input/generic_interaction.rs
domain/ui/ui_runtime/tests/interaction_replay_report.rs
docs-site/src/content/docs/workspace/planning/
docs-site/src/content/docs/design/active/ui-component-platform-generic-interaction-design.md
docs-site/src/content/docs/reports/closeouts/pt-ui-component-platform-012-generic-interaction-closeout.md
```

The visible proof path is renderer-neutral in `ui_runtime` and does not add a product UI or generic gallery/story framework.

The implementation PR did not rewrite Phase 11 base controls. Additive interaction declarations attach through the existing contribution/lowering path and preserve the Phase 11 ownership model.

The implementation PR did not touch app/editor/game command paths.

## Acceptance criteria

Phase 12 is implementation-ready only when the design and planning records answer:

- exact owner files and crates;
- exact public concepts to add or extend;
- exact gallery/story proof fixture path;
- exact deterministic replay/report proof path;
- how `ui_controls` declarations reference or summarize `ui_input` facts without owning input substrate behavior;
- how `ui_runtime` produces interaction facts/events without owning host product behavior;
- how descriptor/catalog/inspection output exposes interaction support read-only;
- how base controls remain package-quality without Phase 11 rewrites;
- how runtime mount eligibility remains controlled by explicit proof gates;
- how app/editor/game command behavior stays outside reusable controls;
- how focus, keyboard, and text-intent seams remain compatible with a later text-editing phase;
- how Phase 13 overlays/layering and later full text editing remain blocked.

Phase 12 implementation is complete only when:

- reusable interaction declarations exist and are catalog/inspection-visible;
- normalized input facts are resolved through the proper input/runtime owners;
- focus, keyboard, and text-intent facts are represented at the correct substrate boundary;
- runtime interaction facts/events can be formed for reusable controls;
- the gallery/story proof visibly shows hover, press, focus, active, disabled/suppressed, and text-intent-probe behavior;
- the deterministic replay report records target resolution, state transitions, facts, events, outcomes, suppressed/no-target evidence, and boundary assertions;
- no host-specific command behavior or product state change is introduced in `ui_controls`;
- no overlay/layering/full-text-editing behavior is introduced;
- focused tests prove the owner boundary and no-bypass rule.

## Validation gate

Expected validation for this planning patch:

```text
python3 tools/docs/validate_docs.py
git diff --check
```

Expected validation for the later implementation PR:

```text
cargo fmt --all --check
cargo check -p ui_controls
cargo check -p ui_input
cargo check -p ui_runtime
cargo test -p ui_controls control_interaction
cargo test -p ui_controls control_catalog
cargo test -p ui_controls base_control
cargo test -p ui_input input
cargo test -p ui_runtime interaction
cargo test -p ui_runtime --test interaction_replay_report
python3 tools/docs/validate_docs.py
git diff --check
```

If exact test filter names differ after implementation, the PR must record the equivalent focused tests and explain the mapping.

## Stop conditions

Stop and redesign if Phase 12 requires:

- app/editor/game command behavior in `ui_controls`;
- OS/window input collection in `ui_controls`;
- routing policy ownership in `ui_controls`;
- product state changes in reusable control code;
- backend renderer behavior;
- overlay, popup, dropdown, tooltip, or layering behavior;
- full text editing behavior;
- broad shared plugin framework extraction;
- `foundation/meta`;
- generic input framework extraction beyond owner-local needs;
- Phase 11 base-control rewrites;
- Phase 13 work.

Also stop and redesign if the proof requires:

- demo-only interaction state that bypasses descriptors, catalog facts, or runtime formation;
- direct activation-to-host-command execution;
- list, tree, or table navigation that mutates product data;
- text-intent proof that requires caret, selection, text buffer, clipboard, undo/redo, or text layout ownership;
- overlay/layer ordering or popup behavior;
- broad story/gallery framework extraction.

## Closeout requirements

A later Phase 12 implementation closeout must record:

- final implemented public vocabulary;
- final owner files and crates touched;
- gallery/story proof path;
- deterministic replay/report proof path;
- base-control interaction matrix coverage;
- negative proof coverage;
- boundary assertion results;
- validation commands and results;
- any intentionally deferred overlay, popup, text-editing, accessibility, or product-command behavior;
- follow-up planning target for Phase 13 and later text editing.

## Relationship to current work

Phase 11 is complete and provides the descriptor-backed base-control package inventory that Phase 12 can reason about.

Phase 12 is complete in PR #43 with package-backed interaction descriptors, catalog/inspection projection, normalized input facts, descriptor-driven runtime replay/report, renderer-neutral visible proof, negative proof cases, read-only text-intent probe behavior, and no-bypass assertions.

Phase 13 remains overlay/popup/layering. Full text editing remains later, but it must consume the interaction substrate shaped by Phase 12 rather than bypassing it.
