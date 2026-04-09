//! File: domain/editor/editor_inspector/src/editing.rs
//! Purpose: Path-based reflective mutation for ECS-backed inspector targets.

use crate::{EcsInspectorBridge, InspectorPath, InspectorPathSegment};
use ecs::reflect::ReflectValueMut;
use editor_core::{ComponentTypeId, EntityId, ResourceTypeId};

#[derive(Debug, Clone, PartialEq)]
pub enum InspectorEditValue {
    Bool(bool),
    Integer(i64),
    Float(f64),
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InspectorEditError {
    TargetNotFound,
    TypeNotRegistered,
    ValueNotAvailable,
    InvalidPath,
    UnsupportedPathSegment,
    UnsupportedValueType {
        actual_type: String,
    },
    IntegerOutOfRange {
        target_type: &'static str,
        value: i64,
    },
    FloatOutOfRange {
        target_type: &'static str,
        value: f64,
    },
}

impl InspectorEditValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Bool(_) => "bool",
            Self::Integer(_) => "integer",
            Self::Float(_) => "float",
            Self::Text(_) => "String",
        }
    }
}

pub fn set_component_field_value<B>(
    world: &mut ecs::World,
    bridge: &B,
    entity_id: EntityId,
    component_type: ComponentTypeId,
    path: &InspectorPath,
    value: InspectorEditValue,
) -> Result<(), InspectorEditError>
where
    B: EcsInspectorBridge,
{
    let entity = bridge
        .resolve_entity(entity_id)
        .ok_or(InspectorEditError::TargetNotFound)?;

    let rust_type_id = bridge
        .resolve_component_rust_type_id(component_type)
        .ok_or(InspectorEditError::TypeNotRegistered)?;

    let reflected = world
        .reflected_component_value_mut(entity, rust_type_id)
        .ok_or(InspectorEditError::ValueNotAvailable)?;

    set_value_at_path(reflected, path.segments(), &value)
}

pub fn set_resource_field_value<B>(
    world: &mut ecs::World,
    bridge: &B,
    resource_type: ResourceTypeId,
    path: &InspectorPath,
    value: InspectorEditValue,
) -> Result<(), InspectorEditError>
where
    B: EcsInspectorBridge,
{
    let rust_type_id = bridge
        .resolve_resource_rust_type_id(resource_type)
        .ok_or(InspectorEditError::TypeNotRegistered)?;

    let reflected = world
        .reflected_resource_value_mut(rust_type_id)
        .ok_or(InspectorEditError::ValueNotAvailable)?;

    set_value_at_path(reflected, path.segments(), &value)
}

fn set_value_at_path(
    current: ReflectValueMut<'_>,
    segments: &[InspectorPathSegment],
    value: &InspectorEditValue,
) -> Result<(), InspectorEditError> {
    match segments.split_first() {
        None => apply_primitive_edit(current, value),
        Some((segment, rest)) => match segment {
            InspectorPathSegment::Field(name) => {
                let mut struct_mut = current
                    .struct_mut()
                    .ok_or(InspectorEditError::InvalidPath)?;

                let child = struct_mut
                    .field_mut(name)
                    .ok_or(InspectorEditError::InvalidPath)?;

                set_value_at_path(child, rest, value)
            }
            InspectorPathSegment::Index(_) => Err(InspectorEditError::UnsupportedPathSegment),
        },
    }
}

fn apply_primitive_edit(
    mut current: ReflectValueMut<'_>,
    value: &InspectorEditValue,
) -> Result<(), InspectorEditError> {
    let actual_type = current.type_info().stable_name.to_string();

    match value {
        InspectorEditValue::Bool(next) => {
            let slot = current.downcast_mut::<bool>().ok_or_else(|| {
                InspectorEditError::UnsupportedValueType {
                    actual_type: actual_type.clone(),
                }
            })?;
            *slot = *next;
            Ok(())
        }
        InspectorEditValue::Integer(next) => {
            if let Some(slot) = current.downcast_mut::<i8>() {
                *slot = i8::try_from(*next).map_err(|_| InspectorEditError::IntegerOutOfRange {
                    target_type: "i8",
                    value: *next,
                })?;
                return Ok(());
            }
            if let Some(slot) = current.downcast_mut::<i16>() {
                *slot =
                    i16::try_from(*next).map_err(|_| InspectorEditError::IntegerOutOfRange {
                        target_type: "i16",
                        value: *next,
                    })?;
                return Ok(());
            }
            if let Some(slot) = current.downcast_mut::<i32>() {
                *slot =
                    i32::try_from(*next).map_err(|_| InspectorEditError::IntegerOutOfRange {
                        target_type: "i32",
                        value: *next,
                    })?;
                return Ok(());
            }
            if let Some(slot) = current.downcast_mut::<i64>() {
                *slot = *next;
                return Ok(());
            }
            if let Some(slot) = current.downcast_mut::<isize>() {
                *slot =
                    isize::try_from(*next).map_err(|_| InspectorEditError::IntegerOutOfRange {
                        target_type: "isize",
                        value: *next,
                    })?;
                return Ok(());
            }
            if let Some(slot) = current.downcast_mut::<u8>() {
                *slot = u8::try_from(*next).map_err(|_| InspectorEditError::IntegerOutOfRange {
                    target_type: "u8",
                    value: *next,
                })?;
                return Ok(());
            }
            if let Some(slot) = current.downcast_mut::<u16>() {
                *slot =
                    u16::try_from(*next).map_err(|_| InspectorEditError::IntegerOutOfRange {
                        target_type: "u16",
                        value: *next,
                    })?;
                return Ok(());
            }
            if let Some(slot) = current.downcast_mut::<u32>() {
                *slot =
                    u32::try_from(*next).map_err(|_| InspectorEditError::IntegerOutOfRange {
                        target_type: "u32",
                        value: *next,
                    })?;
                return Ok(());
            }
            if let Some(slot) = current.downcast_mut::<u64>() {
                *slot =
                    u64::try_from(*next).map_err(|_| InspectorEditError::IntegerOutOfRange {
                        target_type: "u64",
                        value: *next,
                    })?;
                return Ok(());
            }
            if let Some(slot) = current.downcast_mut::<usize>() {
                *slot =
                    usize::try_from(*next).map_err(|_| InspectorEditError::IntegerOutOfRange {
                        target_type: "usize",
                        value: *next,
                    })?;
                return Ok(());
            }

            Err(InspectorEditError::UnsupportedValueType { actual_type })
        }
        InspectorEditValue::Float(next) => {
            if let Some(slot) = current.downcast_mut::<f32>() {
                if !next.is_finite() || *next < f32::MIN as f64 || *next > f32::MAX as f64 {
                    return Err(InspectorEditError::FloatOutOfRange {
                        target_type: "f32",
                        value: *next,
                    });
                }
                *slot = *next as f32;
                return Ok(());
            }
            if let Some(slot) = current.downcast_mut::<f64>() {
                *slot = *next;
                return Ok(());
            }

            Err(InspectorEditError::UnsupportedValueType { actual_type })
        }
        InspectorEditValue::Text(next) => {
            let slot = current.downcast_mut::<String>().ok_or_else(|| {
                InspectorEditError::UnsupportedValueType {
                    actual_type: actual_type.clone(),
                }
            })?;
            *slot = next.clone();
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InspectorPath, StaticEcsInspectorBridge};

    #[derive(Debug, Clone, ecs::Reflect)]
    struct Vec2 {
        x: f32,
        y: f32,
    }

    #[derive(Debug, Clone, ecs::Component, ecs::ReflectComponent)]
    struct Position {
        value: Vec2,
        speed: f32,
        label: String,
    }

    #[derive(Debug, Clone, ecs::Resource, ecs::ReflectResource)]
    struct CameraSettings {
        zoom: f32,
        exposure: f32,
    }

    #[test]
    fn updates_nested_component_float_field_by_path() {
        let mut world = ecs::World::new();
        world.register_component_type::<Position>();

        let entity = world.spawn(Position {
            value: Vec2 { x: 1.0, y: 2.0 },
            speed: 3.5,
            label: "Player".to_string(),
        });

        let bridge = StaticEcsInspectorBridge::new()
            .with_entity(EntityId(1), entity)
            .with_component_type::<Position>(ComponentTypeId(10));

        let path = InspectorPath::root().child_field("value").child_field("x");

        set_component_field_value(
            &mut world,
            &bridge,
            EntityId(1),
            ComponentTypeId(10),
            &path,
            InspectorEditValue::Float(9.25),
        )
        .expect("component field edit should succeed");

        let position = world
            .get::<Position>(entity)
            .expect("component should exist");
        assert_eq!(position.value.x, 9.25);
    }

    #[test]
    fn updates_component_string_field_by_path() {
        let mut world = ecs::World::new();
        world.register_component_type::<Position>();

        let entity = world.spawn(Position {
            value: Vec2 { x: 1.0, y: 2.0 },
            speed: 3.5,
            label: "Player".to_string(),
        });

        let bridge = StaticEcsInspectorBridge::new()
            .with_entity(EntityId(1), entity)
            .with_component_type::<Position>(ComponentTypeId(10));

        let path = InspectorPath::root().child_field("label");

        set_component_field_value(
            &mut world,
            &bridge,
            EntityId(1),
            ComponentTypeId(10),
            &path,
            InspectorEditValue::Text("Enemy".to_string()),
        )
        .expect("component field edit should succeed");

        let position = world
            .get::<Position>(entity)
            .expect("component should exist");
        assert_eq!(position.label, "Enemy");
    }

    #[test]
    fn updates_resource_float_field_by_path() {
        let mut world = ecs::World::new();
        world.register_resource_type::<CameraSettings>();
        world.insert_registered_resource(CameraSettings {
            zoom: 2.0,
            exposure: 1.25,
        });

        let bridge = StaticEcsInspectorBridge::new()
            .with_resource_type::<CameraSettings>(ResourceTypeId(20));

        let path = InspectorPath::root().child_field("exposure");

        set_resource_field_value(
            &mut world,
            &bridge,
            ResourceTypeId(20),
            &path,
            InspectorEditValue::Float(2.5),
        )
        .expect("resource field edit should succeed");

        let settings = world
            .resource::<CameraSettings>()
            .expect("resource should exist");
        assert_eq!(settings.exposure, 2.5);
    }

    #[test]
    fn rejects_invalid_component_path() {
        let mut world = ecs::World::new();
        world.register_component_type::<Position>();

        let entity = world.spawn(Position {
            value: Vec2 { x: 1.0, y: 2.0 },
            speed: 3.5,
            label: "Player".to_string(),
        });

        let bridge = StaticEcsInspectorBridge::new()
            .with_entity(EntityId(1), entity)
            .with_component_type::<Position>(ComponentTypeId(10));

        let path = InspectorPath::root().child_field("missing_field");

        let error = set_component_field_value(
            &mut world,
            &bridge,
            EntityId(1),
            ComponentTypeId(10),
            &path,
            InspectorEditValue::Float(9.25),
        )
        .expect_err("invalid path should fail");

        assert_eq!(error, InspectorEditError::InvalidPath);
    }
}
