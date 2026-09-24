//! File: apps/runenwerk_editor/src/editor_runtime/ratification/mod.rs
//! Purpose: Single governing-change ingress for scene-authoring ratification paths.

use editor_core::{ChangeOrigin, GoverningChangeError, RatifiedChange, TransactionId};
use editor_scene::{SceneCommandIntent, SceneEditorCommand, scene_intent_to_command};

use crate::editor_runtime::{
    RunenwerkEditorRuntime, execute_scene_command_and_push_history_with_origin,
    execute_scene_transaction_and_push_history_with_origin,
    redo_last_scene_transaction_with_origin, undo_last_scene_transaction_with_origin,
};

/// Ratify a scene intent by allocating command/transaction id and executing through the
/// governing mutation pipeline.
pub fn ratify_scene_intent(
    runtime: &mut RunenwerkEditorRuntime,
    transaction_label: impl Into<String>,
    intent: SceneCommandIntent,
    origin: ChangeOrigin,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    let command_id = runtime.allocate_command_id();
    let command = scene_intent_to_command(command_id, intent);
    ratify_scene_command(runtime, transaction_label, command, origin)
}

/// Ratify a pre-built scene command by allocating transaction id and executing through the
/// governing mutation pipeline.
pub fn ratify_scene_command(
    runtime: &mut RunenwerkEditorRuntime,
    transaction_label: impl Into<String>,
    command: SceneEditorCommand,
    origin: ChangeOrigin,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    let transaction_id = runtime.allocate_transaction_id();
    ratify_scene_command_with_transaction_id(
        runtime,
        transaction_label,
        command,
        transaction_id,
        origin,
    )
}

/// Ratify a pre-built scene command using an explicit transaction id.
pub fn ratify_scene_command_with_transaction_id(
    runtime: &mut RunenwerkEditorRuntime,
    transaction_label: impl Into<String>,
    command: SceneEditorCommand,
    transaction_id: TransactionId,
    origin: ChangeOrigin,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    execute_scene_command_and_push_history_with_origin(
        runtime,
        command,
        transaction_label,
        transaction_id,
        origin,
    )
}

/// Ratify a scene transaction by allocating transaction id and executing through the
/// governing mutation pipeline.
pub fn ratify_scene_transaction(
    runtime: &mut RunenwerkEditorRuntime,
    transaction_label: impl Into<String>,
    commands: &mut [SceneEditorCommand],
    origin: ChangeOrigin,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    let transaction_id = runtime.allocate_transaction_id();
    execute_scene_transaction_and_push_history_with_origin(
        runtime,
        transaction_id,
        transaction_label,
        commands,
        origin,
    )
}

/// Ratify undo ingress through the governing mutation pipeline.
pub fn ratify_scene_undo(
    runtime: &mut RunenwerkEditorRuntime,
    origin: ChangeOrigin,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    undo_last_scene_transaction_with_origin(runtime, origin)
}

/// Ratify redo ingress through the governing mutation pipeline.
pub fn ratify_scene_redo(
    runtime: &mut RunenwerkEditorRuntime,
    origin: ChangeOrigin,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    redo_last_scene_transaction_with_origin(runtime, origin)
}
