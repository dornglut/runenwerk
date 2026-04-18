use editor_core::{ChangeOrigin, GoverningChangeError, RatifiedChange};
use editor_scene::{SceneCommandIntent, SceneEditorCommand};

use crate::editor_runtime::{
    RunenwerkEditorRuntime, ratify_scene_command, ratify_scene_intent, ratify_scene_redo,
    ratify_scene_transaction, ratify_scene_undo,
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
    let result = ratify_scene_intent(runtime, transaction_label, intent, origin)?;

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
    let result = ratify_scene_command(runtime, transaction_label, command, origin)?;

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
    let result = ratify_scene_transaction(runtime, transaction_label, commands, origin)?;

    if result.is_none() {
        return Ok(());
    }

    Ok(())
}

pub fn undo_last_scene_change(
    runtime: &mut RunenwerkEditorRuntime,
    origin: ChangeOrigin,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    ratify_scene_undo(runtime, origin)
}

pub fn redo_last_scene_change(
    runtime: &mut RunenwerkEditorRuntime,
    origin: ChangeOrigin,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    ratify_scene_redo(runtime, origin)
}
