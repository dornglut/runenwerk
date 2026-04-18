use editor_core::{
    ChangeOrigin, CommandExecutor, GoverningChangeError, HistoryEntry, RatifiedChange,
    SemanticOperation, TransactionId, TransactionMetadata,
};
use editor_scene::SceneEditorCommand;

use super::ratification::ratify_scene_change;
use crate::editor_runtime::{
    RetainedSceneTransaction, RunenwerkEditorRuntime, sync_selection_after_scene_change,
};
use crate::editor_runtime::parity::assert_scene_projection_parity;

pub(crate) fn execute_scene_transaction_and_push_history_with_origin(
    runtime: &mut RunenwerkEditorRuntime,
    transaction_id: TransactionId,
    transaction_label: impl Into<String>,
    commands: &mut [SceneEditorCommand],
    origin: ChangeOrigin,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    execute_scene_transaction_and_push_history_with_origin_and_causality(
        runtime,
        transaction_id,
        transaction_label,
        commands,
        origin,
        None,
    )
}

pub(crate) fn execute_scene_transaction_and_push_history_with_origin_and_causality(
    runtime: &mut RunenwerkEditorRuntime,
    transaction_id: TransactionId,
    transaction_label: impl Into<String>,
    commands: &mut [SceneEditorCommand],
    origin: ChangeOrigin,
    causality_id: Option<editor_core::CausalityId>,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    let transaction_label = transaction_label.into();
    let before_snapshot = runtime.capture_scene_snapshot();

    let executed =
        runtime.with_scene_command_context(|ctx| -> Result<_, GoverningChangeError> {
            let metadata = TransactionMetadata::new(transaction_id, transaction_label.clone());
            let executed = CommandExecutor::execute_transaction(ctx, metadata.clone(), commands)
                .map_err(|error| GoverningChangeError::mutation_rejected(error.message))?;

            if executed.commands.is_empty() {
                return Ok(None);
            }

            let commands_metadata = executed
                .commands
                .iter()
                .map(|command| command.metadata.clone())
                .collect::<Vec<_>>();

            ctx.session_mut()
                .history_mut()
                .push_applied(HistoryEntry::new(
                    metadata.clone(),
                    commands_metadata.clone(),
                ));

            Ok(Some((metadata, commands_metadata)))
        })?;

    let Some((metadata, commands_metadata)) = executed else {
        return Ok(None);
    };

    let ratified_change = ratify_scene_change(
        runtime,
        metadata,
        commands_metadata,
        origin,
        vec![SemanticOperation::SceneTransactionApplied],
        causality_id,
    );
    runtime.record_ratified_change(ratified_change.clone());
    let after_snapshot = runtime.capture_scene_snapshot();

    runtime.clear_redo_retained_transactions();
    runtime.store_applied_retained_transaction(RetainedSceneTransaction::new(
        transaction_id,
        before_snapshot,
        after_snapshot,
        ratified_change.clone(),
    ));

    sync_selection_after_scene_change(runtime);
    assert_scene_projection_parity(runtime);

    Ok(Some(ratified_change))
}

#[cfg(test)]
mod tests {
    use editor_core::{ChangeOrigin, CommandId, TransactionId};
    use editor_scene::{SceneCommandIntent, scene_intent_to_command};

    use super::*;
    use crate::editor_runtime::RunenwerkEditorRuntime;

    #[test]
    fn executes_transaction_and_pushes_one_history_entry() {
        let mut runtime = RunenwerkEditorRuntime::new();

        let mut commands = vec![
            scene_intent_to_command(
                CommandId(1),
                SceneCommandIntent::CreateEntity {
                    parent: None,
                    display_name: "A".to_string(),
                },
            ),
            scene_intent_to_command(
                CommandId(2),
                SceneCommandIntent::CreateEntity {
                    parent: None,
                    display_name: "B".to_string(),
                },
            ),
        ];

        let executed = execute_scene_transaction_and_push_history_with_origin(
            &mut runtime,
            TransactionId(1),
            "Create Two Entities",
            &mut commands,
            ChangeOrigin::Runtime,
        )
        .expect("transaction should execute");

        assert!(executed.is_some());
        assert_eq!(runtime.session().history().undo_len(), 1);
    }

    #[test]
    fn executes_transaction_and_records_tool_origin() {
        let mut runtime = RunenwerkEditorRuntime::new();

        let mut commands = vec![scene_intent_to_command(
            CommandId(1),
            SceneCommandIntent::CreateEntity {
                parent: None,
                display_name: "A".to_string(),
            },
        )];

        let change = execute_scene_transaction_and_push_history_with_origin_and_causality(
            &mut runtime,
            TransactionId(2),
            "Create One Entity",
            &mut commands,
            ChangeOrigin::ToolInteraction,
            None,
        )
        .expect("transaction should execute")
        .expect("transaction should ratify");

        assert_eq!(change.origin, ChangeOrigin::ToolInteraction);
        assert_eq!(
            change.semantic_operations,
            vec![SemanticOperation::SceneTransactionApplied]
        );
        assert_eq!(runtime.ratified_change_log().len(), 1);
        assert_eq!(
            runtime
                .last_ratified_change()
                .expect("ratified change should be retained")
                .ratification_id,
            change.ratification_id
        );
    }
}
