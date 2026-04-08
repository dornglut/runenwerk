use ui_math::UiRect;
use ui_render_data::UiFrame;

#[derive(Debug, Clone, PartialEq)]
pub struct HostEditorShellFrame {
	pub bounds: UiRect,
	pub frame: UiFrame,
}

impl HostEditorShellFrame {
	pub fn new(
		bounds: UiRect,
		frame: UiFrame,
	) -> Self {
		Self { bounds, frame }
	}
}