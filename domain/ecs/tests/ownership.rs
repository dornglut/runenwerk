use ecs::{OwnerRole, OwnerState, World};

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Component, ecs::Resource)]
struct SharedScore(pub i32);

#[test]
fn controller_ids_are_runtime_local_and_monotonic() {
    let mut world = World::new();

    let first = world.create_owner(OwnerRole::Active);
    let second = world.create_owner(OwnerRole::Active);

    assert!(first.as_raw() > 0);
    assert_eq!(second.as_raw(), first.as_raw() + 1);
}

#[test]
fn spectator_routes_to_no_targets() {
    let mut world = World::new();
    let spectator = world.create_owner(OwnerRole::Observer);
    let controller = world.create_owner(OwnerRole::Active);

    let entity = world.spawn(SharedScore(0));
    assert!(world.assign_entity_owner(entity, OwnerState::OwnedBy(controller)));

    assert!(world.route_owner_entities(spectator).is_empty());
    assert!(world.route_owner_targets(spectator).is_empty());
    assert_eq!(world.owner_owned_target_count(spectator), 0);

    assert_eq!(world.route_owner_entities(controller), vec![entity]);
}

#[test]
fn ownership_transfer_log_tracks_entity_and_resource_changes() {
    let mut world = World::new();
    world.insert_resource(SharedScore(10));

    let controller = world.create_owner(OwnerRole::Active);
    let entity = world.spawn(SharedScore(0));

    let start_sequence = world.ownership_transfer_sequence();

    assert!(world.assign_entity_owner(entity, OwnerState::OwnedBy(controller)));
    assert!(world.assign_resource_owner::<SharedScore>(OwnerState::OwnedBy(controller)));
    assert!(world.transfer_entity_owner(entity, OwnerState::WorldOwned));

    let updates = world.ownership_transfers_since(start_sequence);
    assert_eq!(updates.len(), 3);
    assert_eq!(updates[0].sequence, start_sequence + 1);
    assert_eq!(updates[1].sequence, start_sequence + 2);
    assert_eq!(updates[2].sequence, start_sequence + 3);

    assert_eq!(world.entity_owner(entity), OwnerState::WorldOwned);
    assert_eq!(
        world.resource_owner::<SharedScore>(),
        OwnerState::OwnedBy(controller)
    );
}

#[test]
fn resource_owner_keys_are_stable_per_type() {
    let mut world = World::new();
    world.insert_resource(SharedScore(1));

    let first = world.ensure_resource_owner_key::<SharedScore>();
    let second = world.ensure_resource_owner_key::<SharedScore>();
    assert_eq!(first, second);

    let descriptor = world
        .resource_ownership_descriptor(first)
        .expect("resource descriptor should exist");
    assert_eq!(descriptor.key, first);
    assert!(descriptor.resource_name.ends_with("SharedScore"));
}
