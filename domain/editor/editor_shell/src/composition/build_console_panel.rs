//! File: domain/editor/editor_shell/src/composition/build_console_panel.rs
//! Purpose: Compose console panel widgets.

use crate::{UiNode, label, panel, vscroll, vstack, vstack_with_policies};
use ui_layout::SizePolicy;
use ui_text::FontId;
use ui_theme::{ThemeTokens, UiColor};

use crate::{
    CONSOLE_BODY_WIDGET_ID, CONSOLE_LIST_WIDGET_ID, CONSOLE_PANEL_WIDGET_ID,
    CONSOLE_SCROLL_WIDGET_ID, ConsoleViewModel, PanelInstanceId, ToolSurfaceInstanceId,
    console_line_widget_id,
};

pub fn build_console_panel(
    view_model: &ConsoleViewModel,
    theme: &ThemeTokens,
    _panel_instance_id: PanelInstanceId,
    _active_tool_surface: Option<ToolSurfaceInstanceId>,
) -> UiNode {
    let row_style = theme.body_small_text_style(FontId(1));
    let mut rows = Vec::with_capacity(view_model.lines.len());
    for (index, line) in view_model.lines.iter().enumerate() {
        rows.push(label(
            console_line_widget_id(index),
            line.clone(),
            row_style.clone(),
        ));
    }

    let list = vstack(
        CONSOLE_LIST_WIDGET_ID,
        (theme.spacing.xs * 0.35).max(1.0),
        rows,
    );
    let scroll = vscroll(CONSOLE_SCROLL_WIDGET_ID, theme.clone(), vec![list]);
    let body = vstack_with_policies(
        CONSOLE_BODY_WIDGET_ID,
        theme.spacing.xs,
        vec![SizePolicy::flex(1.0)],
        vec![scroll],
    );
    let mut panel_theme = theme.clone();
    panel_theme.background_panel = UiColor::new(
        (theme.background_panel.r - 0.01).clamp(0.0, 1.0),
        (theme.background_panel.g - 0.005).clamp(0.0, 1.0),
        (theme.background_panel.b + 0.01).clamp(0.0, 1.0),
        0.94,
    );
    panel(CONSOLE_PANEL_WIDGET_ID, panel_theme, vec![body])
}
