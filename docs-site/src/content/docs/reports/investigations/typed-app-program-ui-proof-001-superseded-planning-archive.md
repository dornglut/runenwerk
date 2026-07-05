---
title: Typed App Program UI Proof 001 Superseded Planning Archive
description: Historical archive of the superseded Typed App Program UI Proof 001 planning contract, retained for audit and future proof-pressure extraction.
status: archived
owner: app-program
layer: workspace
canonical: false
last_reviewed: 2026-07-05
related_docs:
  - ../../workspace/planning/typed-app-program-ui-proof-001-planning.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/decision-register.md
  - ../../design/active/ui-framework-app-integration-direction-review.md
  - ../../design/active/typed-app-program-and-ui-proof-design.md
---

# Typed App Program UI Proof 001 Superseded Planning Archive

## Archive status

This document preserves the detailed planning contract that was active before `PT-UI-FRAMEWORK-APP-INTEGRATION-001 — UI Framework App Integration Direction Review` superseded it.

This archive is not active implementation authority. Use it only as historical evidence and as future route/action/replay/report pressure input.

Active authority after supersession:

```text
docs-site/src/content/docs/design/active/ui-framework-app-integration-direction-review.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/decision-register.md
```

## Archived implementation-planning contract

### Original purpose

Define the implementation-planning contract for the first Typed App Program proof:

```text
Typed App Program UI Proof 001 — Headless Counter App Proof
```

The proof demonstrated the full local app-program loop without creating a shared plugin framework, editor integration, game runtime integration, engine subsystem implementation, networking implementation, or `foundation/meta`.

The old plan required a durable app-program core as its own crate. UI was the first proving consumer, not the owner of the app-program architecture.

### Original lifecycle state

```text
active-planning
```

The old document prepared implementation authorization. It did not by itself implement product code. Implementation could begin only after the planning contract was reviewed, accepted, and active-work pointed to that proof as the active implementation focus.

### Original prerequisite authority

Implementation planning built on the merged Typed App Program investigation/design authority from PR #66:

```text
docs-site/src/content/docs/reports/investigations/typed-app-program-current-state-investigation.md
docs-site/src/content/docs/design/active/typed-app-program-and-ui-proof-design.md
docs-site/src/content/docs/reports/investigations/typed-app-program-engine-pressure-and-design-review.md
docs-site/src/content/docs/reports/investigations/typed-app-program-multiplayer-concurrency-design-review.md
docs-site/src/content/docs/reports/investigations/typed-app-program-cross-cutting-design-review.md
```

### Original implementation scope

Implement a dedicated, UI-independent app-program core crate plus a UI-backed headless counter proof:

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

Ownership split:

```text
domain/app_program
  owns generic local app-program contracts: model, action, route-action mapping, reducer, effect plan, replay, reports.

domain/app_program/examples/headless_counter_ui.rs
  owns the demonstration of the first UI-backed headless counter proof.

domain/app_program/tests/headless_counter_replay.rs
  owns validation of the proof behavior.

ui_program / ui_hosts / ui_binding / ui_evaluator / ui_testing
  are proving consumers or dev/test dependencies, not owners of app-program meaning.
```

The first proof used a counter app:

```text
count = 0 -> counter screen
counter.increment route -> CounterAction::Increment
pure reducer increments count
count >= 5 -> win screen
counter.reset route -> CounterAction::Reset
pure reducer resets count to 0
```

### Original required positive scenario

The proof had to execute and report this scenario:

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

### Original required negative scenarios

The proof had to fail closed and report distinct diagnostics for:

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

### Original required reports

Implementation had to produce inspectable proof structures equivalent to:

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

### Original vocabulary required in code

The first implementation should introduce generic app-program vocabulary in `domain/app_program`:

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

The first implementation could use only:

```text
AppActionSource::LocalHeadless
AppEffectPlan::NoEffect
single logical event order
single-threaded deterministic replay
```

The data model could not make those local defaults the only possible future shape.

### Original exact owner

First proof owner:

```text
domain/app_program
```

Reason:

```text
Typed App Program is cross-domain structure, not UI-owned behavior.
A dedicated crate prevents the first proof from burying durable app-program concepts inside ui_testing.
UI is the first proving consumer because UiProgram and headless UI proof infrastructure already exist.
The counter demo belongs in an example/demo surface, not inside production UI crates.
```

Long-term shared plugin/app composition ownership remained undecided and blocked until at least one non-UI proof validated repeated domain-neutral structure. This crate was not allowed to become `AppRecipe`, `PluginSuite`, or a universal framework in the first proof.

### Original allowed files and crates

Allowed implementation files for the first proof:

```text
Cargo.toml
domain/app_program/Cargo.toml
domain/app_program/src/lib.rs
domain/app_program/src/ids.rs
domain/app_program/src/model.rs
domain/app_program/src/action.rs
domain/app_program/src/route_action.rs
domain/app_program/src/reducer.rs
domain/app_program/src/effect.rs
domain/app_program/src/projection.rs
domain/app_program/src/replay.rs
domain/app_program/src/report.rs
domain/app_program/src/counter.rs
domain/app_program/examples/headless_counter_ui.rs
domain/app_program/tests/headless_counter_replay.rs
```

Allowed only if required by compiler/test wiring:

```text
domain/ui/ui_program/src/events/mod.rs
domain/ui/ui_hosts/src/lib.rs
```

Any change outside this list required returning to planning/design with a reason.

### Original dependency rules

Production dependency rules for `domain/app_program`:

```text
The app_program library crate must not depend on UI crates, editor crates, game crates, engine crates, net crates, material_graph, procgen, renderer backends, or foundation/meta.
It may depend only on low-level foundation crates needed for IDs, diagnostics, schema/value shape, or resource refs if those are required by implementation.
```

Dev/test/example dependency rules:

```text
Examples and tests may use ui_program, ui_schema, ui_compiler, ui_artifacts, ui_evaluator, ui_binding, ui_hosts, ui_state, ui_runtime_view, ui_accessibility, ui_geometry, and ui_testing as dev-dependencies to prove UI integration.
```

Forbidden dependencies:

```text
app_program production code must not depend on editor_shell, game runtime, renderer backend, engine scheduler, physics, asset loading, streaming, network, material_graph, procgen, ui_testing, or foundation/meta.
```

### Original non-owned files and crates

Implementation must not change:

```text
foundation/meta
foundation/commands execution behavior
engine scheduler/runtime implementation
engine physics/simulation implementation
engine asset loading/import implementation
engine streaming/LOD implementation
engine renderer/resource implementation
net engine/networking implementation
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

### Original demo and example placement

The first proof had to include a demo/example surface:

```text
domain/app_program/examples/headless_counter_ui.rs
```

Example rules:

```text
The example demonstrates the local counter app replay over UI/UiProgram/headless evaluation.
The example may use UI crates through dev-dependencies only.
The example must not be the only validation surface; tests must assert proof behavior.
The example must not perform windowing, renderer backend work, engine IO, asset loading, networking, or editor/game integration.
```

### Original principle compliance matrix

| Principle | Requirement |
| --- | --- |
| KISS | One app_program crate, one proof app, one headless host mode, local deterministic replay only. |
| DRY | Reuse existing UiProgram, UiEventPacket, ui_hosts, ui_binding, ui_evaluator, and ui_testing proof infrastructure from examples/tests; do not duplicate UI program semantics. |
| YAGNI | Do not implement actors, async workflow, multiplayer, asset loading, streaming, scheduler, editor integration, game integration, hot reload, localization, telemetry, plugin framework, or generic app recipe composition. |
| SOLID | Separate IDs, model, action, route-action resolution, reducer, effect, projection, replay, report, counter fixture, and UI example. |
| Separation of Concerns | app_program owns app structure; UI program emits UI facts/events; hosts remain external; domains own meaning. |
| Avoid Premature Optimization | No parallelism, no streaming, no runtime scheduling, no hot-path optimization. |
| Law of Demeter | Communicate through packets, snapshots, maps, and reports; do not reach into renderer, engine, editor, game, ECS, IO, or network internals. |

### Original module decomposition map

```text
Cargo.toml
  Adds `domain/app_program` to the workspace and, if used elsewhere, workspace dependency metadata.

domain/app_program/Cargo.toml
  Defines the app_program crate, low-level production dependencies, and UI/dev dependencies for tests/examples only.

lib.rs
  Public module exports and crate-level boundary docs.

ids.rs
  AppProgramId, AppProgramVersion, AppModelId, AppModelVersion, AppActionId, AppActionVersion, AppModelRevision.

model.rs
  AppModelSnapshot and generic model revision/state envelope.

action.rs
  AppAction, AppActionPayload, AppActionCapability, AppActionSource.

route_action.rs
  RouteActionMap, RouteActionMapping, RouteActionResolution, fail-closed diagnostics.

reducer.rs
  AppReducerInput, AppReducerOutcome, pure reducer contract helpers.

effect.rs
  AppEffectPlan, AppEffect, NoEffect, future-shape reserved without implementation.

projection.rs
  AppViewProjection, AppViewProjectionReport, UI-independent projection contract.

replay.rs
  AppReplayScenario, AppReplayStep, AppReplayTrace, deterministic replay runner.

report.rs
  AppProgramReport and phase-specific diagnostic/report summaries.

counter.rs
  Counter proof model/action/reducer/projection fixture used by tests/examples. Counter semantics remain demo-owned, not platform meaning.

examples/headless_counter_ui.rs
  Demo/example showing the headless counter replay over UI/UiProgram/headless evaluation.

tests/headless_counter_replay.rs
  Tests asserting the complete proof and negative cases.
```

### Original feature support matrix

| Feature | First Proof Requirement | Future Pressure |
| --- | --- | --- |
| App model snapshot | Required in app_program | Versioned, serializable, migration-aware. |
| Action IDs and versions | Required in app_program | Source/authority/causality metadata later. |
| Route-action mapping | Required in app_program | Host/domain mapping later. |
| Reducer trace | Required in app_program | Undo/redo/history adapters later. |
| Effect plan | NoEffect only | Host/domain/network/asset proposals later. |
| UI proof integration | Required through example/test dev-deps | Other domain proofs later. |
| Host compatibility | Headless capability facts only | Editor/game/world/network hosts later. |
| Deterministic replay | Required | Multiplayer rollback/reconciliation pressure later. |
| Security/permissions | Missing capability negative case only | Policy/authorization owner later. |
| Localization | Stable IDs not visible labels | Content/UI localization later. |
| Multithreading | No shared mutable global state | Thread-safe snapshots/queues later. |
| Multiplayer | Local source metadata only | Authority/replication/prediction later. |
| Engine systems | Not implemented | Inert proposals/host facts later. |

### Original validation commands

Required before implementation PR merge:

```bash
cargo test -p app_program
cargo test -p app_program --test headless_counter_replay
cargo test -p app_program --examples
cargo test -p ui_program event
cargo test -p ui_hosts route
cargo test -p ui_binding host_data
cargo test -p ui_evaluator
cargo test --workspace
python tools/docs/validate_docs.py
git diff --check
```

If exact test filters differed after implementation, the implementation PR had to explain the substitution and show equivalent coverage.

Docs-only planning PR validation:

```bash
python tools/docs/validate_docs.py
git diff --check
```

### Original stop conditions

Stop and return to planning/design if implementation tried to:

```text
put app-program core types back inside domain/ui/ui_testing
make app_program production code depend on UI crates
modify product/editor/game behavior
modify engine subsystem behavior
modify network/multiplayer behavior
modify Phase 17 SpatialCanvas scope
add callback execution to ui_definition or ui_controls
make ui_state the generic app model
make ui_hosts execute effects
make foundation/commands execute or route commands
add AppRecipe/PluginSuite/shared plugin framework behavior
add scheduler, thread pool, or async runtime behavior
perform asset IO, file IO, streaming, LOD, physics, renderer resource, or world mutation work
use scheduler order as replay order
use wall-clock time, unseeded randomness, global mutable state, or hidden callbacks in reducers
store raw private payloads in reports
use visible localized labels as durable action identity
allow rejected actions to mutate model state
skip distinct diagnostics for route/action/reducer/projection/effect/replay failures
```

### Original closeout evidence required

Implementation closeout had to record:

```text
changed files and ownership check
new crate boundary check
production dependency check showing app_program is UI-independent
example/dev-dependency check showing UI usage is test/demo-only
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

### Original open questions deferred by this planning contract

These were intentionally not solved in the first proof:

```text
second non-UI proof domain
shared extraction beyond app_program local contracts
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

Deferring them was acceptable because the design/review artifacts classified their pressure and set stop conditions. The first proof was not allowed to close these questions accidentally.

### Original next action after planning acceptance

After the old planning PR was reviewed and accepted:

```text
Open implementation branch for Typed App Program UI Proof 001.
Create the app_program crate and the headless counter UI example.
Run the full validation envelope.
Close out with evidence.
```

## Archive conclusion

This archived contract remains useful for extracting fail-closed action routing, replay, report, and diagnostics requirements. It is not the active implementation path for the UI framework.
