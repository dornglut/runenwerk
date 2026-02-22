use ecs::{Archetype, ComponentRegistry, EntityAllocator};
use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, PartialEq)]
struct Velocity {
    dx: f32,
    dy: f32,
}

fn make_archetype() -> Archetype {
    let mut registry = ComponentRegistry::new();
    let position_key = registry.register::<Position>("Position");
    let velocity_key = registry.register::<Velocity>("Velocity");
    Archetype::new_from_registry(&[position_key, velocity_key], &registry)
}

#[test]
fn test_add_row() {
    let mut archetype = make_archetype();

    let mut allocator = EntityAllocator::new();
    let entity = allocator.allocate();

    let mut components = HashMap::new();
    components.insert(
        TypeId::of::<Position>(),
        Box::new(Position { x: 1.0, y: 2.0 }) as Box<dyn Any>,
    );
    components.insert(
        TypeId::of::<Velocity>(),
        Box::new(Velocity { dx: 0.5, dy: 1.0 }) as Box<dyn Any>,
    );

    archetype.add_row(entity, components);

    assert_eq!(archetype.len(), 1);

    let pos_col = archetype
        .column::<Position>(TypeId::of::<Position>())
        .unwrap();
    assert_eq!(pos_col.get(0), Some(&Position { x: 1.0, y: 2.0 }));

    let vel_col = archetype
        .column::<Velocity>(TypeId::of::<Velocity>())
        .unwrap();
    assert_eq!(vel_col.get(0), Some(&Velocity { dx: 0.5, dy: 1.0 }));
}

#[test]
fn test_add_and_remove_row() {
    let mut archetype = make_archetype();

    let mut allocator = EntityAllocator::new();
    let entity1 = allocator.allocate();
    let entity2 = allocator.allocate();

    let mut components1 = HashMap::new();
    components1.insert(
        TypeId::of::<Position>(),
        Box::new(Position { x: 1.0, y: 2.0 }) as Box<dyn Any>,
    );
    components1.insert(
        TypeId::of::<Velocity>(),
        Box::new(Velocity { dx: 0.5, dy: 1.0 }) as Box<dyn Any>,
    );
    archetype.add_row(entity1, components1);

    let mut components2 = HashMap::new();
    components2.insert(
        TypeId::of::<Position>(),
        Box::new(Position { x: 3.0, y: 4.0 }) as Box<dyn Any>,
    );
    components2.insert(
        TypeId::of::<Velocity>(),
        Box::new(Velocity { dx: 1.5, dy: 2.0 }) as Box<dyn Any>,
    );
    archetype.add_row(entity2, components2);

    assert_eq!(archetype.len(), 2);

    let removed = archetype.remove_row(0);
    assert_eq!(removed, entity1);
    assert_eq!(archetype.len(), 1);

    let pos_col = archetype
        .column::<Position>(TypeId::of::<Position>())
        .unwrap();
    let vel_col = archetype
        .column::<Velocity>(TypeId::of::<Velocity>())
        .unwrap();
    assert_eq!(pos_col.get(0), Some(&Position { x: 3.0, y: 4.0 }));
    assert_eq!(vel_col.get(0), Some(&Velocity { dx: 1.5, dy: 2.0 }));

    assert!(archetype.validate());
}
