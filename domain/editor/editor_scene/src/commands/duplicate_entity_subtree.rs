//! File: domain/editor/editor_scene/src/commands/duplicate_entity_subtree.rs
//! Purpose: Duplicate scene hierarchy command with undo support.

use editor_core::{EditorMutationError, EntityId};

use crate::SceneCommandContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateEntitySubtreeCommand {
    pub source: EntityId,
    pub new_parent: Option<EntityId>,
    pub name_suffix: String,
    duplicated: Vec<EntityId>,
}

impl DuplicateEntitySubtreeCommand {
    pub fn new(
        source: EntityId,
        new_parent: Option<EntityId>,
        name_suffix: impl Into<String>,
    ) -> Self {
        Self {
            source,
            new_parent,
            name_suffix: name_suffix.into(),
            duplicated: Vec::new(),
        }
    }

    pub fn apply(&mut self, ctx: &mut SceneCommandContext) -> Result<(), EditorMutationError> {
        self.duplicated = ctx.runtime_mut().duplicate_entity_subtree(
            self.source,
            self.new_parent,
            &self.name_suffix,
        )?;
        Ok(())
    }

    pub fn undo(&mut self, ctx: &mut SceneCommandContext) -> Result<(), EditorMutationError> {
        for entity in self.duplicated.iter().rev().copied() {
            let _ = ctx.runtime_mut().delete_entity(entity)?;
        }
        Ok(())
    }
}
