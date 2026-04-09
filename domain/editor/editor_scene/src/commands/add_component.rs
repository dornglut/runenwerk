//! File: domain/editor/editor_scene/src/commands/add_component.rs
//! Purpose: Add component command with undo support.

use editor_core::{ComponentTypeId, EntityId};

use crate::{SceneCommandContext, SceneComponentSnapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddComponentCommand {
    pub entity: EntityId,
    pub component_type: ComponentTypeId,
    removed_snapshot: Option<SceneComponentSnapshot>,
}

impl AddComponentCommand {
    pub fn new(entity: EntityId, component_type: ComponentTypeId) -> Self {
        Self {
            entity,
            component_type,
            removed_snapshot: None,
        }
    }

    pub fn apply(&mut self, ctx: &mut SceneCommandContext) -> Result<(), &'static str> {
        if let Some(snapshot) = self.removed_snapshot.take() {
            ctx.runtime_mut().restore_component(snapshot)?;
            return Ok(());
        }

        ctx.runtime_mut()
            .add_component(self.entity, self.component_type)
    }

    pub fn undo(&mut self, ctx: &mut SceneCommandContext) -> Result<(), &'static str> {
        let snapshot = ctx
            .runtime_mut()
            .remove_component(self.entity, self.component_type)?;
        self.removed_snapshot = Some(snapshot);
        Ok(())
    }
}
