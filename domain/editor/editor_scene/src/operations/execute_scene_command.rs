//! File: domain/editor/editor_scene/src/operations/execute_scene_command.rs
//! Purpose: Single scene-command execution.

use editor_core::{CommandExecutor, CommandMetadata};

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
