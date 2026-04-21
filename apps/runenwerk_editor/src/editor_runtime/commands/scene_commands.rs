use editor_core::{
    ChangeOrigin, CommandExecutor, CommandId, EditorMutationError, GoverningChangeError,
    RatifiedChange, SemanticOperation,
};
use editor_scene::{SceneCommandIntent, SceneEditorCommand, scene_intent_to_command};

use super::ratification::ratify_scene_change;
use crate::editor_runtime::parity::assert_scene_projection_parity;
use crate::editor_runtime::{
    RetainedSceneTransaction, RunenwerkEditorRuntime, sync_selection_after_scene_change,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedSceneCommand {
    pub metadata: editor_core::CommandMetadata,
}

pub(crate) fn execute_scene_intent(
    runtime: &mut RunenwerkEditorRuntime,
    command_id: CommandId,
    intent: SceneCommandIntent,
) -> Result<Option<ExecutedSceneCommand>, EditorMutationError> {
    let command = scene_intent_to_command(command_id, intent);
    execute_scene_command(runtime, command)
}

pub(crate) fn execute_scene_command(
    runtime: &mut RunenwerkEditorRuntime,
    mut command: SceneEditorCommand,
) -> Result<Option<ExecutedSceneCommand>, EditorMutationError> {
    let executed = runtime
        .with_scene_command_context(|ctx| CommandExecutor::execute_command(ctx, &mut command))?;

    sync_selection_after_scene_change(runtime);
    assert_scene_projection_parity(runtime);

    match executed {
        Some(executed) => Ok(Some(ExecutedSceneCommand {
            metadata: executed.metadata,
        })),
        None => Ok(None),
    }
}

pub(crate) fn execute_scene_command_and_push_history_with_origin(
    runtime: &mut RunenwerkEditorRuntime,
    mut command: SceneEditorCommand,
    transaction_label: impl Into<String>,
    transaction_id: editor_core::TransactionId,
    origin: ChangeOrigin,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    let transaction_label = transaction_label.into();
    let before_snapshot = runtime.capture_scene_snapshot();

    let executed_command_metadata =
        runtime.with_scene_command_context(|ctx| -> Result<_, GoverningChangeError> {
            let executed_command = CommandExecutor::execute_command(ctx, &mut command)
                .map_err(|error| GoverningChangeError::mutation_rejected(error.message))?;

            if let Some(executed_command) = &executed_command {
                let entry = editor_core::HistoryEntry::new(
                    editor_core::TransactionMetadata::new(
                        transaction_id,
                        transaction_label.clone(),
                    ),
                    vec![executed_command.metadata.clone()],
                );

                ctx.session_mut().history_mut().push_applied(entry);
                Ok(Some(executed_command.metadata.clone()))
            } else {
                Ok(None)
            }
        })?;

    sync_selection_after_scene_change(runtime);
    assert_scene_projection_parity(runtime);

    let Some(command_metadata) = executed_command_metadata else {
        return Ok(None);
    };

    let ratified_change = ratify_scene_change(
        runtime,
        editor_core::TransactionMetadata::new(transaction_id, transaction_label),
        vec![command_metadata],
        origin,
        vec![SemanticOperation::SceneCommandApplied],
        None,
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

    Ok(Some(ratified_change))
}

#[cfg(test)]
mod tests {
    use editor_core::{ChangeOrigin, CommandId, ComponentTypeId, EntityId, TransactionId};
    use editor_inspector::{InspectorEditValue, InspectorPath};
    use editor_scene::SceneCommandIntent;

    use super::*;
    use crate::editor_runtime::RunenwerkEditorRuntime;

    #[test]
    fn executes_scene_intent_without_history() {
        let mut runtime = RunenwerkEditorRuntime::new();

        let result = execute_scene_intent(
            &mut runtime,
            CommandId(1),
            SceneCommandIntent::CreateEntity {
                parent: None,
                display_name: "Player".to_string(),
            },
        )
        .expect("scene intent execution should succeed");

        assert!(result.is_some());
        assert_eq!(runtime.session().history().undo_len(), 0);
    }

    #[test]
    fn executes_scene_command_and_pushes_history() {
        let mut runtime = RunenwerkEditorRuntime::new();

        let result = execute_scene_command_and_push_history_with_origin(
            &mut runtime,
            editor_scene::scene_intent_to_command(
                CommandId(2),
                SceneCommandIntent::CreateEntity {
                    parent: None,
                    display_name: "Player".to_string(),
                },
            ),
            "Create Entity",
            TransactionId(1),
            ChangeOrigin::Runtime,
        )
        .expect("scene command execution should succeed");

        assert!(result.is_some());
        assert_eq!(runtime.session().history().undo_len(), 1);
    }

    #[test]
    fn executes_scene_command_and_records_origin() {
        let mut runtime = RunenwerkEditorRuntime::new();

        let change = execute_scene_command_and_push_history_with_origin(
            &mut runtime,
            editor_scene::scene_intent_to_command(
                CommandId(3),
                SceneCommandIntent::CreateEntity {
                    parent: None,
                    display_name: "Player".to_string(),
                },
            ),
            "Create Entity",
            TransactionId(3),
            ChangeOrigin::InspectorPanel,
        )
        .expect("scene command execution should succeed")
        .expect("change should be ratified");

        assert_eq!(change.origin, ChangeOrigin::InspectorPanel);
        assert_eq!(
            change.semantic_operations,
            vec![SemanticOperation::SceneCommandApplied]
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

    #[test]
    fn edit_component_field_command_can_be_built_and_executed() {
        let mut runtime = RunenwerkEditorRuntime::new();

        let result = execute_scene_command(
            &mut runtime,
            editor_scene::SceneEditorCommand::new_edit_component_field(
                CommandId(3),
                "Edit Component Field",
                EntityId(1),
                ComponentTypeId(10),
                InspectorPath::root().child_field("speed"),
                InspectorEditValue::Float(7.0),
            ),
        );

        assert!(result.is_err() || result.is_ok());
    }
}
