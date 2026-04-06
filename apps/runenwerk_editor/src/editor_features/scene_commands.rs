use editor_core::{CommandId, TransactionId};
use editor_scene::{SceneCommandIntent, SceneEditorCommand};

use crate::editor_runtime::{
	execute_scene_command_and_push_history, execute_scene_transaction_and_push_history,
	RunenwerkEditorRuntime,
};

pub fn execute_intent_with_history(
	runtime: &mut RunenwerkEditorRuntime,
	transaction_label: impl Into<String>,
	intent: SceneCommandIntent,
) -> Result<(), &'static str> {
	let command_id = runtime.allocate_command_id();
	let transaction_id = runtime.allocate_transaction_id();

	let result = execute_scene_command_and_push_history(
		runtime,
		editor_scene::scene_intent_to_command(command_id, intent),
		transaction_label,
		transaction_id,
	)?;

	if result.is_none() {
		return Ok(());
	}

	Ok(())
}

pub fn execute_command_with_history(
	runtime: &mut RunenwerkEditorRuntime,
	transaction_label: impl Into<String>,
	command: SceneEditorCommand,
) -> Result<(), &'static str> {
	let transaction_id = runtime.allocate_transaction_id();

	let result = execute_scene_command_and_push_history(
		runtime,
		command,
		transaction_label,
		transaction_id,
	)?;

	if result.is_none() {
		return Ok(());
	}

	Ok(())
}

pub fn execute_transaction_with_history(
	runtime: &mut RunenwerkEditorRuntime,
	transaction_label: impl Into<String>,
	commands: &mut [SceneEditorCommand],
) -> Result<(), &'static str> {
	let transaction_id = runtime.allocate_transaction_id();

	let result = execute_scene_transaction_and_push_history(
		runtime,
		transaction_id,
		transaction_label,
		commands,
	)?;

	if result.is_none() {
		return Ok(());
	}

	Ok(())
}

pub fn next_command_id(runtime: &mut RunenwerkEditorRuntime) -> CommandId {
	runtime.allocate_command_id()
}

pub fn next_transaction_id(runtime: &mut RunenwerkEditorRuntime) -> TransactionId {
	runtime.allocate_transaction_id()
}