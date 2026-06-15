//! Retained UI formation target.

use crate::{
    AuthoredUiNodePath, FormedUiEmbed, NormalizedUiTemplate, UiAvailability, UiAvailabilityBinding,
    UiAvailabilityId, UiAxisDefinition, UiCollectionItem, UiCollectionSlotId,
    UiDefinitionDiagnostic, UiEmbedSlotId, UiMenuSlotId, UiNodeDefinition, UiNodeId, UiRouteSlotId,
    UiScrollAxisDefinition, UiScrollInputDefinition, UiScrollInputPolicyDefinition,
    UiScrollOwnershipDefinition, UiSelectionSlotId, UiValue, UiValueBinding, UiValueSlotId,
};
use crate::{FormedInteractionModel, FormedScrollOwner};
use std::collections::{BTreeMap, BTreeSet};
use ui_layout::SizePolicy;
use ui_math::{Axis, UiSize};
use ui_render_data::ViewportSurfaceEmbedSlotId;
use ui_text::FontId;
use ui_theme::ThemeTokens;
use ui_tree::{TableColumn, TableRow, TreeRow, UiNode, UiNodeKind, WidgetId};
use ui_widgets::{
    NumericInputConfig, ScrollInputPolicies, ScrollInputPolicy, button_selected, hdivider, hscroll,
    hstack_with_policies, label, numeric_input, panel, select, split, table, tabs, text_input,
    toggle, tree, vdivider, viewport_surface_embed, vscroll, vstack_with_policies, xy_scroll,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormedUiRoute {
    RouteSlot(UiRouteSlotId),
    CollectionItemRoute {
        collection: UiCollectionSlotId,
        item_key: String,
        route: UiRouteSlotId,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormedRetainedUiProduct {
    pub root: UiNode,
    pub routes_by_widget_id: BTreeMap<WidgetId, FormedUiRoute>,
    pub paths_by_widget_id: BTreeMap<WidgetId, AuthoredUiNodePath>,
    pub embeds_by_widget_id: BTreeMap<WidgetId, FormedUiEmbed>,
    pub diagnostics: Vec<UiDefinitionDiagnostic>,
    pub availability_by_widget_id: BTreeMap<WidgetId, UiAvailability>,
    pub interaction_model: FormedInteractionModel,
}

#[derive(Debug, Clone)]
pub struct UiDefinitionContext {
    pub theme: ThemeTokens,
    pub values: BTreeMap<UiValueSlotId, UiValue>,
    pub collections: BTreeMap<UiCollectionSlotId, Vec<UiCollectionItem>>,
    pub selections: BTreeMap<UiSelectionSlotId, String>,
    pub availability: BTreeMap<UiAvailabilityId, UiAvailability>,
    pub widget_ids_by_path: BTreeMap<AuthoredUiNodePath, WidgetId>,
    pub embed_slots: BTreeMap<UiEmbedSlotId, u16>,
    pub menus: BTreeMap<UiMenuSlotId, Vec<UiCollectionItem>>,
    pub next_widget_id: u64,
    pub widget_id_scope: Option<WidgetIdScope>,
}

impl UiDefinitionContext {
    pub fn new(theme: ThemeTokens) -> Self {
        Self {
            theme,
            values: BTreeMap::new(),
            collections: BTreeMap::new(),
            selections: BTreeMap::new(),
            availability: BTreeMap::new(),
            widget_ids_by_path: BTreeMap::new(),
            embed_slots: BTreeMap::new(),
            menus: BTreeMap::new(),
            next_widget_id: 1_000_000,
            widget_id_scope: None,
        }
    }

    pub fn with_widget_id_scope(mut self, scope: WidgetIdScope) -> Self {
        self.widget_id_scope = Some(scope);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WidgetIdScope {
    base: u64,
}

impl WidgetIdScope {
    pub const fn new(base: u64) -> Self {
        Self { base }
    }

    pub fn scoped_widget_id(self, local_id: u64) -> WidgetId {
        WidgetId(self.base.saturating_add(local_id))
    }
}

pub fn form_retained_ui(
    template: &NormalizedUiTemplate,
    context: &mut UiDefinitionContext,
) -> FormedRetainedUiProduct {
    let mut state = FormationState::default();
    state.diagnostics.extend(template.diagnostics.clone());
    let root_path = AuthoredUiNodePath::root(template.root.id());
    let root = form_node(&template.root, &root_path, template, context, &mut state);
    FormedRetainedUiProduct {
        root,
        routes_by_widget_id: state.routes_by_widget_id,
        paths_by_widget_id: state.paths_by_widget_id,
        embeds_by_widget_id: state.embeds_by_widget_id,
        diagnostics: state.diagnostics,
        availability_by_widget_id: state.availability_by_widget_id,
        interaction_model: state.interaction_model,
    }
}

#[derive(Default)]
struct FormationState {
    routes_by_widget_id: BTreeMap<WidgetId, FormedUiRoute>,
    paths_by_widget_id: BTreeMap<WidgetId, AuthoredUiNodePath>,
    embeds_by_widget_id: BTreeMap<WidgetId, FormedUiEmbed>,
    availability_by_widget_id: BTreeMap<WidgetId, UiAvailability>,
    interaction_model: FormedInteractionModel,
    used_widget_ids: BTreeSet<WidgetId>,
    diagnostics: Vec<UiDefinitionDiagnostic>,
}

fn form_node(
    node: &UiNodeDefinition,
    path: &AuthoredUiNodePath,
    template: &NormalizedUiTemplate,
    context: &mut UiDefinitionContext,
    state: &mut FormationState,
) -> UiNode {
    let widget_id = assign_widget_id(path, context, state);
    state.paths_by_widget_id.insert(widget_id, path.clone());
    let text_style = context.theme.body_text_style(FontId(1));
    let small_text_style = context.theme.body_small_text_style(FontId(1));

    match node {
        UiNodeDefinition::Panel { children, .. } => panel(
            widget_id,
            context.theme.clone(),
            form_children(children, path, template, context, state),
        ),
        UiNodeDefinition::Row { children, .. } => hstack_with_policies(
            widget_id,
            context.theme.spacing.sm,
            vec![SizePolicy::Auto; children.len()],
            form_children(children, path, template, context, state),
        ),
        UiNodeDefinition::Column { children, .. }
        | UiNodeDefinition::Stack {
            axis: UiAxisDefinition::Vertical,
            children,
            ..
        } => vstack_with_policies(
            widget_id,
            context.theme.spacing.xs,
            vec![SizePolicy::Auto; children.len()],
            form_children(children, path, template, context, state),
        ),
        UiNodeDefinition::Stack {
            axis: UiAxisDefinition::Horizontal,
            children,
            ..
        } => hstack_with_policies(
            widget_id,
            context.theme.spacing.xs,
            vec![SizePolicy::Auto; children.len()],
            form_children(children, path, template, context, state),
        ),
        UiNodeDefinition::Scroll {
            axis,
            input,
            ownership,
            children,
            ..
        } => {
            let formed = form_children(children, path, template, context, state);
            let mut node = match axis {
                UiScrollAxisDefinition::Horizontal => {
                    hscroll(widget_id, context.theme.clone(), formed)
                }
                UiScrollAxisDefinition::Vertical => {
                    vscroll(widget_id, context.theme.clone(), formed)
                }
                UiScrollAxisDefinition::Both => xy_scroll(widget_id, context.theme.clone(), formed),
            };
            if let UiNodeKind::Scroll(scroll) = &mut node.kind {
                scroll.input_policies = scroll_input_policies(*input);
            }
            state
                .interaction_model
                .push_scroll_owner(formed_scroll_owner(widget_id, *axis, *ownership));
            node
        }
        UiNodeDefinition::Split {
            axis,
            ratio,
            children,
            ..
        } => split(
            widget_id,
            axis_to_runtime(*axis),
            *ratio,
            context.theme.spacing.sm,
            form_children(children, path, template, context, state),
        ),
        UiNodeDefinition::Spacer { .. } => ui_widgets::spacer(widget_id, UiSize::new(12.0, 4.0)),
        UiNodeDefinition::Separator {
            axis,
            length,
            thickness,
            ..
        } => form_separator(
            widget_id,
            axis.unwrap_or(UiAxisDefinition::Horizontal),
            *length,
            thickness.unwrap_or(1.0),
            context,
        ),
        UiNodeDefinition::Label { label: text, .. } => {
            label(widget_id, resolve_text(text, context), text_style)
        }
        UiNodeDefinition::Control { kind, .. } => {
            state.diagnostics.push(
                UiDefinitionDiagnostic::error(
                    "ui.definition.retained_form.generic_control_unsupported",
                    format!(
                        "generic control '{}' must be formed through UiProgram formation, not retained UiTree formation",
                        kind.as_str()
                    ),
                )
                  .at_path(path.clone()),
            );

            label(
                widget_id,
                format!("Unsupported control: {}", kind.as_str()),
                small_text_style,
            )
        }
        UiNodeDefinition::Button {
            label,
            route,
            availability,
            selected,
            ..
        } => {
            let availability = resolve_availability(availability.as_ref(), context);
            let mut formed = button_selected(
                widget_id,
                resolve_text(label, context),
                text_style,
                context.theme.clone(),
                selected
                    .as_ref()
                    .and_then(|value| resolve_value(value, context).as_bool())
                    .unwrap_or(false),
            );
            if let UiNodeKind::Button(button) = &mut formed.kind {
                button.enabled = availability.is_enabled();
            }
            state
                .availability_by_widget_id
                .insert(widget_id, availability.clone());
            if availability.is_enabled()
                && let Some(route) = route
            {
                state
                    .routes_by_widget_id
                    .insert(widget_id, FormedUiRoute::RouteSlot(route.id.clone()));
            }
            formed
        }
        UiNodeDefinition::Toggle {
            label,
            checked,
            route,
            availability,
            ..
        } => {
            let availability = resolve_availability(availability.as_ref(), context);
            let mut formed = toggle(
                widget_id,
                resolve_text(label, context),
                resolve_value(checked, context).as_bool().unwrap_or(false),
                text_style,
                context.theme.clone(),
            );
            if let UiNodeKind::Toggle(toggle) = &mut formed.kind {
                toggle.enabled = availability.is_enabled();
            }
            state
                .availability_by_widget_id
                .insert(widget_id, availability.clone());
            if availability.is_enabled()
                && let Some(route) = route
            {
                state
                    .routes_by_widget_id
                    .insert(widget_id, FormedUiRoute::RouteSlot(route.id.clone()));
            }
            formed
        }
        UiNodeDefinition::TextInput {
            value,
            placeholder,
            route,
            availability,
            ..
        } => {
            let availability = resolve_availability(availability.as_ref(), context);
            let mut formed = text_input(
                widget_id,
                resolve_text(value, context),
                placeholder.clone().unwrap_or_default(),
                text_style,
                context.theme.clone(),
            );
            if let UiNodeKind::TextInput(input) = &mut formed.kind {
                input.editable = availability.is_enabled();
            }
            state
                .availability_by_widget_id
                .insert(widget_id, availability.clone());
            if availability.is_enabled()
                && let Some(route) = route
            {
                state
                    .routes_by_widget_id
                    .insert(widget_id, FormedUiRoute::RouteSlot(route.id.clone()));
            }
            formed
        }
        UiNodeDefinition::NumericInput {
            value,
            route,
            availability,
            ..
        } => {
            let availability = resolve_availability(availability.as_ref(), context);
            let mut formed = numeric_input(
                widget_id,
                NumericInputConfig::new(
                    resolve_value(value, context)
                        .as_number()
                        .unwrap_or_default(),
                    1.0,
                    None,
                    None,
                    2,
                ),
                text_style,
                context.theme.clone(),
            );
            if let UiNodeKind::NumericInput(input) = &mut formed.kind {
                input.enabled = availability.is_enabled();
            }
            state
                .availability_by_widget_id
                .insert(widget_id, availability.clone());
            if availability.is_enabled()
                && let Some(route) = route
            {
                state
                    .routes_by_widget_id
                    .insert(widget_id, FormedUiRoute::RouteSlot(route.id.clone()));
            }
            formed
        }
        UiNodeDefinition::Select {
            items,
            selected,
            route,
            availability,
            ..
        } => {
            let availability = resolve_availability(availability.as_ref(), context);
            let collection = context
                .collections
                .get(&items.id)
                .cloned()
                .unwrap_or_default();
            let selected_key = selected
                .as_ref()
                .and_then(|slot| context.selections.get(&slot.id));
            let selected_index = collection
                .iter()
                .position(|item| Some(&item.key) == selected_key);
            let mut formed = select(
                widget_id,
                collection.iter().map(|item| item.label.clone()),
                selected_index,
                "Select",
                text_style,
                context.theme.clone(),
            );
            if let UiNodeKind::Select(select) = &mut formed.kind {
                select.enabled = availability.is_enabled();
            }
            state
                .availability_by_widget_id
                .insert(widget_id, availability.clone());
            if availability.is_enabled()
                && let Some(route) = route
            {
                state
                    .routes_by_widget_id
                    .insert(widget_id, FormedUiRoute::RouteSlot(route.id.clone()));
            }
            formed
        }
        UiNodeDefinition::Tabs {
            items,
            selected,
            route,
            ..
        } => {
            let collection = context
                .collections
                .get(&items.id)
                .cloned()
                .unwrap_or_default();
            let selected_key = selected
                .as_ref()
                .and_then(|slot| context.selections.get(&slot.id));
            let selected_index = collection
                .iter()
                .position(|item| Some(&item.key) == selected_key)
                .unwrap_or(0);
            if let Some(route) = route {
                state
                    .routes_by_widget_id
                    .insert(widget_id, FormedUiRoute::RouteSlot(route.id.clone()));
            }
            tabs(
                widget_id,
                collection.iter().map(|item| item.label.clone()),
                selected_index,
                small_text_style,
                context.theme.clone(),
            )
        }
        UiNodeDefinition::Table {
            rows,
            columns,
            route,
            ..
        } => {
            let items = context
                .collections
                .get(&rows.id)
                .cloned()
                .unwrap_or_default();
            if let Some(route) = route {
                state
                    .routes_by_widget_id
                    .insert(widget_id, FormedUiRoute::RouteSlot(route.id.clone()));
            }
            let formed_columns = if columns.is_empty() {
                vec![TableColumn::new("Item", 160.0)]
            } else {
                columns
                    .iter()
                    .map(|column| TableColumn::new(column.label.clone(), column.width))
                    .collect()
            };
            table(
                widget_id,
                formed_columns,
                items.iter().map(|item| {
                    let cells = if columns.is_empty() {
                        vec![item.label.clone()]
                    } else {
                        columns
                            .iter()
                            .map(|column| resolve_item_text(item, &column.value, context))
                            .collect()
                    };
                    let mut row = TableRow::new(cells);
                    row.selected = item.selected;
                    row.enabled = item.enabled;
                    row
                }),
                text_style.clone(),
                small_text_style,
                context.theme.clone(),
            )
        }
        UiNodeDefinition::Tree { rows, route, .. } => {
            let items = context
                .collections
                .get(&rows.id)
                .cloned()
                .unwrap_or_default();
            if let Some(route) = route {
                state
                    .routes_by_widget_id
                    .insert(widget_id, FormedUiRoute::RouteSlot(route.id.clone()));
            }
            tree(
                widget_id,
                items.iter().map(|item| {
                    let depth = item
                        .values
                        .get(&"tree.depth".into())
                        .and_then(UiValue::as_number)
                        .map(|value| value.max(0.0) as usize)
                        .unwrap_or(0);
                    let has_children = item
                        .values
                        .get(&"tree.has_children".into())
                        .and_then(UiValue::as_bool)
                        .unwrap_or(false);
                    let mut row = TreeRow::new(item.label.clone(), depth, has_children);
                    row.selected = item.selected;
                    row.enabled = item.enabled;
                    row.expanded = item
                        .values
                        .get(&"tree.expanded".into())
                        .and_then(UiValue::as_bool)
                        .unwrap_or(false);
                    row
                }),
                text_style,
                context.theme.clone(),
            )
        }
        UiNodeDefinition::Repeat {
            id,
            items,
            template: template_id,
            axis,
        } => {
            let collection = context
                .collections
                .get(&items.id)
                .cloned()
                .unwrap_or_default();
            let Some(child_template) = template.templates.get(template_id) else {
                state.diagnostics.push(
                    UiDefinitionDiagnostic::error(
                        "ui.definition.template.unresolved",
                        format!("unresolved template ref '{}'", template_id),
                    )
                    .at_path(path.clone()),
                );
                return vstack_with_policies(
                    widget_id,
                    context.theme.spacing.xs,
                    Vec::new(),
                    Vec::new(),
                );
            };
            let mut children = Vec::new();
            for item in &collection {
                let child_path = path.repeated_child(id, &item.key, child_template.root.id());
                let previous_values = install_repeat_item_values(item, context);
                children.push(form_node(
                    &child_template.root,
                    &child_path,
                    child_template,
                    context,
                    state,
                ));
                restore_repeat_item_values(previous_values, context);
            }
            match axis.unwrap_or(UiAxisDefinition::Vertical) {
                UiAxisDefinition::Horizontal => hstack_with_policies(
                    widget_id,
                    context.theme.spacing.xs,
                    vec![SizePolicy::Auto; children.len()],
                    children,
                ),
                UiAxisDefinition::Vertical => vstack_with_policies(
                    widget_id,
                    context.theme.spacing.xs,
                    vec![SizePolicy::Auto; children.len()],
                    children,
                ),
            }
        }
        UiNodeDefinition::TemplateRef {
            template: template_id,
            ..
        } => {
            let Some(child_template) = template.templates.get(template_id) else {
                state.diagnostics.push(
                    UiDefinitionDiagnostic::error(
                        "ui.definition.template.unresolved",
                        format!("unresolved template ref '{}'", template_id),
                    )
                    .at_path(path.clone()),
                );
                return vstack_with_policies(
                    widget_id,
                    context.theme.spacing.xs,
                    Vec::new(),
                    Vec::new(),
                );
            };
            form_node(&child_template.root, path, child_template, context, state)
        }
        UiNodeDefinition::MenuSlot { menu, .. } => {
            let items = context.menus.get(&menu.id).cloned().unwrap_or_default();
            let mut children = Vec::new();
            for item in items {
                let item_path = path.child(&UiNodeId::new(item.key.clone()));
                let child_id = assign_widget_id(&item_path, context, state);
                let mut child = button_selected(
                    child_id,
                    item.label,
                    small_text_style.clone(),
                    context.theme.clone(),
                    item.selected,
                );
                if let UiNodeKind::Button(button) = &mut child.kind {
                    button.enabled = item.enabled;
                }
                state.paths_by_widget_id.insert(child_id, item_path);
                if item.enabled {
                    state.routes_by_widget_id.insert(
                        child_id,
                        FormedUiRoute::RouteSlot(UiRouteSlotId::new(item.key.clone())),
                    );
                }
                children.push(child);
            }
            hstack_with_policies(
                widget_id,
                context.theme.spacing.xs,
                vec![SizePolicy::Auto; children.len()],
                children,
            )
        }
        UiNodeDefinition::EmbedSlot { slot, .. } => {
            let raw_slot = context.embed_slots.get(&slot.id).copied().unwrap_or(1);
            let formed =
                viewport_surface_embed(widget_id, 0, ViewportSurfaceEmbedSlotId::new(raw_slot));
            state.embeds_by_widget_id.insert(
                widget_id,
                FormedUiEmbed {
                    slot: slot.id.clone(),
                },
            );
            formed
        }
    }
}

fn form_children(
    children: &[UiNodeDefinition],
    parent_path: &AuthoredUiNodePath,
    template: &NormalizedUiTemplate,
    context: &mut UiDefinitionContext,
    state: &mut FormationState,
) -> Vec<UiNode> {
    children
        .iter()
        .filter_map(|child| {
            if node_is_unavailable(child, context) {
                return None;
            }
            Some(form_node(
                child,
                &parent_path.child(child.id()),
                template,
                context,
                state,
            ))
        })
        .collect()
}

fn node_is_unavailable(node: &UiNodeDefinition, context: &UiDefinitionContext) -> bool {
    let availability = match node {
        UiNodeDefinition::Panel { availability, .. }
        | UiNodeDefinition::Label { availability, .. }
        | UiNodeDefinition::Button { availability, .. }
        | UiNodeDefinition::Toggle { availability, .. }
        | UiNodeDefinition::TextInput { availability, .. }
        | UiNodeDefinition::NumericInput { availability, .. }
        | UiNodeDefinition::Select { availability, .. } => availability.as_ref(),
        _ => None,
    };
    matches!(
        resolve_availability(availability, context),
        UiAvailability::Unavailable { .. }
    )
}

fn assign_widget_id(
    path: &AuthoredUiNodePath,
    context: &mut UiDefinitionContext,
    state: &mut FormationState,
) -> WidgetId {
    let widget_id = if let Some(widget_id) = context.widget_ids_by_path.get(path).copied() {
        widget_id
    } else {
        let widget_id = context
            .widget_id_scope
            .map(|scope| scope.scoped_widget_id(context.next_widget_id))
            .unwrap_or(WidgetId(context.next_widget_id));
        context.next_widget_id += 1;
        context.widget_ids_by_path.insert(path.clone(), widget_id);
        widget_id
    };
    if !state.used_widget_ids.insert(widget_id) {
        state.diagnostics.push(
            UiDefinitionDiagnostic::error(
                "ui.definition.widget_id.duplicate",
                format!(
                    "formed widget id '{}' is assigned more than once",
                    widget_id.0
                ),
            )
            .at_path(path.clone()),
        );
    }
    widget_id
}

fn resolve_value(binding: &UiValueBinding, context: &UiDefinitionContext) -> UiValue {
    match binding {
        UiValueBinding::Static(value) => value.clone(),
        UiValueBinding::Slot(slot) => context
            .values
            .get(slot)
            .cloned()
            .unwrap_or_else(|| UiValue::Text(String::new())),
    }
}

fn resolve_item_text(
    item: &UiCollectionItem,
    binding: &UiValueBinding,
    context: &UiDefinitionContext,
) -> String {
    match binding {
        UiValueBinding::Static(value) => value.as_text(),
        UiValueBinding::Slot(slot) => item
            .values
            .get(slot)
            .or_else(|| context.values.get(slot))
            .cloned()
            .unwrap_or_else(|| UiValue::Text(String::new()))
            .as_text(),
    }
}

#[derive(Default)]
struct RepeatItemBindings {
    values: BTreeMap<UiValueSlotId, Option<UiValue>>,
    collections: BTreeMap<UiCollectionSlotId, Option<Vec<UiCollectionItem>>>,
    selections: BTreeMap<UiSelectionSlotId, Option<String>>,
}

fn install_repeat_item_values(
    item: &UiCollectionItem,
    context: &mut UiDefinitionContext,
) -> RepeatItemBindings {
    let mut values = item.values.clone();
    values.insert("item.key".into(), UiValue::Text(item.key.clone()));
    values.insert("item.label".into(), UiValue::Text(item.label.clone()));
    values.insert("item.selected".into(), UiValue::Bool(item.selected));
    values.insert("item.enabled".into(), UiValue::Bool(item.enabled));

    let mut previous = RepeatItemBindings::default();
    for (slot, value) in values {
        previous
            .values
            .insert(slot.clone(), context.values.insert(slot, value));
    }
    for (slot, collection) in &item.collections {
        previous.collections.insert(
            slot.clone(),
            context.collections.insert(slot.clone(), collection.clone()),
        );
    }
    for (slot, selection) in &item.selections {
        previous.selections.insert(
            slot.clone(),
            context.selections.insert(slot.clone(), selection.clone()),
        );
    }
    previous
}

fn restore_repeat_item_values(
    previous_values: RepeatItemBindings,
    context: &mut UiDefinitionContext,
) {
    for (slot, previous) in previous_values.values {
        match previous {
            Some(value) => {
                context.values.insert(slot, value);
            }
            None => {
                context.values.remove(&slot);
            }
        }
    }
    for (slot, previous) in previous_values.collections {
        match previous {
            Some(collection) => {
                context.collections.insert(slot, collection);
            }
            None => {
                context.collections.remove(&slot);
            }
        }
    }
    for (slot, previous) in previous_values.selections {
        match previous {
            Some(selection) => {
                context.selections.insert(slot, selection);
            }
            None => {
                context.selections.remove(&slot);
            }
        }
    }
}

fn resolve_text(binding: &UiValueBinding, context: &UiDefinitionContext) -> String {
    resolve_value(binding, context).as_text()
}

fn resolve_availability(
    binding: Option<&UiAvailabilityBinding>,
    context: &UiDefinitionContext,
) -> UiAvailability {
    match binding {
        Some(UiAvailabilityBinding::Static(value)) => value.clone(),
        Some(UiAvailabilityBinding::Ref(id)) => context
            .availability
            .get(id)
            .cloned()
            .or_else(|| context.values.get(id).and_then(UiValue::as_availability))
            .or_else(|| {
                context
                    .values
                    .get(id)
                    .and_then(UiValue::as_bool)
                    .map(|enabled| {
                        if enabled {
                            UiAvailability::Available
                        } else {
                            UiAvailability::Disabled {
                                reason: format!("availability '{}' is false", id),
                            }
                        }
                    })
            })
            .unwrap_or(UiAvailability::Unavailable {
                reason: format!("unresolved availability '{}'", id),
            }),
        None => UiAvailability::Available,
    }
}

fn form_separator(
    widget_id: WidgetId,
    axis: UiAxisDefinition,
    length: Option<f32>,
    thickness: f32,
    context: &UiDefinitionContext,
) -> UiNode {
    let length_policy = length.map(SizePolicy::Fixed).unwrap_or(SizePolicy::Auto);
    match axis {
        UiAxisDefinition::Horizontal => hdivider(
            widget_id,
            thickness,
            length_policy,
            context.theme.foreground,
        ),
        UiAxisDefinition::Vertical => vdivider(
            widget_id,
            thickness,
            length_policy,
            context.theme.foreground,
        ),
    }
}

fn axis_to_runtime(axis: UiAxisDefinition) -> Axis {
    match axis {
        UiAxisDefinition::Horizontal => Axis::Horizontal,
        UiAxisDefinition::Vertical => Axis::Vertical,
    }
}

fn scroll_input_policies(input: UiScrollInputDefinition) -> ScrollInputPolicies {
    ScrollInputPolicies::new(
        scroll_input_policy(input.horizontal),
        scroll_input_policy(input.vertical),
    )
}

fn scroll_input_policy(input: UiScrollInputPolicyDefinition) -> ScrollInputPolicy {
    match input {
        UiScrollInputPolicyDefinition::WheelOnly => ScrollInputPolicy::WheelOnly,
        UiScrollInputPolicyDefinition::MiddleDragOnly => ScrollInputPolicy::MiddleDragOnly,
        UiScrollInputPolicyDefinition::WheelAndMiddleDrag => ScrollInputPolicy::WheelAndMiddleDrag,
    }
}

fn formed_scroll_owner(
    widget_id: WidgetId,
    axis: UiScrollAxisDefinition,
    ownership: UiScrollOwnershipDefinition,
) -> FormedScrollOwner {
    FormedScrollOwner {
        widget_id,
        axes: match axis {
            UiScrollAxisDefinition::Horizontal => vec![Axis::Horizontal],
            UiScrollAxisDefinition::Vertical => vec![Axis::Vertical],
            UiScrollAxisDefinition::Both => vec![Axis::Horizontal, Axis::Vertical],
        },
        boundary: ownership.boundary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AuthoredUiTemplate, UiCollectionSlotRef, UiNodeId, UiRouteSlotRef,
        UiScrollBoundaryPolicyDefinition,
    };

    #[test]
    fn disabled_button_forms_without_route() {
        let template = AuthoredUiTemplate {
            id: "test.toolbar".into(),
            root: UiNodeDefinition::Button {
                id: UiNodeId::from("root"),
                label: UiValueBinding::static_text("Add"),
                route: Some(UiRouteSlotRef {
                    id: "route.add".into(),
                }),
                availability: Some(UiAvailabilityBinding::Static(UiAvailability::Disabled {
                    reason: "not implemented".to_string(),
                })),
                selected: None,
            },
            templates: Vec::new(),
            menus: Vec::new(),
        };
        let normalized = crate::normalize_authored_template(template);
        let mut context = UiDefinitionContext::new(ThemeTokens::default());
        let product = form_retained_ui(&normalized, &mut context);
        assert!(product.routes_by_widget_id.is_empty());
        assert!(matches!(product.root.kind, UiNodeKind::Button(_)));
    }

    #[test]
    fn two_axis_scroll_forms_with_explicit_input_policy() {
        let template = AuthoredUiTemplate {
            id: "test.scroll".into(),
            root: UiNodeDefinition::Scroll {
                id: UiNodeId::from("root"),
                axis: UiScrollAxisDefinition::Both,
                input: UiScrollInputDefinition {
                    horizontal: UiScrollInputPolicyDefinition::MiddleDragOnly,
                    vertical: UiScrollInputPolicyDefinition::WheelOnly,
                },
                ownership: UiScrollOwnershipDefinition {
                    boundary: UiScrollBoundaryPolicyDefinition::ConsumeAtBoundary,
                },
                children: vec![UiNodeDefinition::Label {
                    id: "entry".into(),
                    label: UiValueBinding::static_text("line"),
                    availability: None,
                }],
            },
            templates: Vec::new(),
            menus: Vec::new(),
        };
        let normalized = crate::normalize_authored_template(template);
        let mut context = UiDefinitionContext::new(ThemeTokens::default());

        let product = form_retained_ui(&normalized, &mut context);

        let UiNodeKind::Scroll(scroll) = product.root.kind else {
            panic!("scroll definition should form a retained scroll node");
        };
        assert_eq!(scroll.axes, ui_widgets::ScrollAxes::Both);
        assert_eq!(
            scroll.input_policies,
            ScrollInputPolicies::new(
                ScrollInputPolicy::MiddleDragOnly,
                ScrollInputPolicy::WheelOnly,
            )
        );
        assert_eq!(
            product.interaction_model.scroll_owners,
            vec![FormedScrollOwner {
                widget_id: product.root.id,
                axes: vec![Axis::Horizontal, Axis::Vertical],
                boundary: UiScrollBoundaryPolicyDefinition::ConsumeAtBoundary,
            }]
        );
    }

    #[test]
    fn unavailable_children_are_omitted_while_disabled_children_remain_non_interactive() {
        let template = AuthoredUiTemplate {
            id: "test.availability".into(),
            root: UiNodeDefinition::Column {
                id: "root".into(),
                children: vec![
                    UiNodeDefinition::Label {
                        id: "available".into(),
                        label: UiValueBinding::static_text("Available"),
                        availability: None,
                    },
                    UiNodeDefinition::Button {
                        id: "disabled".into(),
                        label: UiValueBinding::static_text("Disabled"),
                        route: Some(UiRouteSlotRef {
                            id: "route.disabled".into(),
                        }),
                        availability: Some(UiAvailabilityBinding::Static(
                            UiAvailability::Disabled {
                                reason: "fixture disabled".to_string(),
                            },
                        )),
                        selected: None,
                    },
                    UiNodeDefinition::Button {
                        id: "unavailable".into(),
                        label: UiValueBinding::static_text("Unavailable"),
                        route: Some(UiRouteSlotRef {
                            id: "route.unavailable".into(),
                        }),
                        availability: Some(UiAvailabilityBinding::Ref(
                            "availability.unavailable".into(),
                        )),
                        selected: None,
                    },
                ],
            },
            templates: Vec::new(),
            menus: Vec::new(),
        };
        let normalized = crate::normalize_authored_template(template);
        let mut context = UiDefinitionContext::new(ThemeTokens::default());
        context.values.insert(
            "availability.unavailable".into(),
            UiValue::Availability(UiAvailability::Unavailable {
                reason: "wrong control kind".to_string(),
            }),
        );

        let product = form_retained_ui(&normalized, &mut context);
        let formed_paths = product
            .paths_by_widget_id
            .values()
            .map(AuthoredUiNodePath::as_str)
            .collect::<BTreeSet<_>>();

        assert!(formed_paths.contains("root/available"));
        assert!(formed_paths.contains("root/disabled"));
        assert!(!formed_paths.contains("root/unavailable"));
        assert!(
            product.routes_by_widget_id.is_empty(),
            "disabled and unavailable nodes must not expose route entries"
        );
    }

    #[test]
    fn vertical_separator_forms_fixed_length_divider() {
        let template = AuthoredUiTemplate {
            id: "test.separator".into(),
            root: UiNodeDefinition::Separator {
                id: UiNodeId::from("root"),
                axis: Some(UiAxisDefinition::Vertical),
                length: Some(18.0),
                thickness: Some(1.0),
            },
            templates: Vec::new(),
            menus: Vec::new(),
        };
        let normalized = crate::normalize_authored_template(template);
        let mut context = UiDefinitionContext::new(ThemeTokens::default());

        let product = form_retained_ui(&normalized, &mut context);

        let UiNodeKind::Divider(divider) = product.root.kind else {
            panic!("separator should form a divider node");
        };
        assert_eq!(divider.axis, Axis::Vertical);
        assert_eq!(divider.thickness, 1.0);
        assert_eq!(divider.length_policy, SizePolicy::Fixed(18.0));
    }

    #[test]
    fn repeat_children_use_source_map_paths_under_repeat_node() {
        let template = AuthoredUiTemplate {
            id: "test.repeat".into(),
            root: UiNodeDefinition::Column {
                id: "root".into(),
                children: vec![UiNodeDefinition::Repeat {
                    id: "rows".into(),
                    items: UiCollectionSlotRef {
                        id: "test.rows".into(),
                    },
                    template: "test.repeat.row".into(),
                    axis: Some(UiAxisDefinition::Vertical),
                }],
            },
            templates: vec![AuthoredUiTemplate {
                id: "test.repeat.row".into(),
                root: UiNodeDefinition::Label {
                    id: "entry".into(),
                    label: UiValueBinding::static_text("row"),
                    availability: None,
                },
                templates: Vec::new(),
                menus: Vec::new(),
            }],
            menus: Vec::new(),
        };
        let normalized = crate::normalize_authored_template(template);
        let mut context = UiDefinitionContext::new(ThemeTokens::default());
        context.collections.insert(
            "test.rows".into(),
            vec![
                UiCollectionItem::new("a", "A"),
                UiCollectionItem::new("b", "B"),
            ],
        );

        let product = form_retained_ui(&normalized, &mut context);
        let formed_paths = product
            .paths_by_widget_id
            .values()
            .map(AuthoredUiNodePath::as_str)
            .collect::<BTreeSet<_>>();

        assert!(formed_paths.contains("root/rows[a]/entry"));
        assert!(formed_paths.contains("root/rows[b]/entry"));
        assert!(!formed_paths.contains("root/rows/rows[a]/entry"));
    }

    #[test]
    fn generated_widget_ids_are_scoped_during_formation() {
        let template = AuthoredUiTemplate {
            id: "test.scoped".into(),
            root: UiNodeDefinition::Column {
                id: "root".into(),
                children: vec![UiNodeDefinition::Button {
                    id: "action".into(),
                    label: UiValueBinding::static_text("Action"),
                    route: Some(UiRouteSlotRef {
                        id: "route.action".into(),
                    }),
                    availability: None,
                    selected: None,
                }],
            },
            templates: Vec::new(),
            menus: Vec::new(),
        };
        let normalized = crate::normalize_authored_template(template);
        let mut first = UiDefinitionContext::new(ThemeTokens::default())
            .with_widget_id_scope(WidgetIdScope::new(10_000_000));
        let mut second = UiDefinitionContext::new(ThemeTokens::default())
            .with_widget_id_scope(WidgetIdScope::new(20_000_000));

        let first_product = form_retained_ui(&normalized, &mut first);
        let second_product = form_retained_ui(&normalized, &mut second);

        assert!(first_product.diagnostics.is_empty());
        assert!(second_product.diagnostics.is_empty());
        assert_eq!(first_product.root.id, WidgetId(11_000_000));
        assert_eq!(second_product.root.id, WidgetId(21_000_000));
        assert_ne!(first_product.root.id, second_product.root.id);
        assert_ne!(
            first_product.routes_by_widget_id.keys().next(),
            second_product.routes_by_widget_id.keys().next(),
            "identical authored surfaces must not collide after scoped formation",
        );
    }
}
