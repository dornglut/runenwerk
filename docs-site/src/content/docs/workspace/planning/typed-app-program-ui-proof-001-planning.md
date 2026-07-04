---
title: Typed App Program UI Proof 001 Planning
description: Implementation-planning contract for the first Typed App Program proof, a headless counter app replay over the existing UI program stack.
status: active
owner: ui
layer: workspace
canonical: false
last_reviewed: 2026-07-04
related_docs:
  - ./active-work.md
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../complete-merge-readiness-gate.md
  - ../../design/active/typed-app-program-and-ui-proof-design.md
  - ../../reports/investigations/typed-app-program-current-state-investigation.md
  - ../../reports/investigations/typed-app-program-engine-pressure-and-design-review.md
  - ../../reports/investigations/typed-app-program-multiplayer-concurrency-design-review.md
  - ../../reports/investigations/typed-app-program-cross-cutting-design-review.md
---

# Typed App Program UI Proof 001 Planning

## Purpose

Define the implementation-planning contract for the first Typed App Program proof:

```text
Typed App Program UI Proof 001 — Headless Counter App Proof
```

The proof demonstrates the full local app-program loop without creating a shared plugin framework, new cross-domain app runtime, editor integration, game runtime integration, engine subsystem implementation, networking implementation, or `foundation/meta`.

## Lifecycle State

```text
active-planning
```

This document prepares implementation authorization. It does not by itself implement product code. Implementation may begin only after this planning contract is reviewed, accepted, and active-work points to this proof as the active implementation focus.

## Prerequisite Authority

Implementation planning builds on the merged Typed App Program investigation/design authority from PR #66:

```text
docs-site/src/content/docs/reports/investigations/typed-app-program-current-state-investigation.md
docs-site/src/content/docs/design/active/typed-app-program-and-ui-proof-design.md
docs-site/src/content/docs/reports/investigations/typed-app-program-engine-pressure-and-design-review.md
docs-site/src/content/docs/reports/investigations/typed-app-program-multiplayer-concurrency-design-review.md
docs-site/src/content/docs/reports/investigations/typed-app-program-cross-cutting-design-review.md
```

If that merged authority changes materially, this planning contract must be reviewed again before implementation.

## Implementation Scope

Implement a local, headless proof of the Typed App Program loop:

```text
AppModelSnapshot
  -> AppViewProjection
  -> UiProgram / UiRuntimeArtifact / UiOutput
  -> UiEventPacket
  -> RouteActionMap
  -> AppAction
  -> AppReducer
  -> AppEffectPlan
  -> AppModelSnapshot revision N+1
  -> AppReplayTrace / AppProgramReport
```

The first proof uses a counter app:

```text
count = 0 -> counter screen
counter.increment route -> CounterAction::Increment
pure reducer increments count
count >= 5 -> win screen
counter.reset route -> CounterAction::Reset
pure reducer resets count to 0
```

## Required Positive Scenario

The proof must execute and report this scenario:

```text
1. initial model snapshot: count = 0, screen = counter
2. project to UI and evaluate first output
3. resolve counter.increment route to CounterAction::Increment
4. reducer transitions count 0 -> 1
5. repeat increments through count 5
6. projection switches to win screen
7. resolve counter.reset route to CounterAction::Reset
8. reducer transitions count 5 -> 0
9. projection returns to counter screen
```

## Required Negative Scenarios

The proof must fail closed and report distinct diagnostics for:

```text
unknown route
wrong route schema version
invalid action payload
missing capability / unauthorized route
route diagnostic rejection
reducer diagnostic rejection
projection diagnostic rejection
no mutation after rejected action
```

## Required Reports

Implementation must produce inspectable proof structures equivalent to:

```text
AppReplayTrace
AppProgramReport
RouteActionResolutionReport
CounterReducerTraceReport
CounterViewProjectionReport
CounterEffectPlanReport
UiProgram build or lowering report
UiCompilerReport
UiEvaluation output diagnostics
Headless reproducibility evidence
```

Report requirements:

```text
stable ordering
distinct diagnostic namespaces
model revisions before and after each reducer step
route/action/effect IDs and versions
local action source metadata or an explicitly reserved source field
safe bounded payload summary
no raw private payload assumption
deterministic pass/fail summary
```

## Vocabulary Required In Code

The first implementation should introduce proof-local equivalents of:

```text
AppProgramId
AppProgramVersion
AppModelId
AppModelVersion
AppModelSnapshot
AppModelRevision
AppActionId
AppActionVersion
AppActionPayload
AppActionCapability
AppActionSource
RouteActionMap
RouteActionMapping
RouteActionResolution
AppReducerInput
AppReducerOutcome
AppEffectPlan
AppEffect
AppReplayScenario
AppReplayStep
AppReplayTrace
AppProgramReport
AppViewProjection
AppViewProjectionReport
```

The first implementation may use only:

```text
AppActionSource::LocalHeadless
AppEffectPlan::NoEffect
single logical event order
single-threaded deterministic replay
```

The data model must not make those local defaults the only possible future shape.

## Exact Owner

First proof owner:

```text
domain/ui/ui_testing
```

Reason:

```text
ui_testing already owns headless proof fixtures and proof-style validation helpers.
The first proof is a proving slice, not production app runtime ownership.
```

Long-term generic ownership remains undecided and blocked until at least one non-UI proof validates repeated domain-neutral structure.

## Allowed Files And Crates

Allowed implementation files for the first proof:

```text
domain/ui/ui_testing/src/lib.rs
domain/ui/ui_testing/src/app_program/mod.rs
domain/ui/ui_testing/src/app_program/ids.rs
domain/ui/ui_testing/src/app_program/model.rs
domain/ui/ui_testing/src/app_program/action.rs
domain/ui/ui_testing/src/app_program/route_action.rs
domain/ui/ui_testing/src/app_program/reducer.rs
domain/ui/ui_testing/src/app_program/effect.rs
domain/ui/ui_testing/src/app_program/projection.rs
domain/ui/ui_testing/src/app_program/replay.rs
domain/ui/ui_testing/src/app_program/report.rs
domain/ui/ui_testing/src/app_program/counter_fixture.rs
domain/ui/ui_testing/tests/typed_app_program_counter.rs
```

Allowed only if required by compiler/test wiring:

```text
domain/ui/ui_testing/Cargo.toml
domain/ui/ui_program/src/events/mod.rs
domain/ui/ui_hosts/src/lib.rs
```

Any change outside this list requires returning to planning/design with a reason.

## Non-Owned Files And Crates

Implementation must not change:

```text
foundation/meta
foundation/commands execution behavior
engine scheduler/runtime implementation
engine physics/simulation implementation
engine asset loading/import implementation
engine streaming/LOD implementation
engine renderer/resource implementation
domain/editor/editor_shell command execution
game runtime/HUD/world-space implementation
material_graph/procgen implementation
ui_definition callback or behavior execution
ui_controls app mutation behavior
ui_state generic app model ownership
ui_hosts host effect execution
AppRecipe / PluginSuite / shared plugin framework files
Phase 17 SpatialCanvas files
```

## Dependency Rules

Allowed dependencies:

```text
ui_testing may use existing ui_program, ui_schema, ui_compiler, ui_artifacts, ui_evaluator, ui_binding, ui_hosts, ui_state, ui_runtime_view, ui_accessibility, and ui_geometry dependencies already present or needed for existing headless proof style.
```

Forbidden dependencies:

```text
ui_testing app_program proof must not depend on editor_shell, game runtime, renderer backend, engine scheduler, physics, asset loading, streaming, network, material_graph, procgen, or foundation/meta.
```

## Principle Compliance Matrix

| Principle | Requirement |
| --- | --- |
| KISS | One proof app, one host mode, local deterministic replay only. |
| DRY | Reuse existing UiProgram, UiEventPacket, ui_hosts, ui_binding, ui_evaluator, and ui_testing proof infrastructure. |
| YAGNI | Do not implement actors, async workflow, multiplayer, asset loading, streaming, scheduler, editor integration, game integration, hot reload, localization, telemetry, or plugin framework. |
| SOLID | Separate IDs, model, action, route-action resolution, reducer, effect, projection, replay, and reports. |
| Separation of Concerns | UI program emits UI facts/events; app proof owns local model/action/reducer/effect trace; host execution remains external. |
| Avoid Premature Optimization | No parallelism, no streaming, no runtime scheduling, no hot-path optimization. |
| Law of Demeter | Communicate through packets, snapshots, maps, and reports; do not reach into renderer, engine, editor, game, ECS, or IO internals. |

## Module Decomposition Map

```text
ids.rs
  AppProgramId, AppProgramVersion, AppModelId, AppModelVersion, AppActionId, AppActionVersion, AppModelRevision.

model.rs
  AppModelSnapshot and counter-specific local model fixture representation.

action.rs
  AppAction, AppActionPayload, AppActionCapability, AppActionSource, CounterAction fixtures.

route_action.rs
  RouteActionMap, RouteActionMapping, RouteActionResolution, fail-closed diagnostics.

reducer.rs
  AppReducerInput, AppReducerOutcome, pure counter reducer, no mutation after rejection.

effect.rs
  AppEffectPlan, AppEffect, NoEffect, future-shape reserved without implementation.

projection.rs
  AppViewProjection, AppViewProjectionReport, counter/win screen projection into UiProgram or reliable UiProgram inputs.

replay.rs
  AppReplayScenario, AppReplayStep, AppReplayTrace, deterministic replay runner.

report.rs
  AppProgramReport and phase-specific diagnostic/report summaries.

counter_fixture.rs
  Headless Counter App Proof fixture and positive/negative scenario builders.

typed_app_program_counter.rs
  Tests asserting the complete proof and negative cases.
```

## Feature Support Matrix

| Feature | First Proof Requirement | Future Pressure |
| --- | --- | --- |
| App model snapshot | Required | Versioned, serializable, migration-aware. |
| Action IDs and versions | Required | Source/authority/causality metadata later. |
| Route-action mapping | Required | Host/domain mapping later. |
| Reducer trace | Required | Undo/redo/history adapters later. |
| Effect plan | NoEffect only | Host/domain/network/asset proposals later. |
| Host compatibility | Headless capability facts only | Editor/game/world/network hosts later. |
| Deterministic replay | Required | Multiplayer rollback/reconciliation pressure later. |
| Security/permissions | Missing capability negative case only | Policy/authorization owner later. |
| Privacy/redaction | Safe bounded payload summary only | Redaction policy later. |
| Accessibility | Preserve UI output/accessibility proof where existing | Full UI accessibility integration later. |
| Localization | Stable IDs not visible labels | Content/UI localization later. |
| Multithreading | No shared mutable global state | Thread-safe snapshots/queues later. |
| Multiplayer | Local source metadata only | Authority/replication/prediction later. |
| Engine systems | Not implemented | Inert proposals/host facts later. |

## Validation Commands

Required before implementation PR merge:

```bash
cargo test -p ui_testing typed_app_program_counter
cargo test -p ui_testing app_program
cargo test -p ui_program event
cargo test -p ui_hosts route
cargo test -p ui_binding host_data
cargo test -p ui_evaluator
cargo test --workspace
python tools/docs/validate_docs.py
git diff --check
```

If exact test filters differ after implementation, the implementation PR must explain the substitution and show equivalent coverage.

Docs-only planning PR validation:

```bash
python tools/docs/validate_docs.py
git diff --check
```

## Stop Conditions

Stop and return to planning/design if implementation tries to:

```text
create a new crate
modify product/editor/game behavior
modify engine subsystem behavior
modify Phase 17 SpatialCanvas scope
add callback execution to ui_definition or ui_controls
make ui_state the generic app model
make ui_hosts execute effects
make foundation/commands execute or route commands
add AppRecipe/PluginSuite/shared plugin framework behavior
add networking, scheduler, thread pool, or async runtime behavior
perform asset IO, file IO, streaming, LOD, physics, renderer resource, or world mutation work
use scheduler order as replay order
use wall-clock time, unseeded randomness, global mutable state, or hidden callbacks in reducers
store raw private payloads in reports
use visible localized labels as durable action identity
allow rejected actions to mutate model state
skip distinct diagnostics for route/action/reducer/projection/effect/replay failures
```

## Closeout Evidence Required

Implementation closeout must record:

```text
changed files and ownership check
positive scenario proof summary
negative scenario proof summary
report structures produced
validation commands and results
proof that no non-owned files/crates changed
proof that no engine/multiplayer/security/hot-reload/localization subsystem was implemented
principle compliance review
module decomposition review
follow-up decision: next proof, editor integration, or hold
```

## Open Questions Deferred By This Planning Contract

These are intentionally not solved in the first proof:

```text
final cross-domain AppProgram crate ownership
second non-UI proof domain
AppRecipe / PluginSuite integration
editor/workbench integration
game HUD integration
world-space UI integration
network/multiplayer authority model
async effect lifecycle implementation
asset loading/streaming/LOD/physics integration
persistence and migration implementation
hot reload implementation
telemetry implementation
full localization implementation
```

Deferring them is acceptable because the design/review artifacts classify their pressure and set stop conditions. The first proof must not close these questions accidentally.

## Next Action After This PR

After this planning PR is reviewed and accepted:

```text
Open implementation branch for Typed App Program UI Proof 001.
Implement only the allowed files and proof behavior.
Run the full validation envelope.
Close out with evidence.
```
