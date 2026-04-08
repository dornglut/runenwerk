---
title: "ECS Crate"
description: "Documentation for ECS crate."
---

# ECS Crate

`ecs` is the ECS runtime foundation in the `domain` layer.

## Quick Overview

- `World`: entities/components/resources, event channels, component indexes, spatial indexes
- Query runtime: `Query`, `QueryState`, `QueryOrphaned`
- Runtime and scheduling bridge: `Runtime`, `IntoSystem`, set ordering (`in_set`, `before`, `after`)
- System params: `Res`, `ResMut`, `ResView`, `Commands`, `BroadcastReader`, `BroadcastWriter`, `BroadcastReader`
- Deferred mutation primitives: `Commands`, `DeferredCommand`, `BatchCommands`
- Stateful tracking: `StatefulComponent`, `component_state`, `mark_stateful_changed`

## Minimal Getting Started

```rust
use ecs::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Position {
    x: f32,
    y: f32,
}

let mut world = World::new();
let entity = world.spawn(Position { x: 1.0, y: 2.0 });
world.require_mut::<Position>(entity).unwrap().x += 1.0;
assert_eq!(world.require::<Position>(entity).unwrap().x, 2.0);
```

## Documentation

- Docs hub: [00-overview.md](./00-overview.md)
- Usage guide: [usage-guide.md](./usage-guide.md)
- Advanced guide: [advanced-guide.md](./advanced-guide.md)
- Architecture (internals): [architecture.md](./architecture.md)
- Feature map: [features.md](./features.md)
