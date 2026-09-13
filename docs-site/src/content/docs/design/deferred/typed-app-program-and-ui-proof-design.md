---
title: Deferred Typed App Program And UI Proof Design
description: Deferred experimental model/action/reducer/effect/replay architecture for app-owned behavior using Runenwerk-local UI proof infrastructure.
status: deferred
owner: ui
layer: design
canonical: false
last_reviewed: 2026-09-13
related_docs:
  - ../../adr/accepted/0019-batteries-included-application-composition.md
  - ../../architecture/runenwerk-platform-architecture.md
  - ../../reports/investigations/typed-app-program-current-state-investigation.md
  - ../implemented/ui-program-architecture.md
  - ../implemented/ui-program-architecture-owner-map.md
  - ../archived/ui-framework-app-integration-direction-review.md
---

# Deferred Typed App Program And UI Proof Design

## Deferred disposition

This document preserves an unaccepted experimental architecture for typed
app-owned model/action/reducer/effect/replay semantics. It is intentionally
postponed and is **not** current Runenwerk application-composition authority or
implementation authorization.

Current durable product-composition law is owned by:

- [ADR 0019: Batteries-Included Application Composition](../../adr/accepted/0019-batteries-included-application-composition.md);
- [Runenwerk Platform Architecture](../../architecture/runenwerk-platform-architecture.md).

Those authorities keep `App` as the one live runtime composition root. This
deferred design must not be read as authorization for a second App runtime,
persistent app-program configuration authority, or a meta-framework beside
ordinary plugins/resources/configuration.

## Experimental model preserved

The proposal explored this explicit app-behavior shape:

```text
AppModelSnapshot
  -> AppViewProjection
  -> UiProgram / runtime artifact / UI output
  -> UiEventPacket
  -> RouteActionMap
  -> AppAction
  -> AppReducer
  -> AppEffectPlan
  -> host policy / effect execution
  -> next app model revision
  -> AppReplayTrace / AppProgramReport
```

Candidate vocabulary included:

```text
AppProgramId / AppProgramVersion
AppModelSnapshot / AppModelRevision
AppAction / AppActionCapability
RouteActionMap / RouteActionResolution
AppReducer / AppReducerOutcome
AppEffectPlan
AppViewProjection / AppViewProjectionReport
AppReplayTrace / AppProgramReport
```

The intended ownership split remains useful pressure:

- UI owns UI semantics and emits typed events/output facts;
- app/domain owners own model/action/reducer semantics;
- effect plans are inert proposals, not hidden execution;
- hosts own concrete mutation/effects and policy;
- renderer output is not product truth;
- ECS is not the default static app-model authority;
- route/schema/capability failures should be explicit and fail closed;
- deterministic replay/proof can be useful evidence.

These are preserved candidate semantics, not accepted public API.

## Why it is deferred

The proposal predates ADR 0019 and tried to answer a broader architectural
question before a concrete product consumer had proven that a durable
`AppProgram` layer was necessary.

Current Runenwerk already has the accepted `App` composition root plus current
Runenwerk-local UI source/program/runtime/host integration. A separate typed
app-program abstraction would add another durable layer and vocabulary. That
cost is not justified without a concrete consumer showing that ordinary
`App`/plugin composition and app-owned typed state/actions cannot express the
required behavior cleanly.

The earlier headless Counter proof and associated planning remain historical
pressure evidence. They do not activate this design.

## Reactivation conditions

Reactivate this design only through a new owning GitHub issue that re-derives the
boundary from current source and accepted authority and proves all of the
following:

1. A concrete Runenwerk consumer needs reusable model/action/reducer/effect/replay
   contracts beyond ordinary ADR-0019 `App`/plugin composition.
2. `App` remains the single live runtime composition root; any helper lowers to
   ordinary App/plugins/resources and does not become persistent parallel
   composition truth.
3. App/domain semantic ownership stays outside UI, renderer, generic ECS storage,
   and foundation utilities.
4. The proposal reuses current UI source/program/event/host contracts instead of
   duplicating them and names exact owner files, dependencies, validation, and
   stop conditions.
5. Any proposed shared extraction is justified by repeated structurally
   different consumers rather than the UI proof alone.
6. If `typed-app-program-counter-proof-design.md` is used as the proving
   consumer, issue #281 must first resolve and that Counter design must be
   separately re-reviewed against the resulting accepted architecture.

Until those gates are met, this document is design evidence only.

## Candidate proof obligations if reactivated

A future proof should retain the strongest useful requirements from the original
proposal:

```text
unknown route -> explicit rejection
schema mismatch -> explicit rejection
invalid payload -> explicit rejection
missing capability -> explicit rejection
reducer rejection/failure -> explicit diagnostics
projection failure -> explicit diagnostics
rejected action -> no app/domain mutation
replay step -> before/input/resolution/outcome/effects/projection/after evidence
```

Callbacks, convenience builders, or visual authoring may later exist, but durable
behavior contracts must remain typed and inspectable if this architecture is
ever accepted.

## Non-authorizations

This deferred record does not authorize:

- an `app_program` crate;
- `AppProgram` as a second runtime root;
- `AppRecipe` / `PluginSuite` machinery;
- `foundation/meta`;
- editor/game/world-space integration;
- Counter implementation or lifecycle changes;
- shared plugin framework extraction;
- any current work item.
