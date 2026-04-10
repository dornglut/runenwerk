//! File: domain/editor/editor_shell/src/composition/build_console_panel.rs
//! Purpose: Compose console panel widgets.

use crate::{UiNode, label, panel, vstack};
use ui_text::FontId;
use ui_theme::ThemeTokens;

use crate::{
    CONSOLE_LIST_WIDGET_ID, CONSOLE_PANEL_WIDGET_ID, CONSOLE_TITLE_WIDGET_ID, ConsoleViewModel,
    console_line_widget_id,
};

pub fn build_console_panel(view_model: &ConsoleViewModel, theme: &ThemeTokens) -> UiNode {
    let title = label(
        CONSOLE_TITLE_WIDGET_ID,
        "Console",
        theme.heading_text_style(FontId(1)),
    );

    let mut rows = Vec::with_capacity(view_model.lines.len());
    for (index, line) in view_model.lines.iter().enumerate() {
        rows.push(label(
            console_line_widget_id(index),
            line.clone(),
            theme.body_small_text_style(FontId(1)),
        ));
    }

    let list = vstack(CONSOLE_LIST_WIDGET_ID, theme.spacing.xs, rows);
    panel(CONSOLE_PANEL_WIDGET_ID, theme.clone(), vec![title, list])
}
