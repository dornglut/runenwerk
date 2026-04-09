//! File: domain/editor/editor_scene/src/commands/edit_component_field.rs
//! Purpose: Edit component field command with undo support.

use editor_core::{ComponentTypeId, EntityId};
use editor_inspector::{InspectorEditError, InspectorEditValue, InspectorPath};

use crate::SceneCommandContext;

#[derive(Debug, Clone, PartialEq)]
pub struct EditComponentFieldCommand {
    pub entity: EntityId,
    pub component_type: ComponentTypeId,
    pub path: InspectorPath,
    pub new_value: InspectorEditValue,
    previous_value: Option<InspectorEditValue>,
}

impl EditComponentFieldCommand {
    pub fn new(
        entity: EntityId,
        component_type: ComponentTypeId,
        path: InspectorPath,
        new_value: InspectorEditValue,
    ) -> Self {
        Self {
            entity,
            component_type,
            path,
            new_value,
            previous_value: None,
        }
    }

    pub fn apply(&mut self, ctx: &mut SceneCommandContext) -> Result<(), &'static str> {
        if self.previous_value.is_none() {
            let value = ctx
                .runtime()
                .read_component_field(self.entity, self.component_type, &self.path)
                .map_err(map_edit_error)?;
            self.previous_value = Some(value);
        }

        ctx.runtime_mut()
            .write_component_field(
                self.entity,
                self.component_type,
                &self.path,
                self.new_value.clone(),
            )
            .map_err(map_edit_error)
    }

    pub fn undo(&mut self, ctx: &mut SceneCommandContext) -> Result<(), &'static str> {
        let Some(previous_value) = self.previous_value.clone() else {
            return Ok(());
        };

        ctx.runtime_mut()
            .write_component_field(self.entity, self.component_type, &self.path, previous_value)
            .map_err(map_edit_error)
    }
}

fn map_edit_error(error: InspectorEditError) -> &'static str {
    match error {
        InspectorEditError::TargetNotFound => "inspector target not found",
        InspectorEditError::TypeNotRegistered => "inspector type not registered",
        InspectorEditError::ValueNotAvailable => "inspector value not available",
        InspectorEditError::InvalidPath => "invalid inspector path",
        InspectorEditError::UnsupportedPathSegment => "unsupported inspector path segment",
        InspectorEditError::UnsupportedValueType { .. } => "unsupported inspector value type",
        InspectorEditError::IntegerOutOfRange { .. } => "integer out of range",
        InspectorEditError::FloatOutOfRange { .. } => "float out of range",
    }
}
