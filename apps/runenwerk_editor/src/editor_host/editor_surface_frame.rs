use crate::editor_host::{HostEditorShellFrame, HostViewportFrameState, HostEditorSurfaceLayout};

#[derive(Debug, Clone, PartialEq)]
pub struct HostEditorSurfaceFrame {
	pub layout: HostEditorSurfaceLayout,
	pub shell: HostEditorShellFrame,
	pub viewport: HostViewportFrameState,
}

impl HostEditorSurfaceFrame {
	pub fn new(
		layout: HostEditorSurfaceLayout,
		shell: HostEditorShellFrame,
		viewport: HostViewportFrameState,
	) -> Self {
		Self {
			layout,
			shell,
			viewport,
		}
	}
}