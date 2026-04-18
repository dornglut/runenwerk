use editor_core::EditorMutationError;

use crate::editor_runtime::RunenwerkEditorRuntime;

pub(crate) fn validate_scene_projection_parity(
    runtime: &RunenwerkEditorRuntime,
) -> Result<(), EditorMutationError> {
    let document = runtime.document();
    let ids = runtime.ids();

    for entity in document.entity_ids() {
        if ids.resolve_entity(entity).is_none() {
            return Err(EditorMutationError::runtime_rejected(
                "projection mapping missing for document entity",
            ));
        }
    }

    for entity in ids.entity_ids() {
        if !document.contains(entity) {
            return Err(EditorMutationError::runtime_rejected(
                "projection contains entity missing from document",
            ));
        }
    }

    Ok(())
}

pub(crate) fn assert_scene_projection_parity(runtime: &RunenwerkEditorRuntime) {
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
