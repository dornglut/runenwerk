//! File: domain/editor/editor_shell/src/composition/build_viewport_panel.rs
//! Purpose: Compose viewport panel widgets.

use crate::{UiNode, label, panel, vstack, vstack_with_policies};
use ui_layout::SizePolicy;
use ui_text::FontId;
use ui_theme::{ThemeTokens, UiColor};

use crate::{
    VIEWPORT_BODY_WIDGET_ID, VIEWPORT_CANVAS_CONTENT_WIDGET_ID, VIEWPORT_CANVAS_LABEL_WIDGET_ID,
    VIEWPORT_CANVAS_WIDGET_ID, VIEWPORT_CHROME_CONTENT_WIDGET_ID, VIEWPORT_CHROME_WIDGET_ID,
    VIEWPORT_PANEL_WIDGET_ID, VIEWPORT_STATUS_WIDGET_ID, VIEWPORT_TITLE_WIDGET_ID,
    ViewportViewModel,
};

pub fn build_viewport_panel(view_model: &ViewportViewModel, theme: &ThemeTokens) -> UiNode {
    let title = label(
        VIEWPORT_TITLE_WIDGET_ID,
        "Viewport",
        theme.heading_text_style(FontId(1)),
    );

    let status = label(
        VIEWPORT_STATUS_WIDGET_ID,
        format!(
            "selected={:?} hovered={:?} dragging={} preview={}",
            view_model.selected_entity,
            view_model.hovered_entity,
            view_model.drag_in_progress,
            view_model.preview_active
        ),
        theme.body_small_text_style(FontId(1)),
    );

    let chrome_content = vstack(
        VIEWPORT_CHROME_CONTENT_WIDGET_ID,
        theme.spacing.sm,
        vec![title, status],
    );
    let mut chrome_theme = theme.clone();
    chrome_theme.background_panel = UiColor::new(
        (theme.background_panel.r + 0.01).clamp(0.0, 1.0),
        (theme.background_panel.g + 0.01).clamp(0.0, 1.0),
        (theme.background_panel.b + 0.05).clamp(0.0, 1.0),
        0.88,
    );
    let chrome_panel = panel(
        VIEWPORT_CHROME_WIDGET_ID,
        chrome_theme,
        vec![chrome_content],
    );

    let canvas_label = label(
        VIEWPORT_CANVAS_LABEL_WIDGET_ID,
        "Viewport Canvas",
        theme.body_small_text_style(FontId(1)),
    );
    let canvas_content = vstack(
        VIEWPORT_CANVAS_CONTENT_WIDGET_ID,
        theme.spacing.xs,
        vec![canvas_label],
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
        vec![SizePolicy::Auto, SizePolicy::flex(1.0)],
        vec![chrome_panel, canvas_panel],
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
