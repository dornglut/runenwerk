use ecs::{World, WorldQueryExt};
use std::any::Any;

#[derive(Debug, PartialEq, Copy, Clone)]
struct Position(f32, f32);

#[derive(Debug, PartialEq, Copy, Clone)]
struct Velocity(f32, f32);

#[derive(Debug, PartialEq)]
struct Health(u32);

#[test]
fn test_query_two_components_filters_entities() {
    let mut world = World::new();
    world.register_component::<Position>();
    world.register_component::<Velocity>();
    world.register_component::<Health>();

    let with_velocity_1 = world.spawn_entity(vec![
        Box::new(Position(1.0, 2.0)) as Box<dyn Any>,
        Box::new(Velocity(0.25, 0.5)) as Box<dyn Any>,
    ]);

    let _without_velocity = world.spawn_entity(vec![
        Box::new(Position(10.0, 20.0)) as Box<dyn Any>,
        Box::new(Health(100)) as Box<dyn Any>,
    ]);

    let with_velocity_2 = world.spawn_entity(vec![
        Box::new(Position(3.0, 4.0)) as Box<dyn Any>,
        Box::new(Velocity(1.0, 1.5)) as Box<dyn Any>,
        Box::new(Health(80)) as Box<dyn Any>,
    ]);

    let mut seen = Vec::new();
    for (entity, (position, velocity)) in world.query().with::<Position>().with::<Velocity>().iter()
    {
        seen.push((entity.id, *position, *velocity));
    }
    seen.sort_by_key(|(id, _, _)| *id);

    assert_eq!(
        seen,
        vec![
            (with_velocity_1.id, Position(1.0, 2.0), Velocity(0.25, 0.5)),
            (with_velocity_2.id, Position(3.0, 4.0), Velocity(1.0, 1.5)),
        ]
    );
}

#[test]
fn test_query_single_component_across_archetypes() {
    let mut world = World::new();
    world.register_component::<Position>();
    world.register_component::<Velocity>();

    let position_only = world.spawn_entity_typed(Position(8.0, 9.0));
    let position_and_velocity = world.spawn_entity(vec![
        Box::new(Position(2.0, 3.0)) as Box<dyn Any>,
        Box::new(Velocity(0.1, 0.2)) as Box<dyn Any>,
    ]);

    let mut ids: Vec<u32> = world
        .query()
        .with::<Position>()
        .iter()
        .map(|(entity, (_position,))| entity.id)
        .collect();
    ids.sort();

    assert_eq!(ids, vec![position_only.id, position_and_velocity.id]);
}

#[test]
fn test_query_mut_updates_components() {
    let mut world = World::new();
    world.register_component::<Position>();
    world.register_component::<Velocity>();

    let entity = world.spawn_entity(vec![
        Box::new(Position(1.0, 2.0)) as Box<dyn Any>,
        Box::new(Velocity(0.5, -1.0)) as Box<dyn Any>,
    ]);

    world.query_mut::<Position, Velocity, _>(|_, position, velocity| {
        position.0 += velocity.0;
        position.1 += velocity.1;
    });

    assert_eq!(
        world.get_component::<Position>(entity),
        Some(&Position(1.5, 1.0))
    );
}
