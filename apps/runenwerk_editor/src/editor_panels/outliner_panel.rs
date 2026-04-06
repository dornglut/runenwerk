use editor_core::EntityId;

use crate::editor_runtime::{OutlinerRow, RunenwerkEditorRuntime};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlinerPanelState {
	pub rows: Vec<OutlinerRow>,
	pub selected_entity: Option<EntityId>,
}

impl OutlinerPanelState {
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
	pub fn build_state(runtime: &RunenwerkEditorRuntime) -> OutlinerPanelState {
		let rows = runtime.outliner_tree().flatten();
		let selected_entity = runtime.selected_entity();

		OutlinerPanelState {
			rows,
			selected_entity,
		}
	}
}