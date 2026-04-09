use crate::editor_runtime::RunenwerkEditorRuntime;

pub fn validate_scene_projection_parity(
    runtime: &RunenwerkEditorRuntime,
) -> Result<(), &'static str> {
    let document = runtime.document();
    let ids = runtime.ids();

    for entity in document.entity_ids() {
        let Some(document_snapshot) = document.entity_snapshot(entity) else {
            return Err("document snapshot missing for known entity");
        };
        let Some(projected_snapshot) = ids.entity_snapshot(entity) else {
            return Err("projection snapshot missing for document entity");
        };

        if document_snapshot != projected_snapshot {
            return Err("document and projection snapshots diverged");
        }
    }

    for entity in ids.entity_ids() {
        if !document.contains(entity) {
            return Err("projection contains entity missing from document");
        }
    }

    Ok(())
}

pub fn assert_scene_projection_parity(runtime: &RunenwerkEditorRuntime) {
    let result = validate_scene_projection_parity(runtime);
    debug_assert!(
        result.is_ok(),
        "scene projection parity check failed: {:?}",
        result.err()
    );

    if let Err(error) = result {
        eprintln!("scene projection parity check failed: {error}");
    }
}
