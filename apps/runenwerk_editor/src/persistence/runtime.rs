use std::collections::BTreeSet;

use asset::{AssetId, AssetSourceId, AssetSourceRevisionId};
use editor_core::{EditorMutationError, EntityId};
use editor_persistence::{
    FormedScenePackageV2, SceneEntityRecordV2, SceneFileV2, SceneMaterialAssignmentsRecord,
    SceneMaterialSlotRecord, SceneMaterialSourceRefRecord, ScenePrimitiveKind,
    ScenePrimitiveRecord, SceneTransformRecord, SdfPrimitiveMaterialSlotAssignmentRecord,
};
use editor_scene::{
    SceneEntitySnapshot, SceneMaterialAssignmentState, SceneMaterialPalette,
    SceneMaterialPaletteEntryId, SceneMaterialSlot, SceneMaterialSlotId, SceneMaterialSourceRef,
    SceneRuntime, SdfPrimitiveMaterialSlotAssignment, SdfPrimitiveSourceId,
};
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

    SceneFileV2::new(entities).with_material_assignments(scene_material_assignments_record(
        runtime.scene_material_assignments(),
    ))
}

pub fn apply_scene_file_to_runtime(
    runtime: &mut RunenwerkEditorRuntime,
    scene_file: &SceneFileV2,
) -> Result<(), EditorMutationError> {
    apply_scene_entities_to_runtime(runtime, &scene_file.entities)?;
    apply_scene_material_assignments_to_runtime(runtime, &scene_file.material_assignments)
}

pub fn apply_formed_scene_to_runtime(
    runtime: &mut RunenwerkEditorRuntime,
    formed_scene: &FormedScenePackageV2,
) -> Result<(), EditorMutationError> {
    apply_scene_entities_to_runtime(runtime, formed_scene.entities())?;
    apply_scene_material_assignments_to_runtime(runtime, formed_scene.material_assignments())
}

fn scene_material_assignments_record(
    state: &SceneMaterialAssignmentState,
) -> SceneMaterialAssignmentsRecord {
    let mut record = SceneMaterialAssignmentsRecord {
        source_revision: state.source_revision(),
        palette_slots: state
            .palette()
            .slots
            .iter()
            .map(scene_material_slot_record)
            .collect(),
        sdf_primitive_assignments: state
            .assignments()
            .map(|assignment| {
                SdfPrimitiveMaterialSlotAssignmentRecord::new(
                    assignment.primitive.entity_id().0,
                    assignment.slot_id.raw(),
                )
            })
            .collect(),
    };
    record.sort_stable();
    record
}

fn scene_material_slot_record(slot: &SceneMaterialSlot) -> SceneMaterialSlotRecord {
    SceneMaterialSlotRecord {
        slot_id: slot.slot_id.raw(),
        palette_entry_id: slot.palette_entry_id.raw(),
        display_name: slot.display_name.clone(),
        source_ref: slot
            .source_ref
            .as_ref()
            .map(scene_material_source_ref_record),
        material_asset_id: slot.material_asset_id.map(|value| value.raw()),
        is_default: slot.is_default,
    }
}

fn scene_material_source_ref_record(
    source_ref: &SceneMaterialSourceRef,
) -> SceneMaterialSourceRefRecord {
    SceneMaterialSourceRefRecord {
        asset_id: source_ref.asset_id.raw(),
        source_id: source_ref.source_id.raw(),
        source_revision_id: source_ref.source_revision_id.map(|value| value.raw()),
        source_revision: source_ref.source_revision.clone(),
    }
}

fn apply_scene_material_assignments_to_runtime(
    runtime: &mut RunenwerkEditorRuntime,
    record: &SceneMaterialAssignmentsRecord,
) -> Result<(), EditorMutationError> {
    let state = scene_material_assignments_from_record(record)?;
    runtime.replace_scene_material_assignments(state);
    Ok(())
}

fn scene_material_assignments_from_record(
    record: &SceneMaterialAssignmentsRecord,
) -> Result<SceneMaterialAssignmentState, EditorMutationError> {
    let palette = SceneMaterialPalette::new(
        record
            .palette_slots
            .iter()
            .map(scene_material_slot_from_record)
            .collect::<Result<Vec<_>, _>>()?,
    )
    .map_err(|_| EditorMutationError::runtime_rejected("invalid scene material palette"))?;
    let assignments = record
        .sdf_primitive_assignments
        .iter()
        .map(|assignment| {
            SdfPrimitiveMaterialSlotAssignment::new(
                SdfPrimitiveSourceId::new(EntityId(assignment.sdf_primitive_entity_id)),
                SceneMaterialSlotId::new(assignment.slot_id),
            )
        })
        .collect::<Vec<_>>();
    SceneMaterialAssignmentState::new(palette, assignments)
        .map(|state| state.with_source_revision(record.source_revision))
        .map_err(|_| EditorMutationError::runtime_rejected("invalid scene material assignment"))
}

fn scene_material_slot_from_record(
    record: &SceneMaterialSlotRecord,
) -> Result<SceneMaterialSlot, EditorMutationError> {
    let mut slot = SceneMaterialSlot::new(
        SceneMaterialSlotId::new(record.slot_id),
        record.display_name.clone(),
    )
    .with_palette_entry_id(SceneMaterialPaletteEntryId::new(record.palette_entry_id))
    .with_default(record.is_default);

    if let Some(source_ref) = &record.source_ref {
        slot = slot.with_source_ref(scene_material_source_ref_from_record(source_ref)?);
    }
    if let Some(asset_id) = record.material_asset_id {
        slot = slot.with_material_asset(required_asset_id(asset_id)?);
    }

    Ok(slot)
}

fn scene_material_source_ref_from_record(
    record: &SceneMaterialSourceRefRecord,
) -> Result<SceneMaterialSourceRef, EditorMutationError> {
    let mut source_ref = SceneMaterialSourceRef::new(
        required_asset_id(record.asset_id)?,
        required_source_id(record.source_id)?,
    );
    if let Some(revision_id) = record.source_revision_id {
        source_ref = source_ref.with_source_revision_id(required_source_revision_id(revision_id)?);
    }
    if let Some(revision) = &record.source_revision {
        source_ref = source_ref.with_source_revision(revision.clone());
    }
    Ok(source_ref)
}

fn required_asset_id(raw: u64) -> Result<AssetId, EditorMutationError> {
    AssetId::try_from_raw(raw)
        .map_err(|_| EditorMutationError::runtime_rejected("invalid asset id"))
}

fn required_source_id(raw: u64) -> Result<AssetSourceId, EditorMutationError> {
    AssetSourceId::try_from_raw(raw)
        .map_err(|_| EditorMutationError::runtime_rejected("invalid asset source id"))
}

fn required_source_revision_id(raw: u64) -> Result<AssetSourceRevisionId, EditorMutationError> {
    AssetSourceRevisionId::try_from_raw(raw)
        .map_err(|_| EditorMutationError::runtime_rejected("invalid asset source revision id"))
}

fn apply_scene_entities_to_runtime(
    runtime: &mut RunenwerkEditorRuntime,
    entities: &[SceneEntityRecordV2],
) -> Result<(), EditorMutationError> {
    if runtime.document().entity_ids().next().is_some() {
        return Err(EditorMutationError::runtime_rejected(
            "scene import requires an empty runtime document",
        ));
    }

    let mut pending = entities.to_vec();
    pending.sort_by_key(|entity| entity.id);

    let mut restored = BTreeSet::new();

    while !pending.is_empty() {
        let mut next_pending = Vec::new();
        let mut progressed = false;

        for entity in pending {
            if let Some(parent) = entity.parent
                && !restored.contains(&parent)
            {
                next_pending.push(entity);
                continue;
            }

            runtime
                .scene_runtime()
                .restore_entity(SceneEntitySnapshot::new(
                    EntityId(entity.id),
                    entity.display_name,
                    entity.parent.map(EntityId),
                ))
                .map_err(|error| EditorMutationError::runtime_rejected(error.message))?;
            restored.insert(entity.id);
            progressed = true;
        }

        if !progressed {
            return Err(EditorMutationError::runtime_rejected(
                "scene file has missing or cyclic parent references",
            ));
        }

        pending = next_pending;
    }

    for entity in entities {
        let editor_entity = EntityId(entity.id);
        if !runtime.entity_has_component(editor_entity, LOCAL_TRANSFORM_COMPONENT_TYPE_ID) {
            runtime
                .scene_runtime()
                .add_component(editor_entity, LOCAL_TRANSFORM_COMPONENT_TYPE_ID)
                .map_err(|error| EditorMutationError::runtime_rejected(error.message))?;
        }
        if !runtime.entity_has_component(editor_entity, EDITOR_PRIMITIVE_COMPONENT_TYPE_ID) {
            runtime
                .scene_runtime()
                .add_component(editor_entity, EDITOR_PRIMITIVE_COMPONENT_TYPE_ID)
                .map_err(|error| EditorMutationError::runtime_rejected(error.message))?;
        }

        runtime.insert_component_for_editor_entity(
            editor_entity,
            local_transform_from_record(entity.transform),
        )?;
        runtime.insert_component_for_editor_entity(
            editor_entity,
            editor_primitive_from_record(entity.primitive),
        )?;
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
        EditorPrimitiveKind::Cylinder => ScenePrimitiveKind::Cylinder,
        EditorPrimitiveKind::Torus => ScenePrimitiveKind::Torus,
        EditorPrimitiveKind::Plane => ScenePrimitiveKind::Plane,
    }
}

fn scene_kind_to_editor(kind: ScenePrimitiveKind) -> EditorPrimitiveKind {
    match kind {
        ScenePrimitiveKind::Box => EditorPrimitiveKind::Box,
        ScenePrimitiveKind::Sphere => EditorPrimitiveKind::Sphere,
        ScenePrimitiveKind::Capsule => EditorPrimitiveKind::Capsule,
        ScenePrimitiveKind::Cylinder => EditorPrimitiveKind::Cylinder,
        ScenePrimitiveKind::Torus => EditorPrimitiveKind::Torus,
        ScenePrimitiveKind::Plane => EditorPrimitiveKind::Plane,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor_app::RunenwerkEditorApp;
    use crate::editor_runtime::{bootstrap_mvp_scene_if_empty, register_mvp_component_types};
    use editor_scene::{
        SceneMaterialPalette, SceneMaterialSlot, SceneMaterialSlotId,
        SdfPrimitiveMaterialSlotAssignment, SdfPrimitiveSourceId,
    };

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
        source_app
            .runtime_mut()
            .insert_component_for_editor_entity(
                entity,
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
            .insert_component_for_editor_entity(entity, primitive)
            .expect("primitive insert should succeed");

        let scene_file = scene_file_from_runtime(source_app.runtime());
        assert_eq!(
            scene_file.version,
            editor_persistence::SCENE_FILE_VERSION_V2
        );
        assert_eq!(scene_file.entities.len(), 2);
        assert!(
            scene_file
                .entities
                .iter()
                .any(|record| record.display_name == "Ground Plane"
                    && record.primitive.kind == ScenePrimitiveKind::Plane),
            "MVP ground plane should persist as a normal primitive entity"
        );

        let mut restored = RunenwerkEditorRuntime::new();
        register_mvp_component_types(&mut restored);
        apply_scene_file_to_runtime(&mut restored, &scene_file)
            .expect("scene apply should succeed");

        let restored_entity = entity;
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

    #[test]
    fn sdf_material_assignments_persist_through_scene_file_roundtrip() {
        let mut source_app = RunenwerkEditorApp::new();
        register_mvp_component_types(source_app.runtime_mut());
        bootstrap_mvp_scene_if_empty(source_app.runtime_mut()).expect("bootstrap should succeed");

        let entity = source_app
            .runtime()
            .document()
            .entity_ids()
            .next()
            .expect("seeded runtime should contain an SDF primitive entity");
        let assigned_slot = SceneMaterialSlotId::new(2);
        let palette = SceneMaterialPalette::new([
            SceneMaterialSlot::default_generated(),
            SceneMaterialSlot::new(assigned_slot, "Assigned"),
        ])
        .expect("valid palette");
        let assignments = SceneMaterialAssignmentState::new(
            palette,
            [SdfPrimitiveMaterialSlotAssignment::new(
                SdfPrimitiveSourceId::new(entity),
                assigned_slot,
            )],
        )
        .expect("valid assignments");
        source_app
            .runtime_mut()
            .replace_scene_material_assignments(assignments);

        let scene_file = scene_file_from_runtime(source_app.runtime());
        assert_eq!(
            scene_file.material_assignments.sdf_primitive_assignments,
            vec![SdfPrimitiveMaterialSlotAssignmentRecord::new(entity.0, 2)]
        );

        let mut restored = RunenwerkEditorRuntime::new();
        register_mvp_component_types(&mut restored);
        apply_scene_file_to_runtime(&mut restored, &scene_file)
            .expect("scene apply should succeed");

        let resolution = restored.material_slot_index_for_entity(entity);
        assert_eq!(resolution, 1);
    }

    #[test]
    fn material_assignment_resolution_ignores_derived_cache_hints() {
        let entity = EntityId(77);
        let assigned_slot = SceneMaterialSlotId::new(2);
        let record = SceneMaterialAssignmentsRecord::new(
            [
                SceneMaterialSlotRecord::default_generated(),
                SceneMaterialSlotRecord {
                    slot_id: assigned_slot.raw(),
                    palette_entry_id: assigned_slot.raw(),
                    display_name: "Authored".to_string(),
                    source_ref: Some(SceneMaterialSourceRefRecord::new(7, 9)),
                    material_asset_id: Some(7),
                    is_default: false,
                },
            ],
            [SdfPrimitiveMaterialSlotAssignmentRecord::new(
                entity.0,
                assigned_slot.raw(),
            )],
        );

        let state = scene_material_assignments_from_record(&record)
            .expect("authored assignments should restore");

        let resolution =
            state.resolve_material_slot_for_sdf_primitive(SdfPrimitiveSourceId::new(entity));
        assert_eq!(resolution.resolved_slot_id, assigned_slot);
        assert_eq!(resolution.material_table_index, 1);
        assert!(
            !state.material_table_identity().contains("artifact")
                && !state.material_table_identity().contains("cache"),
            "authored assignment resolution must not depend on generated artifact/cache hints"
        );
    }
}
