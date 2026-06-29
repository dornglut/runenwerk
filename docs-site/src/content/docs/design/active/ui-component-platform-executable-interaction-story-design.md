---
title: UI Component Platform Executable Interaction Story Design
description: Proposed Tier 5 proof design for replay/live parity, live gallery proof hosts, static frame evidence, and no-bypass reusable UI interaction validation.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-06-29
related_docs:
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
  - ../../workspace/planning/decision-register.md
  - ./ui-component-platform-generic-interaction-design.md
  - ./ui-component-platform-story-proof-envelope-design.md
  - ./ui-component-platform-input-gesture-device-design.md
  - ./editor-ui-runtime-v2-and-interaction-formation-design.md
---

# UI Component Platform Executable Interaction Story Design

## Status

Lifecycle state: `proposed-design`.

Planning ID: `PT-UI-COMPONENT-PLATFORM-012A-PLANNING`.

This is a design intake for the Tier 5 proof standard for reusable UI interaction work. It does not authorize implementation. Active implementation requires exact owner files, scope, validation envelope, evidence expectation, and stop conditions in `active-work.md`.

This design intentionally follows the existing workflow rule that architecture acceptance is not implementation authorization.

## Decision summary

Reusable UI interaction work should not close on static proof alone when the claim is live interaction. The long-term standard should be an **Executable UI Interaction Story**: one story definition that can run in deterministic replay mode and live gallery/proof-host mode while preserving the same normalized input stream, runtime interaction formation path, report shape, visual proof shape, static frame artifact, and no-bypass boundary assertions.

The key invariant is:

```text
Replay mode and live mode may differ only by input source.
Everything after NormalizedInputSample must be shared.
```

This keeps the proof stronger than a static marker frame and safer than a fake live demo. Gallery, Workbench, UI Designer, and later product hosts may display or drive the proof host, but they must not redefine reusable control semantics.

## Problem

`PT-UI-COMPONENT-PLATFORM-012` added package-backed generic interaction descriptors, normalized input facts, deterministic replay/report evidence, renderer-neutral visible proof, and static `UiFrame` mount validation. That is valuable lower-tier evidence, but it does not provide a live gallery/proof-host surface where real pointer/key input visibly updates Button hover, pressed, captured, focus, activation, suppression, no-target, list/tree/table intent, or text-intent probe evidence.

For reusable interaction phases, static proof is necessary but not sufficient. A live-only demo is also insufficient if it can bypass descriptors, catalog facts, normalized input, or runtime interaction formation.

The missing target is an executable proof story that supports both deterministic replay and live input while proving semantic parity between the two.

## Current reality

Current Phase 12 evidence provides:

```text
ControlPackageDescriptor::interaction_descriptors
ControlCatalogIndex / inspection interaction projection
NormalizedInputFact pointer/keyboard/focus/semantic/text-intent facts
MountedInteractionFixture
InteractionReplayScript
InteractionFormationReport
InteractionVisualProof
InteractionProofRenderFrame
UiStaticMountReport::from_frame
```

That proof path is Tier 1 through Tier 3:

```text
Tier 1 — Contract proof
Tier 2 — Deterministic replay proof
Tier 3 — Static visual proof
```

It is not yet Tier 4 or Tier 5:

```text
Tier 4 — Live proof host
Tier 5 — Executable interaction story with replay/live semantic parity
```

Existing Phase 3 story-proof authority says `ui_story` owns story manifests, workflow execution, evidence, reports, diagnostics, and mount verdicts. The component platform may consume story proof; it must not create a parallel control-specific story runner. This design keeps that boundary.

## Goals

- Define Tier 5 as the default proof standard for reusable UI interaction phases.
- Preserve the current Phase 12 contract/replay/static evidence as lower-tier assets.
- Add a clean owner split for executable interaction stories without moving story execution into `ui_controls`.
- Define replay/live semantic parity without requiring pixel-perfect visual parity.
- Define a live gallery/proof-host path that uses real pointer/key/focus/text-intent input.
- Ensure live input is converted into `NormalizedInputSample` before reusable interaction formation.
- Ensure replay and live modes share the same runtime interaction formation path after normalized input.
- Keep host command execution, product mutation, overlays/layering, and full text editing out of this proof.
- Leave seams for future drag/drop, multi-pointer, focus scopes, shortcuts, text editing, overlays, scrolling, docking, accessibility, animation, and regression artifacts.

## Non-goals

This design does not implement or authorize:

- product-facing app/editor/game command dispatch;
- product/app/editor/game state mutation;
- overlay, popup, dropdown, tooltip, modal, or layer behavior;
- full text editing, caret, selection, IME/composition, clipboard, undo/redo, validation, text buffer mutation, or text layout ownership;
- drag/drop behavior;
- docking or workspace surface behavior;
- accessibility tree implementation;
- animation or transition systems;
- backend renderer behavior;
- a broad new story framework parallel to `ui_story`;
- shared plugin framework extraction;
- generic plugin primitives;
- `foundation/meta`.

## Vocabulary

```text
Proof tier
  The evidence level required by a phase or story.

Executable UI Interaction Story
  A reusable story definition that can run deterministic replay and live proof-host modes through the same interaction pipeline.

Replay mode
  Scripted normalized input samples drive the story deterministically.

Live mode
  Actual gallery/proof-host pointer/key/focus/text-intent input is normalized, logged, and applied to the same runtime session path.

Semantic Replay/Live Parity
  A comparison proving that a live input log replayed deterministically produces equivalent semantic evidence.

Live Interaction Proof Host
  A gallery/proof-host integration that collects raw host input, normalizes it, feeds an interaction story session, and renders current proof state.

Interaction Story Session
  Runtime-owned execution state over a mounted interaction fixture, input log, formation report, and visual proof.

Expected Evidence
  Story-owned assertions for required markers, outcomes, suppressions, no-target events, final current states, and boundary counters.
```

## Proof tiers

Use these tiers for future UI planning records:

```text
Tier 1 — Contract proof
  descriptors, package validation, catalog, inspection

Tier 2 — Deterministic replay proof
  scripted normalized input -> runtime report/outcomes

Tier 3 — Static visual proof
  visual proof -> UiFrame -> static mount validation

Tier 4 — Live proof host
  real gallery/proof-host pointer/key input -> same runtime path -> visible state updates

Tier 5 — Executable interaction story
  shared story definition, replay mode, live mode, semantic replay/live parity,
  static frame artifact, and no-bypass boundary assertions
```

Default rule:

```text
If a phase claims reusable interaction behavior, require Tier 5 unless the active-work entry explicitly accepts a contract-only or static-only scope.
```

## Owner map

### `ui_controls`

May own:

- reusable control interaction declarations;
- package-backed interaction descriptors;
- catalog/inspection summaries;
- control story proof requirements as references or summaries when needed.

Must not own:

- story execution;
- live input sessions;
- raw host event collection;
- gallery presentation;
- product commands;
- product state mutation.

### `ui_input`

May own:

- normalized input fact vocabulary;
- pointer, keyboard, focus, semantic, and text-intent facts;
- future extensible fields for pointer id, device kind, modifiers, scroll delta, pressure, click count, and logical timestamp.

Must not own:

- reusable control semantics;
- story execution;
- product behavior;
- gallery rendering.

### `ui_runtime`

May own:

- mounted interaction fixtures or runtime interaction structure;
- `InteractionStorySession` execution mechanics;
- incremental `apply_sample` behavior;
- deterministic replay over the same session path;
- input log capture;
- `InteractionFormationReport` updates;
- `InteractionVisualProof` formation;
- `InteractionProofRenderFrame` projection.

Must not own:

- story registry and discovery authority;
- gallery/proof-host event loops;
- product command routing;
- product state mutation;
- overlay/layer creation;
- full text editing transactions.

### `ui_story`

Should own or remain the authority for:

- executable story identity and manifest concepts;
- story execution envelope;
- workflow reports and diagnostics;
- story evidence and expected-failure semantics;
- mount verdicts when applicable.

It must not absorb reusable control semantics from `ui_controls` or product behavior from apps/editors/games.

### Gallery / proof host

May own:

- visible proof host UI;
- raw host event collection;
- host-event-to-`NormalizedInputSample` adaptation;
- feeding samples into a runtime-owned interaction session;
- rendering the current proof frame or proof views;
- manual verification surface.

Must not own:

- activation semantics;
- fake hover/press/focus state independent of runtime proof;
- reusable descriptor interpretation;
- product commands;
- product state mutation.

### `ui_static_mount`

May own:

- static `UiFrame` mount validation;
- primitive/surface/order diagnostics.

Must not own:

- live interaction;
- story execution;
- control semantics.

### Product/editor/app layers

May later own:

- mapping semantic outcomes to commands;
- product state changes;
- overlay and popup behavior;
- text editing transactions;
- selection, command routing, undo/redo, persistence, and app-specific policy.

They are consumers of reusable interaction outcomes, not owners of reusable interaction semantics.

## Target architecture

```text
ui_controls
  ControlInteractionDescriptor
  package/catalog/inspection interaction facts

ui_input
  NormalizedInputSample
  pointer/key/focus/text-intent facts

ui_story
  Executable story identity, manifest/evidence/report envelope, diagnostics

ui_runtime
  InteractionStorySession
  mounted fixture resolution
  deterministic replay and live incremental apply
  InteractionFormationReport
  InteractionVisualProof
  InteractionProofRenderFrame

gallery/proof host
  raw host input -> NormalizedInputSample
  live session drive
  visible proof render

ui_static_mount
  UiFrame validation

product/editor/app
  later command/state behavior
```

## Replay workflow

```text
Executable UI Interaction Story
  -> replay script
  -> NormalizedInputSample stream
  -> InteractionStorySession::apply_sample
  -> InteractionFormationReport
  -> InteractionVisualProof
  -> InteractionProofRenderFrame
  -> UiStaticMountReport::from_frame
```

Replay mode proves deterministic regression behavior and can run in tests.

## Live workflow

```text
Executable UI Interaction Story
  -> Live Interaction Proof Host
  -> raw pointer/key/focus/text-intent event
  -> NormalizedInputSample
  -> InteractionStorySession::apply_sample
  -> current InteractionFormationReport
  -> current InteractionVisualProof
  -> current InteractionProofRenderFrame
  -> live gallery/proof-host redraw
```

The live host may display the proof. It must not mutate reusable interaction state directly.

## Semantic replay/live parity workflow

```text
Live input log
  -> deterministic replay mode
  -> replayed-live-log report
  -> compare with original live report
```

Compare:

- target resolution;
- focus resolution;
- state transitions;
- runtime facts;
- runtime events;
- semantic outcomes;
- suppressed events;
- no-target events;
- observed markers;
- final current states;
- boundary assertions.

Do not compare:

- wall-clock timing;
- raw OS event ids;
- pixel snapshots;
- exact primitive count after every step;
- text wrapping;
- animation interpolation.

## Candidate public concepts

Final names may differ after code inspection, but responsibilities should remain:

```text
UiProofTier
ExecutableUiInteractionStory
ExecutableUiInteractionStoryId
InteractionStoryExpectedEvidence
ExpectedInteractionMarker
ExpectedCurrentState
ExpectedInteractionOutcome
ExpectedSuppression
ExpectedNoTarget
InteractionStoryBoundaryPolicy
InteractionStorySession
InteractionStoryExecutionMode
InteractionStoryStepEvidence
InteractionStoryRunReport
InteractionReplayLiveParityReport
LiveInteractionProofHost
HostInteractionInputAdapter
```

Preferred split:

```text
ui_story or story/proof layer:
  ExecutableUiInteractionStory identity/definition/evidence requirement

ui_runtime:
  InteractionStorySession, report/proof formation, replay/live apply mechanics

gallery/proof host:
  LiveInteractionProofHost and host input adapter
```

Do not put the whole story framework into `ui_runtime`.

## Phase 12A story content

The first executable generic interaction story should mount:

```text
Button
Disabled Button
Inert Button
ActionPrompt
InspectorField
Read-only InspectorField
ListView
TreeView
TableView
Label
```

Required live interactions:

```text
hover Button
press Button
release Button inside
press Button then release outside
focus Button
keyboard activate Button
focus ActionPrompt and activate
focus List and navigate
focus Tree and navigate
focus Table and navigate
send text-intent to InspectorField
send text-intent to Read-only InspectorField
attempt text-intent on non-text control
press Disabled Button
click outside all controls
```

Required visible evidence:

```text
hovered
pressed
captured
focused
focus-visible
activation-requested
action-intent
list-active-item-intent
tree-node-intent
table-cell-or-row-intent
text-intent-probe
read-only-text-intent-probe
disabled
suppressed
no-target
```

Required final-state distinction:

```text
pressed is observed during press
pressed is not current after release
captured is observed during press
captured is not current after release
focused may remain current
disabled remains current
read-only remains current
```

## Future extension seams

Design for these without implementing them now:

- multi-pointer, touch, pen, pressure, click count, long press, wheel/scroll;
- drag start, drag update, drag cancel, drop intent, drop target resolution;
- focus scopes, roving focus, modal focus trap, surface focus, focus restoration;
- shortcut intent and later product command routing;
- editable text, caret, selection, IME, clipboard, undo/redo, text buffer mutation;
- dropdowns, popups, tooltips, modals, outside-click dismissal, overlay stacking;
- scroll containers, viewport clipping, virtual lists;
- tabs, splits, docking, floating panels, multi-window, surface routing;
- accessibility roles, names, states, keyboard navigation contracts, accessibility proof;
- hover/press/focus animations as separate animation evidence;
- recorded input logs, golden semantic reports, static frame artifacts, CI comparison.

## Alternatives considered

### Keep Phase 12 static proof as the final interaction gate

Rejected for future interaction phases. Static proof is valuable but does not prove live host integration.

### Add a direct live Button demo in Gallery

Rejected as the long-term standard. A live demo can bypass descriptors, normalized input, and runtime formation unless it is tied to replay/live parity.

### Put executable story definitions entirely in `ui_runtime`

Rejected as the preferred architecture. `ui_runtime` should own execution/session mechanics. Story identity, evidence envelope, and discovery should align with `ui_story` authority.

### Make product Gallery execute activation behavior now

Rejected. Product behavior belongs to later product/editor/app adoption, not reusable interaction proof.

## Risks

- Overbuilding a new story framework instead of extending/consuming `ui_story`.
- Gallery host faking hover/press state instead of feeding normalized input into runtime.
- Treating semantic parity as pixel parity and making live proof brittle.
- Expanding Phase 12A into overlays, text editing, docking, or product command behavior.
- Keeping static proof completion language without naming the stronger Tier 5 follow-up.

## Acceptance criteria

Phase 12A design is accepted only when it records:

- owner boundaries for `ui_controls`, `ui_input`, `ui_runtime`, `ui_story`, gallery/proof host, `ui_static_mount`, and product layers;
- proof tier vocabulary and default policy;
- replay/live parity rule;
- live proof-host responsibilities and no-bypass restrictions;
- semantic comparison requirements;
- expected Phase 12A story content;
- future extension seams;
- implementation gate and stop conditions.

Phase 12A implementation is complete only when:

- an executable generic interaction story exists;
- deterministic replay mode runs canonical scripts;
- live proof-host mode accepts actual pointer/key/focus/text-intent input;
- live mode records normalized input logs;
- recorded live input logs replay deterministically;
- replay/live semantic parity passes for outcomes, markers, current states, suppressions, no-target events, and boundaries;
- the proof host visibly updates Button hover, pressed/captured, focus, release-inside activation, release-outside cancellation/suppression, disabled suppression, no-target evidence, list/tree/table intents, and text-intent probes;
- static `UiFrame` mount validation remains green;
- host command, product mutation, overlay, and text-edit transaction counters remain zero.

## Implementation gate

Implementation is not authorized by this design intake.

Before implementation, active-work must name:

```text
exact owner files/crates
exact story/proof-host scope
host adapter location
runtime session API scope
validation envelope
evidence artifacts
manual live validation steps
stop conditions
```

Expected implementation validation should include focused tests equivalent to:

```text
cargo fmt --all --check
cargo check -p ui_controls
cargo check -p ui_input
cargo check -p ui_runtime
cargo check -p ui_static_mount
cargo test -p ui_runtime executable_interaction_story
cargo test -p ui_static_mount phase12_generic_interaction
cargo test -p <gallery_or_story_host_crate> phase12_live_gallery_interaction_story
python3 tools/docs/validate_docs.py
git diff --check
```

If a gallery/proof-host crate does not exist yet or uses a different name, the implementation PR must record the actual focused test mapping.

## Stop conditions

Stop and redesign if implementation requires:

- story execution moving into `ui_controls`;
- a parallel story framework that bypasses `ui_story`;
- live gallery state changes that do not pass through normalized input and runtime interaction formation;
- activation outcomes executing product commands;
- product/app/editor/game state mutation;
- overlay/popup/layering behavior;
- full text editing behavior;
- broad plugin framework extraction;
- `foundation/meta`;
- generic plugin primitives;
- pixel-perfect replay/live parity.

## Relationship to current work

`PT-UI-COMPONENT-PLATFORM-012` remains the lower-tier generic interaction implementation reference: package-backed descriptors, catalog/inspection projection, normalized input facts, deterministic replay/report, renderer-neutral visible proof, and static `UiFrame` mount evidence.

`PT-UI-COMPONENT-PLATFORM-012A` is the proposed follow-up planning focus that upgrades reusable interaction acceptance to Tier 5 before overlay/layering, text editing, or product adoption can claim live reusable interaction behavior.

Phase 13 overlays/layering and later text editing remain future work. They should consume the executable interaction story standard rather than bypass it.
