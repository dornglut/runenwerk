//! File: domain/editor/editor_shell/src/composition/build_viewport_panel.rs
//! Purpose: Compose viewport panel widgets.

use ui_runtime::{label, panel, vstack, UiNode};
use ui_text::FontId;
use ui_theme::ThemeTokens;

use crate::{ViewportViewModel, VIEWPORT_PANEL_WIDGET_ID, VIEWPORT_STATUS_WIDGET_ID, VIEWPORT_TITLE_WIDGET_ID};

pub fn build_viewport_panel(
	view_model: &ViewportViewModel,
	theme: &ThemeTokens,
) -> UiNode {
	let title = label(
		VIEWPORT_TITLE_WIDGET_ID,
		"Viewport",
		theme.heading_text_style(FontId(1)),
	);

	let status = label(
		VIEWPORT_STATUS_WIDGET_ID,
		format!(
			"selected={:?} hovered={:?} dragging={} preview={}",
			view_model.selected_entity,
			view_model.hovered_entity,
			view_model.drag_in_progress,
			view_model.preview_active
		),
		theme.body_small_text_style(FontId(1)),
	);

	let body = vstack(VIEWPORT_PANEL_WIDGET_ID, theme.spacing.sm, vec![title, status]);
	panel(VIEWPORT_PANEL_WIDGET_ID, theme.clone(), vec![body])
}