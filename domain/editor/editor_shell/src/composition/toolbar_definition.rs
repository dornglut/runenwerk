//! File: domain/editor/editor_shell/src/composition/toolbar_definition.rs
//! Purpose: Form the editor toolbar from UI definition data.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    EDITOR_DESIGN_WORKSPACE_PROFILE_ID, MODELLING_WORKSPACE_PROFILE_ID, SCENE_WORKSPACE_PROFILE_ID,
    TOOLBAR_ADD_WORKSPACE_WIDGET_ID, TOOLBAR_EDIT_MENU_WIDGET_ID,
    TOOLBAR_EDITOR_DESIGN_WORKSPACE_WIDGET_ID, TOOLBAR_FILE_MENU_WIDGET_ID,
    TOOLBAR_MENU_POPUP_WIDGET_ID, TOOLBAR_MODELLING_WORKSPACE_WIDGET_ID, TOOLBAR_ROOT_WIDGET_ID,
    TOOLBAR_ROW_WIDGET_ID, TOOLBAR_ROWS_WIDGET_ID, TOOLBAR_SCENE_WORKSPACE_WIDGET_ID,
    TOOLBAR_SCROLL_WIDGET_ID, TOOLBAR_SEPARATOR_WIDGET_ID, TOOLBAR_WINDOW_MENU_WIDGET_ID,
    ToolbarViewModel, WorkspaceProfileId, toolbar_menu_item_widget_id,
    toolbar_workspace_close_overlay_widget_id, toolbar_workspace_close_widget_id,
};
use editor_definition::{EditorDefinitionBindings, EditorToolbarBinding};
use ui_definition::{
    AuthoredUiNodePath, AuthoredUiTemplate, FormedRetainedUiProduct, FormedUiRoute,
    NormalizedUiTemplate, UiAvailability, UiAvailabilityBinding, UiDefinitionContext,
    UiRouteSlotId, UiValue, form_retained_ui, normalize_authored_template,
};
use ui_math::{UiInsets, UiSize};
use ui_text::{FontId, TextVerticalAlign};
use ui_theme::{ThemeTokens, UiColor};
use ui_tree::{
    OverlayAdornmentNode, PopupAlign, PopupFlipPolicy, PopupNode, PopupSide, UiNode, UiNodeKind,
};
use ui_widgets::{button, button_selected};

const TOOLBAR_TEMPLATE_RON: &str = include_str!("../../../../../assets/editor/ui/toolbar.ron");
const EDITOR_BINDINGS_RON: &str =
    include_str!("../../../../../assets/editor/ui/editor_bindings.ron");

pub fn build_defined_toolbar(
    view_model: &ToolbarViewModel,
    theme: &ThemeTokens,
) -> FormedRetainedUiProduct {
    build_defined_toolbar_with_template(view_model, theme, None)
}

pub fn build_defined_toolbar_with_template(
    view_model: &ToolbarViewModel,
    theme: &ThemeTokens,
    template: Option<&NormalizedUiTemplate>,
) -> FormedRetainedUiProduct {
    let checked_in_template: AuthoredUiTemplate =
        ron::from_str(TOOLBAR_TEMPLATE_RON).expect("checked-in toolbar UI fixture must parse");
    let fallback;
    let normalized = if let Some(template) = template {
        template
    } else {
        fallback = normalize_authored_template(checked_in_template);
        &fallback
    };
    let mut context = UiDefinitionContext::new(theme.clone());
    register_toolbar_widget_ids(&mut context);
    populate_toolbar_values(&mut context, view_model);
    let mut product = form_retained_ui(normalized, &mut context);
    insert_dynamic_workspace_buttons(&mut product, view_model, theme);
    project_workspace_close_buttons(&mut product, view_model, theme);
    compact_toolbar_root(&mut product.root, theme);
    product
}

pub fn build_defined_toolbar_menu_popup(
    view_model: &ToolbarViewModel,
    theme: &ThemeTokens,
) -> Option<FormedRetainedUiProduct> {
    build_defined_toolbar_menu_popup_with_binding(view_model, theme, None)
}

pub fn build_defined_toolbar_menu_popup_with_binding(
    view_model: &ToolbarViewModel,
    theme: &ThemeTokens,
    binding: Option<&EditorToolbarBinding>,
) -> Option<FormedRetainedUiProduct> {
    let (active_menu_id, anchor) = active_toolbar_menu_anchor(view_model)?;
    let fallback;
    let toolbar = if let Some(binding) = binding {
        binding
    } else {
        fallback = checked_in_toolbar_binding();
        &fallback
    };
    let mut text_style = theme.body_small_text_style(FontId(1));
    text_style.vertical_align = TextVerticalAlign::CapHeightCenter;
    let mut routes_by_widget_id = BTreeMap::new();
    let mut paths_by_widget_id = BTreeMap::new();
    let mut availability_by_widget_id = BTreeMap::new();
    let mut children = Vec::new();
    let mut used_widget_indices = BTreeSet::new();

    for (fallback_index, item) in toolbar
        .menu_items
        .iter()
        .filter(|item| item.menu_id == active_menu_id)
        .enumerate()
    {
        let preferred_widget_index = view_model
            .buttons
            .iter()
            .position(|button| button.stable_name == item.item_id)
            .unwrap_or(fallback_index);
        let widget_index = next_available_toolbar_menu_item_index(
            preferred_widget_index,
            &mut used_widget_indices,
        );
        let widget_id = toolbar_menu_item_widget_id(widget_index);
        let runtime_enabled = view_model
            .buttons
            .iter()
            .find(|button| button.stable_name == item.item_id)
            .is_none_or(|button| button.enabled);
        let availability = resolve_toolbar_menu_item_availability(
            item.availability.as_ref(),
            view_model,
            runtime_enabled,
        );
        let enabled = availability.is_enabled();
        let mut child = button_selected(
            widget_id,
            item.label.clone(),
            text_style.clone(),
            theme.clone(),
            false,
        );
        if let UiNodeKind::Button(button_node) = &mut child.kind {
            button_node.enabled = enabled;
            button_node.fill_width = true;
        }
        paths_by_widget_id.insert(
            widget_id,
            AuthoredUiNodePath(format!(
                "root/menu_popup/{}/{}",
                active_menu_id, item.item_id
            )),
        );
        availability_by_widget_id.insert(widget_id, availability);
        if enabled {
            routes_by_widget_id.insert(widget_id, FormedUiRoute::RouteSlot(item.route.clone()));
        }
        children.push(child);
    }

    if children.is_empty() {
        return None;
    }

    let root = UiNode::with_children(
        TOOLBAR_MENU_POPUP_WIDGET_ID,
        UiNodeKind::Popup(PopupNode::anchored_outside(
            anchor,
            PopupSide::Bottom,
            PopupAlign::Start,
            PopupFlipPolicy::FlipToFit,
            theme.clone(),
        )),
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

fn next_available_toolbar_menu_item_index(preferred: usize, used: &mut BTreeSet<usize>) -> usize {
    let mut candidate = preferred;
    while used.contains(&candidate) {
        candidate = candidate.saturating_add(1);
    }
    used.insert(candidate);
    candidate
}

fn checked_in_toolbar_binding() -> EditorToolbarBinding {
    ron::from_str::<EditorDefinitionBindings>(EDITOR_BINDINGS_RON)
        .expect("checked-in editor bindings fixture must parse")
        .toolbar
}

fn resolve_toolbar_menu_item_availability(
    binding: Option<&UiAvailabilityBinding>,
    view_model: &ToolbarViewModel,
    runtime_enabled: bool,
) -> UiAvailability {
    let resolved = match binding {
        Some(UiAvailabilityBinding::Static(value)) => value.clone(),
        Some(UiAvailabilityBinding::Ref(id)) => {
            toolbar_availability_for_ref(id.as_str(), view_model)
        }
        None => UiAvailability::Available,
    };
    if runtime_enabled || !resolved.is_enabled() {
        resolved
    } else {
        UiAvailability::Disabled {
            reason: "toolbar menu item unavailable".to_string(),
        }
    }
}

fn toolbar_availability_for_ref(id: &str, view_model: &ToolbarViewModel) -> UiAvailability {
    let enabled = match id {
        "editor.undo.available" => menu_item_enabled(view_model, "edit_undo"),
        "editor.redo.available" => menu_item_enabled(view_model, "edit_redo"),
        _ => None,
    };
    match enabled {
        Some(true) => UiAvailability::Available,
        Some(false) => UiAvailability::Disabled {
            reason: "toolbar menu item unavailable".to_string(),
        },
        None => UiAvailability::Unavailable {
            reason: format!("toolbar availability '{id}' is not supplied"),
        },
    }
}

fn menu_item_enabled(view_model: &ToolbarViewModel, stable_name: &str) -> Option<bool> {
    view_model
        .buttons
        .iter()
        .find(|button| button.stable_name == stable_name)
        .map(|button| button.enabled)
}

fn compact_toolbar_root(root: &mut UiNode, theme: &ThemeTokens) {
    let UiNodeKind::Panel(panel) = &mut root.kind else {
        return;
    };
    panel.padding = UiInsets::new(theme.spacing.xs, 0.0, theme.spacing.xs, theme.spacing.xs);
    apply_compact_toolbar_text_alignment(root);

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

fn apply_compact_toolbar_text_alignment(node: &mut UiNode) {
    if let UiNodeKind::Button(button) = &mut node.kind {
        button.text_style.vertical_align = TextVerticalAlign::CapHeightCenter;
    }
    for child in &mut node.children {
        apply_compact_toolbar_text_alignment(child);
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
        let close_overlay_widget_id = toolbar_workspace_close_overlay_widget_id(profile_id);
        if row
            .children
            .iter()
            .any(|child| child.id == close_widget_id || child.id == close_overlay_widget_id)
        {
            continue;
        }
        let mut close = button(
            close_widget_id,
            "x",
            toolbar_icon_text_style(theme),
            theme.clone(),
        );
        if let UiNodeKind::Button(button) = &mut close.kind {
            style_workspace_close_button(button, theme, anchor_widget_id);
            button.enabled = open_workspace_count > 1;
        }
        row.children.insert(
            anchor_index + 1,
            UiNode::with_children(
                close_overlay_widget_id,
                UiNodeKind::OverlayAdornment(OverlayAdornmentNode::anchored_inside_top_end(
                    anchor_widget_id,
                    theme.spacing.xs,
                )),
                vec![close],
            ),
        );
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

fn toolbar_icon_text_style(theme: &ThemeTokens) -> ui_text::TextStyle {
    let mut text_style = theme.body_small_text_style(FontId(1));
    text_style.vertical_align = TextVerticalAlign::InkBoundsCenter;
    text_style
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

fn register_toolbar_widget_ids(context: &mut UiDefinitionContext) {
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
    ];
    for (path, widget_id) in mappings {
        context
            .widget_ids_by_path
            .insert(AuthoredUiNodePath(path.to_string()), widget_id);
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

fn active_toolbar_menu_anchor(
    view_model: &ToolbarViewModel,
) -> Option<(&'static str, ui_tree::WidgetId)> {
    view_model
        .buttons
        .iter()
        .find_map(|button| match (button.stable_name, button.is_active) {
            ("menu_file", true) => Some(("file", TOOLBAR_FILE_MENU_WIDGET_ID)),
            ("menu_edit", true) => Some(("edit", TOOLBAR_EDIT_MENU_WIDGET_ID)),
            ("menu_window", true) => Some(("window", TOOLBAR_WINDOW_MENU_WIDGET_ID)),
            ("workspace_plus", true) => Some(("workspace", TOOLBAR_ADD_WORKSPACE_WIDGET_ID)),
            _ => None,
        })
}
