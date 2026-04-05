//! File: domain/editor/editor_scene/src/commands/edit_resource_field.rs
//! Purpose: Edit resource field command with undo support.

use editor_core::ResourceTypeId;
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
	/// File: domain/editor/editor_scene/src/commands/edit_resource_field.rs
	/// Method: new
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

	/// File: domain/editor/editor_scene/src/commands/edit_resource_field.rs
	/// Method: apply
	pub fn apply(&mut self, ctx: &mut SceneCommandContext) -> Result<(), &'static str> {
		if self.previous_value.is_none() {
			let value = ctx
				.runtime()
				.read_resource_field(self.resource_type, &self.path)
				.map_err(map_edit_error)?;
			self.previous_value = Some(value);
		}

		ctx.runtime_mut()
			.write_resource_field(
				self.resource_type,
				&self.path,
				self.new_value.clone(),
			)
			.map_err(map_edit_error)
	}

	/// File: domain/editor/editor_scene/src/commands/edit_resource_field.rs
	/// Method: undo
	pub fn undo(&mut self, ctx: &mut SceneCommandContext) -> Result<(), &'static str> {
		let Some(previous_value) = self.previous_value.clone() else {
			return Ok(());
		};

		ctx.runtime_mut()
			.write_resource_field(self.resource_type, &self.path, previous_value)
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