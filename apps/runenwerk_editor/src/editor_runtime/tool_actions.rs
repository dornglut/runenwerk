use editor_core::{ComponentTypeId, EditorMutationError, EntityId};
use editor_inspector::{InspectorEditValue, InspectorPath};
use scene::LocalTransform;

use crate::editor_runtime::{RunenwerkEditorRuntime, ratify_scene_transaction};

pub fn commit_translation_preview_into_local_transform(
    runtime: &mut RunenwerkEditorRuntime,
    entity: EntityId,
    delta: scene::Vec3Value,
) -> Result<(), EditorMutationError> {
    let local_transform_type = find_registered_component_type::<LocalTransform>(runtime)
        .ok_or(EditorMutationError::runtime_rejected(
            "LocalTransform is not registered in editor runtime",
        ))?;

    let ecs_entity = runtime
        .ids()
        .resolve_entity(entity)
        .ok_or(EditorMutationError::runtime_rejected(
            "editor entity is not registered",
        ))?;

    let current = runtime
        .world()
        .get::<LocalTransform>(ecs_entity)
        .ok_or(EditorMutationError::runtime_rejected(
            "entity does not have LocalTransform",
        ))?;

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
