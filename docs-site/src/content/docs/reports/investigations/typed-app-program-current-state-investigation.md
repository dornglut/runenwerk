---
title: Typed App Program Current-State Investigation
description: Current-state investigation for typed app-program architecture, UI proof reuse, app action/reducer/effect replay, and shared-extraction boundaries.
status: active
owner: ui
layer: reports
canonical: false
last_reviewed: 2026-07-04
related_docs:
  - ../../workspace/workflow-lifecycle.md
  - ../../workspace/complete-investigation-gate.md
  - ../../workspace/complete-design-gate.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/production-tracks.md
  - ../../guidelines/domain-program-architecture-pattern.md
  - ../../design/active/runenwerk-domain-workbench-north-star.md
  - ../../design/active/ui-program-architecture.md
  - ../../design/active/ui-program-architecture-owner-map.md
  - ../../design/active/runenwerk-typed-app-composition-plugin-framework-design.md
---

# Typed App Program Current-State Investigation

## Purpose

Investigate whether Runenwerk should add a typed app-program layer for model/action/reducer/effect/replay behavior, how that relates to the existing `UiProgram` implementation, and what must be designed before any implementation work starts.

This report is an investigation artifact only. It does not authorize product code, crate creation, Phase 17 SpatialCanvas implementation, a shared plugin framework, `foundation/meta`, or app/editor/game command execution in `domain/ui`.

## Executive Conclusion

Runenwerk should pursue a typed app-program proof, but not as immediate implementation and not as a generic shared plugin framework.

The correct next direction is:

```text
Typed App Program
  cross-domain architecture concept

UI proves it first
  through existing UiProgram, ui_hosts, ui_testing, and headless proof infrastructure

Shared app/plugin extraction comes later
  only after UI plus at least one non-UI proof expose repeated domain-neutral structure
```

The most important finding is that the repository already contains most of the UI-side program substrate:

```text
ui_program
ui_schema
ui_controls
ui_program_lowering
ui_compiler
ui_artifacts
ui_evaluator
ui_state
ui_binding
ui_hosts
ui_runtime_view
ui_accessibility
ui_geometry
ui_testing
```

The missing layer is not another UI program. The missing layer is an app-model/action/reducer/effect/replay contract that can sit above or beside `UiProgram` without duplicating it.

Recommended first proof after design:

```text
Headless Counter App Proof
```

The proof must include model projection, route event emission, route-to-action mapping, reducer trace, effect plan, deterministic replay report, rejection of unknown/bad/unauthorized events, and a visible state transition such as `count == 5 -> win screen`.

## Workflow Status

This topic triggers the complete investigation and complete design gates because it touches:

- app composition;
- reusable platform capability;
- host integration;
- route/action/effect vocabulary;
- public API shape;
- domain boundary ownership;
- possible new crate pressure;
- future shared extraction pressure;
- proof infrastructure;
- non-UI proving-domain implications.

Immediate implementation is rejected.

Required sequence:

```text
1. Record investigation evidence.
2. Complete a design gate.
3. Decide owner/crate/module placement from the design.
4. Only then authorize a narrow implementation proof.
```

## Source Set Inspected

### Workflow And Planning Authority

- `docs-site/src/content/docs/workspace/workflow-lifecycle.md`
- `docs-site/src/content/docs/workspace/complete-investigation-gate.md`
- `docs-site/src/content/docs/workspace/complete-design-gate.md`
- `docs-site/src/content/docs/workspace/planning/active-work.md`
- `docs-site/src/content/docs/workspace/planning/production-tracks.md`
- PR #65 metadata for `docs/phase-17-spatialcanvas-intake`

### Architecture Authority

- `docs-site/src/content/docs/design/active/runenwerk-domain-workbench-north-star.md`
- `docs-site/src/content/docs/guidelines/domain-program-architecture-pattern.md`
- `docs-site/src/content/docs/design/active/ui-program-architecture.md`
- `docs-site/src/content/docs/design/active/ui-program-architecture-owner-map.md`
- `docs-site/src/content/docs/design/active/runenwerk-typed-app-composition-plugin-framework-design.md`
- `docs-site/src/content/docs/domain/ui/architecture.md`
- `domain/ui/ui-crate-ownership.toml`

### Code Reality

- root `Cargo.toml`
- `domain/ui/ui_program/src/lib.rs`
- `domain/ui/ui_program/src/events/*`
- `domain/ui/ui_program/src/graphs/*`
- `domain/ui/ui_schema/src/lib.rs`
- `domain/ui/ui_state/src/lib.rs`
- `domain/ui/ui_binding/src/lib.rs`
- `domain/ui/ui_hosts/src/lib.rs`
- `domain/ui/ui_artifacts/src/*`
- `domain/ui/ui_compiler/src/*`
- `domain/ui/ui_evaluator/src/*`
- `domain/ui/ui_runtime_view/src/lib.rs`
- `domain/ui/ui_testing/src/*`
- `domain/ui/ui_definition/src/*`
- `domain/ui/ui_program_lowering/src/*`
- `domain/ui/ui_runtime/src/input/*`
- `domain/ui/ui_controls/src/*`
- `domain/ui/ui_surface/src/*`
- `domain/editor/editor_shell/src/workspace/reducer.rs`
- `domain/material_graph/src/*`
- `domain/procgen/src/lib.rs`
- `foundation/commands/src/lib.rs`

## Current Authority Findings

### A. Runenwerk Is Not UI-Only

The Domain Workbench direction frames Runenwerk as a system for typed, versioned, inspectable domain programs across UI, rendering, material graphs, worlds, simulations, assets, and tools.

Therefore typed app-program work must remain cross-domain in concept, even if UI is the first proof.

Correct principle:

```text
Domains own meaning.
The platform owns structure.
```

That means the app-program architecture may standardize structural concepts such as model identity, action identity, schema references, reducers, effect plans, host compatibility, diagnostics, traces, source maps, fixtures, and replay reports.

It must not own meanings such as:

```text
button semantics
material-node semantics
world-region semantics
render-pass semantics
gameplay behavior
editor command behavior
ECS mutation
renderer product truth
```

### B. App Plugin Framework Is A Future Extraction, Not The Next Implementation

The typed app-composition/plugin framework design is directionally useful, but it is explicitly a later shared-extraction target. It should not be implemented from one proving domain.

A shared plugin framework may only be revisited after:

- the UI proving domain uses the architecture spine;
- at least one non-UI proving domain exposes the same domain-neutral pressure;
- an accepted extraction design authorizes exact shared vocabulary;
- the extracted scope excludes domain meaning, execution semantics, universal graph interpretation, and generic compiler/evaluator behavior.

Immediate `AppRecipe`, `PluginSuite`, universal extension registry, or `foundation/meta` work is rejected.

### C. Phase 17 SpatialCanvas Is Separate

The current component platform track names Phase 17 SpatialCanvas as the next future milestone, and PR #65 opens a SpatialCanvas planning/design intake. SpatialCanvas is a reusable component/surface milestone. It must not absorb app-program architecture.

Typed app-program work should be tracked separately from Phase 17.

## Current Code Findings

### Existing UI Program Spine

The workspace already contains the following UI program crates:

```text
ui_schema
ui_program
ui_program_lowering
ui_compiler
ui_artifacts
ui_evaluator
ui_state
ui_binding
ui_hosts
ui_runtime_view
ui_accessibility
ui_geometry
ui_testing
```

This means the correct architecture must reuse the existing spine instead of replacing it.

### `ui_program`

`ui_program` already owns:

- program identity;
- program version;
- graph families;
- source maps;
- diagnostics;
- route IDs;
- route schema versions;
- route capabilities;
- event packets.

It exposes typed graph families:

```text
control
properties
layout
state
style
interaction
binding
visual
accessibility
inspection
```

This is sufficient as the UI-side semantic program contract.

### `UiEventPacket`

`UiEventPacket` already contains:

```text
route
schema_version
source_control
phase
payload
capabilities
source_map
diagnostics
```

This is the correct route event unit for a typed app proof. It should not be replaced with callbacks or ad hoc widget IDs.

### `ui_schema`

`ui_schema` provides UI-local schema/value shape and validation. It already supports route-ref shapes and object validation. It is suitable for UI route payloads and binding payload checks.

A typed app-program design must decide whether app action schemas reuse UI schema for the UI proof or point to a domain-neutral schema vocabulary later. It must not silently couple all future app/domain action payloads to UI-only semantics without design approval.

### `ui_state`

`ui_state` owns UI state buckets:

```text
transient
preview
committed
focus
hover
drag
animation
host-fed
package-owned
```

This is UI state. It must not become generic app model state.

Typed app-program work needs a separate `AppModelSnapshot` / `AppModelContract` concept.

### `ui_binding`

`ui_binding` already owns host-data snapshots, binding snapshots, dirty propagation, authorization, revision policy, collection diffs, and diagnostics.

This is suitable for:

```text
AppModelSnapshot -> host data snapshot -> UI state binding
```

But it does not itself define app selectors, app projection rules, or reducer semantics. Those must be designed separately.

### `ui_hosts`

`ui_hosts` already defines host kinds:

```text
Editor
Game
WorldSpace
Headless
```

It also defines `HostRouteMap`, `HostRouteMapping`, `HostCommand`, optional `DomainCommand`, and route resolution diagnostics.

This is close to the desired event boundary:

```text
UiEventPacket
  -> HostRouteMap
  -> HostCommand / DomainCommand
  -> app/domain-owned execution or proposal handling
```

But it is not a typed app action/reducer contract yet. Current host/domain commands are string ID envelopes, not typed reducer actions with trace evidence.

### `ui_compiler`, `ui_artifacts`, `ui_evaluator`

The compiler already lowers `UiProgram` into `UiRuntimeArtifact`, and the evaluator already evaluates artifacts into `UiOutput` using `UiEvaluationContext` and `UiStateModel`.

Current evaluator behavior proves UI tables, host data, dirty bindings, state rows, interaction rows, visual rows, accessibility rows, inspection rows, and diagnostics.

It does not yet run an app reducer or produce app replay traces.

### `ui_testing`

`ui_testing::HeadlessFixture` already compiles a `UiProgram`, evaluates it with host data, produces accessibility and geometry proof artifacts, checks source-map assertions, checks diagnostic assertions, checks reproducibility, and requires no output diagnostics.

This is the best first proving surface for a typed app-program slice.

### Retained Runtime Interaction Bridge

The retained runtime currently emits semantic interaction results such as:

```text
Activated(WidgetId)
HoveredChanged
PressedChanged
FocusChanged
TextInput
Toggled
NumericStepped
TabSelected
SelectChanged
TableRowSelected
TreeRowSelected
GraphCanvasAction
```

This is useful but not enough. A typed app proof needs a bridge:

```text
UiInteraction + route map + payload schema + capability
  -> UiEventPacket
```

The bridge should be deterministic, source-map capable, and fail closed.

### `ui_definition` And `ui_program_lowering`

`ui_program_lowering` can lower generic authored `UiNodeDefinition::Control` nodes into `UiProgram` graphs using a control package registry snapshot.

It lowers authored route IDs into `RouteId` for interaction handlers.

Gap: authored route validation is looser than `UiProgram` route validation. A design/implementation proof must ensure invalid authored route IDs do not panic or bypass validation during lowering.

### Editor Workspace Reducer

`editor_shell` already has an explicit `WorkspaceMutation` enum and `reduce_workspace` function. This proves that reducer-style domain-owned mutation fits the project style.

However, that reducer is editor-specific and must remain editor-owned. The typed app-program proof should use it as pattern evidence, not as generic UI behavior.

### Foundation Commands

`foundation/commands` provides inert command descriptors and proposals. It explicitly does not execute commands, route proposals, validate domain meaning, register descriptors globally, grant permissions, or map proposals to concrete domain command enums.

This is a candidate vocabulary for future effect plans, but only as inert effect/proposal data. It must not become an execution system.

### Material And Procgen As Non-UI Pressure

`material_graph` and `procgen` already have domain-owned document/catalog/ratification/lowering/product concepts. They are plausible later second-domain proofs for the domain-program architecture pattern.

They should not be pulled into the first UI proof.

## Capability Inventory

### Already Available

```text
UiProgram identity/version
UiProgram typed graph families
UiEventPacket
RouteId / RouteSchemaVersion / RouteCapability
UiSchemaValue / UiSchemaRef / schema validation
ControlPackage registry and package descriptors
UiProgram lowering from authored control nodes
UiCompiler and compiler report
UiRuntimeArtifact manifest and runtime tables
UiEvaluator and UiOutput
UiStateModel
HostDataSnapshot and BindingSnapshotSet
HostRouteMap and host route resolution
Editor/Game/WorldSpace/Headless host kinds
Runtime read model over artifacts
Headless proof fixture
Accessibility proof path
Geometry proof path
Reproducibility assertion
Retained runtime semantic interactions
Editor workspace reducer example
Foundation inert command proposals
```

### Missing Or Insufficient

```text
AppModel identity/version/schema
AppModelSnapshot
AppAction identity/version/schema
RouteActionMap
UiEventPacket -> AppAction mapping
AppReducer contract
ReducerTraceReport
AppEffectPlan
EffectReport
AppViewProjection
AppProgramReport
AppReplayTrace
HostCompatibilityMatrix for app actions/effects
retained UiInteraction -> UiEventPacket bridge
authored route validation before UiProgram lowering
headless dynamic app replay fixture
bad route / bad payload / missing capability rejection proof
model revision and deterministic replay evidence
```

## Owner Matrix

| Responsibility | Current Owner | Candidate Owner | Investigation Decision |
| --- | --- | --- | --- |
| UI semantic program | `ui_program` | Keep | Reuse; do not duplicate. |
| UI schema/value validation | `ui_schema` | Keep for UI proof | Decide later whether cross-domain app schemas use foundation schema. |
| UI control package semantics | `ui_controls` | Keep | Controls declare contracts; they must not mutate app state. |
| UI route event packet | `ui_program` | Keep | Correct event unit for UI proof. |
| UI state buckets | `ui_state` | Keep | Not generic app state. |
| Host data binding | `ui_binding` | Keep | Suitable for projection into UI, not reducer ownership. |
| Host route mapping | `ui_hosts` | Keep | Maps routes to host/domain command envelopes; does not execute effects. |
| App model/action/reducer | None final | Design required | Do not assign until design gate. |
| Effect plan | None final | Design required; maybe references `foundation/commands` | Must remain inert and host/domain-executed. |
| Headless proof harness | `ui_testing` | Use for proof | Test/proof owner only, not production semantics owner. |
| Editor workspace mutation | `editor_shell` | Keep | Product/domain-specific reducer remains editor-owned. |
| Game/world/app execution | app/game/world owners | Keep outside UI | UI may emit route/action proposals only. |
| Shared app/plugin framework | Architecture future | Later extraction | Blocked until UI plus non-UI proof. |

## Boundary Rules

Typed app-program work must obey these rules:

```text
UiProgram stays UI-owned.
App model is not UiStateModel.
App actions are not raw callbacks.
Effects are explicit inert plans, not hidden side effects.
Hosts resolve and execute outside pure UI crates.
Domain meanings stay in their domains.
Renderer consumes UI/render data, not product truth.
ECS executes/stores runtime data where appropriate, not static app semantics.
Shared framework extraction is blocked until at least two proving domains exist.
```

## Gap Analysis

### Gap 1: App Model Contract

Need a durable app model snapshot contract:

```text
AppModelId
AppModelVersion
AppModelSchemaRef
AppModelSnapshot
AppModelRevision
```

For the UI proof, a minimal counter model can be represented as:

```text
counter.count: integer
counter.screen: counter | win
```

But the contract must not hardcode counter semantics.

### Gap 2: App Action Contract

Need action identity and payload schema:

```text
AppActionId
AppActionSchemaVersion
AppActionPayload
AppActionCapability
```

Example proof actions:

```text
counter.increment
counter.reset
```

### Gap 3: Route Action Mapping

Need deterministic map:

```text
RouteId + RouteSchemaVersion + capability + payload schema
  -> AppAction
```

Unknown route, wrong schema version, missing capability, and invalid payload must be rejected with diagnostics.

### Gap 4: Reducer Trace

Need a reducer report that records:

```text
before snapshot
action
authorization/validation result
after snapshot
effect plan
diagnostics
source route/source control/source map
```

### Gap 5: Effect Plan

Need explicit effect data, probably one of:

```text
AppEffectPlan
HostCommand envelope
DomainCommand envelope
foundation/commands::CommandProposal
```

Recommendation for design: define app effect plan as a local proof concept first, and only reference `foundation/commands` if it clearly reduces duplication without moving execution into foundation.

### Gap 6: View Projection

Need app model to UI projection:

```text
AppModelSnapshot
  -> host data snapshots
  -> UiProgram formation inputs or UiEvaluationContext
  -> UiOutput
```

For the counter proof:

```text
count < 5 -> counter screen with count label and increment route
count >= 5 -> win screen with reset route
```

### Gap 7: Retained Runtime Event Bridge

Need bridge from retained runtime interactions to `UiEventPacket`:

```text
Activated(widget)
  -> route lookup
  -> payload build
  -> capability attach
  -> UiEventPacket
```

This bridge must fail closed when a widget has no route, route is unavailable, payload cannot be built, or required capability is missing.

### Gap 8: Headless Replay Fixture

Need a headless app replay fixture that runs a sequence:

```text
initial model
render frame 0
increment event 1
render frame 1
increment event 2
render frame 2
increment event 3
render frame 3
increment event 4
render frame 4
increment event 5
render win screen
reset event
render counter screen
```

The fixture must prove reproducibility and report the route/action/reducer/effect chain.

## Use-Case Pressure Matrix

| Use Case | Fit | Required Proof | Boundary |
| --- | --- | --- | --- |
| Headless counter app | Strong first proof | model/action/reducer/effect/replay | No editor/game/runtime side effects. |
| Editor workbench UI | Strong later proof | host route to editor/domain command proposal | Editor mutations stay in `editor_shell`/app. |
| Game HUD | Strong later proof | game host route/action proposal | Gameplay state and commands stay game-owned. |
| World-space UI | Later pressure | world-space host facts and anchor/visibility contracts | World/viewport/game owners handle anchors/effects. |
| Material graph editor | Strong non-UI pressure | material-owned program/action/effect proof | Material meaning stays `material_graph`. |
| Procgen editor | Strong non-UI pressure | procgen-owned program/action/effect proof | Procgen meaning stays `procgen`. |
| UI Designer / app builder | Strong future product | visual authoring over app model/action/projection contracts | Designer UX is product work, not core proof. |
| Remote/headless CI proof | Strong | replay reports and deterministic output facts | No hidden host callbacks. |

## Alternatives Considered

| Option | Benefit | Cost | Decision |
| --- | --- | --- | --- |
| Manual host wiring only | Already possible | Poor ergonomics and weak replay evidence | Keep as compatibility baseline only. |
| Callback-first UI | Ergonomic | Hidden mutation, weak source maps, poor proofability | Reject. |
| UI-only app helper | Fast | Risks trapping generic app semantics in UI | Accept only as proof-local naming if bounded. |
| Cross-domain typed app program immediately | Architecturally clean | Owner/new-crate/shared-extraction risk | Design first; no implementation yet. |
| Full AppRecipe/plugin framework now | Matches long-term idea | Premature and explicitly blocked | Reject now. |
| Headless counter proof first | High proof value, low side effects | Requires design of app contracts | Best first proof after design. |
| Editor integration first | Product-visible | Too much editor noise and mutation policy | Later. |
| Game/world-space first | Important future pressure | Too many adjacent owners | Pressure only now. |

## Ergonomics Target

The eventual authoring surface should support an ergonomic shape like:

```rust
app_program("counter.demo")
    .model(CounterModel { count: 0 })
    .view(counter_view)
    .route("counter.increment", CounterAction::Increment)
    .route("counter.reset", CounterAction::Reset)
    .reducer(counter_reducer)
    .proof(counter_replay_scenario)
```

But this must lower to explicit, inspectable records:

```text
AppModelContract
AppActionContract
RouteActionMap
ViewProjection
ReducerTrace
EffectPlan
ReplayReport
HostCompatibilityReport
```

Callbacks may be introduced later only as syntax sugar over typed records. They must not become the architecture.

## Feature Support Matrix For First Proof

| Feature | Required In First Proof | Notes |
| --- | --- | --- |
| App model snapshot | Yes | Counter model with revision. |
| App action contract | Yes | Increment and reset. |
| Route action map | Yes | UI routes map to actions. |
| Reducer trace | Yes | Before/after model and diagnostics. |
| Effect plan | Yes | May be empty or explicit no-op for counter. |
| Host route compatibility | Yes | Headless host must accept expected routes. |
| Unknown route rejection | Yes | Must fail closed. |
| Missing capability rejection | Yes | Must fail closed. |
| Bad payload/schema rejection | Yes | Must fail closed. |
| View projection | Yes | Counter screen and win screen. |
| Replay report | Yes | Deterministic trace. |
| Source maps | Yes | Must preserve route/control origin where available. |
| Retained runtime integration | Optional first proof | Can be second slice if headless event packets prove route/action first. |
| Editor integration | No | Future proof. |
| Game/world-space integration | No | Future proof. |
| Shared plugin extraction | No | Blocked. |

## Stop Conditions

Stop before implementation if any of these occur:

```text
owner remains unclear
new crate is required but not justified by design gate
AppProgram duplicates UiProgram graph ownership
UiStateModel is reused as generic app model
ui_definition executes callbacks or mutations
ui_controls mutate app/editor/game state
ui_hosts execute host effects directly
foundation/commands starts routing/executing commands
renderer owns product truth
ECS owns static app/domain semantics
SpatialCanvas absorbs app-program responsibilities
AppRecipe/plugin framework is implemented before UI plus non-UI proof
```

## Recommended Next Design Gate

Create a design document:

```text
docs-site/src/content/docs/design/active/typed-app-program-and-ui-proof-design.md
```

The design must decide:

1. whether the first implementation is proof-local under existing UI/test crates or needs a new bounded owner;
2. exact vocabulary for model/action/reducer/effect/replay;
3. relationship to `UiProgram`, `UiEventPacket`, `ui_hosts`, and `ui_testing`;
4. whether action/effect schemas use `ui_schema`, `foundation/schema`, or a bridge;
5. how route validation is made fail-closed;
6. how reducer traces preserve diagnostics and source maps;
7. how the headless counter proof is structured;
8. exact implementation files/crates allowed;
9. exact non-owned crates;
10. exact validation commands;
11. future non-UI proof criteria;
12. explicit shared-extraction blockade.

## Recommended Implementation Slice After Design

If the design passes, the first implementation slice should be:

```text
Headless Counter App Proof
```

Minimum acceptance:

```text
initial count = 0 renders counter screen
increment route emits UiEventPacket
route maps to CounterAction::Increment
reducer increments model revision and count
fifth increment renders win screen
reset route returns to counter screen
unknown route rejected
missing capability rejected
invalid payload rejected
deterministic replay report produced
source/event/action/reducer/effect chain inspectable
no product/editor/game mutation
no shared plugin framework
```

## Final Recommendation

Proceed with typed app-program work only through the workflow gates.

The correct one-line decision is:

```text
Investigate and design Typed App Program as a cross-domain app-model/action/effect/replay architecture, with UI/headless counter as the first proving slice, not as a UI callback helper and not as a premature shared plugin framework.
```

## Validation Status

This document was prepared from source inspection through the GitHub connector. Product code was not changed.

Before merge, run:

```bash
python tools/docs/validate_docs.py
git diff --check
```

No cargo validation is expected for this report because it is docs-only investigation work.
