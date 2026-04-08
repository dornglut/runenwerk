use editor_core::{CommandExecutor, HistoryEntry, TransactionId, TransactionMetadata};
use editor_scene::{SceneCommandContext, SceneEditorCommand};

use crate::editor_runtime::{RunenwerkEditorRuntime, StoredSceneTransaction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedSceneTransaction {
    pub metadata: TransactionMetadata,
    pub commands: Vec<editor_core::CommandMetadata>,
}

/// File: apps/runenwerk_editor/src/editor_runtime/commands/transactions.rs
/// Method: execute_scene_transaction
pub fn execute_scene_transaction(
    runtime: &mut RunenwerkEditorRuntime,
    transaction_id: TransactionId,
    transaction_label: impl Into<String>,
    commands: &mut [SceneEditorCommand],
) -> Result<ExecutedSceneTransaction, &'static str> {
    let (session, mut scene_runtime) = runtime.session_and_scene_runtime();
    let mut ctx = SceneCommandContext::new(session, &mut scene_runtime);

    let metadata = TransactionMetadata::new(transaction_id, transaction_label);
    let executed = CommandExecutor::execute_transaction(&mut ctx, metadata.clone(), commands)?;

    let commands_metadata = executed
        .commands
        .iter()
        .map(|command| command.metadata.clone())
        .collect::<Vec<_>>();

    Ok(ExecutedSceneTransaction {
        metadata,
        commands: commands_metadata,
    })
}

/// File: apps/runenwerk_editor/src/editor_runtime/commands/transactions.rs
/// Method: execute_scene_transaction_and_push_history
pub fn execute_scene_transaction_and_push_history(
    runtime: &mut RunenwerkEditorRuntime,
    transaction_id: TransactionId,
    transaction_label: impl Into<String>,
    commands: &mut [SceneEditorCommand],
) -> Result<Option<ExecutedSceneTransaction>, &'static str> {
    let transaction_label = transaction_label.into();

    let (metadata, commands_metadata, stored_commands) = {
        let (session, mut scene_runtime) = runtime.session_and_scene_runtime();
        let mut ctx = SceneCommandContext::new(session, &mut scene_runtime);

        let metadata = TransactionMetadata::new(transaction_id, transaction_label.clone());
        let executed = CommandExecutor::execute_transaction(&mut ctx, metadata.clone(), commands)?;

        if executed.commands.is_empty() {
            return Ok(None);
        }

        let stored_commands = commands.to_vec();

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

        (metadata, commands_metadata, stored_commands)
    };

    let store = runtime.command_store_mut();
    store.clear_redo();
    store.store_applied(StoredSceneTransaction::new(transaction_id, stored_commands));

    Ok(Some(ExecutedSceneTransaction {
        metadata,
        commands: commands_metadata,
    }))
}

#[cfg(test)]
mod tests {
    use editor_core::{CommandId, TransactionId};
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

        let executed = execute_scene_transaction_and_push_history(
            &mut runtime,
            TransactionId(1),
            "Create Two Entities",
            &mut commands,
        )
        .expect("transaction should execute");

        assert!(executed.is_some());
        assert_eq!(runtime.session().history().undo_len(), 1);
    }
}
