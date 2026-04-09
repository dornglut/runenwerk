use editor_core::{Command, CommandExecutor, HistoryEntry};
use editor_scene::SceneCommandContext;

use crate::editor_runtime::{RunenwerkEditorRuntime, StoredSceneTransaction};

pub fn undo_last_scene_transaction(
    runtime: &mut RunenwerkEditorRuntime,
) -> Result<Option<HistoryEntry>, &'static str> {
    let Some(history_entry) = runtime.session_mut().history_mut().pop_undo() else {
        return Ok(None);
    };

    let transaction_id = history_entry.transaction.id;
    let Some(mut stored) = runtime.command_store_mut().take_applied(transaction_id) else {
        return Err("missing stored scene transaction for undo");
    };

    {
        let (session, mut scene_runtime) = runtime.session_and_scene_runtime();
        let mut ctx = SceneCommandContext::new(session, &mut scene_runtime);

        for command in stored.commands.iter_mut().rev() {
            command.undo(&mut ctx)?;
        }
    }

    runtime
        .command_store_mut()
        .store_redo(StoredSceneTransaction::new(transaction_id, stored.commands));

    runtime
        .session_mut()
        .history_mut()
        .push_redo(history_entry.clone());

    Ok(Some(history_entry))
}

pub fn redo_last_scene_transaction(
    runtime: &mut RunenwerkEditorRuntime,
) -> Result<Option<HistoryEntry>, &'static str> {
    let Some(history_entry) = runtime.session_mut().history_mut().pop_redo() else {
        return Ok(None);
    };

    let transaction_id = history_entry.transaction.id;
    let Some(mut stored) = runtime.command_store_mut().take_redo(transaction_id) else {
        return Err("missing stored scene transaction for redo");
    };

    {
        let (session, mut scene_runtime) = runtime.session_and_scene_runtime();
        let mut ctx = SceneCommandContext::new(session, &mut scene_runtime);

        for command in stored.commands.iter_mut() {
            CommandExecutor::execute_command(&mut ctx, command)?;
        }
    }

    runtime
        .command_store_mut()
        .store_applied(StoredSceneTransaction::new(transaction_id, stored.commands));

    runtime
        .session_mut()
        .history_mut()
        .push_applied(history_entry.clone());

    Ok(Some(history_entry))
}
