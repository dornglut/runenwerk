//! File: domain/editor/editor_shell/src/composition/build_viewport_panel.rs
//! Purpose: Compose viewport panel widgets.

use crate::{
    UiNode, button, label, panel, viewport_product_button_widget_id, viewport_surface_embed, vstack,
    vstack_with_policies,
};
use ui_render_data::ViewportSurfaceSlot;
use ui_layout::SizePolicy;
use ui_text::FontId;
use ui_theme::{ThemeTokens, UiColor};

use crate::{
    VIEWPORT_BODY_WIDGET_ID, VIEWPORT_CANVAS_CONTENT_WIDGET_ID, VIEWPORT_CANVAS_LABEL_WIDGET_ID,
    VIEWPORT_CANVAS_WIDGET_ID, VIEWPORT_CHROME_CONTENT_WIDGET_ID, VIEWPORT_CHROME_WIDGET_ID,
    VIEWPORT_PANEL_WIDGET_ID, VIEWPORT_PRODUCTS_LIST_WIDGET_ID, VIEWPORT_PRODUCTS_TITLE_WIDGET_ID,
    VIEWPORT_STATUS_WIDGET_ID, VIEWPORT_SURFACE_EMBED_WIDGET_ID, VIEWPORT_TITLE_WIDGET_ID,
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
            "viewport={:?} selected_product={:?} selected_entity={:?} hovered={:?} dragging={} preview={}",
            view_model.viewport_id,
            view_model.selected_primary_product_id,
            view_model.selected_entity,
            view_model.hovered_entity,
            view_model.drag_in_progress,
            view_model.preview_active
        ),
        theme.body_small_text_style(FontId(1)),
    );

    let products_title = label(
        VIEWPORT_PRODUCTS_TITLE_WIDGET_ID,
        "Products",
        theme.body_small_text_style(FontId(1)),
    );
    let product_rows = view_model
        .product_choices
        .iter()
        .enumerate()
        .map(|(index, choice)| {
            let status_prefix = if choice.selected { "• " } else { "" };
            let availability_suffix = if choice.enabled { "" } else { " (unavailable)" };
            let mut node = button(
                viewport_product_button_widget_id(index),
                format!("{status_prefix}{}{}", choice.label, availability_suffix),
                theme.body_small_text_style(FontId(1)),
                theme.clone(),
            );
            if let crate::UiNodeKind::Button(value) = &mut node.kind {
                value.enabled = choice.enabled;
            }
            node
        })
        .collect::<Vec<_>>();
    let products = vstack(
        VIEWPORT_PRODUCTS_LIST_WIDGET_ID,
        (theme.spacing.xs * 0.75).max(2.0),
        product_rows,
    );

    let chrome_content = vstack(
        VIEWPORT_CHROME_CONTENT_WIDGET_ID,
        theme.spacing.sm,
        vec![title, status, products_title, products],
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
    let viewport_surface = viewport_surface_embed(
        VIEWPORT_SURFACE_EMBED_WIDGET_ID,
        1,
        ViewportSurfaceSlot::Primary,
    );
    let canvas_content = vstack_with_policies(
        VIEWPORT_CANVAS_CONTENT_WIDGET_ID,
        theme.spacing.xs,
        vec![SizePolicy::Auto, SizePolicy::flex(1.0)],
        vec![canvas_label, viewport_surface],
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
