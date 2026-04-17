//! File: domain/editor/editor_persistence/src/scene_normalization.rs
//! Purpose: Normalized-reality contracts for scene persistence.

use std::collections::{BTreeMap, BTreeSet};

use crate::{SCENE_FILE_VERSION_V2, SceneEntityRecordV2, SceneFileV2};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneNormalizationError {
    UnsupportedVersion(u32),
    DuplicateEntityId(u64),
    MissingParent { entity_id: u64, parent_id: u64 },
    CyclicParentReference { entity_id: u64 },
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
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedSceneFileV2 {
    entities: Vec<SceneEntityRecordV2>,
}

impl NormalizedSceneFileV2 {
    pub fn entities(&self) -> &[SceneEntityRecordV2] {
        &self.entities
    }

    pub fn into_entities(self) -> Vec<SceneEntityRecordV2> {
        self.entities
    }

    pub fn into_scene_file(self) -> SceneFileV2 {
        SceneFileV2::new(self.entities)
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

    scene_file
        .entities
        .sort_by(|left, right| left.id.cmp(&right.id));

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
        if let Some(parent) = entity.parent {
            if !parent_by_entity.contains_key(&parent) {
                return Err(SceneNormalizationError::MissingParent {
                    entity_id: entity.id,
                    parent_id: parent,
                });
            }
        }

        if has_parent_cycle(entity.id, &parent_by_entity) {
            return Err(SceneNormalizationError::CyclicParentReference {
                entity_id: entity.id,
            });
        }
    }

    Ok(NormalizedSceneFileV2 {
        entities: scene_file.entities,
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
        };

        let error = normalize_scene_file(scene).expect_err("normalization should fail");
        assert!(matches!(
            error,
            SceneNormalizationError::CyclicParentReference { .. }
        ));
    }
}
