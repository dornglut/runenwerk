//! File: domain/editor/editor_scene/src/commands/set_transform.rs
//! Purpose: Set and reset scene transform commands with undo support.

use editor_core::{EditorMutationError, EntityId};

use crate::{SceneCommandContext, SceneTransform};

#[derive(Debug, Clone, PartialEq)]
pub struct SetTransformCommand {
    pub entity: EntityId,
    pub transform: SceneTransform,
    previous: Option<SceneTransform>,
}

impl SetTransformCommand {
    pub fn new(entity: EntityId, transform: SceneTransform) -> Self {
        Self {
            entity,
            transform,
            previous: None,
        }
    }

    pub fn apply(&mut self, ctx: &mut SceneCommandContext) -> Result<(), EditorMutationError> {
        if self.previous.is_none() {
            self.previous = Some(ctx.runtime().read_transform(self.entity)?);
        }
        ctx.runtime_mut()
            .write_transform(self.entity, self.transform)
    }

    pub fn undo(&mut self, ctx: &mut SceneCommandContext) -> Result<(), EditorMutationError> {
        let Some(previous) = self.previous else {
            return Ok(());
        };
        ctx.runtime_mut().write_transform(self.entity, previous)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResetTransformCommand {
    inner: SetTransformCommand,
}

impl ResetTransformCommand {
    pub fn new(entity: EntityId) -> Self {
        Self {
            inner: SetTransformCommand::new(entity, SceneTransform::identity()),
        }
    }

    pub fn apply(&mut self, ctx: &mut SceneCommandContext) -> Result<(), EditorMutationError> {
        self.inner.apply(ctx)
    }

    pub fn undo(&mut self, ctx: &mut SceneCommandContext) -> Result<(), EditorMutationError> {
        self.inner.undo(ctx)
    }
}
