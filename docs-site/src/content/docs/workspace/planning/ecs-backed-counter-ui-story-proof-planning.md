---
title: ECS-backed Counter UI Story Proof Planning
description: Planning/design intake for the first real UI framework proof after the UI Framework App Integration Direction Review.
status: active
owner: ui
layer: workspace
canonical: false
last_reviewed: 2026-07-05
related_docs:
  - ./active-work.md
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../complete-merge-readiness-gate.md
  - ../../design/active/ui-framework-app-integration-direction-review.md
  - ../../design/active/runenwerk-ui-story-driven-golden-workflow-design.md
  - ../../design/active/ui-runtime-rendering-pipeline-roadmap.md
  - ../../design/active/ui-program-architecture.md
  - ../../design/active/editor-ui-runtime-v2-and-interaction-formation-design.md
  - ../../adr/accepted/0009-ui-interaction-formation-v2.md
  - ../../domain/ui/roadmap.md
  - ../../reports/investigations/typed-app-program-ui-proof-001-superseded-planning-archive.md
---

# ECS-backed Counter UI Story Proof Planning

## Status

Lifecycle state: `active-planning`.

Implementation authorization: not granted.

This document opens the first proof-planning slice after `UI Framework App Integration Direction Review`. It selects the long-term owner strategy, but it must remain planning/design intake until exact implementation owners, allowed files, validation commands, evidence expectations, and stop conditions are accepted.

## Purpose

Define the first real Runenwerk UI-framework proof:

```text
ECS-backed Counter UI Story Proof
```

The proof must demonstrate the intended framework loop:

```text
App/ECS host state
  -> UI screen/component source
  -> ui_definition validation/normalization
  -> FormedInteractionModel
  -> UiProgram route/event facts
  -> runtime artifact/output
  -> UI input/event proposal
  -> typed app action
  -> app/ECS-owned mutation
  -> next UI output
  -> UiStoryRunReport
```

The proof exists to validate framework usage, not to build a product app, game UI, editor feature, SpatialCanvas component, shared plugin framework, or general app-program runtime.

## Authority

Primary authority:

```text
docs-site/src/content/docs/design/active/ui-framework-app-integration-direction-review.md
```

Supporting authority:

```text
docs-site/src/content/docs/domain/ui/roadmap.md
docs-site/src/content/docs/design/active/runenwerk-ui-story-driven-golden-workflow-design.md
docs-site/src/content/docs/design/active/ui-runtime-rendering-pipeline-roadmap.md
docs-site/src/content/docs/design/active/ui-program-architecture.md
docs-site/src/content/docs/adr/accepted/0009-ui-interaction-formation-v2.md
```

Historical pressure evidence:

```text
docs-site/src/content/docs/reports/investigations/typed-app-program-ui-proof-001-superseded-planning-archive.md
```

## Current lifecycle relationship

```text
PT-UI-FRAMEWORK-APP-INTEGRATION-001 — completed direction decision through PR #70
PT-UI-FRAMEWORK-APP-INTEGRATION-002 — this planning intake
```

This planning intake may become the implementation contract only after it names exact owner files and passes the complete design gate.

## Core direction to preserve

The public framework direction is:

```text
ECS/App/Plugin-hosted app authoring
+ ui_definition-backed UI source
+ FormedInteractionModel / UiProgram contracts
+ ui_runtime / ui_evaluator runtime output
+ UiStory proof and mount eligibility
+ host/app-owned mutation
```

This proof must not regress to any rejected direction:

```text
raw ECS-owned UI source of truth
manual app_program public framework
external-template-only first step
SpatialCanvas as app integration
callback-first generic UI mutation
renderer-owned UI semantics
```

## Long-term owner decision

Selected long-term owner strategy:

```text
Option C — a small UI-owned app-integration layer.
```

Working owner name:

```text
domain/ui/ui_app_integration
```

The exact crate/module name is not yet implementation authority. The important decision is ownership shape:

```text
A UI-owned integration layer owns the bridge between App/ECS-hosted app authoring and the UI source/program/story pipeline.
```

It should own only:

```text
screen/action registration contracts
screen/router proof wiring
typed action -> route/action lowering policy
app-state snapshot/binding extraction policy
route proposal -> typed app-action dispatch policy
proof facts connecting UI event -> app mutation -> next UI output
```

It must not own:

```text
control semantics
layout semantics
theme semantics
UiProgram graph truth
runtime input semantics
ECS resources/systems themselves
app mutation
engine scheduling
editor/game commands
SpatialCanvas semantics
AppRecipe / PluginSuite / shared plugin framework
```

## Implementation sequence decision

Use this staged sequence:

```text
1. C-internal proof slice:
   Implement the smallest UI-owned app-integration bridge needed for the Counter proof.
   Keep APIs internal or proof-local until owner and dependency direction are validated.

2. Story/proof harness:
   Use ui_story / ui_testing as the proof executor, not as the long-term framework owner.

3. B-style ergonomics later:
   Add AppUiExt-style public registration only after the proof validates the owner boundary.

4. No pure A as architecture:
   A fixture/story-only proof is allowed only as harness support, not as the framework direction.
```

This avoids the two failure modes:

```text
fixture-only proof that never becomes usable framework API
premature public App extension API before dependency and owner boundaries are proven
```

## Why pure A is not enough

A fixture/story-only proof is useful as a safety harness, but it does not prove real App/Plugin/ECS authoring ergonomics. It risks producing another proof island that leaves the app-facing framework path unresolved.

Therefore:

```text
ui_story / ui_testing may execute the proof.
They must not become the long-term app-integration owner.
```

## Why pure B is not first

An AppUiExt-style API is the desired eventual ergonomics, but it must not be introduced before the integration owner and dependency direction are proven.

Therefore:

```text
AppUiExt-style names are reserved target ergonomics.
They are not accepted implementation names in the first slice.
```

## Why C is selected

A small UI-owned app-integration layer gives the cleanest boundary:

```text
engine::App / ECS remains the host/runtime surface
ui_definition remains authored UI source owner
ui_program remains semantic UI contract owner
ui_runtime / ui_evaluator remain runtime output owners
ui_story / ui_testing remain proof owners
app/domain owners remain mutation owners
```

It avoids pushing app integration into `ui_definition`, `ui_program`, `ui_runtime`, or engine core. It also avoids reviving `app_program` as the public framework path.

## Proof scenario

The first proof uses a counter app:

```text
Counter { count: 0 }
Counter screen visible while count < 5
Increment action increases count by 1
Win screen visible while count >= 5
Reset action sets count to 0
```

Required positive path:

```text
1. App/ECS host initializes Counter resource with count = 0.
2. Counter screen/router selects Counter screen.
3. UI source for the Counter screen is emitted through a UI-definition-compatible builder or fixture path.
4. UI source validates and lowers through the accepted UI pipeline.
5. Runtime output contains count text and an increment activation affordance.
6. Pointer or keyboard activation is processed by UI runtime/event machinery.
7. UI emits a route/event proposal with schema/capability/source evidence.
8. Bridge resolves route/event to CounterAction::Increment.
9. App/ECS-owned system mutates Counter from 0 to 1.
10. Next UI output shows count = 1.
11. After count reaches 5, the active output switches to Win screen.
12. Reset activation resolves to CounterAction::Reset.
13. App/ECS-owned system mutates Counter to 0.
14. Counter screen becomes active again.
15. UiStoryRunReport records every stage and pass/fail evidence.
```

Required negative path:

```text
unknown route is rejected
wrong route schema is rejected
missing capability is rejected
disabled control emits no activation
invalid action payload is rejected
missing host/binding data reports diagnostics
rejected action does not mutate Counter
runtime input outside target emits no app action
callback/direct-mutation bypass is absent
UiStory report fails if any mandatory stage is missing
```

## Required report shape

The proof report must make these facts inspectable:

```text
source identity
screen identity
control/action source-map reference
formation diagnostics
UiProgram route/event diagnostics
compiler/runtime artifact diagnostics
runtime view/output facts
input event facts
route proposal facts
host/app action resolution facts
Counter before/after snapshot facts
mutation owner facts
next-output facts
pass/fail summary
```

The report must preserve the pressure from the superseded app-program proof:

```text
stable route/action identity
safe bounded payload summaries
distinct diagnostic namespaces
fail-closed resolution
no visible label as durable action identity
no mutation after rejected action
```

## Candidate public ergonomics target

Target feel only; not current API and not implementation authorization:

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

Target screen feel only:

```rust
fn counter_screen(ui: &mut UiBuilder, counter: Res<Counter>) {
    ui.column()
        .text(format!("Clicked {} / 5", counter.count))
        .button("Click me")
        .action(CounterAction::Increment);
}

fn win_screen(ui: &mut UiBuilder) {
    ui.column()
        .text("You win!")
        .button("Reset")
        .action(CounterAction::Reset);
}
```

These snippets define desired ergonomics, not names accepted for implementation. The implementation plan must choose exact crate/module/API names before code starts.

## Owner model to prove

| Responsibility | Owner in this proof | Rule |
|---|---|---|
| Counter resource and mutation | App/ECS proof host | Generic UI must not mutate it directly. |
| Counter action type | App proof fixture | Must lower to stable UI route/action evidence. |
| App-integration bridge | UI-owned integration layer | Own route/action/screen proof wiring only. |
| Screen declaration | UI app-integration candidate path | Must produce/capture `ui_definition`-compatible source. |
| UI source validation | `ui_definition` / lowering path | Must not be bypassed. |
| Interaction contract | `FormedInteractionModel` / `ui_program` path | Must carry route/schema/capability/source evidence. |
| Runtime output/input | `ui_runtime` / `ui_evaluator` / existing test host path | Must produce observable output and event facts. |
| Story proof | `ui_story` / `ui_testing` | Must own the proof envelope. |
| Mutation commit | App/ECS owner system | Must be absent on rejected actions. |

## Complete design questions before implementation

Implementation may not start until the next revision answers:

```text
1. What exact crate/module owns UI app-integration contracts?
2. Is `domain/ui/ui_app_integration` a new crate, a module in an existing crate, or a proof-local module first?
3. What exact names are accepted for internal bridge types?
4. Which public AppUiExt names remain reserved but not implemented in the first slice?
5. What does `UiBuilder` output internally: authored template records, normalized records, or a proof fixture format?
6. How are code-authored source maps/provenance represented?
7. How are typed actions converted to route IDs and capabilities without manual string maps?
8. How does ECS/app state become host/binding data for `UiProgram` evaluation?
9. Which existing `UiStory`/`ui_testing` runner extension is enough for this proof?
10. Which exact files/crates are allowed to change?
11. Which exact tests prove the positive and negative paths?
12. How does the implementation prove that no public API was prematurely frozen?
```

## Selected implementation owner strategy

Selected strategy:

```text
C-internal first, then B-public later.
```

### First implementation slice

The first implementation slice should be:

```text
A small UI-owned app-integration proof bridge plus a Counter story proof.
```

It should expose no stable public App extension API unless a follow-up design explicitly accepts names.

It should be allowed to use `ui_story` / `ui_testing` to execute the proof.

It should not be a pure fixture-only proof. The bridge must represent the future app-integration owner boundary, even if its API is internal/proof-local.

### Second implementation slice

Only after the first proof passes and dependency direction is reviewed:

```text
Add AppUiExt-style ergonomic registration.
```

Candidate future methods remain examples only:

```text
add_ui_action
add_ui_screen
add_ui_screen_router
```

## Allowed work in this planning PR

Allowed now:

```text
docs-site/src/content/docs/workspace/planning/ecs-backed-counter-ui-story-proof-planning.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/decision-register.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
```

Forbidden now:

```text
product code
crate creation
engine runtime changes
ECS system API changes
ui_definition implementation
ui_runtime implementation
ui_story implementation
ui_testing implementation
SpatialCanvas implementation
app_program implementation
shared plugin framework
foundation/meta
```

## Candidate future implementation files

These are candidates only. The implementation contract must replace this list with exact accepted files.

Potential existing owners:

```text
domain/ui/ui_story/src/*
domain/ui/ui_testing/src/*
domain/ui/ui_definition/src/*
domain/ui/ui_program/src/*
domain/ui/ui_hosts/src/*
domain/ui/ui_binding/src/*
engine/src/app/* or a small integration adapter only if dependency rules permit it
```

Potential new owner, selected as preferred if dependency review accepts it:

```text
domain/ui/ui_app_integration/*
```

Potential tests:

```text
domain/ui/ui_testing/tests/ecs_backed_counter_ui_story.rs
domain/ui/ui_story/tests/ecs_backed_counter_ui_story.rs
domain/ui/ui_app_integration/tests/counter_ui_story_proof.rs
```

## Non-owned responsibilities

This proof must not own:

```text
editor shell commands
product/editor/game mutation beyond the local Counter fixture
renderer backend resources
windowing or OS event loop
networking or multiplayer
async effect lifecycle
asset IO or file IO
hot reload
localization system
UI Designer persistence
SpatialCanvas item semantics
NodeCanvas / PortGraphCanvas semantics
AppRecipe / PluginSuite / shared plugin framework
foundation/meta
```

## Stop conditions

Stop and return to design if implementation requires:

```text
raw ECS entities as durable UI source
callback-first mutation from generic UI controls
bypassing ui_definition validation/normalization
bypassing UiProgram route/event facts
bypassing UiStory reports
new public App extension names without accepted API review
new crate without accepted owner/dependency decision
engine core depending on domain/ui in the wrong direction
ui_definition depending on engine/ECS
ui_program depending on engine/ECS
ui_runtime executing app mutation directly
ui_hosts executing effects directly
spatial canvas or component-platform scope changes
reopening PR #69 as implementation foundation
```

## Validation expectations for this planning PR

Docs-only validation expected before merge:

```bash
python tools/docs/validate_docs.py
git diff --check
```

Implementation validation must be decided later. Expected categories include:

```text
focused story/proof test
focused route/action negative tests
focused no-bypass test
cargo test --workspace
python tools/docs/validate_docs.py
git diff --check
```

## Next action

Revise this planning intake into an implementation contract for:

```text
C-internal first: small UI-owned app-integration proof bridge plus Counter UiStory proof.
```

That implementation contract must name exact files, internal type names, dependency rules, tests, validation commands, and closeout evidence. It must explicitly defer public AppUiExt-style API until after the proof.
