use ecs::World;

#[derive(Debug, PartialEq)]
struct FrameCounter(u64);

#[derive(Debug, PartialEq)]
struct AppTitle(String);

#[test]
fn world_resource_insert_get_replace_and_remove() {
    let mut world = World::new();
    assert!(!world.has_resource::<FrameCounter>());
    assert!(world.get_resource::<FrameCounter>().is_none());

    assert!(world.insert_resource(FrameCounter(1)).is_none());
    assert!(world.has_resource::<FrameCounter>());
    assert_eq!(world.get_resource::<FrameCounter>(), Some(&FrameCounter(1)));

    let previous = world.insert_resource(FrameCounter(2));
    assert_eq!(previous, Some(FrameCounter(1)));
    assert_eq!(world.get_resource::<FrameCounter>(), Some(&FrameCounter(2)));

    let removed = world.remove_resource::<FrameCounter>();
    assert_eq!(removed, Some(FrameCounter(2)));
    assert!(!world.has_resource::<FrameCounter>());
}

#[test]
fn world_resource_get_mut_updates_state() {
    let mut world = World::new();
    world.insert_resource(AppTitle("Grotto Quest".to_string()));

    let title = world
        .get_resource_mut::<AppTitle>()
        .expect("title resource should exist");
    title.0 = "Grotto Quest - Scene Manager".to_string();

    assert_eq!(
        world.get_resource::<AppTitle>(),
        Some(&AppTitle("Grotto Quest - Scene Manager".to_string()))
    );
}
