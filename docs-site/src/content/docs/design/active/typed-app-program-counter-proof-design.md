---
title: Typed App Program Counter Proof Design
description: Counter app-program proof design for model/action/reducer/effect/projection/replay with UI source projection and a derived win screen at count ten.
status: active
owner: ui
layer: design
canonical: false
last_reviewed: 2026-07-08
related:
  - ./typed-app-program-and-ui-proof-design.md
  - ./domain-authoring-source-and-program-pattern.md
  - ./ui-source-projection-and-program-lowering-design.md
  - ./ui-framework-runtime-requirements-design.md
  - ./ui-component-composition-slots-and-authoring-design.md
  - ./ui-data-binding-forms-and-effects-design.md
  - ./ui-reactive-runtime-and-invalidation-design.md
  - ./ui-live-editing-and-preview-design.md
  - ./ui-game-and-worldspace-host-requirements-design.md
  - ./ui-accessibility-internationalization-and-text-conformance-design.md
  - ./ui-testing-conformance-and-proof-matrix-design.md
  - ./ui-program-architecture.md
  - ./runenwerk-typed-app-composition-plugin-framework-design.md
---

# Typed App Program Counter Proof Design

## Status

Active design for the Counter proof target. This document does not authorize
product implementation by itself. Implementation still requires an active-work
entry or equivalent planning contract naming files, validation commands, stop
conditions, and evidence expectations.

`owner: ui` is temporary for the first UI proving slice. The app-program pattern
must not become permanently UI-owned. Counter uses UI because UI is the first
available proof domain.

This document narrows the Counter target so future implementation slices do not
turn the product app into UI runtime plumbing.

## Decision

Counter is a typed app program.

It is not:

```text
an ECS resource as public source truth
a UiFeature as a universal abstraction
a callback-first UI example
a renderer primitive assembly path
a manual UiEventPacket bridge
a generic graph-runtime proof
```

Counter owns app behavior:

```text
CounterModel
CounterAction
Counter reducer
Counter action availability
Counter effect plan
Counter proof scenarios
```

UI owns UI meaning and lowering:

```text
CounterModel -> UiSource projection
UiSource -> AuthoredUiTemplate
AuthoredUiTemplate -> NormalizedUiTemplate
NormalizedUiTemplate -> FormedInteractionModel
FormedInteractionModel -> UiProgram
UiProgram -> UiRuntimeArtifact
UiRuntimeArtifact -> UiOutput / UiFrame / UiEventPacket
```

The host owns concrete effects and runtime integration.

## Target Author-Facing Shape

The following code is illustrative north-star spelling. It locks architecture
shape, not exact API spelling.

```rust
use anyhow::Result;
use engine::plugins::ui::prelude::*;
use engine::prelude::*;

pub const WINDOW_TITLE: &str = "Runenwerk UI Counter Runtime";
pub const WIN_COUNT: i64 = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterModel {
    count: i64,
}

impl Default for CounterModel {
    fn default() -> Self {
        Self { count: 0 }
    }
}

impl CounterModel {
    fn count(&self) -> i64 {
        self.count
    }

    fn screen(&self) -> CounterScreen {
        if self.count >= WIN_COUNT {
            CounterScreen::Win
        } else {
            CounterScreen::Counting
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CounterScreen {
    Counting,
    Win,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CounterAction {
    Increment,
    Decrement,
    Reset,
}

pub fn counter_program() -> AppProgram<CounterModel, CounterAction> {
    app::program("runenwerk.counter")
        .version("0.1.0")
        .title(WINDOW_TITLE)
        .model("counter.model", "1.0.0", CounterModel::default)
        .actions(counter_actions())
        .availability(counter_action_availability)
        .project_ui(counter_ui)
        .reduce(counter_reduce)
        .proofs(counter_proofs())
}

fn counter_actions() -> AppActionSet<CounterAction> {
    app::actions("counter")
        .schema_version("1.0.0")
        .write_capability("counter.write")
        .action(CounterAction::Increment, "increment", "Increment")
        .action(CounterAction::Decrement, "decrement", "Decrement")
        .action(CounterAction::Reset, "reset", "Reset")
}

fn counter_action_availability(
    model: &CounterModel,
    action: CounterAction,
) -> AppActionAvailability {
    match (model.screen(), action) {
        (CounterScreen::Counting, CounterAction::Increment) => AppActionAvailability::enabled(),
        (CounterScreen::Counting, CounterAction::Decrement) if model.count() > 0 => {
            AppActionAvailability::enabled()
        }
        (CounterScreen::Counting, CounterAction::Reset) if model.count() > 0 => {
            AppActionAvailability::enabled()
        }
        (CounterScreen::Win, CounterAction::Reset) => AppActionAvailability::enabled(),
        _ => AppActionAvailability::disabled_by_state(),
    }
}

fn counter_ui(
    model: &CounterModel,
    actions: &UiActionProjection<CounterAction>,
) -> UiSource {
    match model.screen() {
        CounterScreen::Counting => counter_screen(model, actions),
        CounterScreen::Win => win_screen(model, actions),
    }
}

fn counter_screen(
    model: &CounterModel,
    actions: &UiActionProjection<CounterAction>,
) -> UiSource {
    ui::screen("counter.screen")
        .title(WINDOW_TITLE)
        .requires_package("runenwerk.ui.base_controls")
        .root(
            ui::column("screen")
                .gap(UiSpace::Md)
                .padding(UiSpace::Lg)
                .children([
                    ui::text("title", UiText::key("counter.title").fallback(WINDOW_TITLE)),
                    ui::text(
                        "count",
                        UiText::key("counter.count")
                            .arg("count", model.count())
                            .arg("target", WIN_COUNT)
                            .fallback(format!("Count: {} / {}", model.count(), WIN_COUNT)),
                    ),
                    ui::row("actions")
                        .gap(UiSpace::Sm)
                        .children([
                            actions.button(CounterAction::Increment),
                            actions.button(CounterAction::Decrement),
                            actions.button(CounterAction::Reset),
                        ]),
                ]),
        )
}

fn win_screen(
    model: &CounterModel,
    actions: &UiActionProjection<CounterAction>,
) -> UiSource {
    ui::screen("counter.win")
        .title("Counter Complete")
        .requires_package("runenwerk.ui.base_controls")
        .root(
            ui::column("screen")
                .gap(UiSpace::Md)
                .padding(UiSpace::Lg)
                .children([
                    ui::text("title", UiText::key("counter.win.title").fallback("You win!")),
                    ui::text(
                        "count",
                        UiText::key("counter.win.count")
                            .arg("count", model.count())
                            .fallback(format!("Final count: {}", model.count())),
                    ),
                    ui::row("actions")
                        .gap(UiSpace::Sm)
                        .children([
                            actions.button(CounterAction::Reset),
                        ]),
                ]),
        )
}

fn counter_reduce(
    mut draft: CounterModel,
    action: CounterAction,
) -> AppReducerOutcome<CounterModel> {
    match action {
        CounterAction::Increment if draft.count < WIN_COUNT => {
            draft.count = draft.count.saturating_add(1).min(WIN_COUNT);
            AppReducerOutcome::accepted(draft).with_no_effect()
        }
        CounterAction::Decrement if draft.count > 0 && draft.count < WIN_COUNT => {
            draft.count = draft.count.saturating_sub(1);
            AppReducerOutcome::accepted(draft).with_no_effect()
        }
        CounterAction::Reset if draft.count > 0 => {
            draft.count = 0;
            AppReducerOutcome::accepted(draft).with_no_effect()
        }
        _ => AppReducerOutcome::rejected(draft)
            .with_diagnostic("counter.action.disabled_by_state")
            .with_no_effect(),
    }
}

fn counter_proofs() -> AppProofSet {
    app::proofs("counter.proofs")
        .scenario("initial_counting_screen")
        .scenario("increment_once")
        .scenario("increment_to_nine")
        .scenario("increment_to_ten_switches_to_win")
        .scenario("increment_after_win_rejected")
        .scenario("reset_from_win_returns_to_counter")
        .scenario("decrement_at_zero_rejected")
        .scenario("reject_unknown_route")
        .scenario("reject_missing_capability")
        .scenario("reject_invalid_payload")
}

pub fn build_counter_app() -> App {
    AppRecipe::new("runenwerk.counter")
        .version("0.1.0")
        .title(WINDOW_TITLE)
        .profile(ProductProfile::Desktop)
        .host(HostProfile::Desktop)
        .plugin(ui::runtime())
        .program(counter_program())
        .assemble()
        .into_app()
}

pub fn run() -> Result<()> {
    build_counter_app().run()
}
```

## Model Rules

`CounterModel` stores only count:

```text
count: i64
```

The active screen is derived:

```text
Counting if count < 10
Win if count >= 10
```

Do not store `screen` as a separate field. Storing both count and screen would
allow inconsistent states such as `count = 3` with `screen = Win`.

## Action Rules

Actions:

```text
counter.increment
counter.decrement
counter.reset
```

Required capability:

```text
counter.write
```

Rules:

```text
Increment:
  accepted only while count < 10
  count becomes min(count + 1, 10)

Decrement:
  accepted only while 0 < count < 10

Reset:
  accepted while count > 0
  count becomes 0

All other action/state pairs:
  rejected with counter.action.disabled_by_state
```

## Availability Versus Reducer Rejection

Action availability is predictive UI/host metadata. It allows controls to render
disabled state, route catalogs to explain available actions, and preview/proof
systems to show expected affordances.

Reducer rejection is authoritative safety. Even if a stale UI event, replay step,
remote host, or bad fixture submits a disabled action, the reducer must reject it
fail-closed and report diagnostics.

## RouteActionMap Requirement

Implementation must create an explicit route-action map from action declarations.

Input facts:

```text
UiEventPacket
HostRouteMap or host compatibility evidence
Counter action declarations
route schema version
payload schema references
required capabilities
```

Resolution variants:

```text
Accepted(AppAction)
RejectedUnknownRoute
RejectedSchemaVersion
RejectedPayloadShape
RejectedMissingCapability
RejectedDisabledByHost
RejectedDisabledByState
RejectedDiagnostics
```

All rejections must be reportable and fail closed.

## Projection Rules

The app-program projection produces `UiSource`, not `UiSourceAst`.

Counting screen:

```text
screen id: counter.screen
visible text: Count: N / 10
actions: increment, decrement, reset
```

Win screen:

```text
screen id: counter.win
visible text: You win!
visible text: Final count: 10
actions: reset only
```

The switch to the win screen is a pure projection consequence of `count >= 10`.
It is not a reducer side effect and not UI-owned state mutation.

## Localization Requirements

Text source should be represented as:

```text
text key
format arguments
fallback text
source-map provenance
```

Proof sketches may display fallback text. Mature implementation must preserve
keys and report missing localization metadata.

## Reactive Update Requirement

The counter proof must demonstrate reactivity:

```text
counter.increment route accepted
-> CounterModel count changes
-> app model revision changes
-> counter UI projection invalidated
-> count label updates
-> when count becomes 10, win screen is projected
-> UiOutput / UiOutputDelta records the changed screen
```

The reactive update must be reported through `UiUpdateReport` or an equivalent
proof-local update trace.

## Required Reports

A complete implementation must produce or attach these reports:

```text
AppAssemblyReport
PluginGraphReport
HostCompatibilityMatrix
RouteActionResolutionReport
CounterReducerTraceReport
CounterViewProjectionReport
CounterEffectPlanReport
UiSourceValidationReport
UiProgramFormationReport
UiCompilerReport
UiEvaluationReport
UiUpdateReport
AppReplayTrace
ProofManifestReport
```

## Required Proof Scenarios

```text
initial_counting_screen
increment_once
increment_to_nine
increment_to_ten_switches_to_win
increment_after_win_rejected
reset_from_win_returns_to_counter
decrement_at_zero_rejected
reject_unknown_route
reject_missing_capability
reject_invalid_payload
```

Each replay step must record:

```text
before model snapshot
input event packet
route-action resolution
accepted or rejected app action
reducer outcome
effect plan
view projection report
UI update report
UI output summary
after model snapshot
stable diagnostics
```

## Stop Conditions

Stop and redesign if implementation requires:

```text
ECS-owned app model truth
product-owned UiEventPacket construction
product-owned prepared-frame or render primitive plumbing
callback-first UI behavior
duplicated stored screen state
route rejection without diagnostics
missing proof replay trace
renderer-owned product truth
hidden global mutable package registry
```
