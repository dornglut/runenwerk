---
title: Power/Action Doctrine-Aligned Architecture Review Summary
description: Deferred architecture review summary for gameplay actions, powers, runtime, and editor authoring design.
status: deferred
owner: gameplay
layer: domain
canonical: true
last_reviewed: 2026-04-27
---

# Power/Action Doctrine-Aligned Architecture Review Summary

## Final Rating

Final architecture rating after doctrine loop: **9.4 / 10**

The architecture is doctrine-aligned enough to proceed with `domain/gameplay/actions` design implementation planning.

The remaining 0.6 is intentionally not forced to 10 because several final details depend on existing Runenwerk foundation conventions:

- whether doctrine policy enums live in foundation or per-domain modules;
- existing ID/value-object strategy;
- existing diagnostics subject model;
- asset/document infrastructure ownership;
- authority/ratification contract names.

## Loop Result

The v2 architecture already had correct crate split and strong local boundaries.

The doctrine loop added explicit coverage for:

- reality classification;
- ratification policy;
- reconciliation policy;
- stability classes;
- retention strategies;
- migration paths;
- capability requirements;
- observation frames for editor;
- ratification handoff for runtime;
- doctrine metadata modules in source layout.

## Final Crate Shape

```text
domain/gameplay/actions
domain/gameplay/powers
engine/gameplay/action_runtime
engine/gameplay/power_runtime
apps/editor/power_editor
```

This shape remains correct.

## Final Dependency Graph

```text
foundation/diagnostics
foundation/id
foundation/tags? optional
foundation/time? optional

        ↓
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

domain/gameplay/powers
        ↓
apps/editor/power_editor

editor document/asset infrastructure
        ↓
apps/editor/power_editor
```

`action_runtime` must not depend on `powers`.

`power_runtime` may depend on `action_runtime`.

## Doctrine Mapping

### Authored Reality

- `TechniqueDocument`
- `ProfileDocument`
- `TechniqueDraft`
- `PowerProfileDraft`
- optional `ActionDraft`

### Normalized / Formed Reality

- `ActionDefinition`
- `TechniqueDefinition`
- `PowerProfileDefinition`

### Instantiated Reality

- `TechniqueInstance`
- `PowerProfileInstance`

### Simulated Reality

- `ActionRuntimeState`
- `ActiveActionComponent`
- `PowerPoolComponent`
- `ActiveTechniqueComponent`
- `OathStateComponent`
- `ActiveConsequenceComponent`

### Observed Reality

- `ActionContextSnapshot`
- `TechniqueContextSnapshot`
- `ActionValidationReport`
- `TechniqueValidationReport`
- `ScalingBreakdown`
- `GraphLoweringReport`

### Session Reality

- `DocumentViewState`
- `DocumentSelectionState`
- `RuntimeSandboxPreview`
- editor preview state

### Workflow Reality

- `ActionRequest`
- `TechniqueRequest`
- request queues
- document save/export state

### Ratified Reality

Not owned directly by these crates.

Ratified change is produced through authority/mutation contracts outside these crates.

Runtime application results may feed ratification but are not ratified by default.

## Critical Doctrine Rules Added

1. Local action success is not ratification.
2. Preview success is not ratification.
3. Runtime application success is not authoritative by default.
4. No artifact is mergeable by default.
5. Mergeability must be declared by domain/document policy.
6. Contexts are immutable snapshots, not live world access.
7. Definitions, instances, runtime components, and drafts remain separate.
8. Runtime state belongs to simulated reality.
9. Editor view state belongs to session reality.
10. Domain validation reports belong to observed reality.
11. Migration paths must be explicit.
12. Capability-scoped operations are preferred over ambient authority.

## Implementation Recommendation

Implement only:

```text
domain/gameplay/actions
```

first.

Minimal first implementation subset:

- identity;
- schema;
- doctrine metadata stubs/enums if not foundation-provided;
- action definition;
- action request;
- action context snapshot;
- requirements;
- costs;
- effect declarations;
- validation report;
- action plan;
- diagnostics integration;
- tests.

Then implement:

```text
domain/gameplay/powers
```

minimal subset:

- identity;
- schema;
- doctrine metadata stubs/enums if not foundation-provided;
- profile definition/instance;
- technique definition/instance;
- oath lifecycle;
- scaling breakdown;
- technique context snapshot;
- validation report;
- action export contract;
- tests.

Keep these design-only for now:

```text
engine/gameplay/action_runtime
engine/gameplay/power_runtime
apps/editor/power_editor
```

## Remaining Decisions Before Code

Before coding `domain/gameplay/actions`, decide:

1. Where doctrine policy enums live.
2. Whether `foundation/id` exists or IDs are local newtypes.
3. Exact diagnostics subject model.
4. Whether schema versions are local newtypes or shared foundation values.
5. Whether capability markers are implemented now or left as design contracts.

## Final Verdict

The architecture now matches the Runenwerk doctrine structurally and terminologically.

It is no longer just a layered crate plan. It now declares:

- which reality each artifact belongs to;
- which transitions are migrations;
- which results are not ratification;
- which artifacts are retained or rebuildable;
- which artifacts are mergeable or not;
- which operations require capabilities.

Proceed with implementation only at the pure-domain layer.
