//! File: domain/editor/editor_persistence/src/scene_formation.rs
//! Purpose: Formed-reality contracts for runtime-ready scene persistence payloads.

use crate::{NormalizedSceneFileV2, SceneEntityRecordV2, SceneFileV2};

#[derive(Debug, Clone, PartialEq)]
pub struct FormedScenePackageV2 {
    entities: Vec<SceneEntityRecordV2>,
}

impl FormedScenePackageV2 {
    pub fn entities(&self) -> &[SceneEntityRecordV2] {
        &self.entities
    }

    pub fn into_scene_file(self) -> SceneFileV2 {
        SceneFileV2::new(self.entities)
    }
}

pub fn form_scene_for_runtime(normalized: NormalizedSceneFileV2) -> FormedScenePackageV2 {
    FormedScenePackageV2 {
        entities: normalized.into_entities(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ScenePrimitiveRecord, SceneTransformRecord, normalize_scene_file};

    #[test]
    fn form_scene_for_runtime_preserves_normalized_entities() {
        let normalized = normalize_scene_file(SceneFileV2 {
            version: crate::SCENE_FILE_VERSION_V2,
            entities: vec![crate::SceneEntityRecordV2::new(
                1,
                "Root",
                None,
                SceneTransformRecord::default(),
                ScenePrimitiveRecord::default(),
            )],
        })
        .expect("normalization should succeed");

        let formed = form_scene_for_runtime(normalized);
        assert_eq!(formed.entities().len(), 1);
        assert_eq!(formed.entities()[0].id, 1);
    }
}
