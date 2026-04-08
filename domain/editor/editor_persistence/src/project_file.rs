use serde::{Deserialize, Serialize};

pub const PROJECT_FILE_VERSION_V1: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSceneEntryV1 {
    pub scene_id: String,
    pub display_name: String,
    pub file_path: String,
}

impl ProjectSceneEntryV1 {
    pub fn new(
        scene_id: impl Into<String>,
        display_name: impl Into<String>,
        file_path: impl Into<String>,
    ) -> Self {
        Self {
            scene_id: scene_id.into(),
            display_name: display_name.into(),
            file_path: file_path.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectFileV1 {
    pub version: u32,
    pub project_name: String,
    pub startup_scene_id: Option<String>,
    pub scenes: Vec<ProjectSceneEntryV1>,
}

impl ProjectFileV1 {
    pub fn new(
        project_name: impl Into<String>,
        startup_scene_id: Option<String>,
        scenes: Vec<ProjectSceneEntryV1>,
    ) -> Self {
        Self {
            version: PROJECT_FILE_VERSION_V1,
            project_name: project_name.into(),
            startup_scene_id,
            scenes,
        }
    }
}
