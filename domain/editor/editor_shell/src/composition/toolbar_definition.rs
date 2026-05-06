//! File: domain/editor/editor_shell/src/composition/toolbar_definition.rs
//! Purpose: Form the editor toolbar from UI definition data.

use crate::{
    TOOLBAR_ADD_WORKSPACE_WIDGET_ID, TOOLBAR_EDIT_MENU_WIDGET_ID, TOOLBAR_FILE_MENU_WIDGET_ID,
    TOOLBAR_MENU_ROW_WIDGET_ID, TOOLBAR_MODELLING_WORKSPACE_WIDGET_ID, TOOLBAR_ROOT_WIDGET_ID,
    TOOLBAR_ROTATE_BUTTON_WIDGET_ID, TOOLBAR_ROW_WIDGET_ID, TOOLBAR_ROWS_WIDGET_ID,
    TOOLBAR_SCALE_BUTTON_WIDGET_ID, TOOLBAR_SCENE_WORKSPACE_WIDGET_ID, TOOLBAR_SCROLL_WIDGET_ID,
    TOOLBAR_SELECT_BUTTON_WIDGET_ID, TOOLBAR_SEPARATOR_WIDGET_ID,
    TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID, TOOLBAR_WINDOW_MENU_WIDGET_ID, ToolbarViewModel,
    toolbar_menu_item_widget_id,
};
use ui_definition::{
    AuthoredUiNodePath, AuthoredUiTemplate, FormedRetainedUiProduct, UiAvailability,
    UiCollectionItem, UiDefinitionContext, UiNodeId, UiRouteSlotId, UiValue, form_retained_ui,
    normalize_authored_template,
};
use ui_theme::ThemeTokens;

const TOOLBAR_TEMPLATE_RON: &str = include_str!("../../../../../assets/editor/ui/toolbar.ron");

pub fn build_defined_toolbar(
    view_model: &ToolbarViewModel,
    theme: &ThemeTokens,
) -> FormedRetainedUiProduct {
    let template: AuthoredUiTemplate =
        ron::from_str(TOOLBAR_TEMPLATE_RON).expect("checked-in toolbar UI fixture must parse");
    let normalized = normalize_authored_template(template);
    let mut context = UiDefinitionContext::new(theme.clone());
    register_toolbar_widget_ids(&mut context, view_model);
    populate_toolbar_values(&mut context, view_model);
    form_retained_ui(&normalized, &mut context)
}

fn register_toolbar_widget_ids(context: &mut UiDefinitionContext, view_model: &ToolbarViewModel) {
    let mappings = [
        ("root", TOOLBAR_ROOT_WIDGET_ID),
        ("root/scroll", TOOLBAR_SCROLL_WIDGET_ID),
        ("root/scroll/rows", TOOLBAR_ROWS_WIDGET_ID),
        ("root/scroll/rows/top_row", TOOLBAR_ROW_WIDGET_ID),
        (
            "root/scroll/rows/top_row/menu_file",
            TOOLBAR_FILE_MENU_WIDGET_ID,
        ),
        (
            "root/scroll/rows/top_row/menu_edit",
            TOOLBAR_EDIT_MENU_WIDGET_ID,
        ),
        (
            "root/scroll/rows/top_row/menu_window",
            TOOLBAR_WINDOW_MENU_WIDGET_ID,
        ),
        (
            "root/scroll/rows/top_row/separator",
            TOOLBAR_SEPARATOR_WIDGET_ID,
        ),
        (
            "root/scroll/rows/top_row/workspace_scene",
            TOOLBAR_SCENE_WORKSPACE_WIDGET_ID,
        ),
        (
            "root/scroll/rows/top_row/workspace_modelling",
            TOOLBAR_MODELLING_WORKSPACE_WIDGET_ID,
        ),
        (
            "root/scroll/rows/top_row/workspace_plus",
            TOOLBAR_ADD_WORKSPACE_WIDGET_ID,
        ),
        (
            "root/scroll/rows/top_row/select",
            TOOLBAR_SELECT_BUTTON_WIDGET_ID,
        ),
        (
            "root/scroll/rows/top_row/translate",
            TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID,
        ),
        (
            "root/scroll/rows/top_row/rotate",
            TOOLBAR_ROTATE_BUTTON_WIDGET_ID,
        ),
        (
            "root/scroll/rows/top_row/scale",
            TOOLBAR_SCALE_BUTTON_WIDGET_ID,
        ),
        (
            "root/scroll/rows/active_menu_row",
            TOOLBAR_MENU_ROW_WIDGET_ID,
        ),
    ];
    for (path, widget_id) in mappings {
        context
            .widget_ids_by_path
            .insert(AuthoredUiNodePath(path.to_string()), widget_id);
    }

    for (index, button) in view_model.buttons.iter().enumerate() {
        if let Some(route) = route_slot_for_toolbar_name(button.stable_name) {
            let path = format!("root/scroll/rows/active_menu_row/{route}");
            context
                .widget_ids_by_path
                .insert(AuthoredUiNodePath(path), toolbar_menu_item_widget_id(index));
        }
    }
}

fn populate_toolbar_values(context: &mut UiDefinitionContext, view_model: &ToolbarViewModel) {
    let active = |name: &str| {
        view_model
            .buttons
            .iter()
            .find(|button| button.stable_name == name)
            .is_some_and(|button| button.is_active)
    };
    context.values.insert(
        "editor.toolbar.menu.file.active".into(),
        UiValue::Bool(active("menu_file")),
    );
    context.values.insert(
        "editor.toolbar.menu.edit.active".into(),
        UiValue::Bool(active("menu_edit")),
    );
    context.values.insert(
        "editor.toolbar.menu.window.active".into(),
        UiValue::Bool(active("menu_window")),
    );
    context.values.insert(
        "editor.workspace.scene.active".into(),
        UiValue::Bool(active("workspace_scene")),
    );
    context.values.insert(
        "editor.workspace.modelling.active".into(),
        UiValue::Bool(active("workspace_modelling")),
    );
    context.values.insert(
        "editor.tool.select.active".into(),
        UiValue::Bool(active("select")),
    );
    context.values.insert(
        "editor.tool.translate.active".into(),
        UiValue::Bool(active("translate")),
    );
    context.values.insert(
        "editor.tool.rotate.active".into(),
        UiValue::Bool(active("rotate")),
    );
    context.values.insert(
        "editor.tool.scale.active".into(),
        UiValue::Bool(active("scale")),
    );
    context.availability.insert(
        "editor.workspace.create.available".into(),
        UiAvailability::Disabled {
            reason: "workspace authoring is M3.6 scope".to_string(),
        },
    );

    let active_menu_items = view_model
        .buttons
        .iter()
        .filter_map(|button| {
            let route = route_slot_for_toolbar_name(button.stable_name)?;
            let mut item = UiCollectionItem::new(route.0, button.label.clone());
            item.enabled = button.enabled;
            Some(item)
        })
        .collect::<Vec<_>>();
    context
        .menus
        .insert("editor.toolbar.active_menu".into(), active_menu_items);
}

pub fn route_slot_for_toolbar_name(name: &str) -> Option<UiRouteSlotId> {
    let route = match name {
        "file_save" => "editor.toolbar.file.save",
        "file_save_as" => "editor.toolbar.file.save_as",
        "file_open" => "editor.toolbar.file.open",
        "file_open_recent" => "editor.toolbar.file.open_recent",
        "edit_undo" => "editor.toolbar.edit.undo",
        "edit_redo" => "editor.toolbar.edit.redo",
        "edit_preferences" => "editor.toolbar.edit.preferences",
        "window_new_window" => "editor.toolbar.window.new_window",
        "window_next_workspace" => "editor.toolbar.window.next_workspace",
        "window_previous_workspace" => "editor.toolbar.window.previous_workspace",
        "window_save_workspace" => "editor.toolbar.window.save_workspace",
        "window_load_general_scene" => "editor.toolbar.window.load_scene_workspace",
        "window_load_general_modelling" => "editor.toolbar.window.load_modelling_workspace",
        "window_load_custom" => "editor.toolbar.window.load_custom_workspace",
        _ => return None,
    };
    Some(UiRouteSlotId::new(route))
}

#[allow(dead_code)]
fn _typed_path(value: &str) -> AuthoredUiNodePath {
    AuthoredUiNodePath::root(&UiNodeId::new(value))
}
