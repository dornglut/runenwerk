---
title: "Fixed Step Plugin Architecture"
description: "Documentation for Fixed Step Plugin Architecture."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-14
---

# Fixed Step Plugin Architecture

## Ownership Boundary

`FixedStepPlugin` is the explicit capability selector for Runenwerk fixed cadence.

It owns the integration contract for:

- `FixedTimeConfig`
- `CatchupBudget`
- `FixedTimeState`
- private fixed-cadence activation state

It does **not** own simulation identity. In particular, it neither installs nor advances
`SimulationTick`.

## Runtime Contract

When the plugin is selected, the shared App frame lifecycle admits fixed-step work. Each admitted
step executes:

```text
FixedStepBegin
FixedUpdate
```

`FixedStepBegin` is the narrow Runenwerk-owned lifecycle occurrence for owner adapters that must
observe the beginning of a fixed step. `SimulationPlugin` uses it to advance simulation identity.

Without `FixedStepPlugin`, an App still runs the ordinary frame schedules but does not execute
`FixedStepBegin` or `FixedUpdate`. Merely inserting the public cadence resources does not activate
the capability.

## Module Layout

- Capability selector: `engine/src/plugins/fixed_step.rs`
- Cadence state: `engine/src/runtime/fixed_time.rs`
- Executor: `engine/src/runtime/fixed_step_executor.rs`
- Frame activation gate: `engine/src/runtime/frame_lifecycle.rs`
- Lifecycle labels: `engine/src/runtime/schedules.rs`

## Ownership Rules

- Cadence occurrence and progress are Runenwerk runtime semantics.
- `FixedTimeState::total_completed_steps` is cadence progress used by bounded App advancement.
- Simulation/network/replay identity remains with the relevant domain integration.
- Public resource presence is configuration/state, not capability authority.
- No generic capability registry or second scheduler is involved.
