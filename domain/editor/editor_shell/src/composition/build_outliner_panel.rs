//! File: domain/editor/editor_shell/src/composition/build_outliner_panel.rs
//! Purpose: Compose outliner panel widgets.

use crate::{UiNode, label, panel, vscroll, vstack_with_policies};
use ui_layout::SizePolicy;
use ui_text::{FontId, TextOverflow};
use ui_theme::{ThemeTokens, UiColor};
use ui_tree::TreeRow;
use ui_widgets::tree;

use crate::{
    OUTLINER_BODY_WIDGET_ID, OUTLINER_LIST_WIDGET_ID, OUTLINER_PANEL_WIDGET_ID,
    OUTLINER_SCROLL_WIDGET_ID, OUTLINER_TITLE_WIDGET_ID, OutlinerViewModel, PanelInstanceId,
    ToolSurfaceInstanceId,
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
        .map(|row| {
            let mut tree_row = TreeRow::new(row.display_name.clone(), row.depth, row.has_children);
            tree_row.selected = row.is_selected;
            tree_row.expanded = true;
            tree_row
        })
        .collect::<Vec<_>>();

    let list = tree(OUTLINER_LIST_WIDGET_ID, rows, row_style, theme.clone());
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
