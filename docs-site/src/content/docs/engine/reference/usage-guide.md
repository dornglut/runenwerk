---
title: "Engine Usage Guide"
description: "Documentation for Engine Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-14
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
state and advances `SimulationTick` during `FixedStepBegin`. `run_for_fixed_steps` stops on cadence
progress, not on simulation tick identity.

## Windowed Example

```rust
use anyhow::Result;
use engine::plugins::default_plugins;
use engine::prelude::*;

struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(default_plugins());
        app.add_systems(Update, update_title);
    }
}

fn update_title(time: Res<Time>, mut window: ResMut<WindowState>) {
    window.set_title(format!("dt={:.4}", time.delta_seconds));
}

fn main() -> Result<()> {
    let mut app = App::new();
    app.set_title("Engine Window");
    app.with_frame_pacing(FramePacingPolicyResource::continuous_capped(60));
    app.add_plugin(WindowPlugin);
    app.run()
}
```

`TimePlugin` updates time in `PreUpdate`. The Engine lifecycle runs `PreUpdate`
before `Update`, so an `Update` system already observes the current time without
an ECS ordering declaration. ECS `before` / `after` references are schedule-local
and must not be used to represent the Engine's order between schedules.

Windowed apps default to `FramePacingPolicyResource::continuous_capped(60)`.
Use `App::with_frame_pacing(FramePacingPolicyResource::on_demand())` for tools
that should redraw only after explicit input, resize, title, cursor, or app
invalidations. The winit runner maps the policy to `WaitUntil` for capped
animation and `Wait` for on-demand mode; systems publish redraw intent through
`WindowState` / `WindowStateRegistryResource` instead of calling winit directly.

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
