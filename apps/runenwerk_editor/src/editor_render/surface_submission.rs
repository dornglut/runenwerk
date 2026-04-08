use crate::editor_host::{HostEditorSurfaceFrame, HostEditorSurfaceLayout};
use crate::editor_render::{EditorUiRenderSubmission, EditorViewportRenderSubmission};

#[derive(Debug, Clone, PartialEq)]
pub struct EditorSurfaceRenderSubmission {
	pub layout: HostEditorSurfaceLayout,
	pub viewport: EditorViewportRenderSubmission,
	pub ui: EditorUiRenderSubmission,
}

impl EditorSurfaceRenderSubmission {
	pub fn from_host_surface_frame(
		frame: &HostEditorSurfaceFrame,
	) -> Self {
		Self {
			layout: frame.layout,
			viewport: EditorViewportRenderSubmission::from_host_frame(&frame.viewport),
			ui: EditorUiRenderSubmission::from_ui_frame(&frame.shell.frame),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.ui.is_empty() && self.viewport.overlays.is_empty()
	}
}