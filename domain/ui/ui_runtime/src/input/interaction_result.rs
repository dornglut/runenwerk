//! File: domain/ui/ui_runtime/src/input/interaction_result.rs
//! Purpose: Semantic UI interaction outcomes produced by runtime input handling.

use crate::WidgetId;
use ui_input::FocusChange;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiInteraction {
	Activated(WidgetId),
	HoveredChanged {
		previous: Option<WidgetId>,
		current: Option<WidgetId>,
	},
	PressedChanged {
		previous: Option<WidgetId>,
		current: Option<WidgetId>,
	},
	FocusChanged(FocusChange),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UiInteractionResults {
	pub items: Vec<UiInteraction>,
}

impl UiInteractionResults {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn push(
		&mut self,
		interaction: UiInteraction,
	) {
		self.items.push(interaction);
	}

	pub fn is_empty(&self) -> bool {
		self.items.is_empty()
	}
}