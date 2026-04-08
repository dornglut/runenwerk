//! File: domain/editor/editor_scene/src/commands/create_entity.rs
//! Purpose: Create entity command with undo support.

use editor_core::EntityId;

use crate::{SceneCommandContext, SceneEntitySnapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateEntityCommand {
    pub parent: Option<EntityId>,
    pub display_name: String,
    created: Option<SceneEntitySnapshot>,
}

impl CreateEntityCommand {
    /// File: domain/editor/editor_scene/src/commands/create_entity.rs
    /// Method: new
    pub fn new(parent: Option<EntityId>, display_name: impl Into<String>) -> Self {
        Self {
            parent,
            display_name: display_name.into(),
            created: None,
        }
    }

    /// File: domain/editor/editor_scene/src/commands/create_entity.rs
    /// Method: apply
    pub fn apply(&mut self, ctx: &mut SceneCommandContext) -> Result<(), &'static str> {
        if let Some(snapshot) = self.created.clone() {
            ctx.runtime_mut().restore_entity(snapshot)?;
            return Ok(());
        }

        let id = ctx
            .runtime_mut()
            .create_entity(self.parent, &self.display_name)?;

        self.created = Some(SceneEntitySnapshot::new(
            id,
            self.display_name.clone(),
            self.parent,
        ));

        Ok(())
    }

    /// File: domain/editor/editor_scene/src/commands/create_entity.rs
    /// Method: undo
    pub fn undo(&mut self, ctx: &mut SceneCommandContext) -> Result<(), &'static str> {
        let Some(snapshot) = &self.created else {
            return Ok(());
        };

        let deleted = ctx.runtime_mut().delete_entity(snapshot.id)?;
        self.created = Some(deleted);
        Ok(())
    }
}
