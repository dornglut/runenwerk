//! File: domain/editor/editor_shell/src/composition/build_viewport_panel.rs
//! Purpose: Compose viewport panel widgets.

use crate::{UiNode, button_selected, hstack_with_policies, label, vscroll, vstack_with_policies};
use editor_viewport::{ViewportDebugStage, ViewportSurfacePresentationSlot};
use ui_definition::{
    AuthoredUiTemplate, UiDefinitionContext, form_retained_ui, normalize_authored_template,
};
use ui_layout::SizePolicy;
use ui_text::FontId;
use ui_theme::{ThemeTokens, UiColor};
use ui_tree::{PopupAlign, PopupFlipPolicy, PopupNode, PopupSide, RadialMenuNode, UiNodeKind};

use crate::{
    PanelInstanceId, SurfaceWidgetScope, ToolSurfaceInstanceId, VIEWPORT_BODY_WIDGET_ID,
    VIEWPORT_CANVAS_CONTENT_WIDGET_ID, VIEWPORT_CANVAS_WIDGET_ID,
    VIEWPORT_CHROME_CONTENT_WIDGET_ID, VIEWPORT_CHROME_WIDGET_ID, VIEWPORT_DETAILS_LABEL_WIDGET_ID,
    VIEWPORT_DETAILS_PANEL_WIDGET_ID, VIEWPORT_DETAILS_TOGGLE_WIDGET_ID,
    VIEWPORT_OPTIONS_BUTTON_WIDGET_ID, VIEWPORT_OPTIONS_POPUP_LIST_WIDGET_ID,
    VIEWPORT_OPTIONS_POPUP_SCROLL_WIDGET_ID, VIEWPORT_OPTIONS_POPUP_WIDGET_ID,
    VIEWPORT_PANEL_WIDGET_ID, VIEWPORT_RESET_CAMERA_WIDGET_ID,
    VIEWPORT_ROOT_OPAQUE_TOGGLE_WIDGET_ID, VIEWPORT_STATISTICS_LABEL_WIDGET_ID,
    VIEWPORT_STATISTICS_TOGGLE_WIDGET_ID, VIEWPORT_STATUS_WIDGET_ID,
    VIEWPORT_SURFACE_EMBED_WIDGET_ID, VIEWPORT_TOOL_RADIAL_BUTTON_WIDGET_ID,
    VIEWPORT_TOOL_RADIAL_MENU_WIDGET_ID, VIEWPORT_TOOLS_MENU_LIST_WIDGET_ID,
    VIEWPORT_TOOLS_MENU_SCROLL_WIDGET_ID, VIEWPORT_TOOLS_MENU_WIDGET_ID, ViewportViewModel,
    viewport_debug_stage_button_widget_id, viewport_embed_slot_for,
    viewport_product_button_widget_id, viewport_tool_radial_item_widget_id,
};

use super::surface_control_polish::{compact_surface_action_button, compact_surface_toggle};
use super::surface_definition_context::{
    contrast_popup_theme, find_node_mut, register_widget_ids_by_path, scoped_definition_context,
    set_stack_child_main_policies, transparent_panel_background,
};

const VIEWPORT_TEMPLATE_RON: &str =
    include_str!("../../../../../assets/editor/ui/surfaces/viewport.ron");

pub fn build_viewport_panel(
    view_model: &ViewportViewModel,
    theme: &ThemeTokens,
    _panel_instance_id: PanelInstanceId,
    active_tool_surface: Option<ToolSurfaceInstanceId>,
) -> UiNode {
    let template: AuthoredUiTemplate =
        ron::from_str(VIEWPORT_TEMPLATE_RON).expect("checked-in viewport UI fixture must parse");
    let normalized = normalize_authored_template(template);
    let scope = SurfaceWidgetScope::optional(active_tool_surface);
    let mut context = scoped_definition_context(theme, scope);
    register_viewport_widget_ids(&mut context, scope);
    context.embed_slots.insert(
        "viewport.expression_product".into(),
        viewport_embed_slot_for(ViewportSurfacePresentationSlot::Primary).raw(),
    );

    let mut root = form_retained_ui(&normalized, &mut context).root;
    polish_viewport_base(&mut root, view_model, theme, scope);
    inject_viewport_overlays(&mut root, view_model, theme, scope);
    root
}

fn register_viewport_widget_ids(context: &mut UiDefinitionContext, scope: SurfaceWidgetScope) {
    register_widget_ids_by_path(
        context,
        scope,
        [
            ("root", VIEWPORT_PANEL_WIDGET_ID),
            ("root/body", VIEWPORT_BODY_WIDGET_ID),
            ("root/body/canvas", VIEWPORT_CANVAS_WIDGET_ID),
            (
                "root/body/canvas/canvas_content",
                VIEWPORT_CANVAS_CONTENT_WIDGET_ID,
            ),
            (
                "root/body/canvas/canvas_content/viewport_canvas",
                VIEWPORT_SURFACE_EMBED_WIDGET_ID,
            ),
        ],
    );
}

fn polish_viewport_base(
    root: &mut UiNode,
    view_model: &ViewportViewModel,
    theme: &ThemeTokens,
    scope: SurfaceWidgetScope,
) {
    if let UiNodeKind::Panel(panel) = &mut root.kind {
        panel.theme.background_panel = transparent_panel_background(theme, 0.0);
        panel.theme.border = UiColor::new(
            (theme.border.r + 0.05).clamp(0.0, 1.0),
            (theme.border.g + 0.06).clamp(0.0, 1.0),
            (theme.border.b + 0.09).clamp(0.0, 1.0),
            0.95,
        );
    }
    set_stack_child_main_policies(
        root,
        scope.widget_id(VIEWPORT_BODY_WIDGET_ID),
        vec![SizePolicy::flex(1.0)],
    );
    if let Some(body) = find_node_mut(root, scope.widget_id(VIEWPORT_BODY_WIDGET_ID))
        && let UiNodeKind::Stack(stack) = &mut body.kind
    {
        stack.gap = 0.0;
    }
    if let Some(canvas) = find_node_mut(root, scope.widget_id(VIEWPORT_CANVAS_WIDGET_ID))
        && let UiNodeKind::Panel(panel) = &mut canvas.kind
    {
        panel.theme.background_panel = transparent_panel_background(theme, 0.0);
        panel.theme.border = UiColor::new(theme.accent.r, theme.accent.g, theme.accent.b, 0.70);
    }
    if let Some(content) = find_node_mut(root, scope.widget_id(VIEWPORT_CANVAS_CONTENT_WIDGET_ID)) {
        if let UiNodeKind::Stack(stack) = &mut content.kind {
            stack.child_main_policies = if view_model.viewport_id.is_some() {
                vec![SizePolicy::flex(1.0)]
            } else {
                Vec::new()
            };
        }
        if view_model.viewport_id.is_none() {
            content
                .children
                .retain(|child| child.id != scope.widget_id(VIEWPORT_SURFACE_EMBED_WIDGET_ID));
        }
    }
    if let Some(embed) = find_node_mut(root, scope.widget_id(VIEWPORT_SURFACE_EMBED_WIDGET_ID))
        && let UiNodeKind::ViewportSurfaceEmbed(embed) = &mut embed.kind
        && let Some(viewport_id) = view_model.viewport_id
    {
        embed.viewport_id = viewport_id.0;
    }
}

fn inject_viewport_overlays(
    root: &mut UiNode,
    view_model: &ViewportViewModel,
    theme: &ThemeTokens,
    scope: SurfaceWidgetScope,
) {
    let mut overlay_children = vec![viewport_chrome_overlay(view_model, theme, scope)];
    if let Some(tools_menu) = viewport_tools_menu(view_model, theme, scope) {
        overlay_children.push(tools_menu);
    }
    if let Some(tool_radial) = viewport_tool_radial_menu(view_model, theme, scope) {
        overlay_children.push(tool_radial);
    }
    if let Some(options_popup) = viewport_options_popup(view_model, theme, scope) {
        overlay_children.push(options_popup);
    }
    if let Some(status_overlay) = viewport_status_overlay(view_model, theme, scope) {
        overlay_children.push(status_overlay);
    }
    if let Some(canvas) = find_node_mut(root, scope.widget_id(VIEWPORT_CANVAS_WIDGET_ID)) {
        canvas.children.extend(overlay_children);
    }
}

fn viewport_chrome_overlay(
    view_model: &ViewportViewModel,
    theme: &ThemeTokens,
    scope: SurfaceWidgetScope,
) -> UiNode {
    let chrome = hstack_with_policies(
        scope.widget_id(VIEWPORT_CHROME_CONTENT_WIDGET_ID),
        theme.spacing.xs,
        vec![SizePolicy::Auto, SizePolicy::Auto],
        vec![
            button_selected(
                scope.widget_id(VIEWPORT_OPTIONS_BUTTON_WIDGET_ID),
                "Options",
                theme.body_small_text_style(FontId(1)),
                theme.clone(),
                view_model.options_menu_open,
            ),
            button_selected(
                scope.widget_id(VIEWPORT_TOOL_RADIAL_BUTTON_WIDGET_ID),
                "Tools",
                theme.body_small_text_style(FontId(1)),
                theme.clone(),
                view_model.tools_menu_open || view_model.tool_radial_anchor_position.is_some(),
            ),
        ],
    );

    UiNode::with_children(
        scope.widget_id(VIEWPORT_CHROME_WIDGET_ID),
        UiNodeKind::Popup(PopupNode::anchored_top_start(
            scope.widget_id(VIEWPORT_CANVAS_WIDGET_ID),
            transparent_panel_theme(theme, 0.0),
        )),
        vec![chrome],
    )
}

fn viewport_tool_radial_menu(
    view_model: &ViewportViewModel,
    theme: &ThemeTokens,
    scope: SurfaceWidgetScope,
) -> Option<UiNode> {
    view_model
        .tool_radial_anchor_position
        .map(|anchor_position| {
            let mut radial_theme = theme.clone();
            radial_theme.background_panel = UiColor::new(
                theme.background_panel.r,
                theme.background_panel.g,
                theme.background_panel.b,
                0.94,
            );
            let mut radial = RadialMenuNode::anchored_at(anchor_position, radial_theme);
            radial.inner_radius = 18.0;
            radial.outer_radius = 78.0;
            radial.item_size = ui_math::UiSize::new(50.0, 30.0);
            UiNode::with_children(
                scope.widget_id(VIEWPORT_TOOL_RADIAL_MENU_WIDGET_ID),
                UiNodeKind::RadialMenu(radial),
                ["Select", "Move", "Rotate", "Scale"]
                    .into_iter()
                    .enumerate()
                    .map(|(index, label)| {
                        compact_surface_action_button(
                            scope.widget_id(viewport_tool_radial_item_widget_id(index)),
                            label,
                            false,
                            true,
                            theme,
                        )
                    })
                    .collect(),
            )
        })
}

fn viewport_tools_menu(
    view_model: &ViewportViewModel,
    theme: &ThemeTokens,
    scope: SurfaceWidgetScope,
) -> Option<UiNode> {
    view_model.tools_menu_open.then(|| {
        let popup_theme = contrast_popup_theme(theme);
        let items = ["Select", "Move", "Rotate", "Scale"]
            .into_iter()
            .enumerate()
            .map(|(index, label)| {
                compact_surface_action_button(
                    scope.widget_id(viewport_tool_radial_item_widget_id(index)),
                    label,
                    false,
                    true,
                    theme,
                )
            })
            .collect::<Vec<_>>();
        UiNode::with_children(
            scope.widget_id(VIEWPORT_TOOLS_MENU_WIDGET_ID),
            UiNodeKind::Popup(PopupNode::anchored_outside(
                scope.widget_id(VIEWPORT_TOOL_RADIAL_BUTTON_WIDGET_ID),
                PopupSide::Bottom,
                PopupAlign::Start,
                PopupFlipPolicy::FlipToFit,
                popup_theme,
            )),
            vec![scrollable_popup_menu_content(
                scope.widget_id(VIEWPORT_TOOLS_MENU_SCROLL_WIDGET_ID),
                scope.widget_id(VIEWPORT_TOOLS_MENU_LIST_WIDGET_ID),
                theme,
                items,
            )],
        )
    })
}

fn viewport_options_popup(
    view_model: &ViewportViewModel,
    theme: &ThemeTokens,
    scope: SurfaceWidgetScope,
) -> Option<UiNode> {
    view_model.options_menu_open.then(|| {
        let popup_theme = contrast_popup_theme(theme);
        let mut items = vec![
            compact_surface_toggle(
                scope.widget_id(VIEWPORT_DETAILS_TOGGLE_WIDGET_ID),
                "Details",
                view_model.details_visible,
                theme,
            ),
            compact_surface_toggle(
                scope.widget_id(VIEWPORT_STATISTICS_TOGGLE_WIDGET_ID),
                "Statistics",
                view_model.statistics_visible,
                theme,
            ),
        ];
        if view_model.viewport_id.is_some() {
            items.push(compact_surface_action_button(
                scope.widget_id(VIEWPORT_RESET_CAMERA_WIDGET_ID),
                "Reset Camera",
                false,
                true,
                theme,
            ));
            items.push(compact_surface_toggle(
                scope.widget_id(VIEWPORT_ROOT_OPAQUE_TOGGLE_WIDGET_ID),
                "Opaque Root",
                view_model.root_background_opaque,
                theme,
            ));
            for (index, stage) in ViewportDebugStage::ALL.into_iter().enumerate() {
                items.push(compact_surface_action_button(
                    scope.widget_id(viewport_debug_stage_button_widget_id(index)),
                    format!("Debug {}", stage.display_label()),
                    view_model.debug_stage == stage,
                    true,
                    theme,
                ));
            }
        }
        for (index, choice) in view_model.product_choices.iter().enumerate() {
            items.push(compact_surface_action_button(
                scope.widget_id(viewport_product_button_widget_id(index)),
                format!("Product {}", choice.label),
                choice.selected,
                choice.enabled,
                theme,
            ));
        }
        UiNode::with_children(
            scope.widget_id(VIEWPORT_OPTIONS_POPUP_WIDGET_ID),
            UiNodeKind::Popup(PopupNode::anchored_outside(
                scope.widget_id(VIEWPORT_OPTIONS_BUTTON_WIDGET_ID),
                PopupSide::Bottom,
                PopupAlign::Start,
                PopupFlipPolicy::FlipToFit,
                popup_theme,
            )),
            vec![scrollable_popup_menu_content(
                scope.widget_id(VIEWPORT_OPTIONS_POPUP_SCROLL_WIDGET_ID),
                scope.widget_id(VIEWPORT_OPTIONS_POPUP_LIST_WIDGET_ID),
                theme,
                items,
            )],
        )
    })
}

fn scrollable_popup_menu_content(
    scroll_id: crate::WidgetId,
    list_id: crate::WidgetId,
    theme: &ThemeTokens,
    items: Vec<UiNode>,
) -> UiNode {
    vscroll(
        scroll_id,
        contrast_popup_theme(theme),
        vec![vstack_with_policies(
            list_id,
            theme.spacing.xs,
            vec![SizePolicy::Auto; items.len()],
            items,
        )],
    )
}

fn viewport_status_overlay(
    view_model: &ViewportViewModel,
    theme: &ThemeTokens,
    scope: SurfaceWidgetScope,
) -> Option<UiNode> {
    (view_model.details_visible
        || view_model.statistics_visible
        || !view_model.overlay_status_lines.is_empty())
    .then(|| {
        let mut overlay_items = Vec::new();
        if view_model.details_visible {
            overlay_items.push(label(
                scope.widget_id(VIEWPORT_DETAILS_LABEL_WIDGET_ID),
                viewport_details_text(view_model),
                theme.body_small_text_style(FontId(1)),
            ));
        }
        if view_model.statistics_visible {
            overlay_items.push(label(
                scope.widget_id(VIEWPORT_STATISTICS_LABEL_WIDGET_ID),
                viewport_statistics_text(view_model),
                theme.body_small_text_style(FontId(1)),
            ));
        }
        if !view_model.overlay_status_lines.is_empty() {
            overlay_items.push(label(
                scope.widget_id(crate::WidgetId(90_100)),
                view_model
                    .overlay_status_lines
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" | "),
                theme.body_small_text_style(FontId(1)),
            ));
        }
        UiNode::with_children(
            scope.widget_id(VIEWPORT_DETAILS_PANEL_WIDGET_ID),
            UiNodeKind::Popup(PopupNode::anchored_inside_bottom_start(
                scope.widget_id(VIEWPORT_CANVAS_WIDGET_ID),
                transparent_panel_theme(theme, 0.50),
            )),
            vec![hstack_with_policies(
                scope.widget_id(VIEWPORT_STATUS_WIDGET_ID),
                theme.spacing.sm,
                vec![SizePolicy::Auto; overlay_items.len()],
                overlay_items,
            )],
        )
    })
}

fn transparent_panel_theme(theme: &ThemeTokens, alpha: f32) -> ThemeTokens {
    let mut panel_theme = theme.clone();
    panel_theme.background_panel = transparent_panel_background(theme, alpha);
    panel_theme.border = UiColor::new(theme.border.r, theme.border.g, theme.border.b, 0.0);
    panel_theme
}

fn viewport_details_text(view_model: &ViewportViewModel) -> String {
    let viewport = view_model
        .viewport_id
        .map(|viewport| viewport.0.to_string())
        .unwrap_or_else(|| "unbound".to_string());
    let selected_entity = view_model
        .selected_entity
        .map(|entity| entity.0.to_string())
        .unwrap_or_else(|| "none".to_string());
    format!("Details: viewport={viewport} selected_entity={selected_entity}")
}

fn viewport_statistics_text(view_model: &ViewportViewModel) -> String {
    let frame_metrics = match (view_model.frame_rate_fps, view_model.frame_time_ms) {
        (Some(fps), Some(frame_ms)) => format!(" fps={fps:.1} frame_ms={frame_ms:.2}"),
        (Some(fps), None) => format!(" fps={fps:.1}"),
        (None, Some(frame_ms)) => format!(" frame_ms={frame_ms:.2}"),
        (None, None) => String::new(),
    };
    format!(
        "Statistics: drag={} preview={}{}",
        view_model.drag_in_progress, view_model.preview_active, frame_metrics
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UiNodeKind;

    fn find_node(node: &UiNode, id: crate::WidgetId) -> Option<&UiNode> {
        if node.id == id {
            return Some(node);
        }
        node.children.iter().find_map(|child| find_node(child, id))
    }

    #[test]
    fn viewport_options_menu_projects_checkbox_items() {
        let theme = ThemeTokens::default();
        let hidden = build_viewport_panel(
            &ViewportViewModel::default(),
            &theme,
            PanelInstanceId::try_from_raw(1).unwrap(),
            None,
        );
        let options_button = find_node(&hidden, VIEWPORT_OPTIONS_BUTTON_WIDGET_ID)
            .expect("options button should exist");
        assert!(matches!(
            &options_button.kind,
            UiNodeKind::Button(button) if button.label == "Options" && !button.selected
        ));
        assert!(find_node(&hidden, VIEWPORT_DETAILS_TOGGLE_WIDGET_ID).is_none());

        let visible_model = ViewportViewModel {
            options_menu_open: true,
            details_visible: true,
            statistics_visible: true,
            ..Default::default()
        };
        let visible = build_viewport_panel(
            &visible_model,
            &theme,
            PanelInstanceId::try_from_raw(1).unwrap(),
            None,
        );
        let visible_button = find_node(&visible, VIEWPORT_DETAILS_TOGGLE_WIDGET_ID)
            .expect("details button should exist");
        assert!(matches!(
            &visible_button.kind,
            UiNodeKind::Toggle(toggle) if toggle.label == "Details" && toggle.checked
        ));
        let statistics_button = find_node(&visible, VIEWPORT_STATISTICS_TOGGLE_WIDGET_ID)
            .expect("statistics button should exist");
        assert!(matches!(
            &statistics_button.kind,
            UiNodeKind::Toggle(toggle) if toggle.label == "Statistics" && toggle.checked
        ));
    }

    #[test]
    fn viewport_options_menu_projects_product_choices() {
        let theme = ThemeTokens::default();
        let visible_model = ViewportViewModel {
            options_menu_open: true,
            viewport_id: Some(editor_viewport::ViewportId(4)),
            product_choices: vec![
                crate::ViewportProductChoiceViewModel {
                    viewport_id: editor_viewport::ViewportId(4),
                    product_id: editor_viewport::ExpressionProductId(1),
                    label: "Scene Color".to_string(),
                    selected: true,
                    enabled: true,
                },
                crate::ViewportProductChoiceViewModel {
                    viewport_id: editor_viewport::ViewportId(4),
                    product_id: editor_viewport::ExpressionProductId(9),
                    label: "Volume Slice".to_string(),
                    selected: false,
                    enabled: false,
                },
            ],
            ..Default::default()
        };
        let visible = build_viewport_panel(
            &visible_model,
            &theme,
            PanelInstanceId::try_from_raw(1).unwrap(),
            None,
        );

        let selected_product = find_node(&visible, viewport_product_button_widget_id(0))
            .expect("selected product button should exist");
        assert!(matches!(
            &selected_product.kind,
            UiNodeKind::Button(button)
                if button.label == "Product Scene Color" && button.selected && button.enabled
        ));

        let unavailable_product = find_node(&visible, viewport_product_button_widget_id(1))
            .expect("unavailable product button should exist");
        assert!(matches!(
            &unavailable_product.kind,
            UiNodeKind::Button(button)
                if button.label == "Product Volume Slice" && !button.selected && !button.enabled
        ));
    }

    #[test]
    fn viewport_tool_radial_menu_projects_transform_tool_entries() {
        let theme = ThemeTokens::default();
        let hidden = build_viewport_panel(
            &ViewportViewModel::default(),
            &theme,
            PanelInstanceId::try_from_raw(1).unwrap(),
            None,
        );
        assert!(find_node(&hidden, VIEWPORT_TOOL_RADIAL_MENU_WIDGET_ID).is_none());

        let visible_model = ViewportViewModel {
            tool_radial_anchor_position: Some(ui_math::UiPoint::new(120.0, 80.0)),
            ..Default::default()
        };
        let visible = build_viewport_panel(
            &visible_model,
            &theme,
            PanelInstanceId::try_from_raw(1).unwrap(),
            None,
        );
        let menu = find_node(&visible, VIEWPORT_TOOL_RADIAL_MENU_WIDGET_ID)
            .expect("tool radial menu should exist");
        assert!(matches!(
            &menu.kind,
            UiNodeKind::RadialMenu(radial)
                if matches!(
                    radial.anchor,
                    ui_tree::RadialMenuAnchor::Point(point)
                        if point == ui_math::UiPoint::new(120.0, 80.0)
                )
        ));
        for (index, label) in ["Select", "Move", "Rotate", "Scale"]
            .into_iter()
            .enumerate()
        {
            let item = find_node(&visible, viewport_tool_radial_item_widget_id(index))
                .expect("tool radial item should exist");
            assert!(matches!(
                &item.kind,
                UiNodeKind::Button(button) if button.label == label
            ));
        }
    }

    #[test]
    fn viewport_tools_click_menu_projects_transform_tool_entries() {
        let theme = ThemeTokens::default();
        let hidden = build_viewport_panel(
            &ViewportViewModel::default(),
            &theme,
            PanelInstanceId::try_from_raw(1).unwrap(),
            None,
        );
        assert!(find_node(&hidden, VIEWPORT_TOOLS_MENU_WIDGET_ID).is_none());

        let visible_model = ViewportViewModel {
            tools_menu_open: true,
            ..Default::default()
        };
        let visible = build_viewport_panel(
            &visible_model,
            &theme,
            PanelInstanceId::try_from_raw(1).unwrap(),
            None,
        );
        let menu =
            find_node(&visible, VIEWPORT_TOOLS_MENU_WIDGET_ID).expect("tools menu should exist");
        assert!(matches!(&menu.kind, UiNodeKind::Popup(_)));
        for (index, label) in ["Select", "Move", "Rotate", "Scale"]
            .into_iter()
            .enumerate()
        {
            let item = find_node(&visible, viewport_tool_radial_item_widget_id(index))
                .expect("tool menu item should exist");
            assert!(matches!(
                &item.kind,
                UiNodeKind::Button(button) if button.label == label
            ));
        }
    }

    #[test]
    fn viewport_details_content_is_visible_only_when_enabled() {
        let theme = ThemeTokens::default();
        let hidden = build_viewport_panel(
            &ViewportViewModel::default(),
            &theme,
            PanelInstanceId::try_from_raw(1).unwrap(),
            None,
        );
        assert!(find_node(&hidden, VIEWPORT_DETAILS_LABEL_WIDGET_ID).is_none());

        let visible_model = ViewportViewModel {
            details_visible: true,
            viewport_id: Some(editor_viewport::ViewportId(7)),
            selected_entity: Some(editor_core::EntityId(42)),
            drag_in_progress: true,
            preview_active: true,
            ..Default::default()
        };
        let visible = build_viewport_panel(
            &visible_model,
            &theme,
            PanelInstanceId::try_from_raw(1).unwrap(),
            None,
        );
        assert!(find_node(&visible, VIEWPORT_DETAILS_PANEL_WIDGET_ID).is_some());
        let details_label = find_node(&visible, VIEWPORT_DETAILS_LABEL_WIDGET_ID)
            .expect("details label should exist");
        assert!(matches!(
            &details_label.kind,
            UiNodeKind::Label(label)
                if label.text.contains("viewport=7")
                    && label.text.contains("selected_entity=42")
        ));

        let statistics_model = ViewportViewModel {
            statistics_visible: true,
            drag_in_progress: true,
            preview_active: true,
            frame_rate_fps: Some(60.0),
            frame_time_ms: Some(16.67),
            ..Default::default()
        };
        let statistics = build_viewport_panel(
            &statistics_model,
            &theme,
            PanelInstanceId::try_from_raw(1).unwrap(),
            None,
        );
        let statistics_label = find_node(&statistics, VIEWPORT_STATISTICS_LABEL_WIDGET_ID)
            .expect("statistics label should exist");
        assert!(matches!(
            &statistics_label.kind,
            UiNodeKind::Label(label)
                if label.text.contains("drag=true")
                    && label.text.contains("preview=true")
                    && label.text.contains("fps=60.0")
                    && label.text.contains("frame_ms=16.67")
        ));
    }

    #[test]
    fn viewport_status_overlay_projects_generic_overlay_status_lines() {
        let theme = ThemeTokens::default();
        let model = ViewportViewModel {
            overlay_status_lines: vec![
                "Procgen overlay: 2 region(s)".to_string(),
                "density-main [0, 0, 0]-[16, 16, 16]".to_string(),
            ],
            ..Default::default()
        };

        let visible = build_viewport_panel(
            &model,
            &theme,
            PanelInstanceId::try_from_raw(1).unwrap(),
            None,
        );

        let panel = find_node(&visible, VIEWPORT_DETAILS_PANEL_WIDGET_ID)
            .expect("status overlay should exist for generic overlay lines");
        assert!(format!("{:?}", panel).contains("Procgen overlay: 2 region(s)"));
    }
}
