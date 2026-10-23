---
title: "Engine Usage Guide"
description: "Documentation for Engine Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-10-21
publication: primary
---

# Engine Usage Guide

Practical guide for normal `engine` crate workflows.

## Typical Setup

1. Import the prelude and create an app.
2. Register the capabilities/resources/systems the application needs.
3. Run in headless (`run_for_frames`, or `run_for_fixed_steps` after selecting fixed cadence) or windowed (`run`) mode.

A bare `App` does not imply fixed cadence or simulation identity. The ordinary default stack selects
both `FixedStepPlugin` and `SimulationPlugin` explicitly.

## Headless Example

```rust
use anyhow::Result;
use engine::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Component)]
struct Position {
    x: i32,
}

#[derive(Debug, Copy, Clone, PartialEq, Component)]
struct Velocity {
    x: i32,
}

struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, movement);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((Position { x: 0 }, Velocity { x: 1 }));
}

fn movement(mut query: Query<(&mut Position, &Velocity)>) {
    for (position, velocity) in query.iter() {
        position.x += velocity.x;
    }
}

fn main() -> Result<()> {
    let mut app = App::headless();
    app.add_plugin(MovementPlugin);
    let app = app.run_for_frames(3)?;

    let world = app.world();
    let query = world.query::<&Position>();
    let positions: Vec<_> = query.iter(world).map(|position| position.x).collect();
    println!("{positions:?}");
    Ok(())
}
```

## Fixed-Step Headless Example

```rust
use anyhow::Result;
use engine::plugins::{FixedStepPlugin, SimulationPlugin};
use engine::prelude::*;

fn main() -> Result<()> {
    let mut app = App::headless();
    app.add_plugins((FixedStepPlugin, SimulationPlugin));
    app.add_systems(FixedUpdate, simulate);

    let app = app.run_for_fixed_steps(60)?;
    assert_eq!(app.world().resource::<SimulationTick>()?.0, 60);
    Ok(())
}

fn simulate() {}
```

`FixedStepPlugin` activates cadence. `SimulationPlugin` is separate: it supplies simulation owner
state and advances `SimulationTick` during `FixedStepBegin`. The fixed-cadence-owned `AppFixedStepExt::run_for_fixed_steps` stops on cadence
progress, not on simulation tick identity.

## Windowed Example

```rust
use anyhow::Result;
use engine::plugins::default_plugins;
use engine::prelude::*;

fn main() -> Result<()> {
    let mut app = App::new();
    app.set_title("Engine Window");
    app.with_frame_pacing(FramePacingPolicyResource::continuous_capped(60));
    app.add_plugins(default_plugins());
    app.run()
}
```

`TimePlugin` updates time in `PreUpdate`. The Engine lifecycle runs `PreUpdate`
before `Update`, so an `Update` system already observes the current time without
an ECS ordering declaration. ECS `before` / `after` references are schedule-local
and must not be used to represent the Engine's order between schedules.

Windowed apps default to `FramePacingPolicyResource::continuous_capped(60)`.
`AppNativeHostExt::with_frame_pacing` configures the selected native-window Host; with
the trait available through the ordinary prelude, the ergonomic call remains
`app.with_frame_pacing(FramePacingPolicyResource::on_demand())`. Use on-demand pacing for
tools that should redraw only after explicit input, resize, cursor, native-window intent,
or app invalidations. Use
`app.with_frame_pacing(FramePacingPolicyResource::continuous_uncapped())` for
continuous Host turns without an artificial FPS deadline. The winit runner maps
capped animation to `WaitUntil`, and both on-demand and uncapped to event-driven
`Wait`, with uncapped explicitly requesting another redraw on every turn.
The Host still retains one pending-redraw identity per native window and does
not introduce a second scheduling loop. Uncapped Host cadence does **not**
disable Render's current FIFO (VSync) surface presentation or guarantee
unlimited submitted, completed, or physically displayed frames; changing
surface Present mode requires separate capability-aware Render policy.
Fixed-step simulation cadence remains independent. Native pacing is not a headless policy:
configuring it on `App::headless()` is rejected during composition admission. Native
lifecycle/effect intent is keyed by `NativeWindowId` in
`WindowStateRegistryResource`; headless App state does not manufacture a native-window
surrogate.

## Native application launch configuration

Native Host pacing is separate from Render surface presentation policy and fixed
simulation cadence. An application can set it directly through
`AppNativeHostExt::with_frame_pacing`, or accept an optional launch argument.
The effective Host policy remains in the existing App resource, not in a
persistent CLI configuration object.

The application owns **one argument grammar**, including product-specific flags,
`--help`, unknown-argument rejection, and any positional values. When that
grammar encounters a Host option, it can reuse the single typed value parser:

```rust
use anyhow::{Result, bail, ensure};
use engine::app::NativeHostLaunchOverrides;
use engine::prelude::App;

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let mut host = NativeHostLaunchOverrides::default();
    while let Some(arg) = args.next() {
        if let Some(request) = NativeHostLaunchOverrides::parse_option(&arg, &mut args)? {
            ensure!(host.frame_pacing().is_none(), "duplicate --host-pacing argument");
            host = request;
        } else {
            bail!("unknown application argument '{}'", arg.to_string_lossy());
        }
    }

    let mut app = App::new();
    app.set_title("Configured Window");
    // Compose product defaults before lowering explicit launch intent.
    host.apply_to(&mut app)?;
    app.run()
}
```

This minimal example admits only the Host option. Maintained Boids and Arena
define their own complete product grammars in their executable entry points.
They recognize `--help` and `-h` before starting the App; Boids additionally
owns `--species` and `--evidence`.

Both maintained executables accept:

```text
--host-pacing capped          # continuous capped 60 FPS
--host-pacing capped:120      # validated FPS 1..=240
--host-pacing uncapped        # continuous without synthetic Host FPS deadline
--host-pacing on-demand       # redraw only on explicit event/invalidation
```

The equals form (`--host-pacing=capped:120`) is also supported.
Examples:

```text
cargo run -p engine --example boids_render_flow -- --species --host-pacing uncapped
cargo run -p runenwerk_arena -- --host-pacing capped:120
cargo run -p runenwerk_arena -- --help
```

Host parsing occurs **only when the product grammar reaches that option**.
A value-taking product argument consumes its own value before Host matching;
there is no independent pre-scan that can steal a product value. The maintained
executables have no positional options: their `--` terminator and any
unexpected following positional tokens are rejected. Duplicate, missing,
malformed, out-of-range, non-UTF8 Host values and unknown product options fail
before application startup. Boids rejects a Host override combined with
`--evidence`; `--help` exits successfully without starting a window.

An explicit Host override on a headless or already-sealed App is rejected
without applying partial Host state. Omitting the Host flag leaves all
programmatic settings and the default capped-60 policy unchanged. Uncapped
Host redraw does not disable Render FIFO/VSync, guarantee monitor scanout,
change fixed-step cadence, or alter GPU execution. On-demand applications
require explicit native input, window invalidation, or wake to advance;
a continuously animated Boids smoke is not an on-demand idle/wake test.

## Schedules You Will Use Most

- `Startup`
  - one-time setup for resources/entities
- `PreUpdate`
  - input/time/net receive paths and frame-prep logic
- `FixedStepBegin`
  - owner integration adapters at the beginning of each admitted fixed step
- `FixedUpdate`
  - fixed-step systems (0..N times per frame when `FixedStepPlugin` is selected)
- `Update`
  - per-frame gameplay and state updates
- `RenderPrepare`, `RenderSubmit`
  - render-facing preparation and submission
- `FrameEnd`
  - frame cleanup/finalization (for example input pulse clearing)

## Common Workflow References

- Minimal runtime flow:
  - `engine/examples/runtime_minimal/main.rs`
- Window + input flow:
  - `engine/examples/window_input_demo/main.rs`
- Default plugin stack helper:
  - `engine/src/plugins/mod.rs`
- Plugin index:
  - [`../plugins/README.md`](../plugins/README.md)
- Plugin guides:
  - [`plugins/index.md`](plugins/index.md)
