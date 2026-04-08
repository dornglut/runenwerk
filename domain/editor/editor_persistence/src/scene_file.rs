use serde::{Deserialize, Serialize};

pub const SCENE_FILE_VERSION_V1: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneEntityRecordV1 {
    pub id: u64,
    pub display_name: String,
    pub parent: Option<u64>,
}

impl SceneEntityRecordV1 {
    pub fn new(id: u64, display_name: impl Into<String>, parent: Option<u64>) -> Self {
        Self {
            id,
            display_name: display_name.into(),
            parent,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneFileV1 {
    pub version: u32,
    pub entities: Vec<SceneEntityRecordV1>,
}

impl SceneFileV1 {
    pub fn new(mut entities: Vec<SceneEntityRecordV1>) -> Self {
        entities.sort_by(|left, right| left.id.cmp(&right.id));
        Self {
            version: SCENE_FILE_VERSION_V1,
            entities,
        }
    }
}
