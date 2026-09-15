---
title: "Engine Advanced Guide"
description: "Documentation for Engine Advanced Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-14
---

# Engine Advanced Guide

Advanced composition, scheduling, and runtime control patterns for `engine`.

## Custom Schedule Ordering

Use typed schedules and sets to make ordering explicit.

```rust
use engine::prelude::*;

fn plugin_build(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            simulate.in_set(CoreSet::Simulation),
            replicate.after(CoreSet::Simulation).in_set(CoreSet::Replication),
        ),
    );
}

fn simulate() {}
fn replicate() {}
```

## Fixed-Step Tuning

Select fixed cadence explicitly before tuning it:

```rust
use engine::plugins::FixedStepPlugin;
use engine::prelude::*;

fn configure_fixed_step(app: &mut App) {
    app.insert_resource(FixedTimeConfig {
        step_seconds: 1.0 / 120.0,
    });
    app.insert_resource(CatchupBudget {
        max_steps_per_frame: 8,
    });
    app.add_plugin(FixedStepPlugin);
}
```

`FixedStepPlugin` preserves explicitly preinserted cadence resources. Resource presence alone does
not activate fixed-step execution.

Inspect runtime fixed-step status through `FixedTimeState`:

- `accumulator_seconds`
- `steps_ran_last_frame`
- `saturated_frames`
- `total_completed_steps`

## Simulation Identity

Simulation identity is separate from cadence. `SimulationPlugin` provides existing `engine_sim`
owner state and advances `SimulationTick` during `FixedStepBegin` only when fixed cadence is also
active:

```rust
use engine::plugins::{FixedStepPlugin, SimulationPlugin};

app.add_plugins((FixedStepPlugin, SimulationPlugin));
```

Do not use `SimulationTick` as an App/cadence progress counter. Replay/network code may restore or
otherwise reason about simulation identity independently of Runenwerk fixed-step progress.

## Headless Control Patterns

- Use `run_for_frames(n)` for frame-count flows.
- Use `run_for_fixed_steps(n)` for exactly `n` additional completed fixed steps after selecting
  `FixedStepPlugin`.
- `run_for_fixed_steps` uses `FixedTimeState::total_completed_steps`, not `SimulationTick`, as its
  stop condition.
- Set a custom `AppRunner` for test harnesses or tools that need other frame-gating logic.

Primary runner implementations:

- `engine/src/app/domain/runner.rs`

## Plugin Authoring Boundaries

`Plugin::build` should focus on composition only:

- initialize resources
- register systems
- define ordering

Avoid performing long-running runtime work in `build`.

Plugin map:

- [`../plugins/README.md`](../plugins/README.md)
- [`plugins/index.md`](plugins/index.md)

## Network and Replay Integration

Network, replay, world, and scene integrations consume simulation/cadence state as required; they
do not become duplicate default providers. The ordinary `default_plugins()` stack selects Time,
FixedStep, Simulation, and Replay in dependency order.

For network-heavy or replay-heavy stacks, use the dedicated docs:

- Net usage:
  - [`../plugins/net/networking-usage-guide.md`](../plugins/net/networking-usage-guide.md)
- Net runtime flow:
  - [`../plugins/net/network-runtime-flow.md`](../plugins/net/network-runtime-flow.md)
- Replay plugin entry:
  - `engine/src/plugins/replay.rs`
