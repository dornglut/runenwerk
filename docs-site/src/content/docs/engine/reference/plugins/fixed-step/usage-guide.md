---
title: "Fixed Step Plugin Usage Guide"
description: "Documentation for Fixed Step Plugin Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-14
publication: primary
---

# Fixed Step Plugin Usage Guide

## Purpose

Selects Runenwerk fixed cadence and installs the cadence resources used by the shared fixed-step
executor.

## Entry Points

- Module: `engine/src/plugins/fixed_step.rs`
- Entry: `FixedStepPlugin`
- Cadence state: `FixedTimeConfig`, `CatchupBudget`, `FixedTimeState`
- Fixed-step lifecycle: `FixedStepBegin` -> `FixedUpdate`

## Minimal Setup

```rust
use engine::plugins::FixedStepPlugin;

app.add_plugin(FixedStepPlugin);
```

Selecting the plugin is what activates fixed-step execution. Inserting `FixedTimeConfig`,
`CatchupBudget`, or `FixedTimeState` by itself does not activate the capability.

## Simulation Integration

Fixed cadence does not imply simulation identity. Add `SimulationPlugin` when the composition also
needs `SimulationTick`, simulation profile/session/seed state, or simulation-tick advancement:

```rust
use engine::plugins::{FixedStepPlugin, SimulationPlugin};

app.add_plugins((FixedStepPlugin, SimulationPlugin));
```

`SimulationPlugin` advances `SimulationTick` during `FixedStepBegin`, before systems in
`FixedUpdate` observe the step.

## Bounded Headless Advancement

Use fixed-cadence-owned `AppFixedStepExt::run_for_fixed_steps(n)` to execute exactly `n` additional completed fixed steps. The ordinary prelude keeps the ergonomic `app.run_for_fixed_steps(n)` call:

```rust
let app = app.run_for_fixed_steps(60)?;
```

The stop condition uses Runenwerk cadence progress (`FixedTimeState::total_completed_steps`), not
`SimulationTick`. The call returns an error when `FixedStepPlugin` was not selected.

## Runtime Contract

- `FixedStepPlugin` owns cadence activation and default cadence state installation.
- Explicitly preinserted cadence resources are preserved.
- Each admitted step runs `FixedStepBegin` and then `FixedUpdate` exactly once.
- Fixed cadence does not install or advance simulation identity.

## Related

- [Architecture](architecture.md)
- [Simulation Plugin](../simulation/usage-guide.md)
- Plugin guides index: [../index.md](../index.md)
