//! File: domain/editor/editor_shell/src/composition/build_viewport_panel.rs
//! Purpose: Compose viewport panel widgets.

use crate::{
    UiNode, button_selected, hstack_with_policies, label, panel, viewport_surface_embed,
    vstack_with_policies,
};
use editor_viewport::ViewportSurfacePresentationSlot;
use ui_layout::SizePolicy;
use ui_text::FontId;
use ui_theme::{ThemeTokens, UiColor};
use ui_tree::{PopupNode, UiNodeKind};

use crate::{
    PanelInstanceId, ToolSurfaceInstanceId, VIEWPORT_BODY_WIDGET_ID,
    VIEWPORT_CANVAS_CONTENT_WIDGET_ID, VIEWPORT_CANVAS_WIDGET_ID,
    VIEWPORT_CHROME_CONTENT_WIDGET_ID, VIEWPORT_CHROME_WIDGET_ID, VIEWPORT_DETAILS_LABEL_WIDGET_ID,
    VIEWPORT_DETAILS_PANEL_WIDGET_ID, VIEWPORT_DETAILS_TOGGLE_WIDGET_ID,
    VIEWPORT_OPTIONS_BUTTON_WIDGET_ID, VIEWPORT_OPTIONS_POPUP_WIDGET_ID, VIEWPORT_PANEL_WIDGET_ID,
    VIEWPORT_STATISTICS_LABEL_WIDGET_ID, VIEWPORT_STATISTICS_TOGGLE_WIDGET_ID,
    VIEWPORT_STATUS_WIDGET_ID, VIEWPORT_SURFACE_EMBED_WIDGET_ID, ViewportViewModel,
    viewport_embed_slot_for,
};

pub fn build_viewport_panel(
    view_model: &ViewportViewModel,
    theme: &ThemeTokens,
    _panel_instance_id: PanelInstanceId,
    _active_tool_surface: Option<ToolSurfaceInstanceId>,
) -> UiNode {
    let viewport_children = view_model
        .viewport_id
        .map(|viewport_id| {
            vec![viewport_surface_embed(
                VIEWPORT_SURFACE_EMBED_WIDGET_ID,
                viewport_id.0,
                viewport_embed_slot_for(ViewportSurfacePresentationSlot::Primary),
            )]
        })
        .unwrap_or_default();
    let viewport_child_policies = if viewport_children.is_empty() {
        Vec::new()
    } else {
        vec![SizePolicy::flex(1.0)]
    };
    let canvas_content = vstack_with_policies(
        VIEWPORT_CANVAS_CONTENT_WIDGET_ID,
        theme.spacing.xs,
        viewport_child_policies,
        viewport_children,
    );
    let mut canvas_theme = theme.clone();
    canvas_theme.background_panel = UiColor::new(
        theme.background_panel.r,
        theme.background_panel.g,
        theme.background_panel.b,
        0.0,
    );
    canvas_theme.border = UiColor::new(theme.accent.r, theme.accent.g, theme.accent.b, 0.70);
    let chrome = hstack_with_policies(
        VIEWPORT_CHROME_CONTENT_WIDGET_ID,
        theme.spacing.xs,
        vec![SizePolicy::Auto],
        vec![button_selected(
            VIEWPORT_OPTIONS_BUTTON_WIDGET_ID,
            "Options",
            theme.body_small_text_style(FontId(1)),
            theme.clone(),
            view_model.options_menu_open,
        )],
    );

    let chrome_overlay = UiNode::with_children(
        VIEWPORT_CHROME_WIDGET_ID,
        UiNodeKind::Popup(PopupNode::anchored_top_start(
            VIEWPORT_CANVAS_WIDGET_ID,
            transparent_panel_theme(theme, 0.0),
        )),
        vec![chrome],
    );

    let options_popup = view_model.options_menu_open.then(|| {
        let mut popup_theme = theme.clone();
        popup_theme.background_panel = UiColor::new(
            theme.background_panel.r,
            theme.background_panel.g,
            theme.background_panel.b,
            0.96,
        );
        UiNode::with_children(
            VIEWPORT_OPTIONS_POPUP_WIDGET_ID,
            UiNodeKind::Popup(PopupNode::anchored_bottom_start(
                VIEWPORT_OPTIONS_BUTTON_WIDGET_ID,
                popup_theme,
            )),
            vec![
                button_selected(
                    VIEWPORT_DETAILS_TOGGLE_WIDGET_ID,
                    if view_model.details_visible {
                        "Details [x]"
                    } else {
                        "Details [ ]"
                    },
                    theme.body_small_text_style(FontId(1)),
                    theme.clone(),
                    view_model.details_visible,
                ),
                button_selected(
                    VIEWPORT_STATISTICS_TOGGLE_WIDGET_ID,
                    if view_model.statistics_visible {
                        "Statistics [x]"
                    } else {
                        "Statistics [ ]"
                    },
                    theme.body_small_text_style(FontId(1)),
                    theme.clone(),
                    view_model.statistics_visible,
                ),
            ],
        )
    });

    let status_overlay = (view_model.details_visible || view_model.statistics_visible).then(|| {
        let mut overlay_items = Vec::new();
        if view_model.details_visible {
            overlay_items.push(label(
                VIEWPORT_DETAILS_LABEL_WIDGET_ID,
                viewport_details_text(view_model),
                theme.body_small_text_style(FontId(1)),
            ));
        }
        if view_model.statistics_visible {
            overlay_items.push(label(
                VIEWPORT_STATISTICS_LABEL_WIDGET_ID,
                viewport_statistics_text(view_model),
                theme.body_small_text_style(FontId(1)),
            ));
        }
        UiNode::with_children(
            VIEWPORT_DETAILS_PANEL_WIDGET_ID,
            UiNodeKind::Popup(PopupNode::anchored_inside_bottom_start(
                VIEWPORT_CANVAS_WIDGET_ID,
                transparent_panel_theme(theme, 0.50),
            )),
            vec![hstack_with_policies(
                VIEWPORT_STATUS_WIDGET_ID,
                theme.spacing.sm,
                vec![SizePolicy::Auto; overlay_items.len()],
                overlay_items,
            )],
        )
    });

    let mut overlay_children = vec![chrome_overlay];
    if let Some(options_popup) = options_popup {
        overlay_children.push(options_popup);
    }
    if let Some(status_overlay) = status_overlay {
        overlay_children.push(status_overlay);
    }

    let mut canvas_children = vec![canvas_content];
    canvas_children.extend(overlay_children);
    let canvas_panel = panel(VIEWPORT_CANVAS_WIDGET_ID, canvas_theme, canvas_children);

    let body = vstack_with_policies(
        VIEWPORT_BODY_WIDGET_ID,
        0.0,
        vec![SizePolicy::flex(1.0)],
        vec![canvas_panel],
    );

    let mut viewport_theme = theme.clone();
    viewport_theme.background_panel = UiColor::new(
        theme.background_panel.r,
        theme.background_panel.g,
        theme.background_panel.b,
        0.0,
    );
    viewport_theme.border = UiColor::new(
        (theme.border.r + 0.05).clamp(0.0, 1.0),
        (theme.border.g + 0.06).clamp(0.0, 1.0),
        (theme.border.b + 0.09).clamp(0.0, 1.0),
        0.95,
    );
    panel(VIEWPORT_PANEL_WIDGET_ID, viewport_theme, vec![body])
}

fn transparent_panel_theme(theme: &ThemeTokens, alpha: f32) -> ThemeTokens {
    let mut panel_theme = theme.clone();
    panel_theme.background_panel = UiColor::new(
        theme.background_panel.r,
        theme.background_panel.g,
        theme.background_panel.b,
        alpha,
    );
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
    format!(
        "Statistics: drag={} preview={}",
        view_model.drag_in_progress, view_model.preview_active
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
            UiNodeKind::Button(button) if button.label == "Details [x]" && button.selected
        ));
        let statistics_button = find_node(&visible, VIEWPORT_STATISTICS_TOGGLE_WIDGET_ID)
            .expect("statistics button should exist");
        assert!(matches!(
            &statistics_button.kind,
            UiNodeKind::Button(button) if button.label == "Statistics [x]" && button.selected
        ));
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
                if label.text.contains("drag=true") && label.text.contains("preview=true")
        ));
    }
}
