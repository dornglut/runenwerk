---
title: UI Framework App Integration Direction Review
description: Direction correction for making Runenwerk UI a real app-facing framework through App/ECS-hosted authoring, UI-definition-backed source, UiProgram contracts, UiStory proof, and host-owned mutation.
status: active
owner: ui
layer: design
canonical: true
last_reviewed: 2026-07-05
related_docs:
  - ../../architecture/ui-framework-architecture.md
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ./runenwerk-ui-platform-capability-roadmap.md
  - ./runenwerk-ui-story-driven-golden-workflow-design.md
  - ./ui-runtime-rendering-pipeline-roadmap.md
  - ./ui-program-architecture.md
  - ./ui-program-architecture-owner-map.md
  - ./editor-ui-runtime-v2-and-interaction-formation-design.md
  - ../../adr/accepted/0009-ui-interaction-formation-v2.md
  - ./runenwerk-typed-app-composition-plugin-framework-design.md
  - ./typed-app-program-and-ui-proof-design.md
  - ../../workspace/planning/typed-app-program-ui-proof-001-planning.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
  - ../../workspace/planning/decision-register.md
---

# UI Framework App Integration Direction Review

## Status

Lifecycle state: `active-planning`.

This document is a direction correction and planning authority candidate. It does not authorize implementation, product code, crate creation, renderer backend work, engine scheduler work, editor/game integration, shared plugin framework extraction, or `foundation/meta` work.

If accepted, this document supersedes the current `PT-UI-APP-PROGRAM-001` implementation direction as the next active UI-framework planning focus. The existing Typed App Program investigation/design remains useful pressure evidence, but the current headless `app_program` implementation PR must not be merged as the foundation for Runenwerk's real UI framework.

## Relationship to canonical architecture

This document selects the app-facing integration direction. It is not the whole
UI framework architecture.

The whole architecture is summarized in:
[Runenwerk UI Framework Architecture](../../architecture/ui-framework-architecture.md).

Execution-neutral contracts remain the semantic boundary. The first
implementation is ECS-hosted because ECS/App/Plugin is the first app host proof
surface, not because ECS owns UI semantics.

## Purpose

Decide how Runenwerk should become a real app-facing UI framework.

The current repository has strong UI substrate, proof, component, runtime, and story machinery, but the next implementation direction drifted toward a manual `app_program` proof crate. That proof answers whether a local app-program replay IR can exist. It does not answer how real app/plugin authors should use Runenwerk UI ergonomically while preserving source maps, diagnostics, story proof, host policy, and owner boundaries.

This review selects the long-term direction:

```text
App / Plugin / ECS-hosted app authoring
  + ui_definition-backed UI source
  + FormedInteractionModel / UiProgram semantic contracts
  + ui_runtime / ui_evaluator runtime output
  + UiStory proof and mount eligibility
  + host/app-owned mutation
```

This is not raw ECS UI. It is not an `app_program`-first app framework. It is not an external-template-only first step. It is an ECS-hosted framework entry point that lowers through Runenwerk's existing UI architecture.

## Evidence inspected

This direction review is based on the following current authorities and code-state records:

```text
docs-site/src/content/docs/domain/ui/architecture.md
docs-site/src/content/docs/domain/ui/roadmap.md
docs-site/src/content/docs/design/active/ui-program-architecture.md
docs-site/src/content/docs/design/active/ui-program-architecture-owner-map.md
docs-site/src/content/docs/design/active/runenwerk-ui-platform-capability-roadmap.md
docs-site/src/content/docs/design/active/runenwerk-ui-story-driven-golden-workflow-design.md
docs-site/src/content/docs/design/active/ui-runtime-rendering-pipeline-roadmap.md
docs-site/src/content/docs/adr/accepted/0009-ui-interaction-formation-v2.md
docs-site/src/content/docs/design/active/editor-ui-runtime-v2-and-interaction-formation-design.md
docs-site/src/content/docs/design/active/ui-component-platform-base-control-packages-design.md
docs-site/src/content/docs/design/active/ui-component-platform-generic-interaction-design.md
docs-site/src/content/docs/design/active/ui-component-platform-overlay-popup-layering-design.md
docs-site/src/content/docs/design/active/ui-component-platform-text-editing-design.md
docs-site/src/content/docs/design/active/ui-component-platform-generic-text-design.md
docs-site/src/content/docs/design/active/ui-component-platform-surface2d-design.md
docs-site/src/content/docs/design/active/runenwerk-typed-app-composition-plugin-framework-design.md
docs-site/src/content/docs/design/active/runenwerk-typed-app-composition-plugin-framework-implementation-roadmap.md
docs-site/src/content/docs/reports/investigations/typed-app-program-current-state-investigation.md
docs-site/src/content/docs/design/active/typed-app-program-and-ui-proof-design.md
docs-site/src/content/docs/workspace/planning/typed-app-program-ui-proof-001-planning.md
PR #69 App: implement Typed App Program headless counter proof
PR #65 Docs: open Phase 17 SpatialCanvas intake
```

## Current repository reality

Runenwerk already has these UI-framework substrates:

```text
ui_definition:
  authored templates, nodes, slots, validation, normalization, retained formation.

ui_program:
  semantic UI graph, route/event packet, source maps, graph-family facts.

ui_program_lowering / ui_compiler / ui_artifacts:
  formation, compilation, runtime artifact tables.

ui_runtime / ui_evaluator / ui_runtime_view:
  retained execution, input/focus/layout/frame output, artifact-backed runtime views.

ui_controls:
  package-backed reusable control declarations, descriptor validation, catalog, inspection.

ui_binding / ui_hosts:
  host data, authorization, dirty propagation, host kind, host route mapping.

ui_story:
  story manifest, registry, runner, report, mount eligibility.

engine::App / ECS:
  runtime app composition, resources, plugins, systems, schedules, run mode.
```

The missing piece is not another proof crate. The missing piece is the canonical app-facing path:

```text
How does a real app/plugin author declare UI?
How does app state bind into UI source/runtime data?
How does UI input become typed app action proposals?
How does app-owned mutation update the next UI output?
How does UiStory prove the full loop?
```

## Decision

Adopt this direction:

```text
ECS-hosted UI-definition-backed framework integration.
```

Definition:

```text
App / Plugin / ECS is the app/runtime host surface.
ui_definition owns authored UI source and reusable UI structure.
FormedInteractionModel owns execution-neutral interaction contracts.
UiProgram owns semantic UI program facts and route/event contracts.
ui_runtime / ui_evaluator own runtime output and input/event processing.
UiStory owns proof, diagnostics, inspection, preview, and mount eligibility.
Host/app/editor/game owners execute mutation and policy decisions.
```

The normal app authoring path should eventually feel like this target shape:

```rust
struct CounterPlugin;

impl Plugin for CounterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Counter>()
            .add_ui_action::<CounterAction>()
            .add_ui_screen(CounterScreen::Counter, counter_screen)
            .add_ui_screen(CounterScreen::Win, win_screen)
            .add_ui_screen_router(counter_screen_router)
            .add_systems(Update, reduce_counter);
    }
}
```

This is not current API and is not implementation authorization. It is a target ergonomics example for the direction.

The important lowering rule is:

```text
add_ui_screen(...) / UiBuilder output
  -> AuthoredUiTemplate / UiNodeDefinition / slot records
  -> validation and normalization
  -> FormedInteractionModel
  -> UiProgram
  -> UiRuntimeArtifact / runtime view / UiOutput
  -> UiStoryRunReport
```

The framework must not bypass this by spawning semantic UI directly as ECS entities, hardcoding renderer primitives, or executing app mutation from generic UI controls.

## Rejected directions

### A. Continue `app_program` as the public framework foundation

Rejected as the next foundation.

The current `app_program` direction is useful pressure evidence for replay/report vocabulary, but it starts from manual proof IR:

```text
AppModelSnapshot
RouteActionMap
AppAction
AppReducer
AppEffectPlan
AppReplayTrace
AppProgramReport
```

This risks making real app authors write manual snapshots, route maps, action decoders, reducers, projections, reports, and demo-specific wiring before they can build a UI. That is not the desired framework experience.

Allowed future role for `app_program` concepts:

```text
proof report vocabulary
replay envelopes
route/action diagnostic classification
cross-domain evidence structures after repeated proof domains
```

Rejected role:

```text
normal public UI app framework
replacement for App / Plugin / ECS
replacement for UiStory
manual model-snapshot API every app must use
shared plugin framework extraction
```

### B. Raw ECS UI as source of truth

Rejected.

Raw ECS UI would make the world contain semantic UI nodes and would be attractive for runtime convenience, but it conflicts with the current Runenwerk architecture:

```text
weak source maps
weak authored template identity
weak migration/inspection
weak story proof
weak designer/workbench path
risk of ECS-owned UI semantics
risk of app-only mutation shortcuts
```

ECS may host app state, app actions, and app systems. ECS must not own static UI/domain semantics.

### C. External-template-first only

Rejected as the immediate next implementation direction.

External authored templates and story manifests are important long-term, but making them the only first step would require full binding language, template activation, host-data projection, source management, and UI designer workflows before a simple app proof is pleasant.

The next proof may use code-authored UI if that code captures/lower into `ui_definition` source records and story reports. External authored templates remain a first-class later path, not the only first path.

### D. SpatialCanvas as the answer to app-framework integration

Rejected.

SpatialCanvas is a reusable positioned-item surface on top of Surface2D. It is not app composition, not provider registration, not product mutation, not plugin framework, not UI Designer persistence, and not the generic app/UI integration layer.

Phase 17 should remain planning-only until this integration decision is accepted.

## Owner model

| Responsibility | Owner | Rule |
|---|---|---|
| App setup, resources, systems, schedules | `engine::App` / ECS | Runtime host surface; not UI semantic owner. |
| App/domain state mutation | app/editor/game/domain owners | Mutate only through owner systems/commands/reducers. |
| Code-authored UI builder output | candidate UI app-integration layer | Must lower to `ui_definition`, not raw ECS UI truth. |
| Authored UI source | `ui_definition` | Own templates, nodes, slots, bindings, validation, normalization, source maps. |
| Interaction contracts | `FormedInteractionModel` / `ui_definition` | Execution-neutral contracts before retained/runtime execution. |
| Semantic UI program | `ui_program` | Route/event packets, graph facts, source maps, schema/capability contracts. |
| Runtime artifact and output | `ui_compiler`, `ui_artifacts`, `ui_runtime`, `ui_evaluator`, `ui_runtime_view` | Derived runtime products only. |
| Host data / route compatibility | `ui_binding`, `ui_hosts` | Host profile, host data, authorization, route mapping, diagnostics. |
| Story/proof/mount eligibility | `ui_story` / `ui_testing` | Full pipeline report, pass/fail, diagnostics, preview, mount eligibility. |
| Renderer execution | renderer/backend owners | Consume renderer-neutral `UiFrame`/primitive products only. |
| Spatial item surface facts | future SpatialCanvas owners | Generic positioned-item facts only, not app integration. |

## Design answers

### 1. What is the public app-facing UI authoring surface?

Decision:

```text
App / Plugin / ECS-hosted registration plus UI-framework screen/component builders.
```

Target form:

```rust
app.init_resource::<Counter>()
    .add_ui_action::<CounterAction>()
    .add_ui_screen(CounterScreen::Counter, counter_screen)
    .add_ui_screen(CounterScreen::Win, win_screen)
    .add_ui_screen_router(counter_screen_router)
    .add_systems(Update, reduce_counter);
```

This API does not exist today. The next design/implementation must decide exact module/crate ownership and exact names before adding it.

Rules:

```text
The app-facing surface registers contracts.
It does not make ECS the source of UI semantics.
It does not execute generic UI callbacks.
It does not replace UiStory.
```

### 2. What does a screen/component function produce?

Decision:

```text
A screen/component function produces UI-definition-compatible source records.
```

Target form:

```rust
fn counter_screen(ui: &mut UiBuilder, counter: Res<Counter>) {
    ui.column()
        .gap("space.md")
        .padding("space.lg")
        .text(format!("Clicked {} / 5", counter.count))
        .button("Click me")
        .action(CounterAction::Increment);
}
```

The builder output must be capturable as:

```text
AuthoredUiTemplate
UiNodeDefinition
UiRouteSlotRef
UiValueSlotRef
UiCollectionSlotRef
source-map/provenance rows
```

It must not be only immediate retained widgets, only raw ECS entities, or direct `UiProgram` graph-row construction.

### 3. How do actions work?

Decision:

```text
App actions are typed in app code and lower to stable UI route/action contracts.
```

Target form:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum CounterAction {
    Increment,
    Reset,
}

ui.button("Click me").action(CounterAction::Increment);
```

Lowered record chain:

```text
CounterAction::Increment
  -> stable action identity
  -> UiRouteSlotRef / RouteId
  -> RouteSchemaVersion
  -> RouteCapability
  -> UiEventPacket
  -> host/app action proposal
  -> typed app action queue or command input
```

Rejected default model:

```rust
ui.button("Click me", || counter.count += 1);
```

Callbacks may exist only as local ergonomic sugar if they lower into explicit route/action records and do not hide mutation, IO, command execution, or host policy.

### 4. How does app state bind to UI?

Decision:

```text
Use ergonomic ECS reads at authoring/runtime boundary, but record lowered host-data/binding/proof facts.
```

The code-authored path may read resources:

```rust
fn counter_screen(ui: &mut UiBuilder, counter: Res<Counter>) {
    ui.text(format!("Clicked {} / 5", counter.count));
}
```

But the framework must record what this means for proof and inspection:

```text
Counter.count
  -> host/binding value or snapshot row
  -> UiValueBinding / runtime host data evidence
  -> text/control property value
  -> source/proof row
```

The user should not manually write projection maps for the simple case. The framework still needs explicit snapshot/binding policy for reproducibility, diagnostics, stale host data, authorization, and future external template support.

### 5. What is the runtime loop?

Decision:

```text
The framework loop is app state -> UI source/runtime output -> UI input/event -> app action -> app mutation -> next UI output -> proof report.
```

Canonical loop:

```text
1. App/ECS resources and systems exist.
2. Screen router selects active screen/surface.
3. Screen/component builder emits UI-definition-compatible source records.
4. ui_definition validates and normalizes.
5. FormedInteractionModel records interaction contracts.
6. UiProgram records semantic UI facts, route slots, schemas, capabilities, source maps.
7. Compiler/evaluator/runtime produces artifact, runtime view, UiOutput/UiFrame.
8. Runtime processes input and emits UiEventPacket / route proposal.
9. Bridge maps route proposal to typed app action.
10. App/ECS owner system mutates app state.
11. UI is rebuilt/evaluated from the new state.
12. UiStoryRunReport records every stage.
```

### 6. What is the first proof?

Decision:

```text
ECS-backed Counter UI Story Proof.
```

This replaces the current manual headless `app_program` counter proof as the preferred next implementation target.

Required proof:

```text
Counter resource starts at count = 0.
Counter screen is active.
UI output contains count text and increment action.
Pointer or keyboard activation travels through UI runtime/event path.
UiEventPacket or route proposal is emitted.
Route/action mapping resolves to CounterAction::Increment.
App/ECS system mutates Counter count 0 -> 1.
Next UI output shows count = 1.
After count reaches 5, active screen becomes Win.
Reset action returns count to 0 and counter screen becomes active again.
UiStoryRunReport records source, formation, program, compiler, runtime, route, binding, app-action, mutation, next-output, diagnostics, and pass/fail evidence.
```

Negative scenarios:

```text
unknown route rejected
wrong route schema rejected
missing capability rejected
disabled control emits no activation
invalid action payload rejected
missing host data emits diagnostics
rejected action does not mutate app state
runtime input outside target emits no app action
callback/direct mutation bypass is absent
```

## Relationship to PR #69

PR #69 is superseded as the next foundation.

Required action:

```text
Close or keep PR #69 only as an implementation spike archive.
Do not merge it into main as the foundation for the UI framework.
Do not continue polishing it unless the accepted goal explicitly returns to app_program proof IR.
Reuse its useful route/action/replay/report pressure as proof criteria for the new integration path.
```

Reason:

```text
PR #69 follows the older app_program-first planning contract, but the better framework decision is one level higher: app/plugin authors should use an app-facing UI integration layer that lowers through ui_definition, UiProgram, UiStory, and host/app mutation owners.
```

## Relationship to Typed App Program design

`typed-app-program-and-ui-proof-design.md` remains useful pressure evidence. It is not the selected next implementation foundation.

Preserve from that design:

```text
fail-closed route/action resolution
typed actions
safe bounded payload reports
replay/report evidence
rejected action no-mutation rule
host capability checks
separation between UI events and app mutation
```

Supersede for the next active direction:

```text
dedicated app_program crate as first implementation target
manual AppModelSnapshot authoring as the normal app path
manual RouteActionMap / AppReducer / AppViewProjection public framework feel
headless counter replay that does not prove normal framework authoring
```

## Relationship to SpatialCanvas

SpatialCanvas remains valid future Component Platform work. It is not the app/UI integration answer.

Rules:

```text
Phase 17 remains planning-only until this integration direction is accepted.
SpatialCanvas must consume Surface2D and add generic positioned-item facts only.
SpatialCanvas must not become app composition, provider registration, plugin framework, UI Designer persistence, product selection mutation, graph semantics, timeline semantics, camera/scene resources, or renderer backend ownership.
```

## Relationship to ECS

ECS is accepted as runtime fabric and app authoring host.

ECS may own:

```text
resources
systems
schedules
plugin registration
runtime app state
app-owned action queues/events
mutation by app/domain systems
```

ECS must not own:

```text
authored UI identity
static UI semantics
control package meaning
UiProgram facts
story proof truth
renderer product truth
editor/game command semantics
```

This reconciles runtime ergonomics with Runenwerk's source-backed UI architecture.

## Proposed next planning ID

Use this planning ID for the direction review:

```text
PT-UI-FRAMEWORK-APP-INTEGRATION-001
```

Title:

```text
UI Framework App Integration Direction Review
```

Lifecycle:

```text
active-planning
```

Activation condition for implementation:

```text
The direction review is accepted, exact owner files/crates are named, public API names are decided, validation envelope is recorded, and the first ECS-backed Counter UI Story Proof implementation plan is created.
```

## Future implementation outline

The first implementation after this design must be planning-authorized separately.

Candidate name:

```text
ECS-backed Counter UI Story Proof
```

Candidate owner shape:

```text
ui_app integration module/crate: screen/action registration and bridge contracts, if accepted
ui_definition: source records emitted by code-authored screens
ui_program / ui_program_lowering: route/action facts and source maps
ui_story / ui_testing: proof runner and report envelope
engine::App / ECS: app state and mutation system proof host
```

Exact crate/module names are intentionally not decided by this document. They are part of the next complete design gate.

## Stop conditions

Stop and redesign if a follow-up tries to:

```text
merge PR #69 as the framework foundation without resolving this direction
make raw ECS entities the durable UI source of truth
add hidden callbacks as primary interaction behavior
bypass ui_definition / UiProgram / UiStory for visible framework proof
render from authored source directly
mutate editor/game/app state from generic UI controls
make SpatialCanvas own app integration
create AppRecipe / PluginSuite / shared plugin framework
create foundation/meta
add renderer backend handles/resources to domain/ui
add engine scheduler/threading/networking/multiplayer implementation
add product UI Designer persistence
claim production framework readiness without story/proof evidence
```

## Validation expectation for this docs change

Docs-only validation expected:

```bash
python tools/docs/validate_docs.py
git diff --check
```

Implementation validation is intentionally deferred until the next implementation plan names exact files, crates, tests, and evidence.

## Final decision

The best long-term direction is:

```text
ECS/App/Plugin-hosted app authoring,
UI-definition-backed UI source,
UiProgram-backed semantic contracts,
UiStory-backed proof and mount eligibility,
host/app-owned mutation.
```

The next work should not continue the manual `app_program` implementation. It should formalize and then implement the first real framework proof: an ECS-backed Counter UI Story Proof through the existing UI pipeline.
