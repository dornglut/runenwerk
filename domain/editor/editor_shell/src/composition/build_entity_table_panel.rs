//! File: domain/editor/editor_shell/src/composition/build_entity_table_panel.rs
//! Purpose: Compose entity table panel widgets.

use crate::{
    ENTITY_TABLE_BODY_WIDGET_ID, ENTITY_TABLE_HEADER_ROW_WIDGET_ID,
    ENTITY_TABLE_HEADER_SCROLL_WIDGET_ID, ENTITY_TABLE_LIST_WIDGET_ID,
    ENTITY_TABLE_PANEL_WIDGET_ID, ENTITY_TABLE_SCROLL_WIDGET_ID, ENTITY_TABLE_SEARCH_WIDGET_ID,
    ENTITY_TABLE_TABLE_SCROLL_WIDGET_ID, ENTITY_TABLE_TITLE_WIDGET_ID, EntityTableSortKey,
    EntityTableViewModel, PanelInstanceId, TableColumn, TableRow, ToolSurfaceInstanceId, UiNode,
    button_selected, entity_table_sort_button_widget_id, hscroll, hstack_with_policies, label,
    panel, search_field, table, vscroll, vstack_with_policies,
};
use ui_layout::SizePolicy;
use ui_text::{FontId, TextOverflow};
use ui_theme::{ThemeTokens, UiColor};

pub fn build_entity_table_panel(
    view_model: &EntityTableViewModel,
    theme: &ThemeTokens,
    _panel_instance_id: PanelInstanceId,
    _active_tool_surface: Option<ToolSurfaceInstanceId>,
) -> UiNode {
    let title = label(
        ENTITY_TABLE_TITLE_WIDGET_ID,
        "Entity Table",
        theme.heading_text_style(FontId(1)),
    );

    let mut search_style = theme.body_small_text_style(FontId(1));
    search_style.overflow = TextOverflow::Ellipsis;
    let search = search_field(
        ENTITY_TABLE_SEARCH_WIDGET_ID,
        view_model.search_query.clone(),
        search_style,
        theme.clone(),
    );

    let sort_buttons_row = hstack_with_policies(
        ENTITY_TABLE_HEADER_ROW_WIDGET_ID,
        theme.spacing.xs,
        vec![SizePolicy::Auto; 4],
        vec![
            sort_button(0, "Id", EntityTableSortKey::EntityId, view_model, theme),
            sort_button(
                1,
                "Name",
                EntityTableSortKey::DisplayName,
                view_model,
                theme,
            ),
            sort_button(2, "Parent", EntityTableSortKey::Parent, view_model, theme),
            sort_button(
                3,
                "Components",
                EntityTableSortKey::ComponentCount,
                view_model,
                theme,
            ),
        ],
    );
    let sort_buttons = hscroll(
        ENTITY_TABLE_HEADER_SCROLL_WIDGET_ID,
        theme.clone(),
        vec![sort_buttons_row],
    );

    let mut row_style = theme.body_small_text_style(FontId(1));
    row_style.overflow = TextOverflow::Ellipsis;
    let header_style = theme.body_small_text_style(FontId(1));
    let rows = view_model
        .rows
        .iter()
        .map(|row| {
            let mut table_row = TableRow::new([
                row.entity_id_label.clone(),
                row.display_name.clone(),
                row.parent_label.clone(),
                row.component_count.to_string(),
            ]);
            table_row.selected = row.is_selected;
            table_row
        })
        .collect::<Vec<_>>();
    let entity_table = table(
        ENTITY_TABLE_LIST_WIDGET_ID,
        [
            TableColumn::new("Id", 52.0),
            TableColumn::new("Name", 128.0),
            TableColumn::new("Parent", 112.0),
            TableColumn::new("Components", 88.0),
        ],
        rows,
        row_style,
        header_style,
        theme.clone(),
    );
    let table_scroll_x = hscroll(
        ENTITY_TABLE_TABLE_SCROLL_WIDGET_ID,
        theme.clone(),
        vec![entity_table],
    );
    let scroll = vscroll(
        ENTITY_TABLE_SCROLL_WIDGET_ID,
        theme.clone(),
        vec![table_scroll_x],
    );
    let body = vstack_with_policies(
        ENTITY_TABLE_BODY_WIDGET_ID,
        theme.spacing.xs,
        vec![
            SizePolicy::Auto,
            SizePolicy::Auto,
            SizePolicy::Auto,
            SizePolicy::flex(1.0),
        ],
        vec![title, search, sort_buttons, scroll],
    );
    let mut panel_theme = theme.clone();
    panel_theme.background_panel = UiColor::new(
        (theme.background_panel.r + 0.012).clamp(0.0, 1.0),
        (theme.background_panel.g + 0.012).clamp(0.0, 1.0),
        (theme.background_panel.b + 0.012).clamp(0.0, 1.0),
        0.94,
    );
    panel(ENTITY_TABLE_PANEL_WIDGET_ID, panel_theme, vec![body])
}

fn sort_button(
    index: usize,
    label_text: &str,
    key: EntityTableSortKey,
    view_model: &EntityTableViewModel,
    theme: &ThemeTokens,
) -> UiNode {
    let marker = if view_model.sort_key == key {
        if view_model.sort_ascending {
            " ^"
        } else {
            " v"
        }
    } else {
        ""
    };
    button_selected(
        entity_table_sort_button_widget_id(index),
        format!("{label_text}{marker}"),
        theme.body_small_text_style(FontId(1)),
        theme.clone(),
        view_model.sort_key == key,
    )
}
