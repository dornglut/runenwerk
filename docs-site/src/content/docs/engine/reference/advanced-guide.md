---
title: "Engine Advanced Guide"
description: "Documentation for Engine Advanced Guide."
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

Control fixed-step cadence and catchup budget through built-in resources:

- `FixedTimeConfig { step_seconds }`
- `CatchupBudget { max_steps_per_frame }`

```rust
use engine::prelude::*;

fn configure_fixed_step(app: &mut App) {
    app.insert_resource(FixedTimeConfig {
        step_seconds: 1.0 / 120.0,
    });
    app.insert_resource(CatchupBudget {
        max_steps_per_frame: 8,
    });
}
```

Inspect runtime fixed-step status through `FixedTimeState`:

- `accumulator_seconds`
- `steps_ran_last_frame`
- `saturated_frames`

## Headless Control Patterns

- Use `run_for_frames(n)` for deterministic frame-count flows.
- Use `run_for_ticks(n)` when fixed-step progression is your stop condition.
- Set a custom `AppRunner` for test harnesses or simulation tools that need frame-gating logic.

Primary runner implementations:

- [`../../src/app/domain/runner.rs`](../../src/app/domain/runner.rs)

## Plugin Authoring Boundaries

`Plugin::build` should focus on composition only:

- initialize resources
- register systems
- define ordering

Avoid performing long-running runtime work in `build`.

Plugin map:

- [`../../src/plugins/README.md`](../plugins/readme.md)
- [`plugins/index.md`](plugins/index.md)

## Network and Replay Integration

For network-heavy or replay-heavy stacks, use the dedicated docs:

- Net usage:
  - [`../../src/plugins/net/NETWORKING_USAGE_GUIDE.md`](../plugins/net/networking-usage-guide.md)
- Net runtime flow:
  - [`../../src/plugins/net/NETWORK_RUNTIME_FLOW.md`](../plugins/net/network-runtime-flow.md)
- Replay plugin entry:
  - [`../../src/plugins/replay.rs`](../../src/plugins/replay.rs)
