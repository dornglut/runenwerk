use ecs::World;
use std::any::{Any, TypeId};

#[derive(Debug, PartialEq)]
struct Position(f32, f32);

#[derive(Debug, PartialEq)]
struct Velocity(f32, f32);

#[test]
fn test_register_and_spawn_entity() {
    let mut world = World::new();

    world.register_component::<Position>();
    world.register_component::<Velocity>();

    let e1 = world.spawn_entity_typed(Position(1.0, 2.0));
    assert_eq!(world.get_component::<Position>(e1).unwrap(), &Position(1.0, 2.0));

    let e2 = world.spawn_entity(vec![
        Box::new(Position(3.0, 4.0)) as Box<dyn Any>,
        Box::new(Velocity(0.5, 0.5)) as Box<dyn Any>,
    ]);
    assert_eq!(world.get_component::<Position>(e2).unwrap(), &Position(3.0, 4.0));
    assert_eq!(world.get_component::<Velocity>(e2).unwrap(), &Velocity(0.5, 0.5));
}

#[test]
fn test_entities_with() {
    let mut world = World::new();

    world.register_component::<Position>();
    world.register_component::<Velocity>();

    let e1 = world.spawn_entity_typed(Position(0.0, 0.0));
    let e2 = world.spawn_entity(vec![
        Box::new(Position(1.0, 1.0)) as Box<dyn Any>,
        Box::new(Velocity(0.1, 0.1)) as Box<dyn Any>,
    ]);

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
    let e2 = world.spawn_entity(vec![
        Box::new(Position(1.0, 1.0)) as Box<dyn Any>,
        Box::new(Velocity(0.1, 0.1)) as Box<dyn Any>,
    ]);

    world.remove_entity(e2);
    assert!(world.get_component::<Position>(e2).is_none());
    assert!(world.get_component::<Velocity>(e2).is_none());

    assert_eq!(world.get_component::<Position>(e1).unwrap(), &Position(0.0, 0.0));
}

#[test]
fn test_add_entity_with_components() {
    let mut world = World::new();

    world.register_component::<Position>();
    world.register_component::<Velocity>();

    let mut components = std::collections::HashMap::new();
    components.insert(TypeId::of::<Position>(), Box::new(Position(5.0, 5.0)) as Box<dyn Any>);
    components.insert(TypeId::of::<Velocity>(), Box::new(Velocity(1.0, 1.0)) as Box<dyn Any>);

    let e = world.add_entity_with_components(components);
    assert_eq!(world.get_component::<Position>(e).unwrap(), &Position(5.0, 5.0));
    assert_eq!(world.get_component::<Velocity>(e).unwrap(), &Velocity(1.0, 1.0));
}
