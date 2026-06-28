---
title: UI Component Platform Generic Interaction Design
description: Phase 12 planning intake for reusable generic interaction semantics across ui_controls, ui_input, ui_runtime, and host-owned policy.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-06-28
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

This is the Phase 12 planning design intake for `PT-UI-COMPONENT-PLATFORM-012-PLANNING`.

Lifecycle state: `active-planning`.

Phase 12 is intended to define and then enable implementation of generic reusable interaction behavior for descriptor-backed controls. This planning patch does not implement that behavior yet. It defines the owner boundaries, acceptance criteria, validation gate, stop conditions, and later implementation envelope that must be accepted before code changes begin.

## Decision summary

Phase 12 should define generic reusable interaction semantics for descriptor-backed controls without moving host policy, product behavior, OS input collection, game/world input policy, or app/editor/game state changes into `ui_controls`.

An additional Phase 12 design document is required. The existing Phase 5 input/gesture/device design covers declarative input capability facts. The existing editor Interaction V2 design covers editor-retained runtime interaction formation. Neither document by itself owns the component-platform boundary between reusable control declarations, normalized input substrate facts, runtime interaction formation, and host-owned product behavior.

## Problem

The UI track now has descriptor-backed, catalog-visible, package-quality base controls. Generic reusable interaction behavior is the purpose of Phase 12, but it has not been defined or implemented yet.

Phase 12 must define how reusable controls declare interaction needs and how runtime layers produce interaction facts without making `ui_controls` collect input, execute host commands, mutate app/editor/game state, or own product policy.

## Goals

- Define reusable control interaction declarations.
- Define semantic interaction states needed by reusable controls.
- Define control-level interaction requirements as descriptor/catalog/inspection facts.
- Define the seam between normalized input facts and runtime interaction formation.
- Define how runtime interaction facts/events are produced for reusable controls.
- Keep app/editor/game command handling and product state changes outside reusable controls.
- Keep the implementation path narrow enough for one later PR.

## Non-goals

Phase 12 does not implement:

- overlay, popup, dropdown, tooltip, or layering behavior;
- text editing;
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

`ui_controls` must not own OS input collection, runtime hit testing, routing policy, command handling, app/editor/game state changes, backend rendering, overlay/layer policy, or text editing behavior.

### `ui_input` / input substrate

`ui_input` or the existing input substrate may own:

- normalized input packet vocabulary;
- device and gesture facts;
- pointer, key, and focus facts as reusable input data;
- runtime input sample formation.

It must not own reusable control semantics, app/editor/game command handling, or product behavior.

### `ui_runtime`

`ui_runtime` may own:

- runtime interaction formation over emitted or mounted UI;
- resolving normalized input facts against runtime UI structure;
- producing interaction facts/events for reusable controls;
- runtime frame/session evidence for interaction behavior.

It must not own static control package truth, app/editor/game command behavior, or product state changes.

### Hosts, apps, editor, and game layers

Hosts, apps, editor, and game layers own:

- OS/window input collection;
- routing policy;
- command handling;
- app/editor/game state changes;
- game/world input policy;
- product-specific behavior.

They may consume reusable interaction facts and choose product behavior, but they must not redefine reusable control interaction semantics.

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
InteractionFormationReport
```

Do not introduce a universal interaction framework or shared plugin primitive during Phase 12.

## Later implementation write scope

A later implementation PR may touch only the minimum owner files needed to prove the contract:

```text
domain/ui/ui_controls/src/interaction.rs
domain/ui/ui_controls/src/package.rs
domain/ui/ui_controls/src/catalog.rs
domain/ui/ui_controls/src/base_control/compiler.rs
domain/ui/ui_controls/src/base_control/lowering/
domain/ui/ui_input/src/
domain/ui/ui_runtime/src/
domain/ui/ui_controls/tests/
domain/ui/ui_input/tests/
domain/ui/ui_runtime/tests/
docs-site/src/content/docs/workspace/planning/
docs-site/src/content/docs/design/active/ui-component-platform-generic-interaction-design.md
```

The implementation PR must not rewrite Phase 11 base controls. Additive interaction declarations may be attached through the existing contribution/lowering path only if they preserve the Phase 11 ownership model.

The implementation PR must not touch app/editor/game command paths except for optional tests or guards that prove product behavior remains outside reusable control semantics.

## Acceptance criteria

Phase 12 is implementation-ready only when the design and planning records answer:

- exact owner files and crates;
- exact public concepts to add or extend;
- how `ui_controls` declarations reference or summarize `ui_input` facts without owning input substrate behavior;
- how `ui_runtime` produces interaction facts/events without owning host product behavior;
- how descriptor/catalog/inspection output exposes interaction support read-only;
- how base controls remain package-quality without Phase 11 rewrites;
- how runtime mount eligibility remains controlled by explicit proof gates;
- how app/editor/game command behavior stays outside reusable controls;
- how Phase 13 overlays/layering and later text editing remain blocked.

Phase 12 implementation is complete only when:

- reusable interaction declarations exist and are catalog/inspection-visible;
- normalized input facts are resolved through the proper input/runtime owners;
- runtime interaction facts/events can be formed for reusable controls;
- no host-specific command behavior or product state change is introduced in `ui_controls`;
- no overlay/layering/text-editing behavior is introduced;
- focused tests prove the owner boundary.

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
- text editing behavior;
- broad shared plugin framework extraction;
- `foundation/meta`;
- generic input framework extraction beyond owner-local needs;
- Phase 11 base-control rewrites;
- Phase 13 work.

## Relationship to current work

Phase 11 is complete and provides the descriptor-backed base-control package inventory that Phase 12 can reason about.

Phase 12 is the active planning focus. It should prepare one narrow implementation PR for generic reusable interaction behavior, but this planning patch must not implement runtime interaction.

Phase 13 remains overlay/popup/layering. Text editing remains later.
