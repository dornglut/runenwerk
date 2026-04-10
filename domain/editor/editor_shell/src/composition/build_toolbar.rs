//! File: domain/editor/editor_shell/src/composition/build_toolbar.rs
//! Purpose: Compose toolbar widgets from toolbar view model.

use crate::{UiNode, button, hstack, panel};
use ui_text::FontId;
use ui_theme::ThemeTokens;

use crate::{
    TOOLBAR_DEBUG_LOGS_BUTTON_WIDGET_ID, TOOLBAR_LOAD_BUTTON_WIDGET_ID,
    TOOLBAR_REDO_BUTTON_WIDGET_ID, TOOLBAR_ROOT_WIDGET_ID, TOOLBAR_ROW_WIDGET_ID,
    TOOLBAR_SAVE_BUTTON_WIDGET_ID, TOOLBAR_SELECT_BUTTON_WIDGET_ID,
    TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID, TOOLBAR_UNDO_BUTTON_WIDGET_ID, ToolbarViewModel,
    UiNodeKind,
};

pub fn build_toolbar(view_model: &ToolbarViewModel, theme: &ThemeTokens) -> UiNode {
    let text_style = theme.body_text_style(FontId(1));

    let mut buttons = Vec::new();

    for button_vm in &view_model.buttons {
        let widget_id = match button_vm.stable_name {
            "select" => TOOLBAR_SELECT_BUTTON_WIDGET_ID,
            "translate" => TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID,
            "undo" => TOOLBAR_UNDO_BUTTON_WIDGET_ID,
            "redo" => TOOLBAR_REDO_BUTTON_WIDGET_ID,
            "save" => TOOLBAR_SAVE_BUTTON_WIDGET_ID,
            "load" => TOOLBAR_LOAD_BUTTON_WIDGET_ID,
            "debug_logs" => TOOLBAR_DEBUG_LOGS_BUTTON_WIDGET_ID,
            _ => continue,
        };

        let mut node = button(
            widget_id,
            button_vm.label.clone(),
            text_style.clone(),
            theme.clone(),
        );
        if let UiNodeKind::Button(button_node) = &mut node.kind {
            button_node.enabled = button_vm.enabled;
        }
        buttons.push(node);
    }

    let row = hstack(TOOLBAR_ROW_WIDGET_ID, theme.spacing.sm, buttons);
    panel(TOOLBAR_ROOT_WIDGET_ID, theme.clone(), vec![row])
}
