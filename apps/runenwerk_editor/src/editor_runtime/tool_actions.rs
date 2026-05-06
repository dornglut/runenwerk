use editor_core::{ComponentTypeId, EditorMutationError, EntityId};
use editor_inspector::{InspectorEditValue, InspectorPath};
use scene::{LocalTransform, QuatValue, Vec3Value};

use crate::editor_runtime::{
    RunenwerkEditorRuntime, TransformPreviewSession, TransformToolKind, ratify_scene_transaction,
};

pub fn commit_transform_preview_into_local_transform(
    runtime: &mut RunenwerkEditorRuntime,
    preview: &TransformPreviewSession,
) -> Result<(), EditorMutationError> {
    match preview.tool {
        TransformToolKind::Translate => commit_translation_preview_into_local_transform(
            runtime,
            preview.entity,
            preview.translation_delta,
        ),
        TransformToolKind::Rotate => commit_rotation_preview_into_local_transform(
            runtime,
            preview.entity,
            preview.rotation_delta_radians,
        ),
        TransformToolKind::Scale => {
            commit_scale_preview_into_local_transform(runtime, preview.entity, preview.scale_delta)
        }
    }
}

pub fn commit_translation_preview_into_local_transform(
    runtime: &mut RunenwerkEditorRuntime,
    entity: EntityId,
    delta: scene::Vec3Value,
) -> Result<(), EditorMutationError> {
    let local_transform_type = find_registered_component_type::<LocalTransform>(runtime).ok_or(
        EditorMutationError::runtime_rejected("LocalTransform is not registered in editor runtime"),
    )?;

    let ecs_entity =
        runtime
            .ids()
            .resolve_entity(entity)
            .ok_or(EditorMutationError::runtime_rejected(
                "editor entity is not registered",
            ))?;

    let current = runtime.world().get::<LocalTransform>(ecs_entity).ok_or(
        EditorMutationError::runtime_rejected("entity does not have LocalTransform"),
    )?;

    let next_x = current.translation.x + delta.x;
    let next_y = current.translation.y + delta.y;
    let next_z = current.translation.z + delta.z;

    let command_x = runtime.allocate_command_id();
    let command_y = runtime.allocate_command_id();
    let command_z = runtime.allocate_command_id();

    let mut commands = vec![
        editor_scene::scene_intent_to_command(
            command_x,
            editor_scene::SceneCommandIntent::EditComponentField {
                entity,
                component_type: local_transform_type,
                path: InspectorPath::root()
                    .child_field("translation")
                    .child_field("x"),
                value: InspectorEditValue::Float(next_x as f64),
            },
        ),
        editor_scene::scene_intent_to_command(
            command_y,
            editor_scene::SceneCommandIntent::EditComponentField {
                entity,
                component_type: local_transform_type,
                path: InspectorPath::root()
                    .child_field("translation")
                    .child_field("y"),
                value: InspectorEditValue::Float(next_y as f64),
            },
        ),
        editor_scene::scene_intent_to_command(
            command_z,
            editor_scene::SceneCommandIntent::EditComponentField {
                entity,
                component_type: local_transform_type,
                path: InspectorPath::root()
                    .child_field("translation")
                    .child_field("z"),
                value: InspectorEditValue::Float(next_z as f64),
            },
        ),
    ];

    let _ = ratify_scene_transaction(
        runtime,
        "Apply Translation Preview",
        &mut commands,
        editor_core::ChangeOrigin::ToolInteraction,
    )
    .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))?;

    Ok(())
}

pub fn commit_rotation_preview_into_local_transform(
    runtime: &mut RunenwerkEditorRuntime,
    entity: EntityId,
    delta_radians: Vec3Value,
) -> Result<(), EditorMutationError> {
    let current = local_transform(runtime, entity)?;
    let delta = glam::Quat::from_rotation_x(delta_radians.x)
        * glam::Quat::from_rotation_y(delta_radians.y)
        * glam::Quat::from_rotation_z(delta_radians.z);
    let next = (delta * current.rotation.to_glam()).normalize();
    commit_local_transform_fields(
        runtime,
        "Apply Rotation Preview",
        entity,
        current.translation,
        QuatValue::from_glam(next),
        current.scale,
    )
}

pub fn commit_scale_preview_into_local_transform(
    runtime: &mut RunenwerkEditorRuntime,
    entity: EntityId,
    delta: Vec3Value,
) -> Result<(), EditorMutationError> {
    let current = local_transform(runtime, entity)?;
    let next_scale = Vec3Value::new(
        (current.scale.x + delta.x).max(0.01),
        (current.scale.y + delta.y).max(0.01),
        (current.scale.z + delta.z).max(0.01),
    );
    commit_local_transform_fields(
        runtime,
        "Apply Scale Preview",
        entity,
        current.translation,
        current.rotation,
        next_scale,
    )
}

fn local_transform(
    runtime: &RunenwerkEditorRuntime,
    entity: EntityId,
) -> Result<LocalTransform, EditorMutationError> {
    let ecs_entity =
        runtime
            .ids()
            .resolve_entity(entity)
            .ok_or(EditorMutationError::runtime_rejected(
                "editor entity is not registered",
            ))?;

    runtime
        .world()
        .get::<LocalTransform>(ecs_entity)
        .copied()
        .ok_or(EditorMutationError::runtime_rejected(
            "entity does not have LocalTransform",
        ))
}

fn commit_local_transform_fields(
    runtime: &mut RunenwerkEditorRuntime,
    label: &'static str,
    entity: EntityId,
    translation: Vec3Value,
    rotation: QuatValue,
    scale: Vec3Value,
) -> Result<(), EditorMutationError> {
    let local_transform_type = find_registered_component_type::<LocalTransform>(runtime).ok_or(
        EditorMutationError::runtime_rejected("LocalTransform is not registered in editor runtime"),
    )?;

    let fields = [
        (
            InspectorPath::root()
                .child_field("translation")
                .child_field("x"),
            InspectorEditValue::Float(translation.x as f64),
        ),
        (
            InspectorPath::root()
                .child_field("translation")
                .child_field("y"),
            InspectorEditValue::Float(translation.y as f64),
        ),
        (
            InspectorPath::root()
                .child_field("translation")
                .child_field("z"),
            InspectorEditValue::Float(translation.z as f64),
        ),
        (
            InspectorPath::root()
                .child_field("rotation")
                .child_field("x"),
            InspectorEditValue::Float(rotation.x as f64),
        ),
        (
            InspectorPath::root()
                .child_field("rotation")
                .child_field("y"),
            InspectorEditValue::Float(rotation.y as f64),
        ),
        (
            InspectorPath::root()
                .child_field("rotation")
                .child_field("z"),
            InspectorEditValue::Float(rotation.z as f64),
        ),
        (
            InspectorPath::root()
                .child_field("rotation")
                .child_field("w"),
            InspectorEditValue::Float(rotation.w as f64),
        ),
        (
            InspectorPath::root().child_field("scale").child_field("x"),
            InspectorEditValue::Float(scale.x as f64),
        ),
        (
            InspectorPath::root().child_field("scale").child_field("y"),
            InspectorEditValue::Float(scale.y as f64),
        ),
        (
            InspectorPath::root().child_field("scale").child_field("z"),
            InspectorEditValue::Float(scale.z as f64),
        ),
    ];

    let mut commands = fields
        .into_iter()
        .map(|(path, value)| {
            editor_scene::scene_intent_to_command(
                runtime.allocate_command_id(),
                editor_scene::SceneCommandIntent::EditComponentField {
                    entity,
                    component_type: local_transform_type,
                    path,
                    value,
                },
            )
        })
        .collect::<Vec<_>>();

    let _ = ratify_scene_transaction(
        runtime,
        label,
        &mut commands,
        editor_core::ChangeOrigin::ToolInteraction,
    )
    .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))?;

    Ok(())
}

fn find_registered_component_type<T>(runtime: &RunenwerkEditorRuntime) -> Option<ComponentTypeId>
where
    T: 'static,
{
    let target = std::any::TypeId::of::<T>();

    runtime.ids().component_type_ids().find(|component_type| {
        runtime
            .ids()
            .resolve_component_rust_type_id(*component_type)
            == Some(target)
    })
}
