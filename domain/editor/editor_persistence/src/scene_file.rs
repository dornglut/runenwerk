use serde::{Deserialize, Serialize};

pub const SCENE_FILE_VERSION_V2: u32 = 2;
pub const DEFAULT_SCENE_MATERIAL_SLOT_ID_RAW: u64 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SceneTransformRecord {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl Default for SceneTransformRecord {
    fn default() -> Self {
        Self {
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScenePrimitiveKind {
    Box,
    Sphere,
    Capsule,
    Cylinder,
    Torus,
    Plane,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScenePrimitiveRecord {
    pub kind: ScenePrimitiveKind,
    pub box_half_extents: [f32; 3],
    pub sphere_radius: f32,
    pub capsule_radius: f32,
    pub capsule_half_height: f32,
}

impl Default for ScenePrimitiveRecord {
    fn default() -> Self {
        Self {
            kind: ScenePrimitiveKind::Box,
            box_half_extents: [0.5, 0.5, 0.5],
            sphere_radius: 0.6,
            capsule_radius: 0.35,
            capsule_half_height: 0.75,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneEntityRecordV2 {
    pub id: u64,
    pub display_name: String,
    pub parent: Option<u64>,
    pub transform: SceneTransformRecord,
    pub primitive: ScenePrimitiveRecord,
}

impl SceneEntityRecordV2 {
    pub fn new(
        id: u64,
        display_name: impl Into<String>,
        parent: Option<u64>,
        transform: SceneTransformRecord,
        primitive: ScenePrimitiveRecord,
    ) -> Self {
        Self {
            id,
            display_name: display_name.into(),
            parent,
            transform,
            primitive,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneMaterialSourceRefRecord {
    pub asset_id: u64,
    pub source_id: u64,
    pub source_revision_id: Option<u64>,
    pub source_revision: Option<String>,
}

impl SceneMaterialSourceRefRecord {
    pub fn new(asset_id: u64, source_id: u64) -> Self {
        Self {
            asset_id,
            source_id,
            source_revision_id: None,
            source_revision: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneMaterialSlotRecord {
    pub slot_id: u64,
    pub palette_entry_id: u64,
    pub display_name: String,
    pub source_ref: Option<SceneMaterialSourceRefRecord>,
    pub material_asset_id: Option<u64>,
    pub is_default: bool,
}

impl SceneMaterialSlotRecord {
    pub fn default_generated() -> Self {
        Self {
            slot_id: DEFAULT_SCENE_MATERIAL_SLOT_ID_RAW,
            palette_entry_id: DEFAULT_SCENE_MATERIAL_SLOT_ID_RAW,
            display_name: "Default Material".to_string(),
            source_ref: None,
            material_asset_id: None,
            is_default: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SdfPrimitiveMaterialSlotAssignmentRecord {
    pub sdf_primitive_entity_id: u64,
    pub slot_id: u64,
}

impl SdfPrimitiveMaterialSlotAssignmentRecord {
    pub const fn new(sdf_primitive_entity_id: u64, slot_id: u64) -> Self {
        Self {
            sdf_primitive_entity_id,
            slot_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneMaterialAssignmentsRecord {
    pub source_revision: u64,
    pub palette_slots: Vec<SceneMaterialSlotRecord>,
    pub sdf_primitive_assignments: Vec<SdfPrimitiveMaterialSlotAssignmentRecord>,
}

impl Default for SceneMaterialAssignmentsRecord {
    fn default() -> Self {
        Self {
            source_revision: 1,
            palette_slots: vec![SceneMaterialSlotRecord::default_generated()],
            sdf_primitive_assignments: Vec::new(),
        }
    }
}

impl SceneMaterialAssignmentsRecord {
    pub fn new(
        palette_slots: impl IntoIterator<Item = SceneMaterialSlotRecord>,
        sdf_primitive_assignments: impl IntoIterator<Item = SdfPrimitiveMaterialSlotAssignmentRecord>,
    ) -> Self {
        let mut value = Self {
            source_revision: 1,
            palette_slots: palette_slots.into_iter().collect(),
            sdf_primitive_assignments: sdf_primitive_assignments.into_iter().collect(),
        };
        value.sort_stable();
        value
    }

    pub fn sort_stable(&mut self) {
        self.palette_slots
            .sort_by_key(|slot| (slot.slot_id, slot.palette_entry_id));
        self.sdf_primitive_assignments
            .sort_by_key(|assignment| assignment.sdf_primitive_entity_id);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneFileV2 {
    pub version: u32,
    pub entities: Vec<SceneEntityRecordV2>,
    #[serde(default)]
    pub material_assignments: SceneMaterialAssignmentsRecord,
}

impl SceneFileV2 {
    pub fn new(mut entities: Vec<SceneEntityRecordV2>) -> Self {
        entities.sort_by_key(|entity| entity.id);
        Self {
            version: SCENE_FILE_VERSION_V2,
            entities,
            material_assignments: SceneMaterialAssignmentsRecord::default(),
        }
    }

    pub fn with_material_assignments(
        mut self,
        material_assignments: SceneMaterialAssignmentsRecord,
    ) -> Self {
        self.material_assignments = material_assignments;
        self.material_assignments.sort_stable();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_file_material_assignments_do_not_persist_generated_artifact_truth() {
        let scene = SceneFileV2::new(
            [SceneEntityRecordV2::new(
                1,
                "primitive",
                None,
                SceneTransformRecord::default(),
                ScenePrimitiveRecord::default(),
            )]
            .into_iter()
            .collect(),
        )
        .with_material_assignments(SceneMaterialAssignmentsRecord::new(
            [
                SceneMaterialSlotRecord::default_generated(),
                SceneMaterialSlotRecord {
                    slot_id: 2,
                    palette_entry_id: 2,
                    display_name: "Rock".to_string(),
                    source_ref: Some(SceneMaterialSourceRefRecord::new(7, 9)),
                    material_asset_id: Some(7),
                    is_default: false,
                },
            ],
            [SdfPrimitiveMaterialSlotAssignmentRecord::new(1, 2)],
        ));

        let encoded = ron::to_string(&scene).expect("scene should serialize");

        for forbidden in [
            "formed_material_artifact_id",
            "shader_artifact_id",
            "material_cache_key",
            "shader_cache_key",
            "prior_valid",
        ] {
            assert!(
                !encoded.contains(forbidden),
                "authored SceneFileV2 material assignments must not persist {forbidden}"
            );
        }
    }
}
