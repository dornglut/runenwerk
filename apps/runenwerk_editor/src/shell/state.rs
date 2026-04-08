use ui_math::UiRect;
use ui_runtime::{UiRuntime, UiTree};

#[derive(Debug, Default)]
pub struct RunenwerkEditorShellState {
	ui_runtime: UiRuntime,
	last_tree: Option<UiTree>,
	last_bounds: Option<UiRect>,
}

impl RunenwerkEditorShellState {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn ui_runtime(&self) -> &UiRuntime {
		&self.ui_runtime
	}

	pub fn ui_runtime_mut(&mut self) -> &mut UiRuntime {
		&mut self.ui_runtime
	}

	pub fn last_tree(&self) -> Option<&UiTree> {
		self.last_tree.as_ref()
	}

	pub fn set_last_tree(
		&mut self,
		tree: UiTree,
	) {
		self.last_tree = Some(tree);
	}

	pub fn last_bounds(&self) -> Option<UiRect> {
		self.last_bounds
	}

	pub fn set_last_bounds(
		&mut self,
		bounds: UiRect,
	) {
		self.last_bounds = Some(bounds);
	}

	pub fn clear_cached_projection(&mut self) {
		self.last_tree = None;
		self.last_bounds = None;
	}
}