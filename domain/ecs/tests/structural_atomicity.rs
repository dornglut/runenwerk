use ecs::{BatchCommands, CommandError, Commands, EntityError, World};
use std::any::TypeId;

#[derive(Debug, PartialEq, Eq, ecs::Component)]
struct A(u32);

#[derive(Debug, PartialEq, Eq, ecs::Component)]
struct B(u32);

#[derive(Debug, PartialEq, Eq, ecs::Component)]
struct C(u32);

#[derive(Debug, PartialEq, Eq, ecs::Component)]
struct D(u32);

#[derive(Debug, PartialEq, Eq, ecs::Component)]
struct NeverInsertedA;

#[derive(Debug, PartialEq, Eq, ecs::Component)]
struct NeverInsertedB;

#[derive(ecs::Bundle)]
struct DerivedPair {
    a: A,
    b: B,
}

#[test]
fn failed_multi_component_removal_preserves_all_components_and_observations() {
    let mut world = World::new();
    let entity = world.spawn(A(7)).expect("spawn should succeed");
    let changes_before = world.component_changes_since(0);
    let location_before = world
        .__entity_archetype_location(entity)
        .expect("entity should have an archetype location");

    let result = world.remove::<(A, B)>(entity);

    assert!(matches!(
        result,
        Err(EntityError::MissingComponent { component, .. })
            if component == std::any::type_name::<B>()
    ));
    assert_eq!(world.require::<A>(entity).expect("A must remain").0, 7);
    assert!(world.get::<B>(entity).is_none());
    assert_eq!(world.component_changes_since(0), changes_before);
    assert_eq!(
        world.__entity_archetype_location(entity),
        Some(location_before)
    );
}

#[test]
fn duplicate_type_removal_keeps_existing_failure_semantics_without_partial_mutation() {
    let mut world = World::new();
    let entity = world.spawn(A(11)).expect("spawn should succeed");
    let changes_before = world.component_changes_since(0);

    let result = world.remove::<(A, A)>(entity);

    assert!(matches!(
        result,
        Err(EntityError::MissingComponent { component, .. })
            if component == std::any::type_name::<A>()
    ));
    assert_eq!(world.require::<A>(entity).expect("A must remain").0, 11);
    assert_eq!(world.component_changes_since(0), changes_before);
}

#[test]
fn rejected_foreign_insert_does_not_register_bundle_or_publish_changes() {
    let mut first = World::new();
    let mut second = World::new();
    let foreign = second.spawn(A(1)).expect("foreign spawn should succeed");
    let changes_before = first.component_changes_since(0);

    let result = first.insert(foreign, (NeverInsertedA, NeverInsertedB));

    assert!(matches!(result, Err(EntityError::ForeignWorld { .. })));
    assert!(!first.has_registered_component_type(TypeId::of::<NeverInsertedA>()));
    assert!(!first.has_registered_component_type(TypeId::of::<NeverInsertedB>()));
    assert_eq!(first.component_changes_since(0), changes_before);
}

#[test]
fn supported_derived_and_tuple_bundles_commit_complete_structural_operations() {
    let mut world = World::new();
    let entity = world
        .spawn(DerivedPair { a: A(3), b: B(5) })
        .expect("derived bundle spawn should succeed");

    assert_eq!(world.require::<A>(entity).expect("A should exist").0, 3);
    assert_eq!(world.require::<B>(entity).expect("B should exist").0, 5);

    world
        .insert(entity, (C(8), D(13)))
        .expect("tuple insert should succeed");
    assert_eq!(world.require::<C>(entity).expect("C should exist").0, 8);
    assert_eq!(world.require::<D>(entity).expect("D should exist").0, 13);

    let removed = world
        .remove::<DerivedPair>(entity)
        .expect("derived bundle removal should succeed");
    assert_eq!(removed.a, A(3));
    assert_eq!(removed.b, B(5));
    assert!(world.get::<A>(entity).is_none());
    assert!(world.get::<B>(entity).is_none());
    assert_eq!(world.require::<C>(entity).expect("C should remain").0, 8);
    assert_eq!(world.require::<D>(entity).expect("D should remain").0, 13);
}

#[test]
fn component_observation_tick_matches_committed_storage_tick() {
    let mut world = World::new();
    let entity = world.spawn(A(1)).expect("spawn should succeed");
    let first_change = world
        .component_changes_since(0)
        .into_iter()
        .last()
        .expect("spawn should publish an added observation");
    let (_, first_storage_tick) = world
        .__entity_component_ticks::<A>(entity)
        .expect("A should have row metadata");
    assert_eq!(first_storage_tick, first_change.tick);

    world.insert(entity, A(2)).expect("update should succeed");
    let latest_change = world
        .component_changes_since(first_change.tick)
        .into_iter()
        .last()
        .expect("update should publish a modified observation");
    let (_, latest_storage_tick) = world
        .__entity_component_ticks::<A>(entity)
        .expect("A should keep row metadata");
    assert_eq!(latest_storage_tick, latest_change.tick);
    assert_eq!(world.require::<A>(entity).expect("A should exist").0, 2);
}

#[test]
fn commands_fail_stop_without_rolling_back_prior_success_or_partially_applying_failure() {
    let mut world = World::new();
    let entity = world.spawn(A(1)).expect("spawn should succeed");
    let mut commands = Commands::new();
    commands.insert(entity, C(2));
    commands.remove::<(A, B)>(entity);
    commands.insert(entity, D(3));

    let result = commands.apply(&mut world);

    assert!(matches!(
        result,
        Err(CommandError::Entity(EntityError::MissingComponent { .. }))
    ));
    assert_eq!(
        world
            .require::<C>(entity)
            .expect("earlier command commits")
            .0,
        2
    );
    assert_eq!(
        world
            .require::<A>(entity)
            .expect("failed removal is atomic")
            .0,
        1
    );
    assert!(world.get::<D>(entity).is_none());
}

#[test]
fn batch_commands_share_non_transactional_fail_stop_semantics() {
    let mut world = World::new();
    let entity = world.spawn(A(1)).expect("spawn should succeed");
    let mut batch = BatchCommands::new();
    batch.insert(entity, C(2));
    batch.remove::<(A, B)>(entity);
    batch.insert(entity, D(3));

    let result = batch.apply(&mut world);

    assert!(matches!(
        result,
        Err(CommandError::Entity(EntityError::MissingComponent { .. }))
    ));
    assert_eq!(
        world
            .require::<C>(entity)
            .expect("earlier command commits")
            .0,
        2
    );
    assert_eq!(
        world
            .require::<A>(entity)
            .expect("failed removal is atomic")
            .0,
        1
    );
    assert!(world.get::<D>(entity).is_none());
}
