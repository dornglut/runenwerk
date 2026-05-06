//! File: domain/editor/editor_shell/src/composition/shell_chrome_definition.rs
//! Purpose: Form shell chrome from editor UI definition data.

use crate::{
    EditorShellFrameModel, ProjectedTabStackSlot, ToolSurfaceKind, UiNode, WidgetId,
    tab_close_button_widget_id, tab_stack_close_area_button_widget_id,
    tab_stack_duplicate_button_widget_id, tab_stack_kind_select_widget_id,
    tab_stack_lock_type_toggle_widget_id, tab_stack_new_tab_button_widget_id,
    tab_stack_reset_area_button_widget_id, tab_stack_split_horizontal_button_widget_id,
    tab_stack_split_vertical_button_widget_id, tab_strip_scroll_widget_id,
};
use ui_definition::{
    AuthoredUiNodePath, AuthoredUiTemplate, UiCollectionItem, UiDefinitionContext, UiValue,
    form_retained_ui, normalize_authored_template,
};
use ui_theme::ThemeTokens;

const SHELL_CHROME_TEMPLATE_RON: &str =
    include_str!("../../../../../assets/editor/ui/shell_chrome.ron");

pub fn build_defined_tab_strip_from_frame(
    tab_stack: &ProjectedTabStackSlot,
    frame_model: &EditorShellFrameModel,
    theme: &ThemeTokens,
) -> UiNode {
    let template: AuthoredUiTemplate = ron::from_str(SHELL_CHROME_TEMPLATE_RON)
        .expect("checked-in shell chrome fixture must parse");
    let normalized = normalize_authored_template(template);
    let mut context = UiDefinitionContext::new(theme.clone());
    register_shell_chrome_widget_ids(&mut context, tab_stack);
    populate_shell_chrome_values(&mut context, tab_stack, frame_model);
    form_retained_ui(&normalized, &mut context).root
}

fn register_shell_chrome_widget_ids(
    context: &mut UiDefinitionContext,
    tab_stack: &ProjectedTabStackSlot,
) {
    let tab_stack_id = tab_stack.tab_stack_id;
    let mappings = [
        ("root".to_string(), tab_strip_scroll_widget_id(tab_stack_id)),
        (
            "root/tab_stack_chrome".to_string(),
            tab_stack.tab_strip_widget_id,
        ),
        (
            "root/tab_stack_chrome/surface_select".to_string(),
            tab_stack_kind_select_widget_id(tab_stack_id),
        ),
        (
            "root/tab_stack_chrome/new_tab".to_string(),
            tab_stack_new_tab_button_widget_id(tab_stack_id),
        ),
        (
            "root/tab_stack_chrome/tabs".to_string(),
            WidgetId(2_600_000 + tab_stack_id.raw()),
        ),
        (
            "root/tab_stack_chrome/split_horizontal".to_string(),
            tab_stack_split_horizontal_button_widget_id(tab_stack_id),
        ),
        (
            "root/tab_stack_chrome/split_vertical".to_string(),
            tab_stack_split_vertical_button_widget_id(tab_stack_id),
        ),
        (
            "root/tab_stack_chrome/duplicate".to_string(),
            tab_stack_duplicate_button_widget_id(tab_stack_id),
        ),
        (
            "root/tab_stack_chrome/reset".to_string(),
            tab_stack_reset_area_button_widget_id(tab_stack_id),
        ),
        (
            "root/tab_stack_chrome/lock".to_string(),
            tab_stack_lock_type_toggle_widget_id(tab_stack_id),
        ),
        (
            "root/tab_stack_chrome/close".to_string(),
            tab_stack_close_area_button_widget_id(tab_stack_id),
        ),
    ];
    for (path, widget_id) in mappings {
        context
            .widget_ids_by_path
            .insert(AuthoredUiNodePath(path), widget_id);
    }

    for (index, tab) in tab_stack.tabs.iter().enumerate() {
        let tab_key = index.to_string();
        let base_path = format!("root/tab_stack_chrome/tabs[{tab_key}]/tab_item");
        context.widget_ids_by_path.insert(
            AuthoredUiNodePath(format!("{base_path}/tab")),
            tab.widget_id,
        );
        context.widget_ids_by_path.insert(
            AuthoredUiNodePath(format!("{base_path}/close")),
            tab_close_button_widget_id(tab_stack_id, index),
        );
    }
}

fn populate_shell_chrome_values(
    context: &mut UiDefinitionContext,
    tab_stack: &ProjectedTabStackSlot,
    frame_model: &EditorShellFrameModel,
) {
    let active_tool_surface_kind = tab_stack
        .active_panel
        .and_then(|panel| panel.active_tool_surface)
        .and_then(|surface_id| frame_model.surface(surface_id))
        .map(|surface| surface.surface_kind)
        .unwrap_or(ToolSurfaceKind::Placeholder);
    let tool_surface_kinds = super::build_editor_shell::shell_tool_surface_kinds();
    context.collections.insert(
        "shell.surface_kinds".into(),
        tool_surface_kinds
            .iter()
            .map(|kind| {
                let mut item = UiCollectionItem::new(
                    format!("{kind:?}"),
                    super::build_editor_shell::tool_surface_kind_label(*kind),
                );
                item.selected = *kind == active_tool_surface_kind;
                item
            })
            .collect(),
    );
    context.selections.insert(
        "shell.surface.active".into(),
        format!("{active_tool_surface_kind:?}"),
    );
    context.values.insert(
        "shell.area.locked".into(),
        UiValue::Bool(tab_stack.locked_tool_surface_kind.is_some()),
    );

    let active_panel_id = tab_stack.active_panel.map(|panel| panel.panel_instance_id);
    context.collections.insert(
        "shell.tabs".into(),
        tab_stack
            .tabs
            .iter()
            .enumerate()
            .map(|(index, tab)| {
                let title = tab
                    .panel
                    .active_tool_surface
                    .and_then(|surface_id| frame_model.surface(surface_id))
                    .map(|surface| surface.title.clone())
                    .unwrap_or_else(|| {
                        super::build_editor_shell::panel_kind_label(tab.panel.panel_kind)
                            .to_string()
                    });
                let mut item = UiCollectionItem::new(index.to_string(), title.clone());
                item.selected = active_panel_id == Some(tab.panel.panel_instance_id);
                item.values.insert("tab.label".into(), UiValue::Text(title));
                item.values
                    .insert("tab.active".into(), UiValue::Bool(item.selected));
                item
            })
            .collect(),
    );
}
