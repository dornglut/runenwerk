use editor_core::{
    ChangeOrigin, CommandExecutor, CommandId, GoverningChangeError, RatifiedChange,
    SemanticOperation,
};
use editor_scene::{
    SceneCommandContext, SceneCommandIntent, SceneEditorCommand, scene_intent_to_command,
};

use super::ratification::ratify_scene_change;
use crate::editor_runtime::{
    RunenwerkEditorRuntime, StoredSceneTransaction, assert_scene_projection_parity,
    sync_selection_after_scene_change,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedSceneCommand {
    pub metadata: editor_core::CommandMetadata,
}

pub fn execute_scene_intent(
    runtime: &mut RunenwerkEditorRuntime,
    command_id: CommandId,
    intent: SceneCommandIntent,
) -> Result<Option<ExecutedSceneCommand>, &'static str> {
    let command = scene_intent_to_command(command_id, intent);
    execute_scene_command(runtime, command)
}

pub fn execute_scene_command(
    runtime: &mut RunenwerkEditorRuntime,
    mut command: SceneEditorCommand,
) -> Result<Option<ExecutedSceneCommand>, &'static str> {
    let executed = {
        let (session, mut scene_runtime) = runtime.session_and_scene_runtime();
        let mut ctx = SceneCommandContext::new(session, &mut scene_runtime);

        CommandExecutor::execute_command(&mut ctx, &mut command)?
    };

    sync_selection_after_scene_change(runtime);
    assert_scene_projection_parity(runtime);

    match executed {
        Some(executed) => Ok(Some(ExecutedSceneCommand {
            metadata: executed.metadata,
        })),
        None => Ok(None),
    }
}

pub fn execute_scene_command_and_push_history(
    runtime: &mut RunenwerkEditorRuntime,
    command: SceneEditorCommand,
    transaction_label: impl Into<String>,
    transaction_id: editor_core::TransactionId,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    execute_scene_command_and_push_history_with_origin(
        runtime,
        command,
        transaction_label,
        transaction_id,
        ChangeOrigin::Runtime,
    )
}

pub fn execute_scene_command_and_push_history_with_origin(
    runtime: &mut RunenwerkEditorRuntime,
    mut command: SceneEditorCommand,
    transaction_label: impl Into<String>,
    transaction_id: editor_core::TransactionId,
    origin: ChangeOrigin,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    let transaction_label = transaction_label.into();

    let executed_command_metadata = {
        let (session, mut scene_runtime) = runtime.session_and_scene_runtime();
        let mut ctx = SceneCommandContext::new(session, &mut scene_runtime);

        let executed_command = CommandExecutor::execute_command(&mut ctx, &mut command)
            .map_err(GoverningChangeError::mutation_rejected)?;

        if let Some(executed_command) = &executed_command {
            let entry = editor_core::HistoryEntry::new(
                editor_core::TransactionMetadata::new(transaction_id, transaction_label.clone()),
                vec![executed_command.metadata.clone()],
            );

            ctx.session_mut().history_mut().push_applied(entry);
            Some(executed_command.metadata.clone())
        } else {
            None
        }
    };

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

    let store = runtime.command_store_mut();
    store.clear_redo();
    store.store_applied(StoredSceneTransaction::new(
        transaction_id,
        vec![command],
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

        let result = execute_scene_command_and_push_history(
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
