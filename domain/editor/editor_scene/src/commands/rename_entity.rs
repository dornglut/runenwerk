//! File: domain/editor/editor_scene/src/commands/rename_entity.rs
//! Purpose: Rename entity command with undo support.

use editor_core::{EditorMutationError, EntityId};

use crate::SceneCommandContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameEntityCommand {
    pub entity: EntityId,
    pub new_display_name: String,
    previous_display_name: Option<String>,
}

impl RenameEntityCommand {
    pub fn new(entity: EntityId, new_display_name: impl Into<String>) -> Self {
        Self {
            entity,
            new_display_name: new_display_name.into(),
            previous_display_name: None,
        }
    }

    pub fn apply(&mut self, ctx: &mut SceneCommandContext) -> Result<(), EditorMutationError> {
        let previous = ctx
            .runtime_mut()
            .rename_entity(self.entity, &self.new_display_name)?;

        if self.previous_display_name.is_none() {
            self.previous_display_name = Some(previous);
        }

        Ok(())
    }

    pub fn undo(&mut self, ctx: &mut SceneCommandContext) -> Result<(), EditorMutationError> {
        let Some(previous_display_name) = self.previous_display_name.as_deref() else {
            return Ok(());
        };

        let _ = ctx
            .runtime_mut()
            .rename_entity(self.entity, previous_display_name)?;

        Ok(())
    }
}
