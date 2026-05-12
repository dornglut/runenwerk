use ecs::{Component, QueryAccess, Resource, World, query_snapshot_source_generation};

#[derive(Component)]
struct Position(i32);

#[derive(Component)]
struct Velocity;

#[derive(Resource)]
struct ViewportObservation(u64);

#[test]
fn query_snapshot_source_generation_changes_after_component_mutation() {
    let mut world = World::new();
    let entity = world.spawn((Position(1), Velocity));
    let access = QueryAccess::default().with_component_read::<Position>();
    let unrelated_access = QueryAccess::default().with_component_read::<Velocity>();

    let initial_generation = query_snapshot_source_generation(&world, &access);
    let unrelated_initial_generation = query_snapshot_source_generation(&world, &unrelated_access);

    world.get_mut::<Position>(entity).unwrap().0 += 10;

    let mutated_generation = query_snapshot_source_generation(&world, &access);
    let unrelated_generation = query_snapshot_source_generation(&world, &unrelated_access);

    assert!(mutated_generation > initial_generation);
    assert_eq!(unrelated_generation, unrelated_initial_generation);
}

#[test]
fn query_snapshot_source_generation_changes_after_resource_mutation() {
    let mut world = World::new();
    world.insert_resource(ViewportObservation(1));
    let access = QueryAccess::default().with_resource_read::<ViewportObservation>();

    let initial_generation = query_snapshot_source_generation(&world, &access);
    world.resource_mut::<ViewportObservation>().unwrap().0 += 1;
    let mutated_generation = query_snapshot_source_generation(&world, &access);

    assert!(mutated_generation > initial_generation);
}
