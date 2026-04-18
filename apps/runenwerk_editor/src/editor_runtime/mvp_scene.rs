use std::any::TypeId;

use editor_core::{ComponentTypeId, EditorMutationError};
use editor_scene::SceneCommandIntent;
use scene::{LocalTransform, Vec3Value};

use crate::editor_runtime::{execute_scene_intent, RunenwerkEditorRuntime};

pub const LOCAL_TRANSFORM_COMPONENT_TYPE_ID: ComponentTypeId = ComponentTypeId(500);
pub const EDITOR_PRIMITIVE_COMPONENT_TYPE_ID: ComponentTypeId = ComponentTypeId(501);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorPrimitiveKind {
    Box,
    Sphere,
    Capsule,
}

impl Default for EditorPrimitiveKind {
    fn default() -> Self {
        Self::Box
    }
}

impl EditorPrimitiveKind {
    pub fn as_u32(self) -> u32 {
        match self {
            Self::Box => 0,
            Self::Sphere => 1,
            Self::Capsule => 2,
        }
    }

    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => Self::Sphere,
            2 => Self::Capsule,
            _ => Self::Box,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, ecs::Component, ecs::ReflectComponent)]
pub struct EditorPrimitive {
    pub primitive_kind: u32,
    pub box_half_extents: Vec3Value,
    pub sphere_radius: f32,
    pub capsule_radius: f32,
    pub capsule_half_height: f32,
}

impl EditorPrimitive {
    pub fn kind(self) -> EditorPrimitiveKind {
        EditorPrimitiveKind::from_u32(self.primitive_kind)
    }

    pub fn set_kind(&mut self, kind: EditorPrimitiveKind) {
        self.primitive_kind = kind.as_u32();
    }

    pub fn box_extents_for_picking(self) -> Vec3Value {
        match self.kind() {
            EditorPrimitiveKind::Box => self.box_half_extents,
            EditorPrimitiveKind::Sphere => Vec3Value::new(
                self.sphere_radius.max(0.05),
                self.sphere_radius.max(0.05),
                self.sphere_radius.max(0.05),
            ),
            EditorPrimitiveKind::Capsule => Vec3Value::new(
                self.capsule_radius.max(0.05),
                self.capsule_radius.max(0.05) + self.capsule_half_height.max(0.05),
                self.capsule_radius.max(0.05),
            ),
        }
    }
}

impl Default for EditorPrimitive {
    fn default() -> Self {
        Self {
            primitive_kind: EditorPrimitiveKind::Box.as_u32(),
            box_half_extents: Vec3Value::new(0.5, 0.5, 0.5),
            sphere_radius: 0.6,
            capsule_radius: 0.35,
            capsule_half_height: 0.75,
        }
    }
}

pub fn register_mvp_component_types(runtime: &mut RunenwerkEditorRuntime) {
    if runtime
        .ids()
        .resolve_component_rust_type_id(LOCAL_TRANSFORM_COMPONENT_TYPE_ID)
        .is_none()
    {
        runtime.register_component_type::<LocalTransform>(LOCAL_TRANSFORM_COMPONENT_TYPE_ID);
    }

    if runtime
        .ids()
        .resolve_component_rust_type_id(EDITOR_PRIMITIVE_COMPONENT_TYPE_ID)
        .is_none()
    {
        runtime.register_component_type::<EditorPrimitive>(EDITOR_PRIMITIVE_COMPONENT_TYPE_ID);
    }
}

pub fn bootstrap_mvp_scene_if_empty(
    runtime: &mut RunenwerkEditorRuntime,
) -> Result<(), EditorMutationError> {
    if runtime.document().entity_ids().next().is_some() {
        return Ok(());
    }

    let create_command_id = runtime.allocate_command_id();
    execute_scene_intent(
        runtime,
        create_command_id,
        SceneCommandIntent::CreateEntity {
            parent: None,
            display_name: "Graybox".to_string(),
        },
    )?;

    let entity =
        runtime
            .document()
            .entity_ids()
            .next()
            .ok_or(EditorMutationError::runtime_rejected(
                "expected created entity in document",
            ))?;

    let add_transform_command_id = runtime.allocate_command_id();
    execute_scene_intent(
        runtime,
        add_transform_command_id,
        SceneCommandIntent::AddComponent {
            entity,
            component_type: LOCAL_TRANSFORM_COMPONENT_TYPE_ID,
        },
    )?;

    let add_primitive_command_id = runtime.allocate_command_id();
    execute_scene_intent(
        runtime,
        add_primitive_command_id,
        SceneCommandIntent::AddComponent {
            entity,
            component_type: EDITOR_PRIMITIVE_COMPONENT_TYPE_ID,
        },
    )?;

    Ok(())
}

pub fn is_local_transform_component(
    runtime: &RunenwerkEditorRuntime,
    component_type: ComponentTypeId,
) -> bool {
    runtime.ids().resolve_component_rust_type_id(component_type)
        == Some(TypeId::of::<LocalTransform>())
}
