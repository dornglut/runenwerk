use editor_core::{ChangeOrigin, CommandId, GoverningChangeError, RatifiedChange, TransactionId};
use editor_scene::{SceneCommandIntent, SceneEditorCommand};

use crate::editor_runtime::{
    RunenwerkEditorRuntime, execute_scene_command_and_push_history_with_origin,
    execute_scene_transaction_and_push_history_with_origin,
    redo_last_scene_transaction_with_origin, undo_last_scene_transaction_with_origin,
};

pub fn execute_intent_with_history(
    runtime: &mut RunenwerkEditorRuntime,
    transaction_label: impl Into<String>,
    intent: SceneCommandIntent,
) -> Result<(), GoverningChangeError> {
    execute_intent_with_history_from_origin(
        runtime,
        transaction_label,
        intent,
        ChangeOrigin::Runtime,
    )
}

pub fn execute_intent_with_history_from_origin(
    runtime: &mut RunenwerkEditorRuntime,
    transaction_label: impl Into<String>,
    intent: SceneCommandIntent,
    origin: ChangeOrigin,
) -> Result<(), GoverningChangeError> {
    let command_id = runtime.allocate_command_id();
    let transaction_id = runtime.allocate_transaction_id();

    let result = execute_scene_command_and_push_history_with_origin(
        runtime,
        editor_scene::scene_intent_to_command(command_id, intent),
        transaction_label,
        transaction_id,
        origin,
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
) -> Result<(), GoverningChangeError> {
    execute_command_with_history_from_origin(
        runtime,
        transaction_label,
        command,
        ChangeOrigin::Runtime,
    )
}

pub fn execute_command_with_history_from_origin(
    runtime: &mut RunenwerkEditorRuntime,
    transaction_label: impl Into<String>,
    command: SceneEditorCommand,
    origin: ChangeOrigin,
) -> Result<(), GoverningChangeError> {
    let transaction_id = runtime.allocate_transaction_id();

    let result = execute_scene_command_and_push_history_with_origin(
        runtime,
        command,
        transaction_label,
        transaction_id,
        origin,
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
) -> Result<(), GoverningChangeError> {
    execute_transaction_with_history_from_origin(
        runtime,
        transaction_label,
        commands,
        ChangeOrigin::Runtime,
    )
}

pub fn execute_transaction_with_history_from_origin(
    runtime: &mut RunenwerkEditorRuntime,
    transaction_label: impl Into<String>,
    commands: &mut [SceneEditorCommand],
    origin: ChangeOrigin,
) -> Result<(), GoverningChangeError> {
    let transaction_id = runtime.allocate_transaction_id();

    let result = execute_scene_transaction_and_push_history_with_origin(
        runtime,
        transaction_id,
        transaction_label,
        commands,
        origin,
    )?;

    if result.is_none() {
        return Ok(());
    }

    Ok(())
}

pub fn undo_last_scene_change(
    runtime: &mut RunenwerkEditorRuntime,
    origin: ChangeOrigin,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    undo_last_scene_transaction_with_origin(runtime, origin)
}

pub fn redo_last_scene_change(
    runtime: &mut RunenwerkEditorRuntime,
    origin: ChangeOrigin,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    redo_last_scene_transaction_with_origin(runtime, origin)
}

pub fn next_command_id(runtime: &mut RunenwerkEditorRuntime) -> CommandId {
    runtime.allocate_command_id()
}

pub fn next_transaction_id(runtime: &mut RunenwerkEditorRuntime) -> TransactionId {
    runtime.allocate_transaction_id()
}
