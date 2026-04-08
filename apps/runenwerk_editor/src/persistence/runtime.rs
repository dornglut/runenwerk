use std::collections::BTreeSet;

use editor_core::EntityId;
use editor_persistence::{SceneEntityRecordV1, SceneFileV1};
use editor_scene::{SceneEntitySnapshot, SceneRuntime};

use crate::editor_runtime::RunenwerkEditorRuntime;

/// File: apps/runenwerk_editor/src/persistence/runtime.rs
/// Method: scene_file_from_runtime
pub fn scene_file_from_runtime(runtime: &RunenwerkEditorRuntime) -> SceneFileV1 {
    let entities = runtime
        .document()
        .entity_ids()
        .filter_map(|entity| {
            let snapshot = runtime.document().entity_snapshot(entity)?;
            Some(SceneEntityRecordV1::new(
                snapshot.id.0,
                snapshot.display_name,
                snapshot.parent.map(|parent| parent.0),
            ))
        })
        .collect::<Vec<_>>();

    SceneFileV1::new(entities)
}

/// File: apps/runenwerk_editor/src/persistence/runtime.rs
/// Method: apply_scene_file_to_runtime
pub fn apply_scene_file_to_runtime(
    runtime: &mut RunenwerkEditorRuntime,
    scene_file: &SceneFileV1,
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

    Ok(())
}
