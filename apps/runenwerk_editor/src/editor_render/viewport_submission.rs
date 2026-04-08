use crate::editor_host::HostViewportFrameState;
use crate::editor_tools_state::TranslateAxis;
use editor_core::EntityId;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EditorViewportRenderSubmission {
	pub selected_entity: Option<EntityId>,
	pub hovered_entity: Option<EntityId>,
	pub overlays: Vec<ViewportOverlaySubmission>,
}

impl EditorViewportRenderSubmission {
	pub fn from_host_frame(
		frame: &HostViewportFrameState,
	) -> Self {
		let mut overlays = Vec::new();

		if let Some(entity) = frame.selected_entity {
			overlays.push(ViewportOverlaySubmission::SelectionOutline { entity });
		}

		if let Some(entity) = frame.hovered_entity {
			overlays.push(ViewportOverlaySubmission::HoverOutline { entity });
		}

		if let Some(entity) = frame.active_entity {
			overlays.push(ViewportOverlaySubmission::ActiveEntity { entity });
		}

		if let Some(axis) = frame.active_axis {
			overlays.push(ViewportOverlaySubmission::ActiveTranslateAxis { axis });
		}

		if let Some(axis) = frame.active_translate_axis {
			overlays.push(ViewportOverlaySubmission::PreviewTranslateAxis { axis });
		}

		if let Some(preview) = &frame.active_preview {
			overlays.push(ViewportOverlaySubmission::TranslationPreview {
				entity: preview.entity,
				delta: preview.translation_delta,
			});
		}

		if frame.drag_in_progress {
			overlays.push(ViewportOverlaySubmission::DragInProgress);
		}

		Self {
			selected_entity: frame.selected_entity,
			hovered_entity: frame.hovered_entity,
			overlays,
		}
	}

	pub fn overlay_count(&self) -> usize {
		self.overlays.len()
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewportOverlaySubmission {
	SelectionOutline {
		entity: EntityId,
	},
	HoverOutline {
		entity: EntityId,
	},
	ActiveEntity {
		entity: EntityId,
	},
	ActiveTranslateAxis {
		axis: TranslateAxis,
	},
	PreviewTranslateAxis {
		axis: TranslateAxis,
	},
	TranslationPreview {
		entity: EntityId,
		delta: scene::Vec3Value,
	},
	DragInProgress,
}