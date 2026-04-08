//! File: domain/editor/editor_shell/src/composition/build_toolbar.rs
//! Purpose: Compose toolbar widgets from toolbar view model.

use ui_runtime::{button, hstack, panel, UiNode};
use ui_text::FontId;
use ui_theme::ThemeTokens;

use crate::{
	ToolbarViewModel, TOOLBAR_ROOT_WIDGET_ID, TOOLBAR_ROW_WIDGET_ID,
	TOOLBAR_SELECT_BUTTON_WIDGET_ID, TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID,
};

pub fn build_toolbar(
	view_model: &ToolbarViewModel,
	theme: &ThemeTokens,
) -> UiNode {
	let text_style = theme.body_text_style(FontId(1));

	let mut buttons = Vec::new();

	for button_vm in &view_model.buttons {
		let widget_id = match button_vm.stable_name {
			"select" => TOOLBAR_SELECT_BUTTON_WIDGET_ID,
			"translate" => TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID,
			_ => continue,
		};

		buttons.push(button(
			widget_id,
			button_vm.label.clone(),
			text_style.clone(),
			theme.clone(),
		));
	}

	let row = hstack(TOOLBAR_ROW_WIDGET_ID, theme.spacing.sm, buttons);
	panel(TOOLBAR_ROOT_WIDGET_ID, theme.clone(), vec![row])
}