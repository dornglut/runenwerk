//! File: domain/editor/editor_shell/src/composition/build_entity_table_panel.rs
//! Purpose: Compose entity table panel widgets from the checked-in surface fixture.

use crate::{
    ENTITY_TABLE_BODY_WIDGET_ID, ENTITY_TABLE_CLEAR_SEARCH_WIDGET_ID,
    ENTITY_TABLE_COMPONENT_FILTER_SELECT_WIDGET_ID, ENTITY_TABLE_CONTROLS_ROW_WIDGET_ID,
    ENTITY_TABLE_CONTROLS_SCROLL_WIDGET_ID, ENTITY_TABLE_HEADER_ROW_WIDGET_ID,
    ENTITY_TABLE_HEADER_SCROLL_WIDGET_ID, ENTITY_TABLE_LIST_WIDGET_ID,
    ENTITY_TABLE_PANEL_WIDGET_ID, ENTITY_TABLE_ROOTS_ONLY_TOGGLE_WIDGET_ID,
    ENTITY_TABLE_SCROLL_WIDGET_ID, ENTITY_TABLE_SEARCH_WIDGET_ID,
    ENTITY_TABLE_SELECTED_ONLY_TOGGLE_WIDGET_ID, ENTITY_TABLE_TABLE_SCROLL_WIDGET_ID,
    ENTITY_TABLE_TITLE_WIDGET_ID, EntityTableComponentFilter, EntityTableSortKey,
    EntityTableViewModel, PanelInstanceId, SurfaceWidgetScope, ToolSurfaceInstanceId, UiNode,
    UiNodeKind, entity_table_sort_button_widget_id,
};
use ui_definition::{
    AuthoredId, AuthoredUiTemplate, UiCollectionItem, UiDefinitionContext, UiValue,
    form_retained_ui, normalize_authored_template,
};
use ui_layout::SizePolicy;
use ui_text::{FontId, TextOverflow};
use ui_theme::ThemeTokens;

use super::surface_control_polish::{
    apply_compact_surface_control_polish, apply_horizontal_control_rail_polish,
    set_control_min_width,
};
use super::surface_definition_context::{
    apply_panel_background, apply_surface_title_polish, find_node_mut, register_widget_ids_by_path,
    scoped_definition_context, set_stack_child_main_policies, toned_panel_background,
};

const ENTITY_TABLE_TEMPLATE_RON: &str =
    include_str!("../../../../../assets/editor/ui/surfaces/entity_table.ron");

pub fn build_entity_table_panel(
    view_model: &EntityTableViewModel,
    theme: &ThemeTokens,
    _panel_instance_id: PanelInstanceId,
    active_tool_surface: Option<ToolSurfaceInstanceId>,
) -> UiNode {
    let template: AuthoredUiTemplate = ron::from_str(ENTITY_TABLE_TEMPLATE_RON)
        .expect("checked-in entity table UI fixture must parse");
    let normalized = normalize_authored_template(template);
    let scope = SurfaceWidgetScope::optional(active_tool_surface);
    let mut context = scoped_definition_context(theme, scope);
    register_entity_table_widget_ids(&mut context, scope);
    populate_entity_table_context(view_model, &mut context);

    let mut root = form_retained_ui(&normalized, &mut context).root;
    polish_entity_table(&mut root, theme, scope);
    root
}

fn register_entity_table_widget_ids(context: &mut UiDefinitionContext, scope: SurfaceWidgetScope) {
    register_widget_ids_by_path(
        context,
        scope,
        [
            ("root", ENTITY_TABLE_PANEL_WIDGET_ID),
            ("root/body", ENTITY_TABLE_BODY_WIDGET_ID),
            ("root/body/title", ENTITY_TABLE_TITLE_WIDGET_ID),
            (
                "root/body/controls_scroll",
                ENTITY_TABLE_CONTROLS_SCROLL_WIDGET_ID,
            ),
            (
                "root/body/controls_scroll/controls",
                ENTITY_TABLE_CONTROLS_ROW_WIDGET_ID,
            ),
            (
                "root/body/controls_scroll/controls/search",
                ENTITY_TABLE_SEARCH_WIDGET_ID,
            ),
            (
                "root/body/controls_scroll/controls/clear_search",
                ENTITY_TABLE_CLEAR_SEARCH_WIDGET_ID,
            ),
            (
                "root/body/controls_scroll/controls/selected_only",
                ENTITY_TABLE_SELECTED_ONLY_TOGGLE_WIDGET_ID,
            ),
            (
                "root/body/controls_scroll/controls/roots_only",
                ENTITY_TABLE_ROOTS_ONLY_TOGGLE_WIDGET_ID,
            ),
            (
                "root/body/controls_scroll/controls/component_filter",
                ENTITY_TABLE_COMPONENT_FILTER_SELECT_WIDGET_ID,
            ),
            (
                "root/body/header_scroll",
                ENTITY_TABLE_HEADER_SCROLL_WIDGET_ID,
            ),
            (
                "root/body/header_scroll/header_row",
                ENTITY_TABLE_HEADER_ROW_WIDGET_ID,
            ),
            (
                "root/body/header_scroll/header_row/sort_id",
                entity_table_sort_button_widget_id(0),
            ),
            (
                "root/body/header_scroll/header_row/sort_name",
                entity_table_sort_button_widget_id(1),
            ),
            (
                "root/body/header_scroll/header_row/sort_parent",
                entity_table_sort_button_widget_id(2),
            ),
            (
                "root/body/header_scroll/header_row/sort_components",
                entity_table_sort_button_widget_id(3),
            ),
            ("root/body/table_scroll", ENTITY_TABLE_SCROLL_WIDGET_ID),
            (
                "root/body/table_scroll/table_scroll_x",
                ENTITY_TABLE_TABLE_SCROLL_WIDGET_ID,
            ),
            (
                "root/body/table_scroll/table_scroll_x/table",
                ENTITY_TABLE_LIST_WIDGET_ID,
            ),
        ],
    );
}

fn populate_entity_table_context(
    view_model: &EntityTableViewModel,
    context: &mut UiDefinitionContext,
) {
    context.values.insert(
        "entity_table.search".into(),
        UiValue::Text(view_model.search_query.clone()),
    );
    context.values.insert(
        "entity_table.selected_only".into(),
        UiValue::Bool(view_model.query.selected_only),
    );
    context.values.insert(
        "entity_table.roots_only".into(),
        UiValue::Bool(matches!(
            view_model.query.hierarchy_filter,
            crate::EntityTableHierarchyFilter::RootsOnly
        )),
    );
    context.collections.insert(
        "entity_table.component_filters".into(),
        view_model
            .component_filters
            .iter()
            .map(|item| {
                UiCollectionItem::new(component_filter_key(item.filter), item.label.clone())
            })
            .collect(),
    );
    context.selections.insert(
        "entity_table.component_filter.selected".into(),
        component_filter_key(view_model.query.component_filter),
    );
    for (slot_prefix, label, sort_key) in [
        ("id", "Id", EntityTableSortKey::EntityId),
        ("name", "Name", EntityTableSortKey::DisplayName),
        ("parent", "Parent", EntityTableSortKey::Parent),
        (
            "components",
            "Components",
            EntityTableSortKey::ComponentCount,
        ),
    ] {
        let active = view_model.sort_key == sort_key;
        let marker = if active {
            if view_model.sort_ascending {
                " ^"
            } else {
                " v"
            }
        } else {
            ""
        };
        context.values.insert(
            AuthoredId::new(format!("entity_table.sort.{slot_prefix}.label")),
            UiValue::Text(format!("{label}{marker}")),
        );
        context.values.insert(
            AuthoredId::new(format!("entity_table.sort.{slot_prefix}.active")),
            UiValue::Bool(active),
        );
    }
    context.collections.insert(
        "entity_table.rows".into(),
        view_model
            .rows
            .iter()
            .map(|row| {
                let mut item =
                    UiCollectionItem::new(row.entity.0.to_string(), row.display_name.clone());
                item.selected = row.is_selected;
                item.values.insert(
                    "entity_table.row.id".into(),
                    UiValue::Text(row.entity_id_label.clone()),
                );
                item.values.insert(
                    "entity_table.row.name".into(),
                    UiValue::Text(row.display_name.clone()),
                );
                item.values.insert(
                    "entity_table.row.parent".into(),
                    UiValue::Text(row.parent_label.clone()),
                );
                item.values.insert(
                    "entity_table.row.components".into(),
                    UiValue::Text(row.component_count.to_string()),
                );
                item
            })
            .collect(),
    );
}

fn polish_entity_table(root: &mut UiNode, theme: &ThemeTokens, scope: SurfaceWidgetScope) {
    apply_panel_background(root, toned_panel_background(theme, 0.012, 0.94));
    set_stack_child_main_policies(
        root,
        scope.widget_id(ENTITY_TABLE_BODY_WIDGET_ID),
        vec![
            SizePolicy::Auto,
            SizePolicy::Auto,
            SizePolicy::Auto,
            SizePolicy::flex(1.0),
        ],
    );
    apply_surface_title_polish(root, scope.widget_id(ENTITY_TABLE_TITLE_WIDGET_ID), theme);
    if let Some(controls_scroll) = find_node_mut(
        root,
        scope.widget_id(ENTITY_TABLE_CONTROLS_SCROLL_WIDGET_ID),
    ) {
        apply_horizontal_control_rail_polish(controls_scroll, theme);
    }
    if let Some(search) = find_node_mut(root, scope.widget_id(ENTITY_TABLE_SEARCH_WIDGET_ID))
        && matches!(&search.kind, UiNodeKind::TextInput(_))
    {
        apply_compact_surface_control_polish(search, theme);
        set_control_min_width(search, 144.0);
    }
    for (widget_id, min_width) in [
        (ENTITY_TABLE_CLEAR_SEARCH_WIDGET_ID, 56.0),
        (ENTITY_TABLE_SELECTED_ONLY_TOGGLE_WIDGET_ID, 112.0),
        (ENTITY_TABLE_ROOTS_ONLY_TOGGLE_WIDGET_ID, 108.0),
        (ENTITY_TABLE_COMPONENT_FILTER_SELECT_WIDGET_ID, 132.0),
    ] {
        if let Some(node) = find_node_mut(root, scope.widget_id(widget_id)) {
            apply_compact_surface_control_polish(node, theme);
            set_control_min_width(node, min_width);
        }
    }
    for index in 0..4 {
        if let Some(sort_button) = find_node_mut(
            root,
            scope.widget_id(entity_table_sort_button_widget_id(index)),
        ) && matches!(&sort_button.kind, UiNodeKind::Button(_))
        {
            apply_compact_surface_control_polish(sort_button, theme);
        }
    }
    if let Some(table) = find_node_mut(root, scope.widget_id(ENTITY_TABLE_LIST_WIDGET_ID))
        && let UiNodeKind::Table(table) = &mut table.kind
    {
        table.text_style = theme.body_small_text_style(FontId(1));
        table.text_style.overflow = TextOverflow::Ellipsis;
        table.header_text_style = theme.body_small_text_style(FontId(1));
    }
}

fn component_filter_key(filter: EntityTableComponentFilter) -> String {
    match filter {
        EntityTableComponentFilter::All => "all".to_string(),
        EntityTableComponentFilter::Has(component_type) => {
            format!("component:{}", component_type.0)
        }
    }
}
