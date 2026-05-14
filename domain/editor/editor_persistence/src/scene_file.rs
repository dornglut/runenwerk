use serde::{Deserialize, Serialize};

pub const SCENE_FILE_VERSION_V2: u32 = 2;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneFileV2 {
    pub version: u32,
    pub entities: Vec<SceneEntityRecordV2>,
}

impl SceneFileV2 {
    pub fn new(mut entities: Vec<SceneEntityRecordV2>) -> Self {
        entities.sort_by_key(|entity| entity.id);
        Self {
            version: SCENE_FILE_VERSION_V2,
            entities,
        }
    }
}
