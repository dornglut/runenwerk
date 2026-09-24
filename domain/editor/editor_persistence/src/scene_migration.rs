use serde::{Deserialize, Serialize};

use crate::{
    SCENE_FILE_VERSION_V2, SceneEntityRecordV2, SceneFileV2, ScenePrimitiveRecord,
    SceneTransformRecord,
};

pub const SCENE_FILE_VERSION_V1: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneEntityRecordV1 {
    pub id: u64,
    pub display_name: String,
    pub parent: Option<u64>,
    pub transform: SceneTransformRecord,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneFileV1 {
    pub version: u32,
    pub entities: Vec<SceneEntityRecordV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneMigrationPath {
    IdentityV2,
    V1ToV2DefaultPrimitive,
}

impl SceneMigrationPath {
    pub const fn as_static_str(self) -> &'static str {
        match self {
            Self::IdentityV2 => "scene:v2->v2",
            Self::V1ToV2DefaultPrimitive => "scene:v1->v2:default-primitive",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneLoadResult {
    pub scene: SceneFileV2,
    pub migration: SceneMigrationPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct SceneVersionProbe {
    version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneLoadError {
    DecodeFailed,
    UnsupportedVersion(u32),
}

impl SceneLoadError {
    pub const fn as_static_str(&self) -> &'static str {
        match self {
            Self::DecodeFailed => "failed to decode scene file",
            Self::UnsupportedVersion(_) => "unsupported scene file version",
        }
    }
}

pub fn decode_scene_file_with_migration(source: &str) -> Result<SceneLoadResult, SceneLoadError> {
    let probe: SceneVersionProbe =
        ron::from_str(source).map_err(|_| SceneLoadError::DecodeFailed)?;

    match probe.version {
        SCENE_FILE_VERSION_V2 => {
            let scene: SceneFileV2 =
                ron::from_str(source).map_err(|_| SceneLoadError::DecodeFailed)?;
            Ok(SceneLoadResult {
                scene,
                migration: SceneMigrationPath::IdentityV2,
            })
        }
        SCENE_FILE_VERSION_V1 => {
            let scene_v1: SceneFileV1 =
                ron::from_str(source).map_err(|_| SceneLoadError::DecodeFailed)?;
            Ok(SceneLoadResult {
                scene: migrate_scene_v1_to_v2(scene_v1),
                migration: SceneMigrationPath::V1ToV2DefaultPrimitive,
            })
        }
        other => Err(SceneLoadError::UnsupportedVersion(other)),
    }
}

fn migrate_scene_v1_to_v2(scene_v1: SceneFileV1) -> SceneFileV2 {
    let entities = scene_v1
        .entities
        .into_iter()
        .map(|entity| {
            SceneEntityRecordV2::new(
                entity.id,
                entity.display_name,
                entity.parent,
                entity.transform,
                ScenePrimitiveRecord::default(),
            )
        })
        .collect::<Vec<_>>();

    SceneFileV2::new(entities)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ScenePrimitiveKind, SceneTransformRecord};

    #[test]
    fn decodes_v2_without_migration() {
        let source = r#"
(
    version: 2,
    entities: [
        (
            id: 1,
            display_name: "Entity",
            parent: None,
            transform: (
                translation: (
                    0.0,
                    0.0,
                    0.0,
                ),
                rotation: (
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                ),
                scale: (
                    1.0,
                    1.0,
                    1.0,
                ),
            ),
            primitive: (
                kind: Box,
                box_half_extents: (
                    0.5,
                    0.5,
                    0.5,
                ),
                sphere_radius: 0.6,
                capsule_radius: 0.35,
                capsule_half_height: 0.75,
            ),
        ),
    ],
)
"#;

        let result = decode_scene_file_with_migration(source).expect("decode should succeed");
        assert_eq!(result.migration, SceneMigrationPath::IdentityV2);
        assert_eq!(result.scene.version, SCENE_FILE_VERSION_V2);
        assert_eq!(
            result.scene.entities[0].primitive.kind,
            ScenePrimitiveKind::Box
        );
    }

    #[test]
    fn migrates_v1_to_v2_with_default_primitive() {
        let source = r#"
(
    version: 1,
    entities: [
        (
            id: 1,
            display_name: "Entity",
            parent: None,
            transform: (
                translation: (
                    0.0,
                    0.0,
                    0.0,
                ),
                rotation: (
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                ),
                scale: (
                    1.0,
                    1.0,
                    1.0,
                ),
            ),
        ),
    ],
)
"#;

        let result = decode_scene_file_with_migration(source).expect("decode should succeed");
        assert_eq!(result.migration, SceneMigrationPath::V1ToV2DefaultPrimitive);
        assert_eq!(result.scene.version, SCENE_FILE_VERSION_V2);
        assert_eq!(
            result.scene.entities[0].primitive,
            crate::ScenePrimitiveRecord::default()
        );
        assert_eq!(
            result.scene.entities[0].transform,
            SceneTransformRecord::default()
        );
    }
}
