use ecs::{World, WorldQueryExt};

#[derive(Debug, PartialEq, Copy, Clone, ecs::Component)]
struct Position(f32, f32);

#[derive(Debug, PartialEq, Copy, Clone, ecs::Component)]
struct Velocity(f32, f32);

#[derive(Debug, PartialEq, ecs::Component)]
struct Health(u32);

#[test]
fn test_query_two_components_filters_entities() {
    let mut world = World::new();
    world.register_component::<Position>();
    world.register_component::<Velocity>();
    world.register_component::<Health>();

    let with_velocity_1 = world.spawn_bundle((Position(1.0, 2.0), Velocity(0.25, 0.5)));

    let _without_velocity = world.spawn_bundle((Position(10.0, 20.0), Health(100)));

    let with_velocity_2 = world.spawn_bundle((Position(3.0, 4.0), Velocity(1.0, 1.5), Health(80)));

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
    let position_and_velocity = world.spawn_bundle((Position(2.0, 3.0), Velocity(0.1, 0.2)));

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

    let entity = world.spawn_bundle((Position(1.0, 2.0), Velocity(0.5, -1.0)));

    world.query_mut::<Position, Velocity, _>(|_, position, velocity| {
        position.0 += velocity.0;
        position.1 += velocity.1;
    });

    assert_eq!(
        world.get_component::<Position>(entity),
        Some(&Position(1.5, 1.0))
    );
}

#[derive(Debug, PartialEq, Copy, Clone, ecs::Component)]
struct Disabled;

#[test]
fn test_query_mut_builder_supports_with_without_filters() {
    let mut world = World::new();
    world.register_component::<Position>();
    world.register_component::<Velocity>();
    world.register_component::<Health>();
    world.register_component::<Disabled>();

    let e_a = world.spawn_bundle((Position(0.0, 0.0), Velocity(1.0, 0.0), Health(10)));
    let e_b = world.spawn_bundle((Position(0.0, 0.0), Velocity(2.0, 0.0), Disabled));
    let e_c = world.spawn_bundle((Position(0.0, 0.0), Health(20)));

    world
        .query_mut_components::<Position>()
        .with::<Velocity>()
        .without::<Disabled>()
        .for_each_with::<Velocity, _>(|_, position, velocity| {
            position.0 += velocity.0;
            position.1 += velocity.1;
        });

    assert_eq!(
        world.get_component::<Position>(e_a),
        Some(&Position(1.0, 0.0))
    );
    assert_eq!(
        world.get_component::<Position>(e_b),
        Some(&Position(0.0, 0.0))
    );
    assert_eq!(
        world.get_component::<Position>(e_c),
        Some(&Position(0.0, 0.0))
    );
}
