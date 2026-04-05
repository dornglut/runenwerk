use editor_core::EntityId;

use crate::editor_runtime::{
	delete_entity_from_outliner, reparent_entity_from_outliner, rename_entity_from_outliner,
	select_entity_from_outliner, OutlinerRow, RunenwerkEditorRuntime,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlinerPanelState {
	pub rows: Vec<OutlinerRow>,
	pub selected_entity: Option<EntityId>,
}

impl OutlinerPanelState {
	/// File: apps/runenwerk_editor/src/editor_panels/outliner_panel.rs
	/// Method: empty
	pub fn empty() -> Self {
		Self {
			rows: Vec::new(),
			selected_entity: None,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutlinerPanelCommand {
	SelectEntity {
		entity: EntityId,
	},
	RenameEntity {
		entity: EntityId,
		new_display_name: String,
	},
	ReparentEntity {
		entity: EntityId,
		new_parent: Option<EntityId>,
	},
	DeleteEntity {
		entity: EntityId,
	},
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlinerPanelCommandResult {
	pub state: OutlinerPanelState,
}

pub struct OutlinerPanelPresenter;

impl OutlinerPanelPresenter {
	/// File: apps/runenwerk_editor/src/editor_panels/outliner_panel.rs
	/// Method: build_state
	pub fn build_state(runtime: &RunenwerkEditorRuntime) -> OutlinerPanelState {
		let rows = runtime.outliner_tree().flatten();
		let selected_entity = runtime.selected_entity();

		OutlinerPanelState {
			rows,
			selected_entity,
		}
	}

	/// File: apps/runenwerk_editor/src/editor_panels/outliner_panel.rs
	/// Method: dispatch
	pub fn dispatch(
		runtime: &mut RunenwerkEditorRuntime,
		command: OutlinerPanelCommand,
	) -> Result<OutlinerPanelCommandResult, &'static str> {
		match command {
			OutlinerPanelCommand::SelectEntity { entity } => {
				select_entity_from_outliner(runtime, entity)?;
			}
			OutlinerPanelCommand::RenameEntity {
				entity,
				new_display_name,
			} => {
				rename_entity_from_outliner(runtime, entity, new_display_name)?;
			}
			OutlinerPanelCommand::ReparentEntity { entity, new_parent } => {
				reparent_entity_from_outliner(runtime, entity, new_parent)?;
			}
			OutlinerPanelCommand::DeleteEntity { entity } => {
				delete_entity_from_outliner(runtime, entity)?;
			}
		}

		Ok(OutlinerPanelCommandResult {
			state: Self::build_state(runtime),
		})
	}
}