//! File: domain/editor/editor_shell/src/composition/build_inspector_panel.rs
//! Purpose: Compose inspector panel widgets.

use ui_runtime::{label, panel, vstack, UiNode};
use ui_text::FontId;
use ui_theme::ThemeTokens;

use crate::{
	inspector_field_widget_id, InspectorTargetViewModel, InspectorViewModel,
	INSPECTOR_LIST_WIDGET_ID, INSPECTOR_PANEL_WIDGET_ID, INSPECTOR_TITLE_WIDGET_ID,
};

pub fn build_inspector_panel(
	view_model: &InspectorViewModel,
	theme: &ThemeTokens,
) -> UiNode {
	let title = label(
		INSPECTOR_TITLE_WIDGET_ID,
		"Inspector",
		theme.heading_text_style(FontId(1)),
	);

	let target_label = match &view_model.target {
		InspectorTargetViewModel::Empty => "Nothing selected".to_string(),
		InspectorTargetViewModel::Entity { display_name } => {
			format!("Entity: {display_name}")
		}
		InspectorTargetViewModel::Component {
			entity_display_name,
			component_display_name,
		} => format!("{entity_display_name} / {component_display_name}"),
		InspectorTargetViewModel::Resource { display_name } => {
			format!("Resource: {display_name}")
		}
		InspectorTargetViewModel::Unsupported { label } => {
			format!("Unsupported: {label}")
		}
		InspectorTargetViewModel::Error { message } => {
			format!("Error: {message}")
		}
	};

	let mut rows = vec![label(
		INSPECTOR_LIST_WIDGET_ID,
		target_label,
		theme.body_text_style(FontId(1)),
	)];

	for (index, field) in view_model.fields.iter().enumerate() {
		rows.push(label(
			inspector_field_widget_id(index),
			format!("{}: {}", field.label, field.value_summary),
			theme.body_small_text_style(FontId(1)),
		));
	}

	let list = vstack(INSPECTOR_LIST_WIDGET_ID, theme.spacing.xs, rows);
	panel(INSPECTOR_PANEL_WIDGET_ID, theme.clone(), vec![title, list])
}