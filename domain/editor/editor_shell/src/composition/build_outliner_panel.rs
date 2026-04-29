//! File: domain/editor/editor_shell/src/composition/build_outliner_panel.rs
//! Purpose: Compose outliner panel widgets.

use crate::{
    UiNode, UiNodeKind, button_selected, label, panel, vscroll, vstack, vstack_with_policies,
};
use ui_layout::SizePolicy;
use ui_math::{UiInsets, UiSize};
use ui_text::{FontId, TextOverflow};
use ui_theme::{ThemeTokens, UiColor};

use crate::{
    OUTLINER_BODY_WIDGET_ID, OUTLINER_LIST_WIDGET_ID, OUTLINER_PANEL_WIDGET_ID,
    OUTLINER_SCROLL_WIDGET_ID, OUTLINER_TITLE_WIDGET_ID, OutlinerViewModel, PanelInstanceId,
    ToolSurfaceInstanceId, outliner_row_widget_id,
};

pub fn build_outliner_panel(
    view_model: &OutlinerViewModel,
    theme: &ThemeTokens,
    _panel_instance_id: PanelInstanceId,
    _active_tool_surface: Option<ToolSurfaceInstanceId>,
) -> UiNode {
    let title = label(
        OUTLINER_TITLE_WIDGET_ID,
        "Outliner",
        theme.heading_text_style(FontId(1)),
    );

    let mut row_style = theme.body_text_style(FontId(1));
    row_style.overflow = TextOverflow::Ellipsis;

    let rows = view_model
        .rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let indent = "  ".repeat(row.depth);
            compact_outliner_row(button_selected(
                outliner_row_widget_id(index),
                format!("{indent}{}", row.display_name),
                row_style.clone(),
                theme.clone(),
                row.is_selected,
            ))
        })
        .collect::<Vec<_>>();

    let list = vstack(
        OUTLINER_LIST_WIDGET_ID,
        (theme.spacing.xs * 0.60).max(1.5),
        rows,
    );
    let scroll = vscroll(OUTLINER_SCROLL_WIDGET_ID, theme.clone(), vec![list]);
    let body = vstack_with_policies(
        OUTLINER_BODY_WIDGET_ID,
        theme.spacing.xs,
        vec![SizePolicy::Auto, SizePolicy::flex(1.0)],
        vec![title, scroll],
    );
    let mut panel_theme = theme.clone();
    panel_theme.background_panel = UiColor::new(
        (theme.background_panel.r + 0.01).clamp(0.0, 1.0),
        (theme.background_panel.g + 0.01).clamp(0.0, 1.0),
        (theme.background_panel.b + 0.01).clamp(0.0, 1.0),
        0.94,
    );
    panel(OUTLINER_PANEL_WIDGET_ID, panel_theme, vec![body])
}

fn compact_outliner_row(mut node: UiNode) -> UiNode {
    if let UiNodeKind::Button(button) = &mut node.kind {
        let vertical = (button.theme.spacing.xs * 0.45).max(1.0);
        let horizontal = (button.theme.spacing.sm * 0.80).max(2.0);
        button.padding = UiInsets::new(horizontal, vertical, horizontal, vertical);
        let line_height = button
            .text_style
            .line_height_or_default(button.text_style.font_size * 1.2);
        button.min_size = UiSize::new(0.0, (line_height + button.padding.vertical()).max(12.0));
    }
    node
}
