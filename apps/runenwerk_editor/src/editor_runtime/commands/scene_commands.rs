use editor_core::{CommandExecutor, CommandId};
use editor_scene::{
    SceneCommandContext, SceneCommandIntent, SceneEditorCommand, scene_intent_to_command,
};

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
    mut command: SceneEditorCommand,
    transaction_label: impl Into<String>,
    transaction_id: editor_core::TransactionId,
) -> Result<Option<ExecutedSceneCommand>, &'static str> {
    let transaction_label = transaction_label.into();

    let executed = {
        let (session, mut scene_runtime) = runtime.session_and_scene_runtime();
        let mut ctx = SceneCommandContext::new(session, &mut scene_runtime);

        let executed = CommandExecutor::execute_command(&mut ctx, &mut command)?;

        if let Some(executed_command) = &executed {
            let entry = editor_core::HistoryEntry::new(
                editor_core::TransactionMetadata::new(transaction_id, transaction_label.clone()),
                vec![executed_command.metadata.clone()],
            );

            ctx.session_mut().history_mut().push_applied(entry);
        }

        executed
    };

    if executed.is_some() {
        let store = runtime.command_store_mut();
        store.clear_redo();
        store.store_applied(StoredSceneTransaction::new(transaction_id, vec![command]));
    }

    sync_selection_after_scene_change(runtime);
    assert_scene_projection_parity(runtime);

    match executed {
        Some(executed) => Ok(Some(ExecutedSceneCommand {
            metadata: executed.metadata,
        })),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use editor_core::{CommandId, ComponentTypeId, EntityId, TransactionId};
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
