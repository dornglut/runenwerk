use editor_core::{
    ChangeOrigin, Command, CommandExecutor, GoverningChangeError, RatifiedChange, SemanticOperation,
};
use editor_scene::SceneCommandContext;

use crate::editor_runtime::{
    RunenwerkEditorRuntime, StoredSceneTransaction, assert_scene_projection_parity,
    sync_selection_after_scene_change,
};

use crate::editor_runtime::commands::ratification::ratify_scene_change;

pub fn undo_last_scene_transaction(
    runtime: &mut RunenwerkEditorRuntime,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    undo_last_scene_transaction_with_origin(runtime, ChangeOrigin::Runtime)
}

pub fn undo_last_scene_transaction_with_origin(
    runtime: &mut RunenwerkEditorRuntime,
    origin: ChangeOrigin,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    let Some(history_entry) = runtime.session_mut().history_mut().pop_undo() else {
        return Ok(None);
    };

    let transaction_id = history_entry.transaction.id;
    let Some(mut stored) = runtime.command_store_mut().take_applied(transaction_id) else {
        return Err(GoverningChangeError::history_inconsistent(
            "missing stored scene transaction for undo",
        ));
    };

    {
        let (session, mut scene_runtime) = runtime.session_and_scene_runtime();
        let mut ctx = SceneCommandContext::new(session, &mut scene_runtime);

        for command in stored.commands.iter_mut().rev() {
            command
                .undo(&mut ctx)
                .map_err(GoverningChangeError::mutation_rejected)?;
        }
    }

    let causality_id = stored.ratified_change.causality_id;

    runtime
        .command_store_mut()
        .store_redo(StoredSceneTransaction::new(
            transaction_id,
            stored.commands,
            stored.ratified_change,
        ));

    runtime
        .session_mut()
        .history_mut()
        .push_redo(history_entry.clone());

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

pub fn redo_last_scene_transaction(
    runtime: &mut RunenwerkEditorRuntime,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    redo_last_scene_transaction_with_origin(runtime, ChangeOrigin::Runtime)
}

pub fn redo_last_scene_transaction_with_origin(
    runtime: &mut RunenwerkEditorRuntime,
    origin: ChangeOrigin,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    let Some(history_entry) = runtime.session_mut().history_mut().pop_redo() else {
        return Ok(None);
    };

    let transaction_id = history_entry.transaction.id;
    let Some(mut stored) = runtime.command_store_mut().take_redo(transaction_id) else {
        return Err(GoverningChangeError::history_inconsistent(
            "missing stored scene transaction for redo",
        ));
    };

    {
        let (session, mut scene_runtime) = runtime.session_and_scene_runtime();
        let mut ctx = SceneCommandContext::new(session, &mut scene_runtime);

        for command in stored.commands.iter_mut() {
            CommandExecutor::execute_command(&mut ctx, command)
                .map_err(GoverningChangeError::mutation_rejected)?;
        }
    }

    let causality_id = stored.ratified_change.causality_id;

    runtime
        .command_store_mut()
        .store_applied(StoredSceneTransaction::new(
            transaction_id,
            stored.commands,
            stored.ratified_change,
        ));

    runtime
        .session_mut()
        .history_mut()
        .push_applied(history_entry.clone());

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
