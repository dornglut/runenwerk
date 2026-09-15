use editor_core::{
    ChangeOrigin, GoverningChangeError, RatifiedChange, SceneChangeRatificationParams,
    SemanticOperation,
};

use crate::editor_runtime::parity::assert_scene_projection_parity;
use crate::editor_runtime::{RunenwerkEditorRuntime, sync_selection_after_scene_change};

use crate::editor_runtime::commands::ratification::ratify_scene_change;

pub(crate) fn undo_last_scene_transaction_with_origin(
    runtime: &mut RunenwerkEditorRuntime,
    origin: ChangeOrigin,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    let Some(history_entry) = runtime.peek_scene_undo_history_entry().cloned() else {
        return Ok(None);
    };

    runtime
        .restore_scene_snapshot(&history_entry.before_snapshot)
        .map_err(|error| GoverningChangeError::mutation_rejected(error.message))?;

    let transaction = history_entry.ratified_change.transaction.clone();
    let commands = history_entry.ratified_change.command_metadata.clone();
    let causality_id = history_entry.ratified_change.causality_id;

    if !runtime.commit_scene_undo_history_entry() {
        return Err(GoverningChangeError::history_inconsistent(
            "scene undo history changed during restoration",
        ));
    }

    sync_selection_after_scene_change(runtime);
    assert_scene_projection_parity(runtime);

    let ratified_change = ratify_scene_change(
        runtime,
        SceneChangeRatificationParams::new(
            transaction,
            commands,
            origin,
            vec![SemanticOperation::SceneTransactionUndone],
            Some(causality_id),
        ),
    );
    runtime.record_ratified_change(ratified_change.clone());

    Ok(Some(ratified_change))
}

pub(crate) fn redo_last_scene_transaction_with_origin(
    runtime: &mut RunenwerkEditorRuntime,
    origin: ChangeOrigin,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    let Some(history_entry) = runtime.peek_scene_redo_history_entry().cloned() else {
        return Ok(None);
    };

    runtime
        .restore_scene_snapshot(&history_entry.after_snapshot)
        .map_err(|error| GoverningChangeError::mutation_rejected(error.message))?;

    let transaction = history_entry.ratified_change.transaction.clone();
    let commands = history_entry.ratified_change.command_metadata.clone();
    let causality_id = history_entry.ratified_change.causality_id;

    if !runtime.commit_scene_redo_history_entry() {
        return Err(GoverningChangeError::history_inconsistent(
            "scene redo history changed during restoration",
        ));
    }

    sync_selection_after_scene_change(runtime);
    assert_scene_projection_parity(runtime);

    let ratified_change = ratify_scene_change(
        runtime,
        SceneChangeRatificationParams::new(
            transaction,
            commands,
            origin,
            vec![SemanticOperation::SceneTransactionRedone],
            Some(causality_id),
        ),
    );
    runtime.record_ratified_change(ratified_change.clone());

    Ok(Some(ratified_change))
}
