use ecs::Reflect;
use ecs::register_reflect_type;

fn assert_reflect<T: ecs::reflect::Reflect>() {}

#[test]
fn position_implements_reflect() {
    assert_reflect::<Position>();
}

#[derive(Debug, Clone, ecs::Reflect)]
struct Vec2 {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, ecs::Component, ecs::ReflectComponent)]
struct Position {
    value: Vec2,
    speed: f32,
}

#[test]
fn reflected_named_struct_exposes_fields() {
    let info = register_reflect_type::<Position>();
    assert!(info.is_component());

    let struct_info = info.struct_info().expect("Position should be a struct");
    assert_eq!(struct_info.field_count(), 2);
    assert_eq!(struct_info.field_at(0).unwrap().name, "value");
    assert_eq!(struct_info.field_at(1).unwrap().name, "speed");
}

#[test]
fn reflected_value_supports_field_lookup() {
    let position = Position {
        value: Vec2 { x: 1.0, y: 2.0 },
        speed: 3.5,
    };

    let reflected = position.reflect_ref();
    let struct_ref = reflected
        .struct_ref()
        .expect("Position should be struct-reflectable");

    let speed = struct_ref
        .field("speed")
        .expect("speed field should exist")
        .downcast_ref::<f32>()
        .expect("speed should be f32");

    assert_eq!(*speed, 3.5);
}

#[test]
fn reflected_value_supports_field_mutation() {
    let mut position = Position {
        value: Vec2 { x: 1.0, y: 2.0 },
        speed: 3.5,
    };

    let reflected = position.reflect_mut();
    let mut struct_mut = reflected
        .struct_mut()
        .expect("Position should be struct-reflectable");

    let mut speed_value = struct_mut
        .field_mut("speed")
        .expect("speed field should exist");

    let speed = speed_value
        .downcast_mut::<f32>()
        .expect("speed should be f32");

    *speed = 7.0;

    assert_eq!(position.speed, 7.0);
}
