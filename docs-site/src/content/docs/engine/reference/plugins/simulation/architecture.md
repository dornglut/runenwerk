---
title: "Simulation Plugin Architecture"
description: "Ownership and lifecycle contract for Runenwerk simulation integration."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-25
publication: primary
---

# Simulation Plugin Architecture

## Ownership Boundary

`SimulationPlugin` is a Runenwerk integration boundary over the existing `engine_sim` domain. The
simulation-domain types remain owned by `engine_sim`; Runenwerk owns only their application/runtime
composition and schedule adaptation.

The plugin non-overwritingly supplies the simulation state required by current Engine consumers:

```text
SimulationTick
SimulationProfileConfig
SimulationSessionId
SimulationSeed
SimulationRng
```

Bare `App` construction supplies none of these resources.

## Fixed-Step Integration

Fixed cadence and simulation identity are separate capabilities:

```text
FixedStepPlugin
  -> cadence activation
  -> FixedStepBegin
  -> FixedUpdate

SimulationPlugin
  -> simulation owner state
  -> SimulationTick advancement in FixedStepBegin
```

When both plugins are selected, simulation tick advancement occurs exactly once at the beginning of
each admitted fixed step, before `FixedUpdate`. `SimulationPlugin` by itself does not activate fixed
cadence; `FixedStepPlugin` by itself does not manufacture simulation identity.

## Composition Rules

- Explicit owner state inserted before plugin installation is preserved.
- `SimulationSessionId` owns no allocator; `SimulationPlugin` installs the App-local initial
  session identity only when none was supplied.
- `SimulationRng` defaults from the effective `SimulationSeed` only when no RNG was supplied.
- Simulation-specific App ergonomics are owned by `AppSimulationExt`; generic `App` does not
  define Simulation-specific inherent methods.
- `AppSimulationExt` configuration lowers into the same owner state; there is no parallel App
  simulation configuration object.
- `AppSimulationExt::current_tick` queries Simulation-owned `SimulationTick` and preserves the
  established absent-state result of `0`.
- World, Net, Replay, and Scene remain consumers of the simulation integration they require; they do
  not become duplicate default providers.
- No generic capability registry, lifecycle event bus, or second scheduler is introduced.

## Related

- [Usage](usage-guide.md)
- [Fixed Step Architecture](../fixed-step/architecture.md)
- Engine architecture: [../../architecture.md](../../architecture.md)
