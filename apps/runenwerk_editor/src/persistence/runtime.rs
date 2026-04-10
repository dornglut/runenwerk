use std::collections::BTreeSet;

use editor_core::EntityId;
use editor_persistence::{
    SceneEntityRecordV2, SceneFileV2, ScenePrimitiveKind, ScenePrimitiveRecord,
    SceneTransformRecord,
};
use editor_scene::{SceneEntitySnapshot, SceneRuntime};
use scene::{LocalTransform, QuatValue, Vec3Value};

use crate::editor_runtime::{
    EDITOR_PRIMITIVE_COMPONENT_TYPE_ID, EditorPrimitive, EditorPrimitiveKind,
    LOCAL_TRANSFORM_COMPONENT_TYPE_ID, RunenwerkEditorRuntime,
};

pub fn scene_file_from_runtime(runtime: &RunenwerkEditorRuntime) -> SceneFileV2 {
    let entities = runtime
        .document()
        .entity_ids()
        .filter_map(|entity| {
            let snapshot = runtime.document().entity_snapshot(entity)?;
            let transform = entity_transform_record(runtime, entity).unwrap_or_default();
            let primitive = entity_primitive_record(runtime, entity).unwrap_or_default();

            Some(SceneEntityRecordV2::new(
                snapshot.id.0,
                snapshot.display_name,
                snapshot.parent.map(|parent| parent.0),
                transform,
                primitive,
            ))
        })
        .collect::<Vec<_>>();

    SceneFileV2::new(entities)
}

pub fn apply_scene_file_to_runtime(
    runtime: &mut RunenwerkEditorRuntime,
    scene_file: &SceneFileV2,
) -> Result<(), &'static str> {
    if runtime.document().entity_ids().next().is_some() {
        return Err("scene import requires an empty runtime document");
    }

    let mut pending = scene_file.entities.clone();
    pending.sort_by(|left, right| left.id.cmp(&right.id));

    let mut restored = BTreeSet::new();

    while !pending.is_empty() {
        let mut next_pending = Vec::new();
        let mut progressed = false;

        for entity in pending {
            if let Some(parent) = entity.parent {
                if !restored.contains(&parent) {
                    next_pending.push(entity);
                    continue;
                }
            }

            runtime
                .scene_runtime()
                .restore_entity(SceneEntitySnapshot::new(
                    EntityId(entity.id),
                    entity.display_name,
                    entity.parent.map(EntityId),
                ))?;
            restored.insert(entity.id);
            progressed = true;
        }

        if !progressed {
            return Err("scene file has missing or cyclic parent references");
        }

        pending = next_pending;
    }

    for entity in &scene_file.entities {
        let editor_entity = EntityId(entity.id);
        let ecs_entity = runtime
            .ids()
            .resolve_entity(editor_entity)
            .ok_or("scene file references unknown entity id")?;

        if !runtime.entity_has_component(editor_entity, LOCAL_TRANSFORM_COMPONENT_TYPE_ID) {
            runtime
                .scene_runtime()
                .add_component(editor_entity, LOCAL_TRANSFORM_COMPONENT_TYPE_ID)?;
        }
        if !runtime.entity_has_component(editor_entity, EDITOR_PRIMITIVE_COMPONENT_TYPE_ID) {
            runtime
                .scene_runtime()
                .add_component(editor_entity, EDITOR_PRIMITIVE_COMPONENT_TYPE_ID)?;
        }

        runtime
            .world_mut()
            .insert(ecs_entity, local_transform_from_record(entity.transform))
            .map_err(|_| "failed to restore local transform component")?;
        runtime
            .world_mut()
            .insert(ecs_entity, editor_primitive_from_record(entity.primitive))
            .map_err(|_| "failed to restore primitive component")?;
    }

    Ok(())
}

fn entity_transform_record(
    runtime: &RunenwerkEditorRuntime,
    entity: EntityId,
) -> Option<SceneTransformRecord> {
    let ecs_entity = runtime.ids().resolve_entity(entity)?;
    let transform = runtime.world().get::<LocalTransform>(ecs_entity).copied()?;
    Some(SceneTransformRecord {
        translation: [
            transform.translation.x,
            transform.translation.y,
            transform.translation.z,
        ],
        rotation: [
            transform.rotation.x,
            transform.rotation.y,
            transform.rotation.z,
            transform.rotation.w,
        ],
        scale: [transform.scale.x, transform.scale.y, transform.scale.z],
    })
}

fn entity_primitive_record(
    runtime: &RunenwerkEditorRuntime,
    entity: EntityId,
) -> Option<ScenePrimitiveRecord> {
    let ecs_entity = runtime.ids().resolve_entity(entity)?;
    let primitive = runtime
        .world()
        .get::<EditorPrimitive>(ecs_entity)
        .copied()?;
    Some(ScenePrimitiveRecord {
        kind: primitive_kind_to_scene(primitive.kind()),
        box_half_extents: [
            primitive.box_half_extents.x,
            primitive.box_half_extents.y,
            primitive.box_half_extents.z,
        ],
        sphere_radius: primitive.sphere_radius,
        capsule_radius: primitive.capsule_radius,
        capsule_half_height: primitive.capsule_half_height,
    })
}

fn local_transform_from_record(record: SceneTransformRecord) -> LocalTransform {
    LocalTransform {
        translation: Vec3Value::new(
            record.translation[0],
            record.translation[1],
            record.translation[2],
        ),
        rotation: QuatValue::new(
            record.rotation[0],
            record.rotation[1],
            record.rotation[2],
            record.rotation[3],
        ),
        scale: Vec3Value::new(record.scale[0], record.scale[1], record.scale[2]),
    }
}

fn editor_primitive_from_record(record: ScenePrimitiveRecord) -> EditorPrimitive {
    let mut primitive = EditorPrimitive {
        primitive_kind: 0,
        box_half_extents: Vec3Value::new(
            record.box_half_extents[0],
            record.box_half_extents[1],
            record.box_half_extents[2],
        ),
        sphere_radius: record.sphere_radius,
        capsule_radius: record.capsule_radius,
        capsule_half_height: record.capsule_half_height,
    };
    primitive.set_kind(scene_kind_to_editor(record.kind));
    primitive
}

fn primitive_kind_to_scene(kind: EditorPrimitiveKind) -> ScenePrimitiveKind {
    match kind {
        EditorPrimitiveKind::Box => ScenePrimitiveKind::Box,
        EditorPrimitiveKind::Sphere => ScenePrimitiveKind::Sphere,
        EditorPrimitiveKind::Capsule => ScenePrimitiveKind::Capsule,
    }
}

fn scene_kind_to_editor(kind: ScenePrimitiveKind) -> EditorPrimitiveKind {
    match kind {
        ScenePrimitiveKind::Box => EditorPrimitiveKind::Box,
        ScenePrimitiveKind::Sphere => EditorPrimitiveKind::Sphere,
        ScenePrimitiveKind::Capsule => EditorPrimitiveKind::Capsule,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor_app::RunenwerkEditorApp;
    use crate::editor_runtime::{bootstrap_mvp_scene_if_empty, register_mvp_component_types};

    #[test]
    fn scene_file_roundtrip_preserves_transform_and_primitive_payload() {
        let mut source_app = RunenwerkEditorApp::new();
        register_mvp_component_types(source_app.runtime_mut());
        bootstrap_mvp_scene_if_empty(source_app.runtime_mut()).expect("bootstrap should succeed");

        let entity = source_app
            .runtime()
            .document()
            .entity_ids()
            .next()
            .expect("seeded runtime should contain one entity");
        let ecs_entity = source_app
            .runtime()
            .ids()
            .resolve_entity(entity)
            .expect("entity mapping should exist");

        source_app
            .runtime_mut()
            .world_mut()
            .insert(
                ecs_entity,
                LocalTransform::new(
                    Vec3Value::new(3.0, 1.5, -2.0),
                    QuatValue::new(0.0, 0.0, 0.0, 1.0),
                    Vec3Value::new(1.0, 1.0, 1.0),
                ),
            )
            .expect("transform insert should succeed");

        let mut primitive = EditorPrimitive::default();
        primitive.set_kind(EditorPrimitiveKind::Capsule);
        primitive.capsule_radius = 0.45;
        primitive.capsule_half_height = 1.2;
        source_app
            .runtime_mut()
            .world_mut()
            .insert(ecs_entity, primitive)
            .expect("primitive insert should succeed");

        let scene_file = scene_file_from_runtime(source_app.runtime());
        assert_eq!(
            scene_file.version,
            editor_persistence::SCENE_FILE_VERSION_V2
        );
        assert_eq!(scene_file.entities.len(), 1);

        let mut restored = RunenwerkEditorRuntime::new();
        register_mvp_component_types(&mut restored);
        apply_scene_file_to_runtime(&mut restored, &scene_file)
            .expect("scene apply should succeed");

        let restored_entity = restored
            .document()
            .entity_ids()
            .next()
            .expect("restored runtime should contain entity");
        let restored_ecs = restored
            .ids()
            .resolve_entity(restored_entity)
            .expect("restored entity mapping should exist");

        let restored_transform = restored
            .world()
            .get::<LocalTransform>(restored_ecs)
            .expect("restored transform should exist");
        assert_eq!(
            restored_transform.translation,
            Vec3Value::new(3.0, 1.5, -2.0)
        );

        let restored_primitive = restored
            .world()
            .get::<EditorPrimitive>(restored_ecs)
            .expect("restored primitive should exist");
        assert_eq!(restored_primitive.kind(), EditorPrimitiveKind::Capsule);
        assert_eq!(restored_primitive.capsule_radius, 0.45);
        assert_eq!(restored_primitive.capsule_half_height, 1.2);
    }
}
