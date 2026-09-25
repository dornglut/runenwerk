---
title: "Replay Plugin Usage Guide"
description: "Documentation for Replay Plugin Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-14
publication: primary
---

# Replay Plugin Usage Guide

## Purpose

Captures simulation command frames/checkpoints and manages replay state resources.

## Entry Points

- Module: `engine/src/plugins/replay.rs`
- Entry: `ReplayPlugin`
- App runtime controls: `AppReplayExt`
- Local README: not present (file-based plugin module)

## Minimal Setup

Replay state can be installed directly:

```rust
use engine::plugins::ReplayPlugin;

app.add_plugin(ReplayPlugin);
```

For recording progression on Runenwerk fixed simulation steps, compose the required capabilities
explicitly:

```rust
use engine::plugins::{FixedStepPlugin, ReplayPlugin, SimulationPlugin};

app.add_plugins((FixedStepPlugin, SimulationPlugin, ReplayPlugin));
```

The ordinary `default_plugins()` stack already selects these in dependency order.

Replay runtime controls are owned by the Replay integration:

```rust
use engine::prelude::{App, AppReplayExt};
use engine::plugins::{ReplayPlugin, SimulationPlugin};

let mut app = App::headless();
app.add_plugins((SimulationPlugin, ReplayPlugin));
app.start_recording()?;
```

`AppReplayExt` preserves the existing runtime-control timing semantics; these methods are not
reclassified as App composition. Replay control admission requires explicit `ReplayPlugin`
selection rather than inferring activation from public Replay resource presence. Starting a
recording also requires an active `SimulationSessionId` established or preserved by the
Simulation integration; replay does not manufacture fallback provenance when simulation identity
is absent.

## Runtime Contract

- Schedule placement: `PreUpdate`, `FixedUpdate`, `FrameEnd`.
- Ownership: replay recording and playback lifecycle resources.
- Consumes simulation identity when recording fixed-step replay data; it does not provide
  `SimulationTick` or other simulation owner state.
- Does not activate fixed cadence.
- Non-ownership: Scene simulation execution itself.

For deterministic bounded headless progression use `App::run_for_fixed_steps(n)` after fixed cadence
has been selected. That API is bounded by Runenwerk cadence progress rather than by replay or
simulation identity.

## Related

- [Simulation Plugin](../simulation/usage-guide.md)
- [Fixed Step Plugin](../fixed-step/usage-guide.md)
- Plugin guides index: [../index.md](../index.md)
- Plugin source map: [../../../../src/plugins/README.md](../../../plugins/README.md)
