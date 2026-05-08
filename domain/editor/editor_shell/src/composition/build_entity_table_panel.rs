//! File: domain/editor/editor_shell/src/composition/build_entity_table_panel.rs
//! Purpose: Compose entity table panel widgets from the checked-in surface fixture.

use crate::{
    ENTITY_TABLE_BODY_WIDGET_ID, ENTITY_TABLE_CLEAR_SEARCH_WIDGET_ID,
    ENTITY_TABLE_COMPONENT_FILTER_SELECT_WIDGET_ID, ENTITY_TABLE_HEADER_ROW_WIDGET_ID,
    ENTITY_TABLE_HEADER_SCROLL_WIDGET_ID, ENTITY_TABLE_LIST_WIDGET_ID,
    ENTITY_TABLE_PANEL_WIDGET_ID, ENTITY_TABLE_ROOTS_ONLY_TOGGLE_WIDGET_ID,
    ENTITY_TABLE_SCROLL_WIDGET_ID, ENTITY_TABLE_SEARCH_WIDGET_ID,
    ENTITY_TABLE_SELECTED_ONLY_TOGGLE_WIDGET_ID, ENTITY_TABLE_TABLE_SCROLL_WIDGET_ID,
    ENTITY_TABLE_TITLE_WIDGET_ID, EntityTableComponentFilter, EntityTableSortKey,
    EntityTableViewModel, PanelInstanceId, ToolSurfaceInstanceId, UiNode, UiNodeKind,
    entity_table_sort_button_widget_id,
};
use ui_definition::{
    AuthoredId, AuthoredUiNodePath, AuthoredUiTemplate, UiCollectionItem, UiDefinitionContext,
    UiValue, form_retained_ui, normalize_authored_template,
};
use ui_layout::SizePolicy;
use ui_text::{FontId, TextOverflow};
use ui_theme::{ThemeTokens, UiColor};

use super::surface_control_polish::apply_compact_surface_control_polish;

const ENTITY_TABLE_TEMPLATE_RON: &str =
    include_str!("../../../../../assets/editor/ui/surfaces/entity_table.ron");

pub fn build_entity_table_panel(
    view_model: &EntityTableViewModel,
    theme: &ThemeTokens,
    _panel_instance_id: PanelInstanceId,
    _active_tool_surface: Option<ToolSurfaceInstanceId>,
) -> UiNode {
    let template: AuthoredUiTemplate = ron::from_str(ENTITY_TABLE_TEMPLATE_RON)
        .expect("checked-in entity table UI fixture must parse");
    let normalized = normalize_authored_template(template);
    let mut context = UiDefinitionContext::new(theme.clone());
    register_entity_table_widget_ids(&mut context);
    populate_entity_table_context(view_model, &mut context);

    let mut root = form_retained_ui(&normalized, &mut context).root;
    polish_entity_table(&mut root, theme);
    root
}

fn register_entity_table_widget_ids(context: &mut UiDefinitionContext) {
    for (path, widget_id) in [
        ("root", ENTITY_TABLE_PANEL_WIDGET_ID),
        ("root/body", ENTITY_TABLE_BODY_WIDGET_ID),
        ("root/body/title", ENTITY_TABLE_TITLE_WIDGET_ID),
        ("root/body/controls/search", ENTITY_TABLE_SEARCH_WIDGET_ID),
        (
            "root/body/controls/clear_search",
            ENTITY_TABLE_CLEAR_SEARCH_WIDGET_ID,
        ),
        (
            "root/body/controls/selected_only",
            ENTITY_TABLE_SELECTED_ONLY_TOGGLE_WIDGET_ID,
        ),
        (
            "root/body/controls/roots_only",
            ENTITY_TABLE_ROOTS_ONLY_TOGGLE_WIDGET_ID,
        ),
        (
            "root/body/controls/component_filter",
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
    ] {
        context
            .widget_ids_by_path
            .insert(AuthoredUiNodePath(path.to_string()), widget_id);
    }
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

fn polish_entity_table(root: &mut UiNode, theme: &ThemeTokens) {
    if let UiNodeKind::Panel(panel) = &mut root.kind {
        panel.theme.background_panel = UiColor::new(
            (theme.background_panel.r + 0.012).clamp(0.0, 1.0),
            (theme.background_panel.g + 0.012).clamp(0.0, 1.0),
            (theme.background_panel.b + 0.012).clamp(0.0, 1.0),
            0.94,
        );
    }
    if let Some(body) = find_node_mut(root, ENTITY_TABLE_BODY_WIDGET_ID)
        && let UiNodeKind::Stack(stack) = &mut body.kind
    {
        stack.child_main_policies = vec![
            SizePolicy::Auto,
            SizePolicy::Auto,
            SizePolicy::Auto,
            SizePolicy::flex(1.0),
        ];
    }
    if let Some(title) = find_node_mut(root, ENTITY_TABLE_TITLE_WIDGET_ID)
        && let UiNodeKind::Label(label) = &mut title.kind
    {
        label.text_style = theme.heading_text_style(FontId(1));
    }
    if let Some(search) = find_node_mut(root, ENTITY_TABLE_SEARCH_WIDGET_ID)
        && matches!(&search.kind, UiNodeKind::TextInput(_))
    {
        apply_compact_surface_control_polish(search, theme);
    }
    for widget_id in [
        ENTITY_TABLE_CLEAR_SEARCH_WIDGET_ID,
        ENTITY_TABLE_SELECTED_ONLY_TOGGLE_WIDGET_ID,
        ENTITY_TABLE_ROOTS_ONLY_TOGGLE_WIDGET_ID,
        ENTITY_TABLE_COMPONENT_FILTER_SELECT_WIDGET_ID,
    ] {
        if let Some(node) = find_node_mut(root, widget_id) {
            apply_compact_surface_control_polish(node, theme);
        }
    }
    for index in 0..4 {
        if let Some(sort_button) = find_node_mut(root, entity_table_sort_button_widget_id(index))
            && matches!(&sort_button.kind, UiNodeKind::Button(_))
        {
            apply_compact_surface_control_polish(sort_button, theme);
        }
    }
    if let Some(table) = find_node_mut(root, ENTITY_TABLE_LIST_WIDGET_ID)
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

fn find_node_mut(node: &mut UiNode, widget_id: crate::WidgetId) -> Option<&mut UiNode> {
    if node.id == widget_id {
        return Some(node);
    }
    for child in &mut node.children {
        if let Some(found) = find_node_mut(child, widget_id) {
            return Some(found);
        }
    }
    None
}
