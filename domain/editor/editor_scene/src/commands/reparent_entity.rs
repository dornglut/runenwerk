//! File: domain/editor/editor_scene/src/commands/reparent_entity.rs
//! Purpose: Reparent entity command with undo support.

use editor_core::{EditorMutationError, EntityId};

use crate::SceneCommandContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReparentEntityCommand {
    pub entity: EntityId,
    pub new_parent: Option<EntityId>,
    previous_parent: Option<Option<EntityId>>,
}

impl ReparentEntityCommand {
    pub fn new(entity: EntityId, new_parent: Option<EntityId>) -> Self {
        Self {
            entity,
            new_parent,
            previous_parent: None,
        }
    }

    pub fn apply(&mut self, ctx: &mut SceneCommandContext) -> Result<(), EditorMutationError> {
        let old_parent = ctx
            .runtime_mut()
            .reparent_entity(self.entity, self.new_parent)?;
        self.previous_parent = Some(old_parent);
        Ok(())
    }

    pub fn undo(&mut self, ctx: &mut SceneCommandContext) -> Result<(), EditorMutationError> {
        let Some(previous_parent) = self.previous_parent else {
            return Ok(());
        };

        ctx.runtime_mut()
            .reparent_entity(self.entity, previous_parent)?;
        Ok(())
    }
}
