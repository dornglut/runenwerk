//! File: domain/editor/editor_scene/src/commands/delete_entities.rs
//! Purpose: Batch entity deletion command with hierarchy-aware undo support.

use editor_core::{EditorMutationError, EntityId};

use crate::{SceneCommandContext, SceneEntitySnapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteEntitiesCommand {
    pub entities: Vec<EntityId>,
    deleted: Vec<SceneEntitySnapshot>,
}

impl DeleteEntitiesCommand {
    pub fn new(entities: Vec<EntityId>) -> Self {
        Self {
            entities,
            deleted: Vec::new(),
        }
    }

    pub fn apply(&mut self, ctx: &mut SceneCommandContext) -> Result<(), EditorMutationError> {
        self.deleted.clear();
        let mut ordered = Vec::new();
        for entity in self.entities.iter().copied() {
            collect_subtree_postorder(ctx, entity, &mut ordered);
        }

        for entity in ordered {
            self.deleted.push(ctx.runtime_mut().delete_entity(entity)?);
        }
        Ok(())
    }

    pub fn undo(&mut self, ctx: &mut SceneCommandContext) -> Result<(), EditorMutationError> {
        for snapshot in self.deleted.iter().rev().cloned() {
            ctx.runtime_mut().restore_entity(snapshot)?;
        }
        Ok(())
    }
}

fn collect_subtree_postorder(
    ctx: &SceneCommandContext<'_>,
    entity: EntityId,
    ordered: &mut Vec<EntityId>,
) {
    for child in ctx.runtime().children_of(Some(entity)) {
        collect_subtree_postorder(ctx, child, ordered);
    }
    if !ordered.contains(&entity) {
        ordered.push(entity);
    }
}
