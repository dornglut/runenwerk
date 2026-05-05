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

use crate::{
    PanelInstanceId, ToolSurfaceInstanceId, VIEWPORT_BODY_WIDGET_ID,
    VIEWPORT_CANVAS_CONTENT_WIDGET_ID, VIEWPORT_CANVAS_WIDGET_ID, VIEWPORT_CHROME_WIDGET_ID,
    VIEWPORT_DETAILS_LABEL_WIDGET_ID, VIEWPORT_DETAILS_PANEL_WIDGET_ID,
    VIEWPORT_DETAILS_TOGGLE_WIDGET_ID, VIEWPORT_PANEL_WIDGET_ID, VIEWPORT_SURFACE_EMBED_WIDGET_ID,
    ViewportViewModel, viewport_embed_slot_for,
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
    let canvas_panel = panel(
        VIEWPORT_CANVAS_WIDGET_ID,
        canvas_theme,
        vec![canvas_content],
    );

    let chrome = hstack_with_policies(
        VIEWPORT_CHROME_WIDGET_ID,
        theme.spacing.xs,
        vec![SizePolicy::Auto],
        vec![button_selected(
            VIEWPORT_DETAILS_TOGGLE_WIDGET_ID,
            if view_model.details_visible {
                "Hide Details"
            } else {
                "Show Details"
            },
            theme.body_small_text_style(FontId(1)),
            theme.clone(),
            view_model.details_visible,
        )],
    );

    let details_panel = view_model.details_visible.then(|| {
        let mut details_theme = theme.clone();
        details_theme.background_panel = UiColor::new(
            theme.background_panel.r,
            theme.background_panel.g,
            theme.background_panel.b,
            0.78,
        );
        let viewport = view_model
            .viewport_id
            .map(|viewport| viewport.0.to_string())
            .unwrap_or_else(|| "unbound".to_string());
        let selected_entity = view_model
            .selected_entity
            .map(|entity| entity.0.to_string())
            .unwrap_or_else(|| "none".to_string());
        panel(
            VIEWPORT_DETAILS_PANEL_WIDGET_ID,
            details_theme,
            vec![label(
                VIEWPORT_DETAILS_LABEL_WIDGET_ID,
                format!(
                    "Viewport Details: viewport={viewport} selected_entity={selected_entity} drag={} preview={}",
                    view_model.drag_in_progress, view_model.preview_active
                ),
                theme.body_small_text_style(FontId(1)),
            )],
        )
    });

    let mut body_policies = vec![SizePolicy::Auto];
    let mut body_children = vec![chrome];
    if let Some(details_panel) = details_panel {
        body_policies.push(SizePolicy::Auto);
        body_children.push(details_panel);
    }
    body_policies.push(SizePolicy::flex(1.0));
    body_children.push(canvas_panel);

    let body = vstack_with_policies(
        VIEWPORT_BODY_WIDGET_ID,
        theme.spacing.sm,
        body_policies,
        body_children,
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
    fn viewport_details_toggle_label_reflects_visibility() {
        let theme = ThemeTokens::default();
        let hidden = build_viewport_panel(
            &ViewportViewModel::default(),
            &theme,
            PanelInstanceId::new(1),
            None,
        );
        let hidden_button = find_node(&hidden, VIEWPORT_DETAILS_TOGGLE_WIDGET_ID)
            .expect("details button should exist");
        assert!(matches!(
            &hidden_button.kind,
            UiNodeKind::Button(button) if button.label == "Show Details" && !button.selected
        ));

        let visible_model = ViewportViewModel {
            details_visible: true,
            ..Default::default()
        };
        let visible = build_viewport_panel(&visible_model, &theme, PanelInstanceId::new(1), None);
        let visible_button = find_node(&visible, VIEWPORT_DETAILS_TOGGLE_WIDGET_ID)
            .expect("details button should exist");
        assert!(matches!(
            &visible_button.kind,
            UiNodeKind::Button(button) if button.label == "Hide Details" && button.selected
        ));
    }

    #[test]
    fn viewport_details_content_is_visible_only_when_enabled() {
        let theme = ThemeTokens::default();
        let hidden = build_viewport_panel(
            &ViewportViewModel::default(),
            &theme,
            PanelInstanceId::new(1),
            None,
        );
        assert!(find_node(&hidden, VIEWPORT_DETAILS_PANEL_WIDGET_ID).is_none());

        let visible_model = ViewportViewModel {
            details_visible: true,
            viewport_id: Some(editor_viewport::ViewportId(7)),
            selected_entity: Some(editor_core::EntityId(42)),
            drag_in_progress: true,
            preview_active: true,
            ..Default::default()
        };
        let visible = build_viewport_panel(&visible_model, &theme, PanelInstanceId::new(1), None);
        assert!(find_node(&visible, VIEWPORT_DETAILS_PANEL_WIDGET_ID).is_some());
        let details_label = find_node(&visible, VIEWPORT_DETAILS_LABEL_WIDGET_ID)
            .expect("details label should exist");
        assert!(matches!(
            &details_label.kind,
            UiNodeKind::Label(label)
                if label.text.contains("viewport=7")
                    && label.text.contains("selected_entity=42")
                    && label.text.contains("drag=true")
                    && label.text.contains("preview=true")
        ));
    }
}
