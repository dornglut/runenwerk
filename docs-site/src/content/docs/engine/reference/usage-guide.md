---
title: "Engine Usage Guide"
description: "Documentation for Engine Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Engine Usage Guide

Practical guide for normal `engine` crate workflows.

## Typical Setup

1. Import the prelude and create an app.
2. Register plugins/resources/systems.
3. Run in headless (`run_for_frames`, `run_for_ticks`) or windowed (`run`) mode.

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
    let query = world.query_state::<&Position, ()>();
    let positions: Vec<_> = query.iter(world).map(|position| position.x).collect();
    println!("{positions:?}");
    Ok(())
}
```

## Windowed Example

```rust
use anyhow::Result;
use engine::plugins::default_plugins;
use engine::prelude::*;

struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(default_plugins());
        app.add_systems(Update, update_title.after(CoreSet::Time));
    }
}

fn update_title(time: Res<Time>, mut window: ResMut<WindowState>) {
    window.set_title(format!("dt={:.4}", time.delta_seconds));
}

fn main() -> Result<()> {
    let mut app = App::new();
    app.set_title("Engine Window");
    app.add_plugin(WindowPlugin);
    app.run()
}
```

## Schedules You Will Use Most

- `Startup`
  - one-time setup for resources/entities
- `PreUpdate`
  - input/time/net receive paths and frame-prep logic
- `FixedUpdate`
  - fixed-step simulation systems (0..N times per frame)
- `Update`
  - per-frame gameplay and state updates
- `RenderPrepare`, `RenderSubmit`
  - render-facing preparation and submission
- `FrameEnd`
  - frame cleanup/finalization (for example input pulse clearing)

## Common Workflow References

- Minimal runtime flow:
  - [`../../examples/runtime_minimal/main.rs`](../../examples/runtime_minimal/main.rs)
- Window + input flow:
  - [`../../examples/window_input_demo/main.rs`](../../examples/window_input_demo/main.rs)
- Default plugin stack helper:
  - [`../../src/plugins/mod.rs`](../../src/plugins/mod.rs)
- Plugin index:
  - [`../../src/plugins/README.md`](../plugins/README.md)
- Plugin guides:
  - [`plugins/index.md`](plugins/index.md)
