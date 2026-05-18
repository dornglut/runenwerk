//! File: domain/editor/editor_persistence/src/scene_normalization.rs
//! Purpose: Normalized-reality contracts for scene persistence.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    SCENE_FILE_VERSION_V2, SceneEntityRecordV2, SceneFileV2, SceneMaterialAssignmentsRecord,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneNormalizationError {
    UnsupportedVersion(u32),
    DuplicateEntityId(u64),
    MissingParent { entity_id: u64, parent_id: u64 },
    CyclicParentReference { entity_id: u64 },
    MissingDefaultMaterialSlot,
    DuplicateMaterialSlotId(u64),
    DuplicateMaterialPaletteEntryId(u64),
    DuplicateSdfMaterialAssignment(u64),
    MaterialAssignmentReferencesMissingEntity { entity_id: u64 },
    MaterialAssignmentReferencesMissingSlot { entity_id: u64, slot_id: u64 },
}

impl SceneNormalizationError {
    pub const fn as_static_str(&self) -> &'static str {
        match self {
            Self::UnsupportedVersion(_) => "scene normalization only supports v2 scene files",
            Self::DuplicateEntityId(_) => "scene normalization found duplicate entity ids",
            Self::MissingParent { .. } => "scene normalization found missing parent reference",
            Self::CyclicParentReference { .. } => {
                "scene normalization found cyclic parent relationship"
            }
            Self::MissingDefaultMaterialSlot => {
                "scene normalization found missing default material slot"
            }
            Self::DuplicateMaterialSlotId(_) => {
                "scene normalization found duplicate material slot ids"
            }
            Self::DuplicateMaterialPaletteEntryId(_) => {
                "scene normalization found duplicate material palette entry ids"
            }
            Self::DuplicateSdfMaterialAssignment(_) => {
                "scene normalization found duplicate SDF material assignments"
            }
            Self::MaterialAssignmentReferencesMissingEntity { .. } => {
                "scene normalization found material assignment for missing SDF primitive entity"
            }
            Self::MaterialAssignmentReferencesMissingSlot { .. } => {
                "scene normalization found material assignment for missing material slot"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedSceneFileV2 {
    entities: Vec<SceneEntityRecordV2>,
    material_assignments: SceneMaterialAssignmentsRecord,
}

impl NormalizedSceneFileV2 {
    pub fn entities(&self) -> &[SceneEntityRecordV2] {
        &self.entities
    }

    pub fn material_assignments(&self) -> &SceneMaterialAssignmentsRecord {
        &self.material_assignments
    }

    pub fn into_entities(self) -> Vec<SceneEntityRecordV2> {
        self.entities
    }

    pub fn into_scene_file(self) -> SceneFileV2 {
        SceneFileV2::new(self.entities).with_material_assignments(self.material_assignments)
    }
}

pub fn normalize_scene_file(
    mut scene_file: SceneFileV2,
) -> Result<NormalizedSceneFileV2, SceneNormalizationError> {
    if scene_file.version != SCENE_FILE_VERSION_V2 {
        return Err(SceneNormalizationError::UnsupportedVersion(
            scene_file.version,
        ));
    }

    scene_file.entities.sort_by_key(|entity| entity.id);

    for pair in scene_file.entities.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(SceneNormalizationError::DuplicateEntityId(pair[0].id));
        }
    }

    let parent_by_entity = scene_file
        .entities
        .iter()
        .map(|entity| (entity.id, entity.parent))
        .collect::<BTreeMap<_, _>>();

    for entity in &scene_file.entities {
        if let Some(parent) = entity.parent
            && !parent_by_entity.contains_key(&parent)
        {
            return Err(SceneNormalizationError::MissingParent {
                entity_id: entity.id,
                parent_id: parent,
            });
        }

        if has_parent_cycle(entity.id, &parent_by_entity) {
            return Err(SceneNormalizationError::CyclicParentReference {
                entity_id: entity.id,
            });
        }
    }

    let material_assignments =
        normalize_material_assignments(scene_file.material_assignments, &parent_by_entity)?;

    Ok(NormalizedSceneFileV2 {
        entities: scene_file.entities,
        material_assignments,
    })
}

fn has_parent_cycle(entity_id: u64, parent_by_entity: &BTreeMap<u64, Option<u64>>) -> bool {
    let mut visited = BTreeSet::new();
    let mut cursor = Some(entity_id);

    while let Some(current) = cursor {
        if !visited.insert(current) {
            return true;
        }
        cursor = parent_by_entity.get(&current).copied().flatten();
    }

    false
}

fn normalize_material_assignments(
    mut material_assignments: SceneMaterialAssignmentsRecord,
    parent_by_entity: &BTreeMap<u64, Option<u64>>,
) -> Result<SceneMaterialAssignmentsRecord, SceneNormalizationError> {
    material_assignments.sort_stable();

    let mut default_count = 0usize;
    let mut slot_ids = BTreeSet::new();
    let mut palette_entry_ids = BTreeSet::new();
    for slot in &material_assignments.palette_slots {
        if slot.is_default {
            default_count += 1;
        }
        if !slot_ids.insert(slot.slot_id) {
            return Err(SceneNormalizationError::DuplicateMaterialSlotId(
                slot.slot_id,
            ));
        }
        if !palette_entry_ids.insert(slot.palette_entry_id) {
            return Err(SceneNormalizationError::DuplicateMaterialPaletteEntryId(
                slot.palette_entry_id,
            ));
        }
    }
    if default_count != 1 {
        return Err(SceneNormalizationError::MissingDefaultMaterialSlot);
    }

    let mut assignments = BTreeSet::new();
    for assignment in &material_assignments.sdf_primitive_assignments {
        if !assignments.insert(assignment.sdf_primitive_entity_id) {
            return Err(SceneNormalizationError::DuplicateSdfMaterialAssignment(
                assignment.sdf_primitive_entity_id,
            ));
        }
        if !parent_by_entity.contains_key(&assignment.sdf_primitive_entity_id) {
            return Err(
                SceneNormalizationError::MaterialAssignmentReferencesMissingEntity {
                    entity_id: assignment.sdf_primitive_entity_id,
                },
            );
        }
        if !slot_ids.contains(&assignment.slot_id) {
            return Err(
                SceneNormalizationError::MaterialAssignmentReferencesMissingSlot {
                    entity_id: assignment.sdf_primitive_entity_id,
                    slot_id: assignment.slot_id,
                },
            );
        }
    }

    Ok(material_assignments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ScenePrimitiveRecord, SceneTransformRecord};

    #[test]
    fn normalize_scene_file_sorts_entities_by_identity() {
        let scene = SceneFileV2 {
            version: SCENE_FILE_VERSION_V2,
            entities: vec![
                SceneEntityRecordV2::new(
                    2,
                    "child",
                    Some(1),
                    SceneTransformRecord::default(),
                    ScenePrimitiveRecord::default(),
                ),
                SceneEntityRecordV2::new(
                    1,
                    "root",
                    None,
                    SceneTransformRecord::default(),
                    ScenePrimitiveRecord::default(),
                ),
            ],
            material_assignments: Default::default(),
        };

        let normalized = normalize_scene_file(scene).expect("normalization should succeed");
        assert_eq!(normalized.entities()[0].id, 1);
        assert_eq!(normalized.entities()[1].id, 2);
    }

    #[test]
    fn normalize_scene_file_rejects_duplicate_entity_ids() {
        let scene = SceneFileV2 {
            version: SCENE_FILE_VERSION_V2,
            entities: vec![
                SceneEntityRecordV2::new(
                    1,
                    "A",
                    None,
                    SceneTransformRecord::default(),
                    ScenePrimitiveRecord::default(),
                ),
                SceneEntityRecordV2::new(
                    1,
                    "B",
                    None,
                    SceneTransformRecord::default(),
                    ScenePrimitiveRecord::default(),
                ),
            ],
            material_assignments: Default::default(),
        };

        let error = normalize_scene_file(scene).expect_err("normalization should fail");
        assert!(matches!(
            error,
            SceneNormalizationError::DuplicateEntityId(1)
        ));
    }

    #[test]
    fn normalize_scene_file_rejects_missing_parent() {
        let scene = SceneFileV2 {
            version: SCENE_FILE_VERSION_V2,
            entities: vec![SceneEntityRecordV2::new(
                2,
                "child",
                Some(999),
                SceneTransformRecord::default(),
                ScenePrimitiveRecord::default(),
            )],
            material_assignments: Default::default(),
        };

        let error = normalize_scene_file(scene).expect_err("normalization should fail");
        assert!(matches!(
            error,
            SceneNormalizationError::MissingParent {
                entity_id: 2,
                parent_id: 999
            }
        ));
    }

    #[test]
    fn normalize_scene_file_rejects_cyclic_parent_reference() {
        let scene = SceneFileV2 {
            version: SCENE_FILE_VERSION_V2,
            entities: vec![
                SceneEntityRecordV2::new(
                    1,
                    "A",
                    Some(2),
                    SceneTransformRecord::default(),
                    ScenePrimitiveRecord::default(),
                ),
                SceneEntityRecordV2::new(
                    2,
                    "B",
                    Some(1),
                    SceneTransformRecord::default(),
                    ScenePrimitiveRecord::default(),
                ),
            ],
            material_assignments: Default::default(),
        };

        let error = normalize_scene_file(scene).expect_err("normalization should fail");
        assert!(matches!(
            error,
            SceneNormalizationError::CyclicParentReference { .. }
        ));
    }

    #[test]
    fn normalize_scene_file_validates_and_sorts_sdf_material_assignments() {
        let scene = SceneFileV2 {
            version: SCENE_FILE_VERSION_V2,
            entities: vec![
                SceneEntityRecordV2::new(
                    2,
                    "B",
                    None,
                    SceneTransformRecord::default(),
                    ScenePrimitiveRecord::default(),
                ),
                SceneEntityRecordV2::new(
                    1,
                    "A",
                    None,
                    SceneTransformRecord::default(),
                    ScenePrimitiveRecord::default(),
                ),
            ],
            material_assignments: SceneMaterialAssignmentsRecord::new(
                [
                    crate::SceneMaterialSlotRecord::default_generated(),
                    crate::SceneMaterialSlotRecord {
                        slot_id: 2,
                        palette_entry_id: 2,
                        display_name: "Red".to_string(),
                        source_ref: None,
                        material_asset_id: None,
                        is_default: false,
                    },
                ],
                [
                    crate::SdfPrimitiveMaterialSlotAssignmentRecord::new(2, 2),
                    crate::SdfPrimitiveMaterialSlotAssignmentRecord::new(1, 1),
                ],
            ),
        };

        let normalized = normalize_scene_file(scene).expect("normalization should succeed");

        assert_eq!(
            normalized
                .material_assignments()
                .sdf_primitive_assignments
                .iter()
                .map(|assignment| assignment.sdf_primitive_entity_id)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    #[test]
    fn normalize_scene_file_rejects_sdf_material_assignment_to_missing_entity() {
        let scene = SceneFileV2 {
            version: SCENE_FILE_VERSION_V2,
            entities: vec![SceneEntityRecordV2::new(
                1,
                "A",
                None,
                SceneTransformRecord::default(),
                ScenePrimitiveRecord::default(),
            )],
            material_assignments: SceneMaterialAssignmentsRecord::new(
                [crate::SceneMaterialSlotRecord::default_generated()],
                [crate::SdfPrimitiveMaterialSlotAssignmentRecord::new(99, 1)],
            ),
        };

        let error = normalize_scene_file(scene).expect_err("normalization should fail");
        assert!(matches!(
            error,
            SceneNormalizationError::MaterialAssignmentReferencesMissingEntity { entity_id: 99 }
        ));
    }
}
