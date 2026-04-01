use ecs::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Component, ecs::StatefulComponent)]
struct StatefulHealth(i32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Component)]
struct Velocity(i32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Component)]
struct Marker;

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Component)]
struct PlainCounter(i32);

fn assert_is_stateful<T: StatefulComponent>() {}

#[test]
fn stateful_component_derive_and_prelude_exports_work() {
    assert_is_stateful::<StatefulHealth>();

    let mut world = World::new();
    let entity = world.spawn(StatefulHealth(1));
    let state: ComponentState = world
        .component_state::<StatefulHealth>(entity)
        .expect("state must exist");

    assert_eq!(state.generation, 1);
    assert_eq!(state.version, 0);
}

#[test]
fn read_only_access_does_not_change_component_state() {
    let mut world = World::new();
    let entity = world.spawn(StatefulHealth(5));

    let state_before = world
        .component_state::<StatefulHealth>(entity)
        .expect("state must exist");
    let _ = world
        .get::<StatefulHealth>(entity)
        .expect("component must exist");
    let _ = world
        .require::<StatefulHealth>(entity)
        .expect("component must exist");
    let state_after = world
        .component_state::<StatefulHealth>(entity)
        .expect("state must exist");

    assert_eq!(state_after, state_before);
}

#[test]
fn state_version_only_changes_on_explicit_mark_stateful_changed() {
    let mut world = World::new();
    let entity = world.spawn(StatefulHealth(5));

    let changed = world.query_state::<(Entity, &StatefulHealth), Changed<StatefulHealth>>();
    assert_eq!(
        changed
            .iter(&world)
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>(),
        vec![entity]
    );
    assert!(changed.iter(&world).next().is_none());

    world
        .get_mut::<StatefulHealth>(entity)
        .expect("component must exist")
        .0 += 1;
    assert_eq!(
        world
            .component_state::<StatefulHealth>(entity)
            .expect("state must exist")
            .version,
        0
    );
    assert_eq!(
        changed
            .iter(&world)
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>(),
        vec![entity]
    );

    assert!(world.mark_stateful_changed::<StatefulHealth>(entity));
    assert_eq!(
        world
            .component_state::<StatefulHealth>(entity)
            .expect("state must exist")
            .version,
        1
    );
    assert_eq!(
        changed
            .iter(&world)
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>(),
        vec![entity]
    );
}

#[test]
fn ordinary_mutable_access_keeps_changed_behavior_without_state_version_bump() {
    let mut world = World::new();
    let entity = world.spawn((StatefulHealth(10), Velocity(2)));

    let changed = world.query_state::<(Entity, &StatefulHealth), Changed<StatefulHealth>>();
    assert!(changed.iter(&world).next().is_some());
    assert!(changed.iter(&world).next().is_none());

    for (health, velocity) in world
        .query_state::<(&mut StatefulHealth, &Velocity), ()>()
        .iter(&mut world)
    {
        health.0 += velocity.0;
    }

    assert_eq!(world.require::<StatefulHealth>(entity).unwrap().0, 12);
    assert_eq!(
        world
            .component_state::<StatefulHealth>(entity)
            .expect("state must exist")
            .version,
        0
    );
    assert_eq!(
        changed
            .iter(&world)
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>(),
        vec![entity]
    );
}

#[test]
fn remove_and_reinsert_creates_new_generation_and_resets_version() {
    let mut world = World::new();
    let entity = world.spawn(StatefulHealth(9));

    assert!(world.mark_stateful_changed::<StatefulHealth>(entity));
    assert!(world.mark_stateful_changed::<StatefulHealth>(entity));

    let state_before = world
        .component_state::<StatefulHealth>(entity)
        .expect("state must exist");
    assert_eq!(state_before.generation, 1);
    assert_eq!(state_before.version, 2);

    let _: StatefulHealth = world.remove(entity).expect("component must be removable");
    world
        .insert(entity, StatefulHealth(100))
        .expect("reinsertion must succeed");

    let state_after = world
        .component_state::<StatefulHealth>(entity)
        .expect("state must exist");
    assert_eq!(state_after.generation, 2);
    assert_eq!(state_after.version, 0);
}

#[test]
fn archetype_moves_preserve_state_generation_and_version_within_lifetime() {
    let mut world = World::new();
    let entity = world.spawn((StatefulHealth(1), Marker));

    assert!(world.mark_stateful_changed::<StatefulHealth>(entity));
    let initial = world
        .component_state::<StatefulHealth>(entity)
        .expect("state must exist");

    world
        .insert(entity, Velocity(3))
        .expect("insert must move archetype");
    let after_insert_move = world
        .component_state::<StatefulHealth>(entity)
        .expect("state must exist");

    let _: Velocity = world.remove(entity).expect("remove must move archetype");
    let after_remove_move = world
        .component_state::<StatefulHealth>(entity)
        .expect("state must exist");

    assert_eq!(after_insert_move, initial);
    assert_eq!(after_remove_move, initial);
}

#[test]
fn non_stateful_components_keep_existing_changed_semantics() {
    let mut world = World::new();
    let entity = world.spawn(PlainCounter(1));

    let changed = world.query_state::<(Entity, &PlainCounter), Changed<PlainCounter>>();
    assert_eq!(
        changed
            .iter(&world)
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>(),
        vec![entity]
    );
    assert!(changed.iter(&world).next().is_none());

    world
        .get_mut::<PlainCounter>(entity)
        .expect("component must exist")
        .0 += 1;
    assert_eq!(world.require::<PlainCounter>(entity).unwrap().0, 2);
    assert_eq!(
        changed
            .iter(&world)
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>(),
        vec![entity]
    );
}

#[test]
fn mark_stateful_changed_returns_false_when_component_missing() {
    let mut world = World::new();
    let with_state = world.spawn(StatefulHealth(1));
    let without_state = world.spawn(PlainCounter(9));

    assert!(world.mark_stateful_changed::<StatefulHealth>(with_state));
    assert!(!world.mark_stateful_changed::<StatefulHealth>(without_state));
    assert_eq!(
        world.component_state::<StatefulHealth>(without_state),
        None,
        "stateful component is absent on entity",
    );
}
