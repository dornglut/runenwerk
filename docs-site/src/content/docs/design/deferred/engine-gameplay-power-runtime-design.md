---
title: engine/gameplay/power_runtime Design
description: Deferred design for runtime integration of gameplay power profiles, techniques, cooldowns, consequences, and power state.
status: deferred
owner: gameplay
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# `engine/gameplay/power_runtime` Design

## Purpose

`engine/gameplay/power_runtime` adapts the pure power-domain model to the engine world.

The crate owns ECS/runtime integration for power profile references, power profile instances, power pools, flow states, known techniques, active techniques, cooldowns, regeneration, oath state, consequences, context snapshot construction, technique request processing, and technique-to-action execution.

It does not define the semantics of powers or techniques. Those belong in `domain/gameplay/powers`.

## Doctrine Alignment

This crate belongs to the runtime simulation layer and participates in mutation/ratification handoff for persistent power state.

It owns simulated reality for power runtime state. It may apply domain-defined power consequences, but authority policy decides whether those become ratified governing changes.

Doctrine stance:

- power runtime components are simulated reality;
- profile/technique instance references connect instantiated/retained state to ECS state;
- oath/consequence runtime state may require retention and ratification;
- technique context snapshots are observed boundary products;
- technique requests are workflow/simulated inputs;
- generic effects are delegated to action runtime;
- power-specific persistent state stays in power runtime and authority/save infrastructure.

## Scope

In scope:

- power ECS components;
- profile instance runtime references;
- known technique runtime state;
- active technique runtime state;
- technique cooldown runtime state;
- power pool runtime state;
- oath state runtime;
- active consequence runtime;
- power instability runtime;
- power regeneration systems;
- power state systems;
- technique request systems;
- technique execution systems;
- technique event bridges;
- technique context snapshot construction from ECS/world;
- bridge from accepted technique to generic action runtime;
- runtime application of power-specific consequences;
- ratification handoff metadata for persistent power changes.

## Non-scope

This crate must not own:

- pure power semantics;
- technique validation rules;
- oath validity rules;
- affinity compatibility rules;
- technique authoring model;
- editor UI;
- renderer effects;
- animation authoring;
- input binding definitions;
- generic action semantics;
- generic action effect application unless delegated through `action_runtime`;
- asset persistence;
- save file schema;
- authority ratification decision;
- retention backend;
- reconciliation engine.

## Architectural Position

Correct dependency shape:

```text
domain/gameplay/actions
        ↓
domain/gameplay/powers

domain/gameplay/actions
        ↓
engine/gameplay/action_runtime

domain/gameplay/powers
        ↓
engine/gameplay/power_runtime

engine/gameplay/action_runtime
        ↓
engine/gameplay/power_runtime
```

Meaning:

```text
power_runtime depends on powers and action_runtime.
action_runtime does not depend on powers.
```

Allowed dependencies:

- `domain/gameplay/powers`;
- `domain/gameplay/actions`;
- `engine/gameplay/action_runtime`;
- ECS/world crates;
- scheduler/event crates;
- foundation diagnostics if needed;
- authority/ratification contracts if these are foundation/engine-level contracts and do not create app coupling.

Forbidden dependencies:

- editor UI crates;
- app crates;
- renderer-specific authoring logic.

## Reality Classification

| Artifact | Reality | Owner | Notes |
|---|---|---|---|
| `PowerProfileComponent` | Simulated reality | `engine/gameplay/power_runtime` | ECS reference/cache for profile state. |
| `PowerProfileInstanceComponent` | Simulated link to instantiated/retained reality | Runtime/save owner | Connects ECS actor to profile instance. |
| `PowerPoolComponent` | Simulated reality; retained if policy requires | Runtime/save owner | Hot resource values. |
| `KnownTechniquesComponent` | Simulated link to instantiated/retained reality | Runtime/save owner | References technique instances. |
| `ActiveTechniqueComponent` | Simulated reality | Runtime | Hot execution/channel state. |
| `TechniqueCooldownComponent` | Simulated reality; retained if policy requires | Runtime/save owner | Cooldown state. |
| `OathStateComponent` | Simulated + retained + possibly ratified reality | Runtime/save/authority | Persistent oath state. |
| `ActiveConsequenceComponent` | Simulated + retained + possibly ratified reality | Runtime/save/authority | Consequence lifecycle state. |
| `TechniqueContextSnapshot` | Observed boundary product | Runtime builder + domain type | Immutable validation facts. |
| `TechniqueRequestQueue` | Workflow/simulated reality | Runtime | Ordered runtime input. |
| `TechniqueEventQueue` | Observed/shared/retained depending on sink | Runtime/event infrastructure | May feed UI, replay, logs, replication. |

## Ratification Policy

Power runtime distinguishes local simulation from governing power change.

Potentially ratifiable changes:

- profile instance mutation;
- technique instance mutation;
- learning/unlearning a technique;
- oath binding;
- oath violation;
- consequence application;
- power lockout;
- persistent seal;
- long-lived instability;
- retained cooldown if gameplay requires it.

Non-ratifying changes:

- editor preview;
- runtime sandbox preview;
- local-only visual prediction;
- AI planning validation;
- ephemeral diagnostic observation.

For authority-scoped gameplay, successful runtime application must feed a ratification contract outside this crate.

## Reconciliation Policy

Default policies:

| Artifact | Reconciliation |
|---|---|
| `PowerPoolComponent` | Authority-owned or simulation-owned; not mergeable by default. |
| `KnownTechniquesComponent` | Reject-on-ratification or authority-owned. |
| `ActiveTechniqueComponent` | Forbidden/authority-owned. |
| `TechniqueCooldownComponent` | Authority-owned or simulation-owned; not mergeable by default. |
| `OathStateComponent` | Forbidden or authority-owned. |
| `ActiveConsequenceComponent` | Forbidden or authority-owned. |
| `TechniqueRequestQueue` | Authority-ordered. |
| `TechniqueEventQueue` | Ordered by event stream policy. |

No persistent power runtime state is mergeable by default.

## Stability and Retention

| Artifact | Stability Class | Retention Strategy |
|---|---|---|
| `PowerPoolComponent` | Partition-stable | Ephemeral, state-retained, or checkpoint-retained |
| `KnownTechniquesComponent` | Partition-stable or replay-stable if authoritative | State-retained |
| `ActiveTechniqueComponent` | Partition-stable | Ephemeral or checkpoint-retained |
| `TechniqueCooldownComponent` | Partition-stable | Ephemeral, state-retained, or checkpoint-retained |
| `OathStateComponent` | Partition-stable or replay-stable if authoritative | State-retained, checkpoint-retained, or journal-retained |
| `ActiveConsequenceComponent` | Partition-stable or replay-stable if authoritative | State-retained, checkpoint-retained, or journal-retained |
| `TechniqueContextSnapshot` | Replay-stable if captured | Ephemeral or journal-retained |
| `TechniqueEventQueue` | Presentation-stable to replay-stable depending on stream | Ephemeral, journal-retained, or audit-retained |

## Migration Paths

Important migration paths:

```text
PowerProfileInstance / TechniqueInstance
  -> Power runtime ECS components
```

Instantiation into simulated reality.

```text
TechniqueRequestQueue
  -> TechniqueContextSnapshot
  -> TechniqueOutcome
  -> ActionRequest?
```

Technique execution path.

```text
TechniqueOutcome
  -> Persistent power-state transition?
  -> RatifiedChange?
```

Power-state mutation and ratification handoff path.

```text
OathState
  -> ConsequenceState
  -> Retained/Ratified power mutation
```

Persistent oath/consequence lifecycle path.

```text
TechniqueEventQueue
  -> Observed/Shared/Retained feeds
```

Observation, sharing, retention propagation path.

## Capability Requirements

Suggested capabilities:

- `BuildPowerContextSnapshotCapability`;
- `ExecuteTechniqueRuntimeCapability`;
- `ApplyPowerConsequenceCapability`;
- `MutatePowerProfileInstanceCapability`;
- `MutateTechniqueInstanceCapability`;
- `BindOathRuntimeCapability`;
- `RequestPowerRatificationCapability`;
- `ObservePowerRuntimeCapability`;
- `RunPowerSandboxPreviewCapability`.

Runtime APIs should avoid broad ambient mutation where a scoped capability can express intent.

## Core Responsibility

The runtime owns this pipeline:

```text
Input / AI / script / network / replay
  ↓
TechniqueRequest
  ↓
PowerContextSnapshotBuilder
  ↓
domain/gameplay/powers validation
  ↓
TechniqueOutcome
  ↓
TechniqueActionBridge
  ↓
ActionRequest
  ↓
engine/gameplay/action_runtime
  ↓
ECS/world mutation
```

For persistent power-state transitions, the runtime may apply domain-defined consequences directly through power-specific adapters and feed ratification handoff if required.

## Definition, Instance, Runtime Component

The runtime must preserve the domain distinction:

```text
PowerProfileDefinition
  Authored/static profile template owned by domain/assets.

PowerProfileInstance
  Character-owned mutable profile state, save-facing.

PowerProfileComponent
  ECS reference to a profile instance or cached runtime profile handle.

PowerPoolComponent
  Hot runtime pool values.

TechniqueDefinition
  Authored/static technique owned by domain/assets.

TechniqueInstance
  Character-owned learned/bound technique state.

KnownTechniquesComponent
  ECS reference/list of technique instances.

ActiveTechniqueComponent
  Hot runtime execution state.
```

Runtime components should reference domain ids or instance ids. They should not redefine semantics.

## ECS Components

The crate may define:

- `PowerProfileComponent`;
- `PowerProfileInstanceComponent`;
- `PowerPoolComponent`;
- `PowerStateComponent`;
- `PowerFlowComponent`;
- `KnownTechniquesComponent`;
- `ActiveTechniqueComponent`;
- `TechniqueCooldownComponent`;
- `TechniqueLockoutComponent`;
- `OathStateComponent`;
- `ActiveConsequenceComponent`;
- `PowerInstabilityComponent`.

Components store runtime state or references to domain/save-facing state.

They must not contain hardcoded validation rules.

## Runtime Resources

Resources may include:

- `PowerRegistryResource`;
- `TechniqueRegistryResource`;
- `PowerProfileInstanceStore` if not owned elsewhere;
- `TechniqueInstanceStore` if not owned elsewhere;
- `PowerRuntimeConfig`;
- `TechniqueRequestQueue`;
- `TechniqueEventQueue`.

Registries provide access to ratified domain definitions.

Asset loading and persistence belong elsewhere.

## Context Snapshot Construction

`PowerContextSnapshotBuilder` converts ECS/world state into `TechniqueContextSnapshot`.

It may query:

- actor power profile component;
- profile instance state;
- technique instance state;
- current power pool;
- flow state;
- known techniques;
- cooldowns;
- oath states;
- active consequences;
- target facts;
- line of sight;
- range;
- target tags/classes;
- environment modifiers;
- spoken/declared intent metadata.

The builder must collect facts, not validate semantics.

The produced context must not contain live ECS references.

## Technique Request Processing

Technique requests can come from:

- player input;
- AI;
- scripts;
- network;
- replay;
- editor preview.

A request is converted into domain input, validated, and either rejected or exported into an action.

Rejected requests emit events with diagnostics.

Accepted requests proceed through action runtime when they require generic world effects.

Pure power-state transitions or consequences may be applied by power runtime.

## Technique-to-Action Bridge

`TechniqueActionBridge` maps accepted technique outcomes to `ActionRequest`.

The bridge should preserve metadata:

- source technique definition id;
- technique instance id;
- actor;
- target;
- costs;
- effects;
- scaling result;
- diagnostics;
- correlation id;
- authority scope hint;
- runtime hints.

The generic action runtime should not need power-specific knowledge.

## Persistent Power State

Not all power behavior is an action.

Power runtime owns persistent power-state transitions such as:

- oath state changes;
- seals;
- lockouts;
- instability;
- known technique mutation;
- profile instance mutation;
- passive pressure;
- long-lived consequences;
- delayed backlash.

These are domain-defined but runtime-applied.

## Oath State Runtime

Runtime should track oath state explicitly.

Examples:

- bound;
- active;
- suspended;
- violated;
- fulfilled;
- broken;
- consequence pending;
- consequence applied.

Oath state should be inspectable for UI, debugging, replay, save/load, retention, and ratification handoff.

## Consequence Lifecycle Runtime

Consequences may be:

- immediate;
- delayed;
- periodic;
- persistent;
- reversible;
- permanent;
- cleansable.

Runtime owns the ticking/application lifecycle.

Domain owns the consequence meaning.

Runtime must report unsupported consequences explicitly.

## Power Regeneration

Runtime owns ticking and regeneration.

Regeneration may depend on:

- current power state;
- flow state;
- lockout;
- injuries/status;
- active consequences;
- environment;
- runtime config;
- profile-derived rates.

Pure formulas may be delegated to domain if they are semantic rules. Ticking and component mutation belong here.

## Cooldowns and Active Techniques

Runtime owns:

- ticking technique cooldowns;
- starting cooldowns after accepted execution;
- active technique/channel state;
- cancellation;
- interruption;
- lockout;
- recovery.

The domain may describe cooldown meaning, but runtime stores and updates time.

## Events

The crate should emit events such as:

- technique requested;
- technique rejected;
- technique accepted;
- technique started;
- technique committed;
- technique cancelled;
- technique completed;
- oath bound;
- oath violated;
- consequence queued;
- consequence applied;
- backlash triggered;
- power state changed;
- power ratification requested;
- power ratification accepted;
- power ratification rejected.

Events should preserve enough data for tooling, combat logs, replay, debugging, UI, and ratification.

## Diagnostics

Runtime diagnostic domain prefix:

```text
power_runtime.*
```

Examples:

- `power_runtime.context.missing_profile`;
- `power_runtime.context.unknown_technique`;
- `power_runtime.context.missing_component`;
- `power_runtime.bridge.action_export_failed`;
- `power_runtime.consequence.unsupported`;
- `power_runtime.oath_state.invalid_transition`;
- `power_runtime.ratification.handoff_failed`.

Use structured diagnostics where possible.

## Runtime Ordering

Power runtime must define deterministic ordering for:

- regeneration;
- request processing;
- cooldown ticking;
- active technique progression;
- oath checks;
- consequence application;
- event emission;
- ratification handoff.

Ordering must be compatible with action runtime ordering.

## Invariants

1. Runtime does not define power semantics.
2. Runtime must call domain validation before executing a technique.
3. Runtime context construction must be explicit and testable.
4. Context snapshots contain facts, not live ECS references.
5. Technique definitions come from ratified domain definitions.
6. ECS components store runtime state or references, not domain rules.
7. Power-specific requests compile into generic actions when generic world effects are needed.
8. Rejected technique requests must not mutate the world.
9. Runtime application failures must be visible.
10. `action_runtime` stays generic and does not learn power rules.
11. Persistent power state is handled by power runtime, not action runtime.
12. Oath and consequence lifecycle state is explicit.
13. Successful runtime application is not automatically ratified governing change.
14. Persistent power runtime state is not mergeable by default.
15. Capability-scoped mutation is preferred over ambient mutation.

## Suggested Source Layout

```text
engine/gameplay/power_runtime/
  README.md
  design.md
  roadmap.md
  Cargo.toml
  src/
    lib.rs

    systems/
      mod.rs
      power_regeneration_system.rs
      power_state_system.rs
      technique_request_system.rs
      technique_execution_system.rs
      technique_cooldown_system.rs
      technique_event_system.rs
      oath_state_system.rs
      consequence_application_system.rs
      power_instability_system.rs

    ecs/
      mod.rs
      power_profile_component.rs
      power_profile_instance_component.rs
      power_pool_component.rs
      power_state_component.rs
      power_flow_component.rs
      known_techniques_component.rs
      active_technique_component.rs
      technique_cooldown_component.rs
      technique_lockout_component.rs
      oath_state_component.rs
      active_consequence_component.rs
      power_instability_component.rs

    doctrine/
      mod.rs
      power_runtime_reality.rs
      power_runtime_reconciliation_policy.rs
      power_runtime_stability_class.rs
      power_runtime_retention_hint.rs
      power_runtime_ratification_handoff.rs

    adapters/
      mod.rs
      power_context_snapshot_builder.rs
      technique_action_bridge.rs
      technique_consequence_applier.rs
      technique_event_bridge.rs

    application/
      mod.rs
      technique_application_result.rs
      consequence_application_result.rs

    ordering/
      mod.rs
      power_runtime_tick.rs
      power_runtime_phase.rs
      power_event_order.rs

    resources/
      mod.rs
      power_registry_resource.rs
      technique_registry_resource.rs
      power_runtime_config.rs
      technique_request_queue.rs
```

## Relationship to `domain/gameplay/powers`

The domain owns:

- profiles;
- technique definitions;
- technique instances as domain/save-facing state;
- requirements;
- constraints;
- oaths;
- consequences;
- validation;
- scaling;
- technique export contract;
- doctrine metadata.

The runtime owns:

- ECS state;
- context snapshot construction;
- ticking;
- cooldowns;
- event emission;
- consequence application;
- oath state runtime;
- action runtime delegation;
- ratification handoff metadata.

## Relationship to `engine/gameplay/action_runtime`

Power runtime delegates generic world effects through action runtime.

It should not duplicate generic effect application.

Power-specific consequences stay in power runtime.

## Testing Strategy

Tests should cover:

- context snapshot construction from mock ECS/world;
- missing profile rejection;
- unknown technique rejection;
- valid technique produces action request;
- rejected technique does not call action runtime;
- cooldown starts after accepted execution;
- regeneration changes power pool correctly;
- oath state transitions;
- consequence lifecycle;
- unsupported consequence reports failure;
- deterministic event ordering;
- ratification handoff metadata;
- doctrine metadata mapping;
- events contain diagnostics.

## Implementation Readiness

This crate should be implemented only after:

1. `domain/gameplay/powers` has stable validation/export contracts.
2. `engine/gameplay/action_runtime` has stable transaction semantics.
3. Profile definition/instance/runtime component ownership is accepted.
4. Oath/consequence runtime lifecycle is accepted.
5. Ratification handoff and retention expectations are known.
