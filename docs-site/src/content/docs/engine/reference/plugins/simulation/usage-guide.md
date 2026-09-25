---
title: "Simulation Plugin Usage Guide"
description: "Runenwerk integration for engine_sim simulation state and fixed-step identity advancement."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-25
---

# Simulation Plugin Usage Guide

## Purpose

`SimulationPlugin` is Runenwerk's explicit integration provider for existing `engine_sim` owner
state. It does not make simulation identity a bare-App invariant and it does not activate fixed
cadence.

## Provided State

The plugin non-overwritingly provides:

- `SimulationTick`
- `SimulationProfileConfig`
- `SimulationSessionId`
- `SimulationSeed`
- `SimulationRng`

When a seed exists but an RNG does not, the RNG is initialized from the effective seed. Explicitly
supplied owner state is preserved.

`SimulationSessionId` is a passive value type and owns no process-global allocator. When no session
identity was supplied, `SimulationPlugin` installs an App-local initial identity; independent Apps
therefore do not consume shared process identity state.

## Minimal Setup

```rust
use engine::plugins::SimulationPlugin;

app.add_plugin(SimulationPlugin);
```

This installs simulation integration state without causing `FixedUpdate` to run.

For a simulation that advances on Runenwerk fixed cadence, compose it with `FixedStepPlugin`:

```rust
use engine::plugins::{FixedStepPlugin, SimulationPlugin};

app.add_plugins((FixedStepPlugin, SimulationPlugin));
```

For every admitted fixed step, `SimulationPlugin` advances `SimulationTick` in `FixedStepBegin`,
before `FixedUpdate` systems observe the step.

## App Integration API

Simulation-specific App ergonomics are owned by `AppSimulationExt`:

- `set_simulation_profile`
- `set_authority_role`
- `set_simulation_seed`
- `current_tick`

The three configuration methods are explicit composition commands. They may materialize the same
owner configuration state before plugin installation; later `SimulationPlugin` installation
preserves it rather than creating a competing configuration authority.

`current_tick` is a Simulation-owned runtime query. It preserves the established behavior of
returning `0` when no `SimulationTick` state exists. The extension changes API ownership only; it
does not make Simulation a bare-App capability or change query fallibility semantics.

## Related

- [Architecture](architecture.md)
- [Fixed Step Plugin](../fixed-step/usage-guide.md)
- Plugin guides index: [../index.md](../index.md)
