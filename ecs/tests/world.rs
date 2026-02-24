use ecs::{
    ComponentChangeKind, EntityDespawnedEvent, EntitySpawnedEvent, ResourceChangeKind, World,
};
use std::any::{Any, TypeId};

#[derive(Debug, PartialEq, ecs::Component)]
struct Position(f32, f32);

#[derive(Debug, PartialEq, ecs::Component)]
struct Velocity(f32, f32);

#[derive(Debug, Clone, PartialEq, Eq, ecs::Component)]
struct Name(String);

#[test]
fn test_register_and_spawn_entity() {
    let mut world = World::new();

    world.register_component::<Position>();
    world.register_component::<Velocity>();

    let e1 = world.spawn_entity_typed(Position(1.0, 2.0));
    assert_eq!(
        world.get_component::<Position>(e1).unwrap(),
        &Position(1.0, 2.0)
    );

    let e2 = world.spawn_bundle((Position(3.0, 4.0), Velocity(0.5, 0.5)));
    assert_eq!(
        world.get_component::<Position>(e2).unwrap(),
        &Position(3.0, 4.0)
    );
    assert_eq!(
        world.get_component::<Velocity>(e2).unwrap(),
        &Velocity(0.5, 0.5)
    );
}

#[test]
fn test_entities_with() {
    let mut world = World::new();

    world.register_component::<Position>();
    world.register_component::<Velocity>();

    let e1 = world.spawn_entity_typed(Position(0.0, 0.0));
    let e2 = world.spawn_bundle((Position(1.0, 1.0), Velocity(0.1, 0.1)));

    let positions: Vec<_> = world.entities_with::<Position>().collect();
    assert!(positions.contains(&e1));
    assert!(positions.contains(&e2));

    let velocities: Vec<_> = world.entities_with::<Velocity>().collect();
    assert!(velocities.contains(&e2));
    assert!(!velocities.contains(&e1));
}

#[test]
fn test_remove_entity() {
    let mut world = World::new();

    world.register_component::<Position>();
    world.register_component::<Velocity>();

    let e1 = world.spawn_entity_typed(Position(0.0, 0.0));
    let e2 = world.spawn_bundle((Position(1.0, 1.0), Velocity(0.1, 0.1)));

    world.remove_entity(e2);
    assert!(world.get_component::<Position>(e2).is_none());
    assert!(world.get_component::<Velocity>(e2).is_none());

    assert_eq!(
        world.get_component::<Position>(e1).unwrap(),
        &Position(0.0, 0.0)
    );
}

#[test]
fn test_add_entity_with_components() {
    let mut world = World::new();

    world.register_component::<Position>();
    world.register_component::<Velocity>();

    let mut components = std::collections::HashMap::new();
    components.insert(
        TypeId::of::<Position>(),
        Box::new(Position(5.0, 5.0)) as Box<dyn Any>,
    );
    components.insert(
        TypeId::of::<Velocity>(),
        Box::new(Velocity(1.0, 1.0)) as Box<dyn Any>,
    );

    let e = world.add_entity_with_components(components);
    assert_eq!(
        world.get_component::<Position>(e).unwrap(),
        &Position(5.0, 5.0)
    );
    assert_eq!(
        world.get_component::<Velocity>(e).unwrap(),
        &Velocity(1.0, 1.0)
    );
}

#[test]
fn test_secondary_index_lookup_tracks_spawn_mutation_and_remove() {
    let mut world = World::new();
    world.register_component::<Name>();
    world.ensure_component_index::<Name, String>(|name| name.0.clone());
    world.ensure_component_index_named::<Name, usize>("len", |name| name.0.len());

    let e = world.spawn_entity_typed(Name("hero".to_string()));
    assert_eq!(
        world.find_entity_by_index::<Name, String>(&"hero".to_string()),
        Some(e)
    );
    assert_eq!(
        world
            .find_component_by_index::<Name, String>(&"hero".to_string())
            .map(|name| name.0.clone()),
        Some("hero".to_string())
    );

    world
        .get_component_mut::<Name>(e)
        .expect("name should exist")
        .0 = "villain".to_string();
    assert!(
        world
            .find_entity_by_index::<Name, String>(&"hero".to_string())
            .is_none()
    );
    assert_eq!(
        world.find_entity_by_index::<Name, String>(&"villain".to_string()),
        Some(e)
    );
    assert_eq!(
        world.find_entity_by_index_named::<Name, usize>("len", &7usize),
        Some(e)
    );

    world.remove_entity(e);
    assert!(
        world
            .find_entity_by_index::<Name, String>(&"villain".to_string())
            .is_none()
    );
    assert!(
        world
            .find_entity_by_index_named::<Name, usize>("len", &7usize)
            .is_none()
    );
}

#[test]
fn test_change_tracking_for_components_and_resources() {
    #[derive(Debug, PartialEq)]
    struct FrameCounter(u64);

    let mut world = World::new();
    world.register_component::<Position>();
    let start = world.current_change_tick();
    assert!(!world.component_changed_since::<Position>(start));

    let entity = world.spawn_entity_typed(Position(1.0, 2.0));
    assert!(world.component_changed_since::<Position>(start));
    let after_spawn = world.current_change_tick();

    world
        .get_component_mut::<Position>(entity)
        .expect("position should exist")
        .0 += 1.0;
    assert!(world.component_changed_since::<Position>(after_spawn));
    world.remove_entity(entity);

    let before_resource = world.current_change_tick();
    world.insert_resource(FrameCounter(1));
    world
        .get_resource_mut::<FrameCounter>()
        .expect("resource should exist")
        .0 += 1;
    let _ = world.remove_resource::<FrameCounter>();
    assert!(world.resource_changed_since::<FrameCounter>(before_resource));

    let component_changes = world.component_changes_since(start);
    assert!(
        component_changes
            .iter()
            .any(|change| change.kind == ComponentChangeKind::Added)
    );
    assert!(
        component_changes
            .iter()
            .any(|change| change.kind == ComponentChangeKind::Modified)
    );
    assert!(
        component_changes
            .iter()
            .any(|change| change.kind == ComponentChangeKind::Removed)
    );

    let resource_changes = world.resource_changes_since(before_resource);
    assert!(
        resource_changes
            .iter()
            .any(|change| change.kind == ResourceChangeKind::Inserted)
    );
    assert!(
        resource_changes
            .iter()
            .any(|change| change.kind == ResourceChangeKind::Modified)
    );
    assert!(
        resource_changes
            .iter()
            .any(|change| change.kind == ResourceChangeKind::Removed)
    );
}

#[test]
fn test_entity_lifecycle_events_emitted() {
    let mut world = World::new();
    world.register_component::<Position>();

    let entity = world.spawn_entity_typed(Position(0.0, 0.0));
    world.remove_entity(entity);

    let spawned = world.drain_events::<EntitySpawnedEvent>();
    let despawned = world.drain_events::<EntityDespawnedEvent>();

    assert_eq!(spawned.len(), 1);
    assert_eq!(spawned[0].entity, entity);
    assert_eq!(despawned.len(), 1);
    assert_eq!(despawned[0].entity, entity);
}
