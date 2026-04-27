---
title: domain/gameplay/powers Design
description: Deferred design for Runenwerk's personal power-system domain.
status: deferred
owner: gameplay
layer: domain
canonical: true
last_reviewed: 2026-04-27
---

# `domain/gameplay/powers` Design

## Purpose

`domain/gameplay/powers` defines Runenwerk's personal power-system domain.

The crate models power identity, profile compatibility, affinities, expressions, flow, control, output, authored techniques, requirements, constraints, oaths, risks, consequences, scaling, validation, and technique-to-action export.

This is not a generic ability list and not an ECS runtime system. It is the pure domain model for a character-specific, rule-driven power system.

## Doctrine Alignment

This crate is a meaning-domain crate for power semantics.

It owns power meaning, validation rules, legal operations, and doctrine metadata for power-domain artifacts. It does not own runtime application, editor documents, asset persistence, authority ratification, or retention backends.

Doctrine stance:

- power drafts and documents are authored reality;
- ratified definitions are normalized or formed reality;
- profile and technique instances are instantiated/retained character state;
- context snapshots are observed boundary products;
- validation reports and scaling breakdowns are observed reality;
- active technique state is simulated reality and belongs to runtime;
- oath/consequence state may be simulated, retained, and ratified depending on scope.

## Scope

In scope:

- power profile definitions;
- power profile instances as domain/save-facing concepts;
- power sources;
- power affinities;
- power expressions;
- power flow model;
- power control model;
- power output model;
- power pool model;
- technique definitions;
- technique instances as learned/bound character-owned concepts;
- technique requirements;
- technique constraints;
- technique oaths;
- oath lifecycle;
- technique risks;
- technique consequences;
- consequence timing;
- technique costs;
- technique effects;
- technique targeting;
- technique scaling rules;
- scaling breakdowns;
- technique validation;
- technique diagnostics;
- technique requests;
- technique outcomes;
- technique events;
- technique export to generic action requests/plans;
- authoring drafts and ratification contracts;
- doctrine-facing metadata required for stability, retention, reconciliation, and ratification handoff.

## Non-scope

This crate must not own:

- ECS components;
- ECS systems;
- runtime ticking;
- runtime regeneration scheduling;
- runtime cooldown ticking;
- direct world mutation;
- renderer effects;
- animation playback;
- audio playback;
- input bindings;
- UI panels;
- editor graph widgets;
- networking transport;
- save file format;
- asset database storage;
- retention backend;
- ratification authority;
- inventory storage;
- quest state;
- generic item use;
- generic dialogue actions;
- generic crafting actions.

Those belong in runtime, action, editor, asset/document, retention, authority, or other domain crates.

## Architectural Position

Correct dependency shape:

```text
foundation/diagnostics
foundation/ids
        ↓
domain/gameplay/actions
        ↓
domain/gameplay/powers
```

Runtime shape:

```text
domain/gameplay/powers
        ↓
engine/gameplay/power_runtime

domain/gameplay/actions
        ↓
engine/gameplay/action_runtime
        ↓
engine/gameplay/power_runtime
```

`domain/gameplay/powers` may depend on `domain/gameplay/actions`.

`domain/gameplay/actions` must not depend on `domain/gameplay/powers`.

Forbidden dependencies:

- ECS crates;
- engine runtime crates;
- editor crates;
- renderer crates;
- app crates.

## Reality Classification

| Artifact | Reality | Owner | Notes |
|---|---|---|---|
| `PowerProfileDraft` | Authored reality | Editor/authoring infrastructure + domain ratifier contract | May be invalid/incomplete. |
| `TechniqueDraft` | Authored reality | Editor/authoring infrastructure + domain ratifier contract | May include incomplete authoring state. |
| `PowerProfileDefinition` | Normalized/Formed reality | Domain contract + asset infrastructure | Reusable profile definition/archetype. |
| `TechniqueDefinition` | Normalized/Formed reality | Domain contract + asset infrastructure | Ratified reusable technique. |
| `PowerProfileInstance` | Instantiated + retained reality | Character/domain save owner | Character-owned mutable state. |
| `TechniqueInstance` | Instantiated + retained reality | Character/domain save owner | Learned/bound/customized technique. |
| `TechniqueContextSnapshot` | Observed boundary product | Runtime/tooling builder | Immutable validation facts. |
| `TechniqueValidationReport` | Observed reality | `domain/gameplay/powers` | Rebuildable derived report. |
| `ScalingBreakdown` | Observed reality | `domain/gameplay/powers` | Explainable derived scaling output. |
| `TechniqueRequest` | Workflow reality or simulated input | Runtime/tooling source | Attempt to use a technique. |
| `TechniqueOutcome` | Domain observed result | `domain/gameplay/powers` | Does not mutate world. |
| `OathBinding` | Instantiated/retained reality | Power domain + runtime/save owner | Depending on whether persistent. |
| `OathState` | Simulated + retained + possibly ratified reality | Power runtime/save/authority | Runtime-owned state from domain rules. |
| `ConsequenceState` | Simulated + retained + possibly ratified reality | Power runtime/save/authority | Runtime-owned lifecycle state. |
| `ActiveTechniqueRuntimeState` | Simulated reality | `engine/gameplay/power_runtime` | Not owned by this crate. |

## Ratification Policy

This crate ratifies drafts into definitions only in the content/authoring sense.

It does not ratify runtime world change.

Power-domain outputs may require runtime/authority ratification when they affect governing state.

Examples:

| Change | Ratification Class |
|---|---|
| `TechniqueDraft -> TechniqueDefinition` | Content ratification through asset/document pipeline |
| `PowerProfileDraft -> PowerProfileDefinition` | Content ratification through asset/document pipeline |
| learning a technique | Authority or save-domain ratification |
| binding an oath | Authority/save-domain ratification if persistent |
| violating an oath | Authority ratification if governing gameplay state |
| applying a consequence | Authority ratification if it changes governing state |
| editor preview technique use | Non-ratifying session change |
| AI planning validation | Non-ratifying observed/planning product |

A successful `TechniqueValidationReport` does not mean a power change is ratified.

## Reconciliation Policy

Default policies:

| Artifact | Reconciliation |
|---|---|
| `TechniqueDraft` | Structure-merged only if editor document infrastructure supports it. |
| `PowerProfileDraft` | Structure-merged only if editor document infrastructure supports it. |
| `TechniqueDefinition` | Replace-only or schema-migrated unless domain-specific merge is designed. |
| `PowerProfileDefinition` | Replace-only or schema-migrated unless domain-specific merge is designed. |
| `TechniqueInstance` | Reject-on-ratification or authority-owned. |
| `PowerProfileInstance` | Reject-on-ratification or authority-owned. |
| `OathState` | Usually forbidden to merge; authority-owned. |
| `ConsequenceState` | Usually forbidden to merge; authority-owned. |
| `TechniqueContextSnapshot` | Not mergeable; rebuild/regenerate. |
| `TechniqueValidationReport` | Rebuildable; not mergeable. |
| `ScalingBreakdown` | Rebuildable; not mergeable. |

No power artifact is mergeable by default.

## Stability and Retention

| Artifact | Stability Class | Retention Strategy |
|---|---|---|
| `TechniqueDefinition` | Replay-stable within schema/version assumptions | State-retained by asset infrastructure |
| `PowerProfileDefinition` | Replay-stable within schema/version assumptions | State-retained by asset infrastructure |
| `TechniqueInstance` | Partition-stable or replay-stable depending on authority/save policy | State-retained or checkpoint-retained |
| `PowerProfileInstance` | Partition-stable or replay-stable depending on authority/save policy | State-retained or checkpoint-retained |
| `TechniqueContextSnapshot` | Replay-stable if captured for replay | Ephemeral or journal-retained by policy |
| `TechniqueValidationReport` | Observationally stable for same inputs | Rebuildable |
| `ScalingBreakdown` | Observationally stable for same inputs | Rebuildable |
| `OathState` | Partition-stable or replay-stable if authoritative | State-retained, checkpoint-retained, or journal-retained |
| `ConsequenceState` | Partition-stable or replay-stable if authoritative | State-retained, checkpoint-retained, or journal-retained |
| `TechniqueRequest` | Replay-stable if used for replay/network | Ephemeral, journal-retained, or audit-retained by policy |

The minimal domain crate only promises deterministic validation and scaling for identical inputs.

## Migration Paths

The crate defines domain contracts used by migration paths.

Important migration paths:

```text
TechniqueDraft
  -> TechniqueDefinition
```

Authored to normalized/formed content. Requires content ratification.

```text
PowerProfileDraft
  -> PowerProfileDefinition
```

Authored to normalized/formed content. Requires content ratification.

```text
TechniqueDefinition
  -> TechniqueInstance
```

Formation/instantiation into character-owned state. Requires learn/bind policy.

```text
PowerProfileDefinition
  -> PowerProfileInstance
```

Formation/instantiation into character-owned state.

```text
TechniqueRequest + TechniqueContextSnapshot
  -> TechniqueOutcome
```

Workflow/simulated input to domain result.

```text
TechniqueOutcome
  -> ActionRequest?
```

Power domain export into action domain. Only for generic action-compatible effects.

```text
OathBinding
  -> OathState
  -> ConsequenceState?
```

Persistent power-state lifecycle. Runtime/authority owns application and retention.

## Capability Requirements

Suggested capabilities:

- `ValidateTechniqueCapability`;
- `RatifyTechniqueDraftCapability`;
- `BindOathCapability`;
- `UseTechniqueCapability`;
- `ExportTechniqueToActionCapability`;
- `ApplyPowerConsequenceCapability`;
- `ObservePowerDiagnosticsCapability`;
- `MutatePowerProfileInstanceCapability`.

This crate may define marker contracts later if foundation capability infrastructure exists. Runtime/editor APIs should not rely on ambient mutation authority.

## Core Idea

A character does not simply own spells.

A character has a power profile and may know, bind, or develop techniques. A technique is valid only if it is compatible with the user's profile, the technique instance state, and the current context snapshot.

Core pipeline:

```text
PowerProfileInstance or PowerProfileDefinition
  + TechniqueDefinition
  + TechniqueInstance?
  + TechniqueContextSnapshot
  ↓
TechniqueValidator
  ↓
TechniqueValidationReport
  ↓
TechniquePlanner / TechniqueOutcome
  ↓
TechniqueActionExporter
  ↓
ActionRequest or ActionPlan input
```

Technique meaning must be complete before action export. Action export is an adapter/export step, not the definition of a technique.

## Definition, Instance, Runtime State, and Draft

The crate must keep these concepts separate.

```text
PowerProfileDefinition
  Authored reusable profile archetype or template.

PowerProfileInstance
  Character-owned mutable domain/save-facing profile state.

PowerProfileRuntimeComponent
  Engine-owned ECS reference or hot runtime state. Not owned here.

PowerProfileDraft
  Authoring representation. May be incomplete or invalid.

TechniqueDefinition
  Authored reusable ratified technique description.

TechniqueInstance
  Character-owned learned/bound/customized technique state.

ActiveTechniqueRuntimeState
  Engine-owned active execution/channel/cooldown state. Not owned here.

TechniqueDraft
  Authoring representation. May be incomplete or invalid.
```

This distinction prevents mixing authored asset data, save-game character state, and frame-runtime ECS data.

## Identity, Version, and Causality

The crate should define or use stable value types for:

- `PowerProfileDefinitionId`;
- `PowerProfileInstanceId`;
- `PowerProfileVersion`;
- `TechniqueDefinitionId`;
- `TechniqueDefinitionVersion`;
- `TechniqueInstanceId`;
- `TechniqueSchemaVersion`;
- `TechniqueRequestId`;
- `TechniqueCorrelationId`;
- `TechniqueContentHash`;
- optional `TechniqueCausalityId` if not provided by foundation.

Definitions and drafts are authored content and must be versionable.

The crate defines schema/version value types. Asset infrastructure owns persistence and migration execution.

Identities should declare:

- scope;
- restart stability;
- replay safety;
- ownership domain.

## Power Profile

`PowerProfileDefinition` describes reusable profile design.

`PowerProfileInstance` represents a character-owned profile state.

A profile may include:

- source;
- primary affinity;
- secondary affinities;
- control;
- output;
- pool/capacity model;
- flow tendencies;
- learned traits;
- seals;
- instability;
- limitations;
- progression state.

ECS runtime may cache or reference these, but ECS components do not define profile semantics.

## Power Source

`PowerSource` describes where power originates.

Examples:

- inner;
- external;
- inherited;
- bound;
- borrowed;
- artifact-mediated;
- environmental;
- pact-based.

This must remain extensible.

## Power Affinity and Expression

`PowerAffinity` belongs primarily to the user/profile.

`PowerExpression` belongs primarily to the technique.

Examples of expressions:

- strengthening;
- shaping;
- directing;
- changing;
- sending;
- binding.

Affinity affects:

- validation;
- efficiency;
- cost;
- stability;
- risk;
- scaling;
- learnability.

Affinity should not be a rigid class system unless explicitly chosen by game design.

## Technique Definition

`TechniqueDefinition` is the stable, ratified domain object for a usable or learnable technique.

It should include:

- technique definition id;
- schema version;
- definition version;
- display/localization key;
- expression;
- medium;
- targeting;
- rank or complexity;
- requirements;
- constraints;
- oaths;
- costs;
- risks;
- consequences;
- effects;
- scaling rules;
- diagnostics metadata;
- stability and retention hints where needed;
- source lineage metadata where inspection/round-tripping matters.

It should not include:

- ECS entity ids;
- UI graph node positions;
- renderer handles;
- input bindings;
- direct engine callbacks.

## Technique Instance

`TechniqueInstance` represents a character-owned instance of a technique.

It may include:

- technique definition id;
- owner/profile instance id;
- learned state;
- mastery level;
- personalized modifiers;
- bound oath state references;
- unlocked variants;
- degradation or instability;
- save-facing state.

Cooldowns and active execution state belong in runtime unless they are persistent save-facing state.

## Technique Draft and Ratification

`TechniqueDraft` is an authoring object.

It may be incomplete, invalid, or inconsistent.

Drafts may live here initially to support ratification, but if authoring grows large they should move to a future crate:

```text
domain/gameplay/power_authoring
```

Ratification pipeline:

```text
TechniqueDraft
  ↓
TechniqueRatifier
  ↓
TechniqueValidationReport
  ↓
TechniqueDefinition
```

The editor manipulates drafts. The domain ratifies drafts into definitions.

## Requirement, Constraint, Oath

The system must distinguish three concepts.

### Requirement

A normal precondition.

Examples:

- target must be visible;
- user must have active flow;
- user must have enough control;
- target must be within range.

Failure means the technique cannot currently execute.

### Constraint

A narrowing rule that limits the technique and may strengthen it.

Examples:

- requires touch;
- requires spoken phrase;
- only usable once per day;
- only usable while wounded;
- only usable against a declared target class.

Failure may reject execution, reduce stability, or prevent scaling benefits depending on rule semantics.

### Oath

A serious self-binding rule with consequence on violation.

Examples:

- never use this against innocents;
- only use this against a named enemy;
- if broken, seal own power;
- if missed, suffer backlash.

Oaths are first-class and must have lifecycle semantics.

## Oath Lifecycle

Oaths require explicit lifecycle modeling.

Concepts:

- `OathBinding`;
- `OathScope`;
- `OathTrigger`;
- `OathViolation`;
- `OathLifecycle`;
- `OathConsequencePolicy`.

Questions every oath must answer:

- When is it bound?
- Who or what is bound?
- Is it attached to the user, technique, target, relationship, item, or pact?
- When is it checked?
- Can it be revoked?
- Can it be suspended?
- Can it be inherited?
- Can violation be accidental?
- What consequence triggers on violation?
- What ratification policy governs binding and violation?

Check timings may include:

- authoring/ratification;
- learning/binding;
- activation;
- start;
- commit;
- hit;
- completion;
- aftermath;
- persistent periodic check.

## Risks and Consequences

Risks describe what can go wrong.

Consequences describe what happens when a risk or violation triggers.

Examples:

- self-damage;
- power lockout;
- permanent scar;
- temporary seal;
- loss of technique;
- delayed backlash;
- lingering corruption;
- instability increase.

Consequences need timing:

- immediate;
- on commit;
- on miss;
- on cancel;
- on violation;
- delayed;
- periodic;
- permanent until cleansed.

The domain defines consequence meaning and timing. Runtime applies it.

## Technique Scaling

Scaling must be explainable.

The crate should prefer rule-composition-based scaling over arbitrary hidden formulas.

Core concepts:

- `ScalingInput`;
- `ScalingRule`;
- `ScalingModifier`;
- `ScalingBreakdown`;
- `ScalingResult`.

Scaling may derive from:

- affinity;
- control;
- output;
- flow;
- cost severity;
- constraint severity;
- oath severity;
- risk severity;
- targeting difficulty;
- technique rank;
- environment facts.

Editor and diagnostics should be able to explain:

```text
Base strength: 1.0
Affinity modifier: +0.25
Touch constraint: +0.15
Named-target oath: +0.70
Instability penalty: -0.20
Final multiplier: 1.90
```

Core principle:

```text
Narrower valid use + higher real consequence may justify stronger output.
```

## Technique Validation

Validation is pure and explainable.

Inputs:

- `PowerProfileDefinition` or `PowerProfileInstance`;
- `TechniqueDefinition`;
- optional `TechniqueInstance`;
- `TechniqueContextSnapshot`.

Output:

- `TechniqueValidationReport`.

Validation report may include:

- rejection diagnostics;
- warnings;
- efficiency notes;
- risk warnings;
- scaling breakdown;
- required missing facts.

Validation must not mutate the world.

## Technique Context Snapshot

`TechniqueContextSnapshot` is supplied by runtime, editor preview, tests, AI, or replay.

It contains immutable facts, not live ECS references.

Possible facts:

- target facts;
- range facts;
- line-of-sight facts;
- spoken oath confirmation;
- current flow state;
- cooldown facts;
- declared target class;
- environment facts;
- resource availability facts;
- current restrictions;
- relevant tags.

The power domain must not query ECS directly.

## Technique Export to Actions

Accepted techniques may export into generic action requests/plans.

```text
TechniqueRequest
  + TechniqueDefinition
  + TechniqueValidationReport
  ↓
TechniqueActionExporter
  ↓
ActionRequest
```

Export converts:

- technique actor to action actor;
- technique target to action target;
- generic costs to action costs;
- generic effects to action effects;
- generic requirements to action requirements;
- metadata and correlation ids.

Not all power behavior is an action.

Power runtime owns persistent power-state transitions and long-lived consequences such as:

- oath state;
- seals;
- instability;
- power lockout;
- passive pressure;
- profile mutation.

## Diagnostics

Use `foundation/diagnostics`.

Diagnostic domain prefix:

```text
power.*
```

Example codes:

- `power.profile.insufficient_control`;
- `power.profile.incompatible_affinity`;
- `power.technique.missing_expression`;
- `power.technique.invalid_target`;
- `power.technique.oath_would_be_broken`;
- `power.technique.constraint_not_satisfied`;
- `power.technique.insufficient_power`;
- `power.technique.unstable_scaling`;
- `power.oath.violation_detected`;
- `power.scaling.unexplainable_modifier`.

Diagnostic subjects should support:

- profile definition id;
- profile instance id;
- technique definition id;
- technique instance id;
- oath id;
- constraint id;
- scaling rule id;
- context fact id.

## Asset and Schema Policy

The crate defines domain contracts and schema version value types.

It does not own:

- asset files;
- persistence;
- editor document storage;
- registry population;
- migration execution.

Asset/document infrastructure owns storage and migration.

The domain crate should expose enough metadata for migration and diagnostics.

## Invariants

1. Power validation is pure and side-effect-free.
2. Runtime state is supplied as immutable context snapshots.
3. Technique drafts may be invalid; technique definitions must be ratified.
4. Definitions, instances, runtime components, and drafts remain separate concepts.
5. Oaths and consequences are first-class domain concepts.
6. Oaths have explicit lifecycle and scope.
7. Scaling is explainable through a breakdown.
8. Technique export to actions must not bypass validation.
9. Rejected technique usage must carry diagnostics.
10. The crate must be usable without ECS or engine startup.
11. The crate may depend on generic actions, but actions must not depend on powers.
12. Game-facing poetic naming must not make code semantics unclear.
13. The domain must be deterministic enough for tests, replay, and tooling.
14. Runtime success is not automatically ratified power-state change.
15. Oath and consequence state are not mergeable by default.
16. Doctrine metadata must be represented in types where it governs behavior.

## Suggested Source Layout

```text
domain/gameplay/powers/
  README.md
  design.md
  roadmap.md
  Cargo.toml
  src/
    lib.rs

    identity/
      mod.rs
      power_profile_definition_id.rs
      power_profile_instance_id.rs
      technique_definition_id.rs
      technique_instance_id.rs
      technique_request_id.rs
      technique_correlation_id.rs

    doctrine/
      mod.rs
      power_reality.rs
      power_reconciliation_policy.rs
      power_stability_class.rs
      power_retention_hint.rs
      power_ratification_hint.rs

    schema/
      mod.rs
      power_profile_schema_version.rs
      technique_schema_version.rs
      technique_definition_version.rs
      technique_content_hash.rs

    profile/
      mod.rs
      power_profile_definition.rs
      power_profile_instance.rs
      power_source.rs
      power_affinity.rs
      power_expression.rs
      power_pool.rs
      power_state.rs
      power_flow.rs
      power_control.rs
      power_output.rs

    technique/
      mod.rs
      technique_definition.rs
      technique_instance.rs
      technique_kind.rs
      technique_medium.rs
      technique_targeting.rs
      technique_rank.rs

    oath/
      mod.rs
      oath_binding.rs
      oath_scope.rs
      oath_trigger.rs
      oath_violation.rs
      oath_lifecycle.rs

    constraints/
      mod.rs
      technique_requirement.rs
      technique_constraint.rs
      technique_consequence.rs
      technique_risk.rs

    scaling/
      mod.rs
      scaling_input.rs
      scaling_rule.rs
      scaling_modifier.rs
      scaling_breakdown.rs
      scaling_result.rs

    costs/
      mod.rs
      technique_cost.rs
      power_cost.rs

    effects/
      mod.rs
      technique_effect.rs
      power_effect.rs

    execution/
      mod.rs
      technique_request.rs
      technique_context_snapshot.rs
      technique_outcome.rs
      technique_event.rs
      technique_action_exporter.rs

    validation/
      mod.rs
      technique_validator.rs
      profile_validator.rs
      technique_validation_report.rs
      technique_diagnostic.rs

    authoring/
      mod.rs
      power_profile_draft.rs
      technique_draft.rs
      authored_constraint.rs
      authored_oath.rs
      technique_ratifier.rs
```

## Public API Shape

Expose:

- identity;
- doctrine metadata;
- schema;
- profile definitions and instances;
- technique definitions and instances;
- oath lifecycle types;
- constraints/consequences;
- scaling breakdown;
- costs/effects;
- validation reports;
- technique requests/outcomes;
- technique action exporter;
- authoring drafts and ratifiers.

Avoid exposing internal helper modules broadly.

## Relationship to `domain/gameplay/actions`

Powers uses actions to export accepted technique execution into the generic action pipeline.

Rule:

```text
Powers may produce actions.
Actions do not know about powers.
Technique meaning exists before action export.
```

## Relationship to `engine/gameplay/power_runtime`

Runtime owns:

- ECS components;
- power regeneration;
- cooldown ticking;
- active technique state;
- building context snapshots from ECS/world;
- applying consequences;
- emitting engine events;
- delegating generic effects to action runtime;
- authority/ratification handoff for governing power changes.

The power domain owns:

- semantics;
- validation;
- scaling;
- oath lifecycle model;
- consequence model;
- technique export contract;
- diagnostics;
- doctrine metadata for its artifacts.

## Testing Strategy

Test without ECS.

Required tests:

- valid technique accepted;
- invalid target rejected;
- insufficient control rejected;
- insufficient power rejected;
- incompatible affinity warning/rejection;
- oath binding lifecycle;
- oath violation detection;
- consequence timing classification;
- scaling breakdown stability;
- draft ratification rejects incomplete drafts;
- technique export rejects invalid validation report;
- diagnostics are stable;
- context snapshots are immutable facts;
- doctrine metadata mapping is correct.

## Naming Policy

Use clear English code names:

- `PowerProfileDefinition`;
- `PowerProfileInstance`;
- `TechniqueDefinition`;
- `TechniqueInstance`;
- `TechniqueConstraint`;
- `TechniqueOath`;
- `TechniqueConsequence`;
- `PowerAffinity`;
- `PowerExpression`.

Use German-inspired/lore names only in content, localization, UI, and worldbuilding.

## Implementation Readiness

This crate is ready after:

1. `domain/gameplay/actions` has stable request/context/effect/planning contracts.
2. Oath lifecycle is accepted.
3. Scaling breakdown model is accepted.
4. Definition/instance/runtime/draft separation is accepted.
5. Doctrine policy enum location is decided.
