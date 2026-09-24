//! File: domain/editor/editor_scene/src/operations/execute_scene_transaction.rs
//! Purpose: Scene transaction execution.

use editor_core::{CommandExecutor, CommandMetadata, GoverningChangeError, TransactionMetadata};

use crate::{SceneCommandContext, SceneEditorCommand};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedSceneTransaction {
    pub transaction: TransactionMetadata,
    pub command_metadata: Vec<CommandMetadata>,
}

pub fn execute_scene_transaction(
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

    Ok(Some(ExecutedSceneTransaction {
        transaction,
        command_metadata,
    }))
}
