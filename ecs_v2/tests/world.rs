use ecs_v2::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, ecs_v2::Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, ecs_v2::Component)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, ecs_v2::Component)]
struct Player;

#[derive(Debug, Copy, Clone, PartialEq, ecs_v2::Component)]
struct Disabled;

#[derive(Debug, Copy, Clone, PartialEq, ecs_v2::Component)]
struct Health(i32);

#[derive(Debug, Clone, PartialEq, Eq, ecs_v2::Component)]
struct Name(String);

#[derive(Debug, PartialEq, Eq)]
struct Frame(u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct TickEvent;

#[derive(Debug, PartialEq, ecs_v2::Bundle)]
struct CombatBundle {
    health: Health,
    name: Name,
}

#[test]
fn spawn_query_and_entity_access_work() {
    let mut world = World::new();
    let entity = world.spawn((
        Player,
        Position { x: 1.0, y: 2.0 },
        Velocity { x: 0.5, y: -1.0 },
    ));

    let position = world.require::<Position>(entity).unwrap();
    assert_eq!(position.x, 1.0);
    assert_eq!(position.y, 2.0);

    let entity_ref = world.entity(entity).unwrap();
    assert!(entity_ref.contains::<Player>());
    assert!(entity_ref.contains::<Velocity>());

    let seen: Vec<_> = world
        .query::<(Entity, &Position)>()
        .with::<Player>()
        .iter()
        .map(|(entity, position)| (entity, position.x, position.y))
        .collect();
    assert_eq!(seen, vec![(entity, 1.0, 2.0)]);
}

#[test]
fn mutable_queries_support_filters() {
    let mut world = World::new();
    let active = world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 2.0, y: 1.0 }));
    let disabled = world.spawn((
        Position { x: 5.0, y: 5.0 },
        Velocity { x: 9.0, y: 9.0 },
        Disabled,
    ));

    let mut query = world
        .query_mut::<(&mut Position, &Velocity)>()
        .without::<Disabled>();
    for (position, velocity) in query.iter_mut() {
        position.x += velocity.x;
        position.y += velocity.y;
    }

    assert_eq!(
        world.require::<Position>(active).unwrap(),
        &Position { x: 2.0, y: 1.0 }
    );
    assert_eq!(
        world.require::<Position>(disabled).unwrap(),
        &Position { x: 5.0, y: 5.0 }
    );
}

#[test]
fn entity_mut_bundle_insert_and_remove_work() {
    let mut world = World::new();
    let entity = world.spawn(Player);

    {
        let mut entity_mut = world.entity_mut(entity).unwrap();
        entity_mut
            .insert(CombatBundle {
                health: Health(10),
                name: Name("hero".to_string()),
            })
            .unwrap();
        entity_mut.require_mut::<Health>().unwrap().0 -= 3;
    }

    assert_eq!(world.require::<Health>(entity).unwrap(), &Health(7));

    let removed: CombatBundle = world.remove(entity).unwrap();
    assert_eq!(removed.health, Health(7));
    assert_eq!(removed.name, Name("hero".to_string()));
    assert!(world.get::<Health>(entity).is_none());
}

#[test]
fn resources_and_change_ticks_work() {
    let mut world = World::new();
    let start = world.current_change_tick();
    world.insert_resource(Frame(1));
    assert!(world.resource_changed_since::<Frame>(start));

    {
        let mut frame = world.resource_mut::<Frame>().unwrap();
        frame.0 += 1;
    }

    assert_eq!(world.resource::<Frame>().unwrap().0, 2);
}

#[test]
fn commands_apply_spawn_insert_and_despawn() {
    let mut world = World::new();
    let existing = world.spawn(Position { x: 1.0, y: 1.0 });
    let doomed = world.spawn(Position { x: 99.0, y: 99.0 });

    let mut commands = world.commands();
    commands.spawn((Position { x: 3.0, y: 4.0 }, Velocity { x: 0.0, y: 1.0 }));
    commands.insert(existing, Velocity { x: 5.0, y: 6.0 });
    commands.despawn(doomed);
    commands.apply(&mut world).unwrap();

    assert!(world.contains(existing));
    assert!(world.require::<Velocity>(existing).is_ok());
    assert!(!world.contains(doomed));

    let positions: Vec<_> = world
        .query::<&Position>()
        .iter()
        .map(|position| *position)
        .collect();
    assert_eq!(positions.len(), 2);
    assert!(positions.contains(&Position { x: 1.0, y: 1.0 }));
    assert!(positions.contains(&Position { x: 3.0, y: 4.0 }));
}

#[test]
fn secondary_indexes_track_updates() {
    let mut world = World::new();
    world.ensure_component_index::<Name, String>(|name| name.0.clone());

    let entity = world.spawn(Name("hero".to_string()));
    assert_eq!(
        world.find_entity_by_index::<Name, String>(&"hero".to_string()),
        Some(entity)
    );

    world.require_mut::<Name>(entity).unwrap().0 = "villain".to_string();
    assert_eq!(
        world.find_entity_by_index::<Name, String>(&"hero".to_string()),
        None
    );
    assert_eq!(
        world.find_entity_by_index::<Name, String>(&"villain".to_string()),
        Some(entity)
    );
}

#[test]
fn event_channels_support_emit_drain_and_frame_transient_cleanup() {
    let mut world = World::new();
    world.configure_event_channel::<TickEvent>(EventChannelConfig {
        capacity: Some(1),
        overflow: OverflowPolicy::DropOldest,
        lifetime: EventLifetime::FrameTransient,
        tracing: EventTracingPolicy::Disabled,
    });

    world.emit_event(TickEvent);
    world.emit_event(TickEvent);
    assert_eq!(world.event_count::<TickEvent>(), 1);

    let stats = world.event_channel_stats::<TickEvent>().unwrap();
    assert_eq!(stats.emitted, 2);
    assert_eq!(stats.dropped, 1);

    world.finish_event_frame();
    assert_eq!(world.event_count::<TickEvent>(), 0);
}
