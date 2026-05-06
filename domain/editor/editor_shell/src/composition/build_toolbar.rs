//! File: domain/editor/editor_shell/src/composition/build_toolbar.rs
//! Purpose: Compose toolbar widgets from toolbar view model.

use crate::{
    UiNode, UiNodeKind, button_selected, hscroll, hstack, label, panel, vstack_with_policies,
};
use ui_layout::SizePolicy;
use ui_text::FontId;
use ui_theme::{ThemeTokens, UiColor};

use crate::{
    TOOLBAR_ADD_WORKSPACE_WIDGET_ID, TOOLBAR_DEBUG_LOGS_BUTTON_WIDGET_ID,
    TOOLBAR_EDIT_MENU_WIDGET_ID, TOOLBAR_FILE_MENU_WIDGET_ID, TOOLBAR_LOAD_BUTTON_WIDGET_ID,
    TOOLBAR_MENU_ROW_WIDGET_ID, TOOLBAR_MODELLING_WORKSPACE_WIDGET_ID,
    TOOLBAR_REDO_BUTTON_WIDGET_ID, TOOLBAR_ROOT_WIDGET_ID, TOOLBAR_ROTATE_BUTTON_WIDGET_ID,
    TOOLBAR_ROW_WIDGET_ID, TOOLBAR_ROWS_WIDGET_ID, TOOLBAR_SAVE_BUTTON_WIDGET_ID,
    TOOLBAR_SCALE_BUTTON_WIDGET_ID, TOOLBAR_SCENE_WORKSPACE_WIDGET_ID, TOOLBAR_SCROLL_WIDGET_ID,
    TOOLBAR_SELECT_BUTTON_WIDGET_ID, TOOLBAR_SEPARATOR_WIDGET_ID,
    TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID, TOOLBAR_UNDO_BUTTON_WIDGET_ID,
    TOOLBAR_WINDOW_MENU_WIDGET_ID, ToolbarViewModel, toolbar_menu_item_widget_id,
};

pub fn build_toolbar(view_model: &ToolbarViewModel, theme: &ThemeTokens) -> UiNode {
    let text_style = theme.body_text_style(FontId(1));

    let mut top_row = Vec::new();
    let mut menu_row = Vec::new();

    for (index, button_vm) in view_model.buttons.iter().enumerate() {
        let widget_id = match button_vm.stable_name {
            "menu_file" => TOOLBAR_FILE_MENU_WIDGET_ID,
            "menu_edit" => TOOLBAR_EDIT_MENU_WIDGET_ID,
            "menu_window" => TOOLBAR_WINDOW_MENU_WIDGET_ID,
            "separator" => TOOLBAR_SEPARATOR_WIDGET_ID,
            "workspace_scene" => TOOLBAR_SCENE_WORKSPACE_WIDGET_ID,
            "workspace_modelling" => TOOLBAR_MODELLING_WORKSPACE_WIDGET_ID,
            "workspace_plus" => TOOLBAR_ADD_WORKSPACE_WIDGET_ID,
            "select" => TOOLBAR_SELECT_BUTTON_WIDGET_ID,
            "translate" => TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID,
            "rotate" => TOOLBAR_ROTATE_BUTTON_WIDGET_ID,
            "scale" => TOOLBAR_SCALE_BUTTON_WIDGET_ID,
            "undo" => TOOLBAR_UNDO_BUTTON_WIDGET_ID,
            "redo" => TOOLBAR_REDO_BUTTON_WIDGET_ID,
            "save" => TOOLBAR_SAVE_BUTTON_WIDGET_ID,
            "load" => TOOLBAR_LOAD_BUTTON_WIDGET_ID,
            "debug_logs" => TOOLBAR_DEBUG_LOGS_BUTTON_WIDGET_ID,
            name if is_menu_item_name(name) => toolbar_menu_item_widget_id(index),
            _ => toolbar_menu_item_widget_id(index),
        };

        let node = if button_vm.stable_name == "separator" {
            label(widget_id, button_vm.label.clone(), text_style.clone())
        } else {
            let mut node = button_selected(
                widget_id,
                button_vm.label.clone(),
                text_style.clone(),
                theme.clone(),
                button_vm.is_active,
            );
            if let UiNodeKind::Button(button_node) = &mut node.kind {
                button_node.enabled = button_vm.enabled;
            }
            node
        };
        if is_menu_item_name(button_vm.stable_name) {
            menu_row.push(node);
        } else {
            top_row.push(node);
        }
    }

    let top_row = hstack(TOOLBAR_ROW_WIDGET_ID, theme.spacing.sm, top_row);
    let mut rows = vec![top_row];
    let mut row_policies = vec![SizePolicy::Auto];
    if !menu_row.is_empty() {
        rows.push(hstack(
            TOOLBAR_MENU_ROW_WIDGET_ID,
            theme.spacing.xs,
            menu_row,
        ));
        row_policies.push(SizePolicy::Auto);
    }
    let toolbar_rows = vstack_with_policies(
        TOOLBAR_ROWS_WIDGET_ID,
        theme.spacing.xs * 0.5,
        row_policies,
        rows,
    );
    let row_scroll = hscroll(TOOLBAR_SCROLL_WIDGET_ID, theme.clone(), vec![toolbar_rows]);
    let mut toolbar_theme = theme.clone();
    toolbar_theme.background_panel = UiColor::new(
        (theme.background_panel.r + 0.02).clamp(0.0, 1.0),
        (theme.background_panel.g + 0.02).clamp(0.0, 1.0),
        (theme.background_panel.b + 0.03).clamp(0.0, 1.0),
        0.95,
    );
    panel(TOOLBAR_ROOT_WIDGET_ID, toolbar_theme, vec![row_scroll])
}

fn is_menu_item_name(stable_name: &str) -> bool {
    stable_name.starts_with("file_")
        || stable_name.starts_with("edit_")
        || stable_name.starts_with("window_")
}
