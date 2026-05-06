//! File: domain/editor/editor_scene/src/operations/execute_scene_command.rs
//! Purpose: Single scene-command execution and history orchestration.

use editor_core::{
    CommandExecutor, CommandMetadata, GoverningChangeError, HistoryEntry, TransactionMetadata,
};

use crate::{SceneCommandContext, SceneEditorCommand};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedSceneCommand {
    pub metadata: CommandMetadata,
}

pub fn execute_scene_command(
    context: &mut SceneCommandContext<'_>,
    command: &mut SceneEditorCommand,
) -> Result<Option<ExecutedSceneCommand>, editor_core::EditorMutationError> {
    let executed = CommandExecutor::execute_command(context, command)?;
    Ok(executed.map(|executed| ExecutedSceneCommand {
        metadata: executed.metadata,
    }))
}

pub fn execute_scene_command_and_push_history(
    context: &mut SceneCommandContext<'_>,
    command: &mut SceneEditorCommand,
    transaction: TransactionMetadata,
) -> Result<Option<ExecutedSceneCommand>, GoverningChangeError> {
    let executed = CommandExecutor::execute_command(context, command)
        .map_err(|error| GoverningChangeError::mutation_rejected(error.message))?;

    let Some(executed) = executed else {
        return Ok(None);
    };

    context
        .session_mut()
        .history_mut()
        .push_applied(HistoryEntry::new(
            transaction,
            vec![executed.metadata.clone()],
        ));

    Ok(Some(ExecutedSceneCommand {
        metadata: executed.metadata,
    }))
}
