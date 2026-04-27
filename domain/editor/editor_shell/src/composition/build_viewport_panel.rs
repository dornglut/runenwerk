//! File: domain/editor/editor_shell/src/composition/build_viewport_panel.rs
//! Purpose: Compose viewport panel widgets.

use crate::{UiNode, panel, viewport_surface_embed, vstack_with_policies};
use editor_viewport::ViewportSurfacePresentationSlot;
use ui_layout::SizePolicy;
use ui_theme::{ThemeTokens, UiColor};

use crate::{
    PanelInstanceId, ToolSurfaceInstanceId, VIEWPORT_BODY_WIDGET_ID,
    VIEWPORT_CANVAS_CONTENT_WIDGET_ID, VIEWPORT_CANVAS_WIDGET_ID, VIEWPORT_PANEL_WIDGET_ID,
    VIEWPORT_SURFACE_EMBED_WIDGET_ID, ViewportViewModel, viewport_embed_slot_for,
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

    let body = vstack_with_policies(
        VIEWPORT_BODY_WIDGET_ID,
        theme.spacing.sm,
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
