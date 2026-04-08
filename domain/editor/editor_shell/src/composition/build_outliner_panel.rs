//! File: domain/editor/editor_shell/src/composition/build_outliner_panel.rs
//! Purpose: Compose outliner panel widgets.

use crate::{UiNode, button, label, panel, vstack};
use ui_text::FontId;
use ui_theme::ThemeTokens;

use crate::{
    OUTLINER_LIST_WIDGET_ID, OUTLINER_PANEL_WIDGET_ID, OUTLINER_TITLE_WIDGET_ID, OutlinerViewModel,
    outliner_row_widget_id,
};

pub fn build_outliner_panel(view_model: &OutlinerViewModel, theme: &ThemeTokens) -> UiNode {
    let title = label(
        OUTLINER_TITLE_WIDGET_ID,
        "Outliner",
        theme.heading_text_style(FontId(1)),
    );

    let rows = view_model
        .rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let indent = "  ".repeat(row.depth);
            let prefix = if row.is_selected { "• " } else { "" };
            button(
                outliner_row_widget_id(index),
                format!("{indent}{prefix}{}", row.display_name),
                theme.body_text_style(FontId(1)),
                theme.clone(),
            )
        })
        .collect::<Vec<_>>();

    let list = vstack(OUTLINER_LIST_WIDGET_ID, theme.spacing.xs, rows);
    panel(OUTLINER_PANEL_WIDGET_ID, theme.clone(), vec![title, list])
}
