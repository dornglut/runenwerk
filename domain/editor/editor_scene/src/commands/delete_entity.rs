//! File: domain/editor/editor_scene/src/commands/delete_entity.rs
//! Purpose: Delete entity command with undo support.

use editor_core::EntityId;

use crate::{SceneCommandContext, SceneEntitySnapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteEntityCommand {
    pub entity: EntityId,
    deleted: Option<SceneEntitySnapshot>,
}

impl DeleteEntityCommand {
    pub fn new(entity: EntityId) -> Self {
        Self {
            entity,
            deleted: None,
        }
    }

    pub fn apply(&mut self, ctx: &mut SceneCommandContext) -> Result<(), &'static str> {
        let deleted = ctx.runtime_mut().delete_entity(self.entity)?;
        self.deleted = Some(deleted);
        Ok(())
    }

    pub fn undo(&mut self, ctx: &mut SceneCommandContext) -> Result<(), &'static str> {
        let Some(snapshot) = self.deleted.clone() else {
            return Ok(());
        };

        ctx.runtime_mut().restore_entity(snapshot)?;
        Ok(())
    }
}
