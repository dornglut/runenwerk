//! File: domain/editor/editor_scene/src/command.rs
//! Purpose: Executable scene command placeholders built from scene intents.

use editor_core::{
	Command, CommandContext, CommandId, CommandMetadata, CommandOutcome,
};

use crate::SceneCommandIntent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneEditorCommand {
	metadata: CommandMetadata,
	pub intent: SceneCommandIntent,
}

impl SceneEditorCommand {
	pub fn new(
		id: CommandId,
		label: impl Into<String>,
		intent: SceneCommandIntent,
	) -> Self {
		Self {
			metadata: CommandMetadata::new(id, label),
			intent,
		}
	}
}

impl Command for SceneEditorCommand {
	type Context = editor_core::EditorSession;
	type Error = &'static str;

	fn metadata(&self) -> &CommandMetadata {
		&self.metadata
	}

	fn apply(&mut self, _ctx: &mut Self::Context) -> Result<CommandOutcome, Self::Error> {
		Ok(CommandOutcome::Applied)
	}

	fn undo(&mut self, _ctx: &mut Self::Context) -> Result<CommandOutcome, Self::Error> {
		Ok(CommandOutcome::Applied)
	}
}