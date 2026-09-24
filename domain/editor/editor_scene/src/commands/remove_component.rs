//! File: domain/editor/editor_scene/src/commands/remove_component.rs
//! Purpose: Remove component command with undo support.

use editor_core::{ComponentTypeId, EditorMutationError, EntityId};

use crate::{SceneCommandContext, SceneComponentSnapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveComponentCommand {
    pub entity: EntityId,
    pub component_type: ComponentTypeId,
    removed_snapshot: Option<SceneComponentSnapshot>,
}

impl RemoveComponentCommand {
    pub fn new(entity: EntityId, component_type: ComponentTypeId) -> Self {
        Self {
            entity,
            component_type,
            removed_snapshot: None,
        }
    }

    pub fn apply(&mut self, ctx: &mut SceneCommandContext) -> Result<(), EditorMutationError> {
        let snapshot = ctx
            .runtime_mut()
            .remove_component(self.entity, self.component_type)?;
        self.removed_snapshot = Some(snapshot);
        Ok(())
    }

    pub fn undo(&mut self, ctx: &mut SceneCommandContext) -> Result<(), EditorMutationError> {
        let Some(snapshot) = self.removed_snapshot.clone() else {
            return Ok(());
        };

        ctx.runtime_mut().restore_component(snapshot)?;
        Ok(())
    }
}
