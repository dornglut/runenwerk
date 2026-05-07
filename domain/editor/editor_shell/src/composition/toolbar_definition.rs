//! File: domain/editor/editor_shell/src/composition/toolbar_definition.rs
//! Purpose: Form the editor toolbar from UI definition data.

use std::collections::BTreeMap;

use crate::{
    EDITOR_DESIGN_WORKSPACE_PROFILE_ID, MODELLING_WORKSPACE_PROFILE_ID, SCENE_WORKSPACE_PROFILE_ID,
    TOOLBAR_ADD_WORKSPACE_WIDGET_ID, TOOLBAR_EDIT_MENU_WIDGET_ID,
    TOOLBAR_EDITOR_DESIGN_WORKSPACE_WIDGET_ID, TOOLBAR_FILE_MENU_WIDGET_ID,
    TOOLBAR_MENU_POPUP_WIDGET_ID, TOOLBAR_MODELLING_WORKSPACE_WIDGET_ID, TOOLBAR_ROOT_WIDGET_ID,
    TOOLBAR_ROTATE_BUTTON_WIDGET_ID, TOOLBAR_ROW_WIDGET_ID, TOOLBAR_ROWS_WIDGET_ID,
    TOOLBAR_SCALE_BUTTON_WIDGET_ID, TOOLBAR_SCENE_WORKSPACE_WIDGET_ID, TOOLBAR_SCROLL_WIDGET_ID,
    TOOLBAR_SELECT_BUTTON_WIDGET_ID, TOOLBAR_SEPARATOR_WIDGET_ID,
    TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID, TOOLBAR_WINDOW_MENU_WIDGET_ID, ToolbarViewModel,
    WorkspaceProfileId, toolbar_menu_item_widget_id, toolbar_workspace_close_widget_id,
};
use ui_definition::{
    AuthoredUiNodePath, AuthoredUiTemplate, FormedRetainedUiProduct, FormedUiRoute, UiAvailability,
    UiDefinitionContext, UiNodeId, UiRouteSlotId, UiValue, form_retained_ui,
    normalize_authored_template,
};
use ui_math::{UiInsets, UiSize};
use ui_text::FontId;
use ui_theme::{ThemeTokens, UiColor};
use ui_tree::{PopupNode, UiNode, UiNodeKind};
use ui_widgets::{button, button_selected};

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
    let mut product = form_retained_ui(&normalized, &mut context);
    insert_dynamic_workspace_buttons(&mut product, view_model, theme);
    project_workspace_close_buttons(&mut product, view_model, theme);
    compact_toolbar_root(&mut product.root, theme);
    product
}

pub fn build_defined_toolbar_menu_popup(
    view_model: &ToolbarViewModel,
    theme: &ThemeTokens,
) -> Option<FormedRetainedUiProduct> {
    let anchor = active_toolbar_menu_anchor(view_model)?;
    let text_style = theme.body_small_text_style(FontId(1));
    let mut routes_by_widget_id = BTreeMap::new();
    let mut paths_by_widget_id = BTreeMap::new();
    let mut availability_by_widget_id = BTreeMap::new();
    let mut children = Vec::new();

    for (index, button) in view_model.buttons.iter().enumerate() {
        let Some(route) = route_slot_for_toolbar_name(button.stable_name) else {
            continue;
        };
        let widget_id = toolbar_menu_item_widget_id(index);
        let mut child = button_selected(
            widget_id,
            button.label.clone(),
            text_style.clone(),
            theme.clone(),
            button.is_active,
        );
        if let UiNodeKind::Button(button_node) = &mut child.kind {
            button_node.enabled = button.enabled;
            button_node.fill_width = true;
        }
        paths_by_widget_id.insert(
            widget_id,
            AuthoredUiNodePath(format!("root/menu_popup/{}", route.as_str())),
        );
        availability_by_widget_id.insert(
            widget_id,
            if button.enabled {
                UiAvailability::Available
            } else {
                UiAvailability::Disabled {
                    reason: "toolbar menu item unavailable".to_string(),
                }
            },
        );
        if button.enabled {
            routes_by_widget_id.insert(widget_id, FormedUiRoute::RouteSlot(route));
        }
        children.push(child);
    }

    if children.is_empty() {
        return None;
    }

    let root = UiNode::with_children(
        TOOLBAR_MENU_POPUP_WIDGET_ID,
        UiNodeKind::Popup(PopupNode::anchored_bottom_start(anchor, theme.clone())),
        children,
    );
    paths_by_widget_id.insert(
        TOOLBAR_MENU_POPUP_WIDGET_ID,
        AuthoredUiNodePath("root/menu_popup".to_string()),
    );

    Some(FormedRetainedUiProduct {
        root,
        routes_by_widget_id,
        paths_by_widget_id,
        embeds_by_widget_id: BTreeMap::new(),
        diagnostics: Vec::new(),
        availability_by_widget_id,
    })
}

fn compact_toolbar_root(root: &mut UiNode, theme: &ThemeTokens) {
    let UiNodeKind::Panel(panel) = &mut root.kind else {
        return;
    };
    panel.padding = UiInsets::new(theme.spacing.xs, 0.0, theme.spacing.xs, theme.spacing.xs);

    if let Some(row) = root
        .children
        .iter_mut()
        .flat_map(|scroll| scroll.children.iter_mut())
        .flat_map(|rows| rows.children.iter_mut())
        .find(|node| node.id == TOOLBAR_ROW_WIDGET_ID)
        && let UiNodeKind::Stack(stack) = &mut row.kind
    {
        stack.gap = theme.spacing.sm;
    }
}

fn insert_dynamic_workspace_buttons(
    product: &mut FormedRetainedUiProduct,
    view_model: &ToolbarViewModel,
    theme: &ThemeTokens,
) {
    let Some(editor_design) = view_model
        .buttons
        .iter()
        .find(|button| button.stable_name == "workspace_editor_design")
    else {
        return;
    };
    let Some(row) = find_node_mut(&mut product.root, TOOLBAR_ROW_WIDGET_ID) else {
        return;
    };
    if row
        .children
        .iter()
        .any(|child| child.id == TOOLBAR_EDITOR_DESIGN_WORKSPACE_WIDGET_ID)
    {
        return;
    }
    let mut node = button_selected(
        TOOLBAR_EDITOR_DESIGN_WORKSPACE_WIDGET_ID,
        editor_design.label.clone(),
        theme.body_small_text_style(FontId(1)),
        theme.clone(),
        editor_design.is_active,
    );
    if let UiNodeKind::Button(button) = &mut node.kind {
        button.enabled = editor_design.enabled;
    }
    let insert_at = row
        .children
        .iter()
        .position(|child| child.id == TOOLBAR_ADD_WORKSPACE_WIDGET_ID)
        .unwrap_or(row.children.len());
    row.children.insert(insert_at, node);
    let route = UiRouteSlotId::new("editor.workspace.editor_design.activate");
    product.routes_by_widget_id.insert(
        TOOLBAR_EDITOR_DESIGN_WORKSPACE_WIDGET_ID,
        FormedUiRoute::RouteSlot(route),
    );
    product.paths_by_widget_id.insert(
        TOOLBAR_EDITOR_DESIGN_WORKSPACE_WIDGET_ID,
        AuthoredUiNodePath("root/scroll/rows/top_row/workspace_editor_design".to_string()),
    );
}

fn find_node_mut(node: &mut UiNode, widget_id: ui_tree::WidgetId) -> Option<&mut UiNode> {
    if node.id == widget_id {
        return Some(node);
    }
    for child in &mut node.children {
        if let Some(found) = find_node_mut(child, widget_id) {
            return Some(found);
        }
    }
    None
}

fn project_workspace_close_buttons(
    product: &mut FormedRetainedUiProduct,
    view_model: &ToolbarViewModel,
    theme: &ThemeTokens,
) {
    let open_workspace_count = view_model
        .buttons
        .iter()
        .filter(|button| workspace_profile_for_stable_name(button.stable_name).is_some())
        .count();
    for (profile_id, anchor_widget_id, route) in [
        (
            SCENE_WORKSPACE_PROFILE_ID,
            TOOLBAR_SCENE_WORKSPACE_WIDGET_ID,
            "editor.workspace.scene.close",
        ),
        (
            MODELLING_WORKSPACE_PROFILE_ID,
            TOOLBAR_MODELLING_WORKSPACE_WIDGET_ID,
            "editor.workspace.modelling.close",
        ),
        (
            EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
            TOOLBAR_EDITOR_DESIGN_WORKSPACE_WIDGET_ID,
            "editor.workspace.editor_design.close",
        ),
    ] {
        let close_widget_id = toolbar_workspace_close_widget_id(profile_id);
        let Some(row) = find_node_mut(&mut product.root, TOOLBAR_ROW_WIDGET_ID) else {
            return;
        };
        let Some(anchor_index) = row
            .children
            .iter()
            .position(|child| child.id == anchor_widget_id)
        else {
            continue;
        };
        if row.children.iter().any(|child| child.id == close_widget_id) {
            continue;
        }
        let mut close = button(
            close_widget_id,
            "x",
            theme.body_small_text_style(FontId(1)),
            theme.clone(),
        );
        if let UiNodeKind::Button(button) = &mut close.kind {
            style_workspace_close_button(button, theme, anchor_widget_id);
            button.enabled = open_workspace_count > 1;
        }
        row.children.insert(anchor_index + 1, close);
        product.routes_by_widget_id.insert(
            close_widget_id,
            FormedUiRoute::RouteSlot(UiRouteSlotId::new(route)),
        );
        product.paths_by_widget_id.insert(
            close_widget_id,
            AuthoredUiNodePath(format!("root/scroll/rows/top_row/{route}")),
        );
    }
}

fn workspace_profile_for_stable_name(name: &str) -> Option<WorkspaceProfileId> {
    match name {
        "workspace_scene" => Some(SCENE_WORKSPACE_PROFILE_ID),
        "workspace_modelling" => Some(MODELLING_WORKSPACE_PROFILE_ID),
        "workspace_editor_design" => Some(EDITOR_DESIGN_WORKSPACE_PROFILE_ID),
        _ => None,
    }
}

fn style_workspace_close_button(
    button: &mut ui_tree::ButtonNode,
    theme: &ThemeTokens,
    anchor: ui_tree::WidgetId,
) {
    let mut close_theme = theme.clone();
    close_theme.background_panel = UiColor::new(
        theme.background_panel.r,
        theme.background_panel.g,
        theme.background_panel.b,
        0.50,
    );
    button.theme = close_theme;
    button.padding = UiInsets::ZERO;
    button.min_size = UiSize::new(18.0, 18.0);
    button.corner_radius = Some(f32::MAX);
    button.reveal_on_hover_anchor = Some(anchor);
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
            "root/scroll/rows/top_row/workspace_editor_design",
            TOOLBAR_EDITOR_DESIGN_WORKSPACE_WIDGET_ID,
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
        "editor.toolbar.menu.workspace.active".into(),
        UiValue::Bool(active("workspace_plus")),
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
        "editor.workspace.editor_design.active".into(),
        UiValue::Bool(active("workspace_editor_design")),
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
}

fn active_toolbar_menu_anchor(view_model: &ToolbarViewModel) -> Option<ui_tree::WidgetId> {
    view_model
        .buttons
        .iter()
        .find_map(|button| match (button.stable_name, button.is_active) {
            ("menu_file", true) => Some(TOOLBAR_FILE_MENU_WIDGET_ID),
            ("menu_edit", true) => Some(TOOLBAR_EDIT_MENU_WIDGET_ID),
            ("menu_window", true) => Some(TOOLBAR_WINDOW_MENU_WIDGET_ID),
            ("workspace_plus", true) => Some(TOOLBAR_ADD_WORKSPACE_WIDGET_ID),
            _ => None,
        })
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
        "window_load_custom" => "editor.toolbar.window.load_custom_workspace",
        "workspace_menu_scene" => "editor.workspace.scene.activate",
        "workspace_menu_modelling" => "editor.workspace.modelling.activate",
        "workspace_menu_editor_design" => "editor.workspace.editor_design.activate",
        "workspace_scene_close" => "editor.workspace.scene.close",
        "workspace_modelling_close" => "editor.workspace.modelling.close",
        "workspace_editor_design_close" => "editor.workspace.editor_design.close",
        _ => return None,
    };
    Some(UiRouteSlotId::new(route))
}

#[allow(dead_code)]
fn _typed_path(value: &str) -> AuthoredUiNodePath {
    AuthoredUiNodePath::root(&UiNodeId::new(value))
}
