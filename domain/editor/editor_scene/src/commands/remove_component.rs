//! File: domain/editor/editor_scene/src/commands/remove_component.rs
//! Purpose: Remove component command with undo support.

use editor_core::{ComponentTypeId, EntityId};

use crate::{SceneCommandContext, SceneComponentSnapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveComponentCommand {
	pub entity: EntityId,
	pub component_type: ComponentTypeId,
	removed_snapshot: Option<SceneComponentSnapshot>,
}

impl RemoveComponentCommand {
	/// File: domain/editor/editor_scene/src/commands/remove_component.rs
	/// Method: new
	pub fn new(entity: EntityId, component_type: ComponentTypeId) -> Self {
		Self {
			entity,
			component_type,
			removed_snapshot: None,
		}
	}

	/// File: domain/editor/editor_scene/src/commands/remove_component.rs
	/// Method: apply
	pub fn apply(&mut self, ctx: &mut SceneCommandContext) -> Result<(), &'static str> {
		let snapshot = ctx
			.runtime_mut()
			.remove_component(self.entity, self.component_type)?;
		self.removed_snapshot = Some(snapshot);
		Ok(())
	}

	/// File: domain/editor/editor_scene/src/commands/remove_component.rs
	/// Method: undo
	pub fn undo(&mut self, ctx: &mut SceneCommandContext) -> Result<(), &'static str> {
		let Some(snapshot) = self.removed_snapshot.clone() else {
			return Ok(());
		};

		ctx.runtime_mut().restore_component(snapshot)?;
		Ok(())
	}
}