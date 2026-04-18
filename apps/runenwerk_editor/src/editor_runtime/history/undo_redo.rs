use editor_core::{ChangeOrigin, GoverningChangeError, RatifiedChange, SemanticOperation};

use crate::editor_runtime::parity::assert_scene_projection_parity;
use crate::editor_runtime::{
    sync_selection_after_scene_change, RetainedSceneTransaction, RunenwerkEditorRuntime,
};

use crate::editor_runtime::commands::ratification::ratify_scene_change;

pub(crate) fn undo_last_scene_transaction_with_origin(
    runtime: &mut RunenwerkEditorRuntime,
    origin: ChangeOrigin,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    let Some(history_entry) = runtime.pop_undo_history_entry() else {
        return Ok(None);
    };

    let transaction_id = history_entry.transaction.id;
    let Some(stored) = runtime.take_applied_retained_transaction(transaction_id) else {
        return Err(GoverningChangeError::history_inconsistent(
            "missing stored scene transaction for undo",
        ));
    };

    runtime
        .restore_scene_snapshot(&stored.before_snapshot)
        .map_err(|error| GoverningChangeError::mutation_rejected(error.message))?;

    let causality_id = stored.ratified_change.causality_id;

    runtime.store_redo_retained_transaction(RetainedSceneTransaction::new(
        transaction_id,
        stored.before_snapshot,
        stored.after_snapshot,
        stored.ratified_change,
    ));

    runtime.push_redo_history_entry(history_entry.clone());

    sync_selection_after_scene_change(runtime);
    assert_scene_projection_parity(runtime);

    let ratified_change = ratify_scene_change(
        runtime,
        history_entry.transaction,
        history_entry.commands,
        origin,
        vec![SemanticOperation::SceneTransactionUndone],
        Some(causality_id),
    );
    runtime.record_ratified_change(ratified_change.clone());

    Ok(Some(ratified_change))
}

pub(crate) fn redo_last_scene_transaction_with_origin(
    runtime: &mut RunenwerkEditorRuntime,
    origin: ChangeOrigin,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    let Some(history_entry) = runtime.pop_redo_history_entry() else {
        return Ok(None);
    };

    let transaction_id = history_entry.transaction.id;
    let Some(stored) = runtime.take_redo_retained_transaction(transaction_id) else {
        return Err(GoverningChangeError::history_inconsistent(
            "missing stored scene transaction for redo",
        ));
    };

    runtime
        .restore_scene_snapshot(&stored.after_snapshot)
        .map_err(|error| GoverningChangeError::mutation_rejected(error.message))?;

    let causality_id = stored.ratified_change.causality_id;

    runtime.store_applied_retained_transaction(RetainedSceneTransaction::new(
        transaction_id,
        stored.before_snapshot,
        stored.after_snapshot,
        stored.ratified_change,
    ));

    runtime.push_applied_history_entry(history_entry.clone());

    sync_selection_after_scene_change(runtime);
    assert_scene_projection_parity(runtime);

    let ratified_change = ratify_scene_change(
        runtime,
        history_entry.transaction,
        history_entry.commands,
        origin,
        vec![SemanticOperation::SceneTransactionRedone],
        Some(causality_id),
    );
    runtime.record_ratified_change(ratified_change.clone());

    Ok(Some(ratified_change))
}
