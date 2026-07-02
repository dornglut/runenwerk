//! File: domain/editor/editor_shell/src/composition/toolbar_definition.rs
//! Purpose: Form the editor toolbar from UI definition data.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    EDITOR_DESIGN_WORKSPACE_PROFILE_ID, MATERIAL_WORKSPACE_PROFILE_ID,
    MODELLING_WORKSPACE_PROFILE_ID, SCENE_WORKSPACE_PROFILE_ID, TOOLBAR_ADD_WORKSPACE_WIDGET_ID,
    TOOLBAR_EDIT_MENU_WIDGET_ID, TOOLBAR_EDITOR_DESIGN_WORKSPACE_WIDGET_ID,
    TOOLBAR_FILE_MENU_WIDGET_ID, TOOLBAR_MATERIALS_WORKSPACE_WIDGET_ID,
    TOOLBAR_MENU_POPUP_LIST_WIDGET_ID, TOOLBAR_MENU_POPUP_SCROLL_WIDGET_ID,
    TOOLBAR_MENU_POPUP_WIDGET_ID, TOOLBAR_MODELLING_WORKSPACE_WIDGET_ID, TOOLBAR_ROOT_WIDGET_ID,
    TOOLBAR_ROW_WIDGET_ID, TOOLBAR_ROWS_WIDGET_ID, TOOLBAR_SCENE_WORKSPACE_WIDGET_ID,
    TOOLBAR_SCROLL_WIDGET_ID, TOOLBAR_SEPARATOR_WIDGET_ID, TOOLBAR_WINDOW_MENU_WIDGET_ID,
    ToolbarViewModel, WidgetId, WorkspaceProfileId, toolbar_menu_item_widget_id,
    toolbar_workspace_active_indicator_widget_id, toolbar_workspace_chrome_widget_id,
    toolbar_workspace_close_widget_id,
};
use editor_core::ToolId;
use editor_definition::{EditorDefinitionBindings, EditorToolbarBinding};
use ui_definition::{
    AuthoredId, AuthoredUiNodePath, AuthoredUiTemplate, FormedChromeSlot, FormedInteractionModel,
    FormedMenuSizing, FormedMenuStackScope, FormedRetainedUiProduct, FormedScrollOwner,
    FormedUiRoute, NormalizedUiTemplate, UiAvailability, UiAvailabilityBinding,
    UiChromeSlotInputPolicyDefinition, UiChromeSlotKindDefinition, UiDefinitionContext,
    UiMenuDismissPolicyDefinition, UiMenuItemWidthDefinition, UiMenuOverflowDefinition,
    UiRouteSlotId, UiScrollBoundaryPolicyDefinition, UiValue, form_retained_ui,
    normalize_authored_template,
};
use ui_math::Axis;
use ui_math::{UiInsets, UiSize};
use ui_text::FontId;
use ui_theme::{ThemeTokens, UiColor};
use ui_tree::{PopupAlign, PopupFlipPolicy, PopupNode, PopupSide, UiNode, UiNodeKind};
use ui_widgets::{button, button_selected, hstack_with_policies};

use super::surface_definition_context::contrast_popup_theme;

const TOOLBAR_TEMPLATE_RON: &str = include_str!("../../../../../assets/editor/ui/toolbar.ron");
const EDITOR_BINDINGS_RON: &str =
    include_str!("../../../../../assets/editor/ui/editor_bindings.ron");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolbarWorkspaceButtonDefinition {
    pub profile_id: WorkspaceProfileId,
    pub tool_id: ToolId,
    pub widget_id: WidgetId,
    pub stable_name: &'static str,
    pub label: &'static str,
    pub activate_route: &'static str,
    pub close_route: &'static str,
    pub menu_item_tool_id: ToolId,
    pub menu_item_stable_name: &'static str,
}

const TOOLBAR_WORKSPACE_BUTTON_DEFINITIONS: [ToolbarWorkspaceButtonDefinition; 4] = [
    ToolbarWorkspaceButtonDefinition {
        profile_id: SCENE_WORKSPACE_PROFILE_ID,
        tool_id: ToolId(3_001),
        widget_id: TOOLBAR_SCENE_WORKSPACE_WIDGET_ID,
        stable_name: "workspace_scene",
        label: "Scene",
        activate_route: "editor.workspace.scene.activate",
        close_route: "editor.workspace.scene.close",
        menu_item_tool_id: ToolId(2_400),
        menu_item_stable_name: "workspace_menu_scene",
    },
    ToolbarWorkspaceButtonDefinition {
        profile_id: MODELLING_WORKSPACE_PROFILE_ID,
        tool_id: ToolId(3_002),
        widget_id: TOOLBAR_MODELLING_WORKSPACE_WIDGET_ID,
        stable_name: "workspace_modelling",
        label: "Modelling",
        activate_route: "editor.workspace.modelling.activate",
        close_route: "editor.workspace.modelling.close",
        menu_item_tool_id: ToolId(2_401),
        menu_item_stable_name: "workspace_menu_modelling",
    },
    ToolbarWorkspaceButtonDefinition {
        profile_id: EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
        tool_id: ToolId(3_004),
        widget_id: TOOLBAR_EDITOR_DESIGN_WORKSPACE_WIDGET_ID,
        stable_name: "workspace_editor_design",
        label: "Editor Design",
        activate_route: "editor.workspace.editor_design.activate",
        close_route: "editor.workspace.editor_design.close",
        menu_item_tool_id: ToolId(2_402),
        menu_item_stable_name: "workspace_menu_editor_design",
    },
    ToolbarWorkspaceButtonDefinition {
        profile_id: MATERIAL_WORKSPACE_PROFILE_ID,
        tool_id: ToolId(3_005),
        widget_id: TOOLBAR_MATERIALS_WORKSPACE_WIDGET_ID,
        stable_name: "workspace_materials",
        label: "Materials",
        activate_route: "editor.workspace.materials.activate",
        close_route: "editor.workspace.materials.close",
        menu_item_tool_id: ToolId(2_403),
        menu_item_stable_name: "workspace_menu_materials",
    },
];

pub fn toolbar_workspace_button_definitions() -> &'static [ToolbarWorkspaceButtonDefinition] {
    &TOOLBAR_WORKSPACE_BUTTON_DEFINITIONS
}

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
    let text_style = theme.body_small_text_style(FontId(1));
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
            contrast_popup_theme(theme),
        )),
        vec![ui_widgets::vscroll(
            TOOLBAR_MENU_POPUP_SCROLL_WIDGET_ID,
            contrast_popup_theme(theme),
            vec![ui_widgets::vstack_with_policies(
                TOOLBAR_MENU_POPUP_LIST_WIDGET_ID,
                theme.spacing.xs,
                vec![ui_layout::SizePolicy::Auto; children.len()],
                children,
            )],
        )],
    );
    paths_by_widget_id.insert(
        TOOLBAR_MENU_POPUP_WIDGET_ID,
        AuthoredUiNodePath("root/menu_popup".to_string()),
    );

    let mut interaction_model = FormedInteractionModel::default();
    interaction_model.push_menu_scope(FormedMenuStackScope {
        scope_id: format!("toolbar.{active_menu_id}"),
        popup_widget_id: TOOLBAR_MENU_POPUP_WIDGET_ID,
        anchor_widget_id: anchor,
        parent_scope_id: None,
        dismiss: UiMenuDismissPolicyDefinition::OutsidePointerDown,
        focus_return: Some(anchor),
    });
    interaction_model.push_menu_sizing(FormedMenuSizing {
        popup_widget_id: TOOLBAR_MENU_POPUP_WIDGET_ID,
        list_widget_id: TOOLBAR_MENU_POPUP_LIST_WIDGET_ID,
        item_width: UiMenuItemWidthDefinition::FillToMenuWidth,
        overflow: UiMenuOverflowDefinition::ScrollWhenClamped,
    });
    interaction_model.push_scroll_owner(FormedScrollOwner {
        widget_id: TOOLBAR_MENU_POPUP_SCROLL_WIDGET_ID,
        axes: vec![Axis::Vertical],
        boundary: UiScrollBoundaryPolicyDefinition::ConsumeAtBoundary,
    });

    Some(FormedRetainedUiProduct {
        root,
        routes_by_widget_id,
        paths_by_widget_id,
        embeds_by_widget_id: BTreeMap::new(),
        diagnostics: Vec::new(),
        availability_by_widget_id,
        interaction_model,
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
    for child in &mut node.children {
        apply_compact_toolbar_text_alignment(child);
    }
}

fn insert_dynamic_workspace_buttons(
    product: &mut FormedRetainedUiProduct,
    view_model: &ToolbarViewModel,
    theme: &ThemeTokens,
) {
    for definition in toolbar_workspace_button_definitions() {
        let Some(view_button) = view_model
            .buttons
            .iter()
            .find(|button| button.stable_name == definition.stable_name)
        else {
            continue;
        };
        let Some(row) = find_node_mut(&mut product.root, TOOLBAR_ROW_WIDGET_ID) else {
            return;
        };
        if row
            .children
            .iter()
            .any(|child| child.id == definition.widget_id)
        {
            continue;
        }
        let mut node = button_selected(
            definition.widget_id,
            view_button.label.clone(),
            theme.body_small_text_style(FontId(1)),
            theme.clone(),
            view_button.is_active,
        );
        if let UiNodeKind::Button(button) = &mut node.kind {
            button.enabled = view_button.enabled;
        }
        let insert_at = row
            .children
            .iter()
            .position(|child| child.id == TOOLBAR_ADD_WORKSPACE_WIDGET_ID)
            .unwrap_or(row.children.len());
        row.children.insert(insert_at, node);
        product.routes_by_widget_id.insert(
            definition.widget_id,
            FormedUiRoute::RouteSlot(UiRouteSlotId::new(definition.activate_route)),
        );
        product.paths_by_widget_id.insert(
            definition.widget_id,
            AuthoredUiNodePath(format!(
                "root/scroll/rows/top_row/{}",
                definition.stable_name
            )),
        );
    }
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
    for definition in toolbar_workspace_button_definitions() {
        let profile_id = definition.profile_id;
        let anchor_widget_id = definition.widget_id;
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
        let active_indicator_widget_id = toolbar_workspace_active_indicator_widget_id(profile_id);
        let chrome_widget_id = toolbar_workspace_chrome_widget_id(profile_id);
        if row.children.iter().any(|child| {
            child.id == close_widget_id
                || child.id == chrome_widget_id
                || child.id == active_indicator_widget_id
        }) {
            continue;
        }
        let anchor = row.children.remove(anchor_index);
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
            anchor_index,
            hstack_with_policies(
                chrome_widget_id,
                theme.spacing.xs,
                vec![
                    ui_layout::SizePolicy::Auto,
                    ui_layout::SizePolicy::Auto,
                    ui_layout::SizePolicy::Auto,
                ],
                vec![
                    close,
                    anchor,
                    workspace_active_indicator_node(
                        active_indicator_widget_id,
                        workspace_button_is_active(view_model, profile_id),
                        theme,
                    ),
                ],
            ),
        );
        push_workspace_chrome_slots(
            &mut product.interaction_model,
            chrome_widget_id,
            close_widget_id,
            anchor_widget_id,
            active_indicator_widget_id,
        );
        product.routes_by_widget_id.insert(
            close_widget_id,
            FormedUiRoute::RouteSlot(UiRouteSlotId::new(definition.close_route)),
        );
        product.paths_by_widget_id.insert(
            close_widget_id,
            AuthoredUiNodePath(format!(
                "root/scroll/rows/top_row/{}:close",
                definition.stable_name
            )),
        );
    }
}

fn push_workspace_chrome_slots(
    model: &mut FormedInteractionModel,
    host_widget_id: ui_tree::WidgetId,
    close_widget_id: ui_tree::WidgetId,
    label_widget_id: ui_tree::WidgetId,
    active_indicator_widget_id: ui_tree::WidgetId,
) {
    model.push_chrome_slot(FormedChromeSlot {
        host_widget_id,
        slot_widget_id: close_widget_id,
        kind: UiChromeSlotKindDefinition::CloseAffordance,
        input_policy: UiChromeSlotInputPolicyDefinition::Command,
        order: 0,
    });
    model.push_chrome_slot(FormedChromeSlot {
        host_widget_id,
        slot_widget_id: close_widget_id,
        kind: UiChromeSlotKindDefinition::CommandArea,
        input_policy: UiChromeSlotInputPolicyDefinition::Command,
        order: 0,
    });
    model.push_chrome_slot(FormedChromeSlot {
        host_widget_id,
        slot_widget_id: label_widget_id,
        kind: UiChromeSlotKindDefinition::Label,
        input_policy: UiChromeSlotInputPolicyDefinition::Activate,
        order: 1,
    });
    model.push_chrome_slot(FormedChromeSlot {
        host_widget_id,
        slot_widget_id: label_widget_id,
        kind: UiChromeSlotKindDefinition::DragRegion,
        input_policy: UiChromeSlotInputPolicyDefinition::Drag,
        order: 1,
    });
    model.push_chrome_slot(FormedChromeSlot {
        host_widget_id,
        slot_widget_id: active_indicator_widget_id,
        kind: UiChromeSlotKindDefinition::ActiveIndicator,
        input_policy: UiChromeSlotInputPolicyDefinition::None,
        order: 2,
    });
}

fn workspace_profile_for_stable_name(name: &str) -> Option<WorkspaceProfileId> {
    toolbar_workspace_button_definitions()
        .iter()
        .find(|definition| definition.stable_name == name)
        .map(|definition| definition.profile_id)
}

fn workspace_button_is_active(
    view_model: &ToolbarViewModel,
    profile_id: WorkspaceProfileId,
) -> bool {
    view_model.buttons.iter().any(|button| {
        workspace_profile_for_stable_name(button.stable_name) == Some(profile_id)
            && button.is_active
    })
}

fn toolbar_icon_text_style(theme: &ThemeTokens) -> ui_text::TextStyle {
    theme.body_small_text_style(FontId(1))
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

fn workspace_active_indicator_node(
    id: ui_tree::WidgetId,
    active: bool,
    theme: &ThemeTokens,
) -> UiNode {
    let mut indicator_theme = theme.clone();
    indicator_theme.background_panel = if active {
        theme.accent
    } else {
        UiColor::new(0.0, 0.0, 0.0, 0.0)
    };
    indicator_theme.border = if active {
        theme.accent
    } else {
        theme.foreground_muted
    };
    indicator_theme.border_width = theme.border_width.max(1.0);
    let mut node = button(id, "", toolbar_icon_text_style(theme), indicator_theme);
    if let UiNodeKind::Button(button) = &mut node.kind {
        button.padding = UiInsets::ZERO;
        button.min_size = UiSize::new(18.0, 18.0);
        button.corner_radius = Some(f32::MAX);
        button.selected = active;
        button.selected_fill = Some(theme.accent);
        button.selected_border = Some(theme.accent);
    }
    node
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
            "root/scroll/rows/top_row/workspace_plus",
            TOOLBAR_ADD_WORKSPACE_WIDGET_ID,
        ),
    ];
    for (path, widget_id) in mappings {
        context
            .widget_ids_by_path
            .insert(AuthoredUiNodePath(path.to_string()), widget_id);
    }
    for definition in toolbar_workspace_button_definitions() {
        context.widget_ids_by_path.insert(
            AuthoredUiNodePath(format!(
                "root/scroll/rows/top_row/{}",
                definition.stable_name
            )),
            definition.widget_id,
        );
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
    for definition in toolbar_workspace_button_definitions() {
        context.values.insert(
            AuthoredId::new(format!(
                "{}.active",
                definition.activate_route.trim_end_matches(".activate")
            )),
            UiValue::Bool(active(definition.stable_name)),
        );
    }
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
