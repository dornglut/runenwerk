use ecs::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Player;

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Disabled;

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Health(i32);

#[derive(Debug, Clone, PartialEq, Eq, ecs::Component)]
struct Name(String);

#[derive(Debug, PartialEq, Eq)]
struct Frame(u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct TickEvent;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct DamageEvent(u32);

#[derive(Debug, PartialEq, ecs::Bundle)]
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
fn resource_lifecycle_and_change_logs_work() {
    let mut world = World::new();
    assert!(!world.has_resource::<Frame>());

    let start = world.current_change_tick();
    world.insert_resource(Frame(10));
    {
        let mut frame = world.resource_mut::<Frame>().unwrap();
        frame.0 += 5;
    }
    let removed = world.remove_resource::<Frame>();

    assert_eq!(removed, Some(Frame(15)));
    assert!(!world.has_resource::<Frame>());

    let changes = world.resource_changes_since(start);
    let kinds: Vec<_> = changes.iter().map(|change| change.kind).collect();
    assert_eq!(
        kinds,
        vec![
            ResourceChangeKind::Inserted,
            ResourceChangeKind::Modified,
            ResourceChangeKind::Removed,
        ]
    );
    assert!(
        changes
            .iter()
            .all(|change| change.resource_name.ends_with("Frame"))
    );
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
fn secondary_index_helpers_and_component_change_logs_work() {
    let mut world = World::new();
    world.ensure_component_index::<Name, String>(|name| name.0.clone());
    world.ensure_component_index_named::<Name, char>("initial", |name| {
        name.0.chars().next().unwrap_or_default()
    });

    let hero = world.spawn((Name("hero".to_string()), Health(10)));
    let helper = world.spawn((Name("healer".to_string()), Health(7)));
    let villain = world.spawn((Name("villain".to_string()), Health(9)));

    assert_eq!(
        world.find_entities_by_index_named::<Name, char>("initial", &'h'),
        vec![hero, helper]
    );
    assert_eq!(
        world.find_component_by_index::<Name, String>(&"villain".to_string()),
        Some(&Name("villain".to_string()))
    );

    let start = world.current_change_tick();
    world.require_mut::<Name>(hero).unwrap().0 = "hunter".to_string();
    world.despawn(villain).unwrap();

    let changes = world.component_changes_since(start);
    assert!(changes.iter().any(|change| {
        change.entity == hero
            && change.component_name.ends_with("Name")
            && change.kind == ComponentChangeKind::Modified
    }));
    assert!(changes.iter().any(|change| {
        change.entity == villain
            && change.component_name.ends_with("Name")
            && change.kind == ComponentChangeKind::Removed
    }));
    assert!(changes.iter().any(|change| {
        change.entity == villain
            && change.component_name.ends_with("Health")
            && change.kind == ComponentChangeKind::Removed
    }));
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

#[test]
fn event_observers_and_drain_helpers_work() {
    let mut world = World::new();
    assert!(!world.has_event_channel::<TickEvent>());
    assert!(world.ensure_event_channel::<TickEvent>());
    assert!(!world.ensure_event_channel::<TickEvent>());
    assert!(world.has_event_channel::<TickEvent>());

    assert!(world.observe_events::<TickEvent>("tick_emit", ObserverTrigger::OnEmit));
    assert!(world.observe_events::<TickEvent>("tick_drain", ObserverTrigger::OnDrain));
    assert!(world.observe_events::<TickEvent>("tick_frame", ObserverTrigger::EndOfFrame));

    world.emit_event(TickEvent);
    world.emit_event(TickEvent);
    let mapped = world.drain_events_map::<TickEvent, _, _>(|_| "tick");
    assert_eq!(mapped, vec!["tick", "tick"]);

    world.emit_event(DamageEvent(1));
    world.emit_event(DamageEvent(2));
    let filtered = world.drain_events_filter::<DamageEvent, _>(|event| event.0 % 2 == 0);
    assert_eq!(filtered, vec![DamageEvent(2)]);

    world.emit_event(TickEvent);
    world.finish_event_frame();

    assert_eq!(world.event_observer_invocations("tick_emit"), Some(3));
    assert_eq!(world.event_observer_invocations("tick_drain"), Some(1));
    assert_eq!(world.event_observer_invocations("tick_frame"), Some(1));

    let notifications = world.drain_event_observer_notifications();
    assert!(notifications.iter().any(|notification| {
        notification.observer_id == "tick_emit"
            && notification.trigger == ObserverTrigger::OnEmit
            && notification.event_count == 1
    }));
    assert!(notifications.iter().any(|notification| {
        notification.observer_id == "tick_drain"
            && notification.trigger == ObserverTrigger::OnDrain
            && notification.event_count == 2
    }));
    assert!(notifications.iter().any(|notification| {
        notification.observer_id == "tick_frame"
            && notification.trigger == ObserverTrigger::EndOfFrame
            && notification.event_count == 1
    }));

    assert!(world.remove_event_observer("tick_frame"));
    assert!(!world.remove_event_observer("tick_frame"));
}
