//! File: domain/editor/editor_scene/src/operations/execute_scene_transaction.rs
//! Purpose: Scene transaction execution and history orchestration.

use editor_core::{
    CommandExecutor, CommandMetadata, GoverningChangeError, HistoryEntry, TransactionMetadata,
};

use crate::{SceneCommandContext, SceneEditorCommand};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedSceneTransaction {
    pub transaction: TransactionMetadata,
    pub command_metadata: Vec<CommandMetadata>,
}

pub fn execute_scene_transaction_and_push_history(
    context: &mut SceneCommandContext<'_>,
    transaction: TransactionMetadata,
    commands: &mut [SceneEditorCommand],
) -> Result<Option<ExecutedSceneTransaction>, GoverningChangeError> {
    let executed = CommandExecutor::execute_transaction(context, transaction.clone(), commands)
        .map_err(|error| GoverningChangeError::mutation_rejected(error.message))?;

    if executed.commands.is_empty() {
        return Ok(None);
    }

    let command_metadata = executed
        .commands
        .iter()
        .map(|command| command.metadata.clone())
        .collect::<Vec<_>>();

    context
        .session_mut()
        .history_mut()
        .push_applied(HistoryEntry::new(
            transaction.clone(),
            command_metadata.clone(),
        ));

    Ok(Some(ExecutedSceneTransaction {
        transaction,
        command_metadata,
    }))
}
