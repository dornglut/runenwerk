//! File: domain/editor/editor_scene/src/commands/create_sdf_primitive.rs
//! Purpose: Create SDF primitive scene entities through runtime-owned ECS integration.

use editor_core::{EditorMutationError, EntityId};

use crate::{SceneCommandContext, SceneEntitySnapshot, SdfPrimitiveSpec};

#[derive(Debug, Clone, PartialEq)]
pub struct CreateSdfPrimitiveCommand {
    pub parent: Option<EntityId>,
    pub display_name: String,
    pub primitive: SdfPrimitiveSpec,
    created: Option<SceneEntitySnapshot>,
}

impl CreateSdfPrimitiveCommand {
    pub fn new(
        parent: Option<EntityId>,
        display_name: impl Into<String>,
        primitive: SdfPrimitiveSpec,
    ) -> Self {
        Self {
            parent,
            display_name: display_name.into(),
            primitive,
            created: None,
        }
    }

    pub fn apply(&mut self, ctx: &mut SceneCommandContext) -> Result<(), EditorMutationError> {
        let entity = ctx.runtime_mut().create_sdf_primitive(
            self.parent,
            &self.display_name,
            self.primitive.clone(),
        )?;
        self.created = Some(SceneEntitySnapshot::new(
            entity,
            self.display_name.clone(),
            self.parent,
        ));
        Ok(())
    }

    pub fn undo(&mut self, ctx: &mut SceneCommandContext) -> Result<(), EditorMutationError> {
        let Some(snapshot) = &self.created else {
            return Ok(());
        };
        let _ = ctx.runtime_mut().delete_entity(snapshot.id)?;
        Ok(())
    }
}
