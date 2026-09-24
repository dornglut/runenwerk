//! File: domain/editor/editor_scene/src/commands/edit_resource_field.rs
//! Purpose: Edit resource field command with undo support.

use editor_core::{EditorMutationError, ResourceTypeId};
use editor_inspector::{InspectorEditError, InspectorEditValue, InspectorPath};

use crate::SceneCommandContext;

#[derive(Debug, Clone, PartialEq)]
pub struct EditResourceFieldCommand {
    pub resource_type: ResourceTypeId,
    pub path: InspectorPath,
    pub new_value: InspectorEditValue,
    previous_value: Option<InspectorEditValue>,
}

impl EditResourceFieldCommand {
    pub fn new(
        resource_type: ResourceTypeId,
        path: InspectorPath,
        new_value: InspectorEditValue,
    ) -> Self {
        Self {
            resource_type,
            path,
            new_value,
            previous_value: None,
        }
    }

    pub fn apply(&mut self, ctx: &mut SceneCommandContext) -> Result<(), EditorMutationError> {
        if self.previous_value.is_none() {
            let value = ctx
                .runtime()
                .read_resource_field(self.resource_type, &self.path)
                .map_err(map_edit_error)?;
            self.previous_value = Some(value);
        }

        ctx.runtime_mut()
            .write_resource_field(self.resource_type, &self.path, self.new_value.clone())
            .map_err(map_edit_error)
    }

    pub fn undo(&mut self, ctx: &mut SceneCommandContext) -> Result<(), EditorMutationError> {
        let Some(previous_value) = self.previous_value.clone() else {
            return Ok(());
        };

        ctx.runtime_mut()
            .write_resource_field(self.resource_type, &self.path, previous_value)
            .map_err(map_edit_error)
    }
}

fn map_edit_error(error: InspectorEditError) -> EditorMutationError {
    match error {
        InspectorEditError::TargetNotFound => {
            EditorMutationError::inspector_rejected("inspector target not found")
        }
        InspectorEditError::TypeNotRegistered => {
            EditorMutationError::inspector_rejected("inspector type not registered")
        }
        InspectorEditError::ValueNotAvailable => {
            EditorMutationError::inspector_rejected("inspector value not available")
        }
        InspectorEditError::InvalidPath => {
            EditorMutationError::inspector_rejected("invalid inspector path")
        }
        InspectorEditError::UnsupportedPathSegment => {
            EditorMutationError::inspector_rejected("unsupported inspector path segment")
        }
        InspectorEditError::UnsupportedValueType { .. } => {
            EditorMutationError::inspector_rejected("unsupported inspector value type")
        }
        InspectorEditError::IntegerOutOfRange { .. } => {
            EditorMutationError::inspector_rejected("integer out of range")
        }
        InspectorEditError::FloatOutOfRange { .. } => {
            EditorMutationError::inspector_rejected("float out of range")
        }
        InspectorEditError::ExpectedEnumField => {
            EditorMutationError::inspector_rejected("expected inspector enum field")
        }
        InspectorEditError::InvalidEnumOption { .. } => {
            EditorMutationError::inspector_rejected("invalid inspector enum option")
        }
    }
}
