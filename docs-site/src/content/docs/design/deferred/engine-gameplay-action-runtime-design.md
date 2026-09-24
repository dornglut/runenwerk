---
title: engine/gameplay/action_runtime Design
description: Deferred design for runtime execution, ordering, event emission, and ratification handoff of gameplay actions.
status: deferred
owner: gameplay
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# `engine/gameplay/action_runtime` Design

## Purpose

`engine/gameplay/action_runtime` adapts the pure action-domain contract to the engine world.

The crate owns runtime machinery that turns action requests, context snapshots, accepted plans, costs, and effect declarations into ECS/world changes, runtime events, scheduling work, and application results.

It does not define core action semantics. It executes action contracts produced by `domain/gameplay/actions` and other domain crates.

## Doctrine Alignment

This crate belongs to the runtime simulation layer and participates in mutation/ratification handoff.

It owns simulated reality for generic action execution. It does not own authoritative ratification, retention policy execution, asset storage, or editor documents.

Doctrine stance:

- runtime action components are simulated reality;
- action context snapshots are observed boundary products created from simulated reality;
- action application results are simulated products and possible ratification inputs;
- successful local application is not automatically ratified governing change;
- retention and authority policies decide whether action execution becomes durable or authoritative.

## Scope

In scope:

- action ECS components;
- runtime action registries/resources;
- action request collection;
- action context snapshot construction;
- action validation invocation;
- action planning invocation;
- application planning;
- transaction-aware action effect application;
- cost payment/reservation/refund application;
- action event bridging;
- cooldown runtime state;
- active action runtime state;
- deterministic runtime ordering;
- ratification handoff metadata;
- integration with engine scheduling.

## Non-scope

This crate must not own:

- generic action semantics;
- power technique semantics;
- power validation;
- item domain ownership;
- quest domain ownership;
- dialogue database ownership;
- editor UI;
- rendering logic;
- animation authoring;
- networking protocol design;
- save file schema;
- authority ratification decision;
- retention backend;
- reconciliation engine.

It may integrate with those systems through adapters, events, contracts, or extension points.

## Architectural Position

Correct dependency shape:

```text
domain/gameplay/actions
        ↓
engine/gameplay/action_runtime
```

Allowed dependencies:

- `domain/gameplay/actions`;
- engine ECS/world crates;
- engine event/scheduler crates;
- foundation diagnostics if needed for runtime diagnostics;
- authority/ratification contracts if these are foundation/engine-level contracts and do not create app coupling.

Forbidden dependencies:

- `domain/gameplay/powers`;
- editor UI crates;
- app crates.

Power-specific bridging belongs in `engine/gameplay/power_runtime`.

## Reality Classification

| Artifact | Reality | Owner | Notes |
|---|---|---|---|
| `ActionSetComponent` | Simulated reality | `engine/gameplay/action_runtime` | Runtime ECS state. |
| `ActiveActionComponent` | Simulated reality | `engine/gameplay/action_runtime` | Hot runtime execution state. |
| `ActionCooldownComponent` | Simulated reality; retained only if policy requires | Runtime/save owner | Hot cooldown state. |
| `ActionRequestQueue` | Workflow/simulated reality | Runtime | Ordered runtime input. |
| `ActionContextSnapshot` | Observed boundary product | Runtime builder + domain type | Immutable validation facts. |
| `ActionApplicationPlan` | Simulated workflow reality | Runtime | Runtime preflight/application plan. |
| `ActionApplicationResult` | Simulated result; ratification input if authoritative | Runtime + authority handoff | Not automatically ratified. |
| `ActionRuntimeEvent` | Observed/shared/retained depending on sink | Runtime/event infrastructure | May feed logs, UI, replay, network. |

## Ratification Boundary

`ActionApplicationResult` is not automatically a `RatifiedChange`.

Possible outcomes:

- non-ratifying local simulation;
- non-ratifying editor/runtime sandbox preview;
- rollbackable prediction;
- authoritative application pending ratification;
- ratified authoritative mutation;
- rejected ratification after local prediction.

For authoritative gameplay mutation, successful application must produce or feed a ratification contract outside this crate.

A ratification handoff should include:

- correlation id;
- causality id;
- authority scope;
- affected domains;
- affected runtime scopes/partitions if known;
- base versions if known;
- result versions if known;
- semantic operation summary;
- reversibility hint;
- retention hint.

## Reconciliation Policy

Default policies:

| Artifact | Reconciliation |
|---|---|
| `ActionRequestQueue` | Authority-ordered; not mergeable by default. |
| `ActiveActionComponent` | Forbidden or authority-owned. |
| `ActionCooldownComponent` | Authority-owned or simulation-owned; not mergeable by default. |
| `ActionApplicationPlan` | Not mergeable; rebuild. |
| `ActionApplicationResult` | Reject-on-ratification or authority-ordered if authoritative. |
| `ActionRuntimeEvent` | Ordered by event stream policy. |

No runtime action state is mergeable by default.

## Stability and Retention

| Artifact | Stability Class | Retention Strategy |
|---|---|---|
| `ActionRequestQueue` | Partition-stable or replay-stable if authoritative | Ephemeral, journal-retained, or audit-retained |
| `ActionContextSnapshot` | Replay-stable if captured | Ephemeral or journal-retained |
| `ActionApplicationPlan` | Replay-stable for same runtime/domain inputs | Rebuildable |
| `ActiveActionComponent` | Partition-stable | Ephemeral or checkpoint-retained |
| `ActionCooldownComponent` | Partition-stable | Ephemeral, state-retained, or checkpoint-retained |
| `ActionApplicationResult` | Partition-stable or replay-stable | Journal-retained/audit-retained if authoritative |
| `ActionRuntimeEvent` | Presentation-stable to replay-stable depending on stream | Ephemeral, journal-retained, or audit-retained |

## Migration Paths

Important migration paths:

```text
ActionRequestQueue
  -> ActionContextSnapshot
  -> ActionPlan
  -> ActionApplicationPlan
  -> ActionApplicationResult
```

Simulated runtime execution path.

```text
ActionApplicationResult
  -> RatifiedChange?
```

Ratification handoff path. Owned by authority/mutation layer.

```text
ActionRuntimeEvent
  -> Observed/Shared/Retained feeds
```

Observation, sharing, retention propagation path. Owned by event/observation/retention infrastructure.

## Capability Requirements

Suggested capabilities:

- `BuildActionContextSnapshotCapability`;
- `ExecuteActionRuntimeCapability`;
- `ApplyActionEffectCapability`;
- `ApplyActionCostCapability`;
- `EmitActionRuntimeEventCapability`;
- `RequestActionRatificationCapability`;
- `ObserveActionRuntimeCapability`.

Runtime APIs should avoid broad ambient mutation where a scoped capability can express intent.

## Core Responsibility

The action runtime owns this pipeline:

```text
Runtime input / AI / script / network / replay
  ↓
ActionRequest
  ↓
ActionContextSnapshotBuilder
  ↓
domain/gameplay/actions validation
  ↓
domain/gameplay/actions planning
  ↓
ActionPlan
  ↓
ActionApplicationPreflight
  ↓
ActionApplicationTransaction
  ↓
ECS/world mutation
  ↓
ActionApplicationResult
  ↓
Ratification handoff? + ActionEventBridge
```

The runtime builds the context snapshot. The domain validates and plans. The runtime applies. Authority decides whether governing change was ratified.

## ECS Components

The crate may define thin runtime components.

Examples:

- `ActionSetComponent`;
- `ActiveActionComponent`;
- `ActionCooldownComponent`;
- `QueuedActionComponent`;
- `ActionLockComponent`.

Components should store runtime state and references to domain IDs, not redefine domain semantics.

## Runtime Resources

Resources may include:

- `ActionRegistryResource`;
- `ActionRuntimeConfig`;
- `ActionRequestQueue`;
- `ActionEventQueue`;
- `ActionApplicationQueue`.

Registries reference ratified domain action definitions or handles to loaded definitions.

Asset loading and persistence belong elsewhere.

## Context Snapshot Construction

`ActionContextSnapshotBuilder` converts ECS/world facts into `ActionContextSnapshot`.

It may query:

- actor state;
- target state;
- inventory state;
- distance/range data;
- line-of-sight results;
- cooldown data;
- tags;
- environment state;
- resource availability.

The builder produces immutable facts, not live references.

The domain should never query ECS directly.

## Application Transaction Model

Runtime application must have explicit transaction semantics.

Default policy:

```text
1. Build immutable context snapshot.
2. Validate and plan in domain.
3. Preflight all required cost and effect adapters.
4. Reserve or pay costs according to cost timing.
5. Apply effects in deterministic order.
6. Emit application result and events.
7. Feed ratification handoff if policy requires.
```

Default strategy should be fail-fast-before-mutation where possible.

If partial application is allowed, it must be explicit in `ActionApplicationResult`.

Supported transaction policies may include:

- fail-fast-before-mutation;
- phase-based commit;
- explicit partial application;
- compensating rollback where supported.

Silent partial mutation is forbidden.

## Runtime Failure Semantics

Runtime failure is distinct from domain rejection.

Examples:

- missing component during application;
- unsupported effect adapter;
- stale cost state;
- target despawned after validation;
- adapter returned failure;
- application order conflict;
- ratification handoff failure.

Runtime failures should produce `ActionApplicationResult` with diagnostics.

## Effect Adapter Policy

Initial implementation should use explicit statically wired adapters.

Avoid dynamic plugin registries until a concrete need exists.

An adapter maps one effect declaration or effect namespace to runtime behavior.

Unsupported effect declarations must produce diagnostics/events.

Domain-specific effects should be handled by domain-specific runtime bridge crates instead of bloating this generic runtime.

## Cost Application

Runtime applies costs according to domain cost timing.

Examples:

- remove inventory item;
- reduce stamina;
- reduce power pool via adapter;
- consume charge;
- start cooldown;
- reserve resource.

Cost behavior must distinguish:

- reserve;
- commit;
- refund;
- consume;
- fail.

If payment fails after validation due to stale state, runtime emits a clear failure event.

## Deterministic Runtime Ordering

The crate must define deterministic order for:

- request dequeueing;
- validation;
- planning;
- cost application;
- effect application;
- event emission;
- ratification handoff;
- cooldown updates;
- phase transitions.

Introduce concepts such as:

- `ActionRuntimeTick`;
- `ActionRuntimePhase`;
- `ActionApplicationOrder`;
- `ActionEventOrder`.

Deterministic ordering is required for replay, rollback, multiplayer, testing, and debugging.

## Scheduling and Phases

The crate may support action phases:

- instant;
- windup;
- channel;
- commit;
- recovery;
- cooldown.

The domain may describe phases abstractly. Runtime schedules them.

Runtime is responsible for ticking time and moving active actions through phases.

## Events

The crate should emit runtime-facing events such as:

- action requested;
- action rejected;
- action planned;
- action preflight failed;
- action started;
- action committed;
- action cancelled;
- action completed;
- action failed during application;
- action effect applied;
- action ratification requested;
- action ratification accepted;
- action ratification rejected.

Events should include domain and runtime diagnostics when relevant.

## Diagnostics

Runtime diagnostic domain prefix:

```text
action_runtime.*
```

Examples:

- `action_runtime.context.missing_actor_component`;
- `action_runtime.application.unsupported_effect`;
- `action_runtime.application.stale_cost_state`;
- `action_runtime.application.target_despawned`;
- `action_runtime.transaction.partial_application`;
- `action_runtime.ratification.handoff_failed`.

Use `foundation/diagnostics` if diagnostics surface to tools.

## Invariants

1. Runtime does not define core action semantics.
2. Runtime builds immutable context snapshots and applies accepted plans.
3. ECS components remain thin and runtime-oriented.
4. Unsupported effects fail visibly, not silently.
5. Domain validation is not bypassed before application.
6. Runtime application failure is reported distinctly from domain rejection.
7. Power-specific logic does not leak into the generic action runtime.
8. Runtime can process requests from input, AI, scripts, network, replay, or editor preview.
9. Application transaction policy is explicit.
10. Runtime ordering is deterministic.
11. Successful runtime application is not automatically ratified governing change.
12. Runtime state is not mergeable by default.
13. Capability-scoped mutation is preferred over ambient mutation.

## Suggested Source Layout

```text
engine/gameplay/action_runtime/
  README.md
  design.md
  roadmap.md
  Cargo.toml
  src/
    lib.rs

    systems/
      mod.rs
      action_request_system.rs
      action_execution_system.rs
      action_event_system.rs
      action_cooldown_system.rs
      action_phase_system.rs

    ecs/
      mod.rs
      action_set_component.rs
      active_action_component.rs
      action_cooldown_component.rs
      queued_action_component.rs
      action_lock_component.rs

    doctrine/
      mod.rs
      action_runtime_reality.rs
      action_runtime_reconciliation_policy.rs
      action_runtime_stability_class.rs
      action_runtime_retention_hint.rs
      action_runtime_ratification_handoff.rs

    adapters/
      mod.rs
      action_context_snapshot_builder.rs
      action_effect_adapter.rs
      action_cost_adapter.rs
      action_event_bridge.rs

    application/
      mod.rs
      action_application_plan.rs
      action_application_step.rs
      action_application_result.rs
      application_transaction.rs
      application_failure.rs
      application_preflight.rs

    ordering/
      mod.rs
      action_runtime_tick.rs
      action_runtime_phase.rs
      action_application_order.rs
      action_event_order.rs

    resources/
      mod.rs
      action_registry_resource.rs
      action_runtime_config.rs
      action_request_queue.rs
      action_application_queue.rs
```

## Relationship to `domain/gameplay/actions`

The domain crate owns:

- action definitions;
- action requests;
- context snapshot type;
- requirements;
- costs;
- effect declarations;
- validation;
- planning;
- outcomes.

The runtime crate owns:

- ECS query;
- context snapshot construction;
- cost application;
- effect application;
- event emission;
- scheduling;
- transaction semantics;
- application results;
- ratification handoff metadata.

## Relationship to `engine/gameplay/power_runtime`

`power_runtime` may compile a power technique into an `ActionRequest` and then invoke `action_runtime`.

`action_runtime` must not know power semantics.

## Testing Strategy

Tests should cover:

- context snapshot construction from mock world state;
- accepted action application;
- rejected action does not mutate world;
- unsupported effect reports failure;
- preflight prevents mutation;
- cost payment failure reports failure;
- partial application policy;
- deterministic event ordering;
- cooldown state changes;
- phase progression;
- ratification handoff metadata;
- doctrine metadata mapping.

Integration tests may use test-support crates if ECS/world setup is heavy.

## Implementation Readiness

This crate should be implemented after `domain/gameplay/actions` stabilizes.

Do not implement before the transaction model, deterministic ordering policy, and ratification handoff contract are accepted.
