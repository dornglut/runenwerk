---
title: domain/gameplay/actions Design
description: Deferred design for gameplay action definitions, requests, requirements, costs, effects, planning, and validation.
status: deferred
owner: gameplay
layer: domain
canonical: true
last_reviewed: 2026-04-27
---

# `domain/gameplay/actions` Design

## Purpose

`domain/gameplay/actions` defines Runenwerk's generic gameplay action contract.

The crate models actor intent, target selection, immutable context facts, requirements, costs, effect declarations, validation, planning, outcomes, and action diagnostics without depending on ECS storage, input devices, rendering, networking, UI, save files, or concrete world mutation.

The crate exists so combat, interaction, item use, crafting, dialogue, scripted behavior, AI planning, and power techniques can share one execution contract without sharing one semantic owner.

This crate is not a gameplay god crate. It defines the action protocol. Domain-specific crates own their own semantics.

## Doctrine Alignment

This crate is a meaning-domain crate for generic gameplay action semantics.

It participates in the Runenwerk reality model as a domain contract, not as a runtime owner, asset database, editor document, or authority system.

Doctrine stance:

- actions are a meaning domain;
- action definitions are authored/normalized/formed content products depending on stage;
- action requests are workflow/simulated inputs;
- action context snapshots are observed boundary products;
- action plans are formed/simulated planning products;
- action runtime state belongs to runtime crates, not this crate;
- action application may feed ratification, but this crate does not ratify world change.

## Scope

This crate owns the pure action-domain contract.

In scope:

- stable action identifiers;
- action definition identity and schema versioning;
- action requests;
- action actors;
- action targets;
- immutable action context snapshots;
- action requirements;
- action costs;
- action effect declarations;
- action validation;
- action planning;
- action outcomes;
- action events;
- action diagnostics;
- action correlation metadata for replay/debugging;
- doctrine-facing metadata required for stability, retention, and ratification handoff.

The crate may expose deterministic, side-effect-free validation and planning helpers.

The crate may define common primitive effect declarations, but it must not directly apply them to ECS/world state.

## Non-scope

This crate must not own:

- ECS components;
- ECS systems;
- world mutation;
- physics queries;
- rendering;
- animation;
- audio;
- input bindings;
- UI;
- editor panels;
- asset loading;
- asset persistence;
- asset migration storage;
- inventory storage;
- quest state;
- dialogue database ownership;
- crafting recipe database ownership;
- networking transport;
- save/load format;
- scripting VM integration;
- runtime scheduling;
- ratification authority;
- retention backend;
- reconciliation engine.

Those concerns belong in runtime, engine, domain-specific, adapter, asset/document, authority, retention, or app crates.

## Architectural Position

The crate sits in the domain layer.

Correct dependency shape:

```text
foundation/diagnostics
foundation/id
foundation/tags?       optional later
foundation/time?       optional later
        ↓
domain/gameplay/actions
        ↓
domain/gameplay/powers
```

Runtime dependency shape:

```text
domain/gameplay/actions
        ↓
engine/gameplay/action_runtime
```

Forbidden dependency direction:

```text
domain/gameplay/actions
  must not depend on:
    domain/gameplay/powers
    engine/gameplay/action_runtime
    engine/gameplay/power_runtime
    apps/editor/*
    apps/game/*
```

Allowed dependencies:

- small foundation crates;
- `foundation/diagnostics`;
- stable ID/value-object crates;
- doctrine policy value types if they exist;
- serialization behind feature flags if required by asset or network infrastructure.

## Design Principle

The action layer answers:

```text
What is this actor attempting?
What facts are known?
Is the attempt allowed?
What plan would result?
What abstract effects should be requested?
What diagnostics explain acceptance or rejection?
What doctrine metadata is needed for runtime handoff?
```

It does not answer:

```text
How does ECS store this?
How is health mutated?
How is an animation played?
How is a dialogue database advanced?
How is a power oath broken?
How is a network packet encoded?
Which authority ratifies this mutation?
How is the action retained durably?
```

## Reality Classification

| Artifact | Reality | Owner | Notes |
|---|---|---|---|
| `ActionDraft` | Authored reality | Editor/asset authoring infrastructure | Optional future representation; not minimal core. |
| `ActionDefinition` | Normalized or formed reality | `domain/gameplay/actions` contract + asset infrastructure storage | Ratified/canonical action data product, not runtime state. |
| `ActionRequest` | Workflow reality or simulated input | Runtime/tooling source | Request to attempt an action; not governing change. |
| `ActionContextSnapshot` | Observed boundary product | Runtime/tooling builder | Immutable facts used for validation/planning. |
| `ActionValidationReport` | Observed reality | `domain/gameplay/actions` | Derived report; rebuildable from inputs. |
| `ActionPlan` | Formed/simulated planning product | `domain/gameplay/actions` | Deterministic plan derived from definition/request/context. |
| `ActionOutcome` | Observed/domain result | `domain/gameplay/actions` | Domain result, not world mutation. |
| `ActionApplicationResult` | Simulated or ratification input | Runtime/authority layer | May feed ratification but not owned here. |
| `ActionRuntimeState` | Simulated reality | Runtime crate | ECS/runtime state outside this crate. |

## Ratification Policy

`domain/gameplay/actions` does not ratify world change.

An accepted `ActionOutcome` or `ActionPlan` means:

```text
The action is domain-valid and has a deterministic plan.
```

It does not mean:

```text
The world has changed.
The change is authoritative.
The change is retained.
The change is visible to remote peers.
```

A runtime application may remain:

- local simulated change;
- non-ratifying session preview;
- authority-ratified gameplay mutation;
- rollbackable prediction;
- replay reconstruction.

Authoritative mutation must be represented by the relevant authority/ratification contract, such as `RatifiedChange`, outside this crate.

## Reconciliation Policy

Default policies:

| Artifact | Reconciliation |
|---|---|
| `ActionDefinition` | Replace-only or schema-migrated by asset/content infrastructure. |
| `ActionRequest` | Authority-ordered or reject-on-ratification in authoritative contexts. |
| `ActionContextSnapshot` | Not mergeable; rebuild or regenerate. |
| `ActionPlan` | Not mergeable; rebuild from inputs. |
| `ActionValidationReport` | Rebuildable; not mergeable. |
| `ActionRuntimeState` | Runtime/authority-owned; usually forbidden or authority-ordered. |

No action artifact is mergeable by default.

## Stability and Retention

| Artifact | Stability Class | Retention Strategy |
|---|---|---|
| `ActionDefinition` | Replay-stable within schema/version assumptions | State-retained by asset infrastructure |
| `ActionRequest` | Replay-stable if used for replay/network | Ephemeral, journal-retained, or audit-retained by authority policy |
| `ActionContextSnapshot` | Replay-stable if captured for authoritative replay | Ephemeral or journal-retained by policy |
| `ActionPlan` | Replay-stable for same definition/request/context | Rebuildable |
| `ActionValidationReport` | Observationally stable for same inputs | Rebuildable |
| `ActionOutcome` | Observationally stable | Ephemeral or journal-retained by policy |
| `ActionApplicationResult` | Partition-stable or replay-stable depending on runtime policy | Runtime/authority-owned |

The minimal domain crate only promises deterministic validation and planning for identical inputs.

## Migration Paths

The crate defines contracts used by migration paths but does not execute migrations.

Important migration paths:

```text
ActionDraft
  -> ActionDefinition
```

Authored to normalized/formed content. Owned by editor/asset infrastructure.

```text
ActionRequest + ActionContextSnapshot
  -> ActionPlan
```

Workflow/simulated input to formed/simulated planning product. Owned by this domain.

```text
ActionPlan
  -> ActionApplicationResult
```

Domain plan to runtime application. Owned by `engine/gameplay/action_runtime`.

```text
ActionApplicationResult
  -> RatifiedChange?
```

Runtime application to authority-ratified change. Owned by mutation/ratification layer, not this crate.

## Capability Requirements

The crate defines data contracts. Runtime/tooling layers must require capabilities for operations.

Suggested capabilities:

- `BuildActionContextSnapshotCapability`;
- `ValidateActionCapability`;
- `PlanActionCapability`;
- `ApplyActionPlanCapability`;
- `RatifyActionApplicationCapability`;
- `ObserveActionDiagnosticsCapability`.

This crate may define marker types later if foundation capability infrastructure exists. It should not rely on ambient authority in runtime APIs.

## Definition, Instance, Runtime State, and Draft

The design must keep four concepts separate.

```text
ActionDefinition
  Authored or generated reusable action description.

ActionRequest
  One actor's attempt to perform an action.

ActionPlan
  Deterministic resolved plan produced from definition + request + context snapshot.

ActionRuntimeState
  Engine-owned ECS/runtime state such as active action phase, cooldown, lock, queue entry.

ActionDraft
  Optional future editor/authoring representation. Not part of the minimal core unless needed.
```

The domain crate owns definitions, requests, snapshots, validation reports, and plans.

Runtime crates own active runtime state.

Editor/asset crates own drafts unless this crate later needs a small authoring model.

## Identity and Causality

The crate should define or use stable value types for:

- `ActionDefinitionId`;
- `ActionDefinitionVersion`;
- `ActionSchemaVersion`;
- `ActionRequestId`;
- `ActionCorrelationId`;
- `ActionPlanId`;
- `ActionEventId`;
- optional `ActionCausalityId` if not provided by foundation.

`ActionCorrelationId` links request, validation, plan, runtime application, diagnostics, logs, replay, and editor preview.

IDs must not be raw ECS entity ids.

Identities should declare:

- scope;
- restart stability;
- replay safety;
- ownership domain.

## Core Pipeline

The core pipeline is:

```text
ActionDefinition
  + ActionRequest
  + ActionContextSnapshot
  ↓
ActionValidator
  ↓
ActionValidationReport
  ↓
ActionPlanner
  ↓
ActionPlan
  ↓
ActionOutcome
```

Runtime application is deliberately outside this crate:

```text
ActionPlan
  ↓
engine/gameplay/action_runtime
  ↓
ActionApplicationResult
```

The action domain may define `ActionApplicationRequest` and `ActionApplicationResult` data contracts if needed, but it must not apply them.

## Context Snapshot Policy

`ActionContextSnapshot` is an immutable collection of facts supplied by runtime or tooling.

It must contain facts, not live ECS references.

Good examples:

- actor is alive;
- target is interactable;
- distance to target is `1.7`;
- actor has item `health_potion`;
- cooldown remaining is `0`;
- target tags include `world.door`;
- line of sight is available.

Bad examples:

- mutable ECS world handle;
- live entity query;
- closure that checks inventory later;
- pointer to a runtime component.

Reason:

- deterministic validation;
- replay;
- rollback;
- editor preview;
- AI planning;
- network reconciliation;
- testability.

## Validation vs Planning vs Application

The crate must not collapse these phases.

### Validation

Validation decides whether the request is currently allowed.

Output:

- accepted/rejected state;
- diagnostics;
- warnings;
- missing facts if the context is incomplete.

### Planning

Planning produces a deterministic `ActionPlan` from an accepted validation result.

The plan may include:

- ordered plan steps;
- abstract costs;
- abstract effects;
- event declarations;
- runtime hints;
- phase metadata.

### Application

Application is runtime-owned.

Runtime may fail to apply a plan because the world changed, an adapter is missing, or a component is absent. Such failure is not the same as domain rejection.

## Action Definition

`ActionDefinition` should contain:

- stable definition id;
- schema version;
- definition version;
- action kind;
- requirements;
- costs;
- effect declarations;
- execution metadata;
- optional classification tags;
- optional diagnostics metadata;
- stability and retention hints where needed;
- source lineage metadata where inspection/round-tripping matters.

It should not contain:

- ECS entity references;
- world pointers;
- closures that mutate engine state;
- UI presentation state;
- input key bindings;
- renderer objects;
- runtime component handles.

## Action Request

`ActionRequest` is one actor's attempt to perform an action.

It should contain:

- request id;
- correlation id;
- causality id if needed;
- actor reference as domain/runtime-neutral identity;
- action definition reference;
- target;
- declared intent;
- request source;
- optional authority scope hint;
- optional metadata.

Request source examples:

- player input;
- AI planner;
- script;
- replay;
- network command;
- editor preview.

Requests should be structurally inspectable and preferably serializable behind feature flags.

## Requirements

Requirements are preconditions.

Examples:

- actor must be alive;
- actor must be in range;
- actor must have line of sight;
- actor must have a required item;
- target must have a required tag;
- target must be interactable;
- action must be off cooldown;
- environment must allow this action.

A failed requirement must produce a stable diagnostic.

## Costs

Costs represent resources consumed, reserved, or scheduled by an action.

Examples:

- consume item;
- consume stamina;
- consume power through a runtime adapter;
- spend materials;
- spend charge;
- start cooldown;
- reserve focus;
- consume action point.

Cost timing must be explicit:

- on request;
- on start;
- on commit;
- on completion;
- reserved during channel;
- refunded on cancel;
- never refunded.

The action domain describes cost intent and validates against context facts. Runtime applies payment.

## Effect Ownership and Extension Policy

`domain/gameplay/actions` must not become a dumping ground for all gameplay effects.

The crate may define:

- generic effect envelopes;
- common primitive effects;
- domain-neutral effect metadata;
- effect namespaces;
- extension points for domain-owned effects.

Examples of acceptable generic primitive effects:

- damage;
- heal;
- apply generic status;
- set generic target state;
- emit gameplay event.

Examples that should usually remain domain-owned:

- suppress power flow;
- break oath;
- apply power consequence;
- advance dialogue node;
- unlock quest objective;
- consume crafting recipe;
- mutate faction reputation.

Domain-specific effects should be represented by typed domain payloads and adapted by runtime bridge crates.

Power-specific effects belong in `domain/gameplay/powers` and `engine/gameplay/power_runtime`, then may be lowered into generic action effects only when appropriate.

## Outcomes

`ActionOutcome` describes the result of domain validation and planning.

It should distinguish:

- accepted;
- rejected;
- cancelled before planning;
- deferred;
- partially planned;
- planning failed.

Runtime application failures belong in `ActionApplicationResult`.

Rejected outcomes must include diagnostics.

## Diagnostics

The crate should use `foundation/diagnostics`.

Diagnostic domain prefix:

```text
action.*
```

Example codes:

- `action.requirement.target_out_of_range`;
- `action.requirement.missing_item`;
- `action.cost.insufficient_resource`;
- `action.target.invalid_kind`;
- `action.plan.unsupported_effect`;
- `action.context.missing_fact`.

Diagnostic subjects should support:

- action definition id;
- action request id;
- action target;
- requirement id;
- cost id;
- effect id;
- plan step id;
- context fact id.

## Asset and Schema Policy

This crate defines action data contracts. It does not own asset storage.

Asset/document infrastructure owns:

- persistence;
- loading;
- migration;
- editor documents;
- import/export;
- registry population.

This crate should define schema version value types and migration-facing metadata but not the migration storage system.

## Invariants

1. Action validation is pure and side-effect-free.
2. Action planning is deterministic for the same definition, request, and context snapshot.
3. Action definitions do not mutate runtime state.
4. Action effects are declarations, not direct world writes.
5. Rejected actions always carry explainable diagnostics.
6. Runtime-specific state does not enter core domain types.
7. The generic action layer must not know about power-specific semantics.
8. Public IDs and values must be stable enough for tests, tooling, replay, and editor diagnostics.
9. The crate must be usable without starting the engine.
10. Validation, planning, and application must remain separate concepts.
11. Local action success is not ratification.
12. No action artifact is mergeable by default.
13. Doctrine metadata must be represented in types where it governs behavior.

## Suggested Source Layout

```text
domain/gameplay/actions/
  README.md
  design.md
  roadmap.md
  Cargo.toml
  src/
    lib.rs

    identity/
      mod.rs
      action_definition_id.rs
      action_request_id.rs
      action_correlation_id.rs
      action_plan_id.rs
      action_event_id.rs

    doctrine/
      mod.rs
      action_reality.rs
      action_reconciliation_policy.rs
      action_stability_class.rs
      action_retention_hint.rs
      action_ratification_hint.rs

    schema/
      mod.rs
      action_schema_version.rs
      action_definition_version.rs

    core/
      mod.rs
      action_kind.rs
      action_definition.rs
      action_request.rs
      action_actor.rs
      action_target.rs
      action_context_snapshot.rs
      action_outcome.rs

    requirements/
      mod.rs
      action_requirement.rs
      requirement_check.rs
      requirement_result.rs

    costs/
      mod.rs
      action_cost.rs
      cost_timing.rs
      cost_payment.rs
      cost_result.rs

    effects/
      mod.rs
      action_effect.rs
      effect_namespace.rs
      effect_payload.rs

    planning/
      mod.rs
      action_planner.rs
      action_plan.rs
      action_plan_step.rs
      action_plan_diagnostic.rs

    events/
      mod.rs
      action_event.rs
      action_event_kind.rs

    validation/
      mod.rs
      action_validator.rs
      action_validation_report.rs
      action_diagnostic.rs
```

## Public API Shape

Expose a curated public API through `lib.rs`.

Recommended public groups:

- identity;
- doctrine metadata;
- schema;
- core action types;
- context snapshots;
- requirement types;
- cost types;
- effect declaration types;
- validation report;
- planning types;
- event types.

Avoid broad wildcard-style exports until the crate stabilizes.

## Relationship to `domain/gameplay/powers`

`domain/gameplay/powers` may compile accepted technique usage into an `ActionRequest` or `ActionPlan` input.

Rule:

```text
Powers may produce actions.
Actions do not know about powers.
```

## Relationship to Runtime

Runtime crates are responsible for:

- building `ActionContextSnapshot`;
- applying action plans;
- paying costs;
- invoking effect adapters;
- emitting runtime events;
- scheduling delayed actions;
- ticking cooldowns;
- integrating with animation, audio, networking, save/load, authority, retention, and ECS.

## Testing Strategy

Tests should cover:

- requirement validation;
- cost validation;
- context snapshot use;
- effect declaration construction;
- plan construction;
- outcome construction;
- diagnostic stability;
- invalid target rejection;
- insufficient resource rejection;
- deterministic planning;
- doctrine metadata mapping;
- absence of ECS dependencies.

## Implementation Readiness

This crate is ready to implement after the following are decided:

1. Foundation ID/value-object strategy.
2. Minimal diagnostic integration.
3. Initial generic effect extension policy.
4. Minimal schema/versioning value types.
5. Whether doctrine policy enums live in foundation or per-domain modules.
