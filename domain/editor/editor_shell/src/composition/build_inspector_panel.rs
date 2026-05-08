//! File: domain/editor/editor_shell/src/composition/build_inspector_panel.rs
//! Purpose: Compose inspector panel widgets from the checked-in surface fixture.

use crate::{
    INSPECTOR_BODY_WIDGET_ID, INSPECTOR_LIST_WIDGET_ID, INSPECTOR_PANEL_WIDGET_ID,
    INSPECTOR_SCROLL_WIDGET_ID, INSPECTOR_TARGET_WIDGET_ID, INSPECTOR_TITLE_WIDGET_ID,
    InspectorFieldControlKind, InspectorTargetViewModel, InspectorViewModel, PanelInstanceId,
    ToolSurfaceInstanceId, UiNode, UiNodeKind, inspector_field_focus_widget_id,
    inspector_field_widget_id,
};
use ui_definition::{
    AuthoredUiNodePath, AuthoredUiTemplate, UiAvailability, UiCollectionItem, UiDefinitionContext,
    UiValue, form_retained_ui, normalize_authored_template,
};
use ui_layout::SizePolicy;
use ui_text::{FontId, TextOverflow};
use ui_theme::{ThemeTokens, UiColor};

use super::surface_control_polish::apply_compact_surface_control_polish;

const INSPECTOR_TEMPLATE_RON: &str =
    include_str!("../../../../../assets/editor/ui/surfaces/inspector.ron");

pub fn build_inspector_panel(
    view_model: &InspectorViewModel,
    theme: &ThemeTokens,
    _panel_instance_id: PanelInstanceId,
    _active_tool_surface: Option<ToolSurfaceInstanceId>,
) -> UiNode {
    let template: AuthoredUiTemplate =
        ron::from_str(INSPECTOR_TEMPLATE_RON).expect("checked-in inspector UI fixture must parse");
    let normalized = normalize_authored_template(template);
    let mut context = UiDefinitionContext::new(theme.clone());
    register_inspector_widget_ids(view_model, &mut context);
    populate_inspector_context(view_model, &mut context);

    let mut root = form_retained_ui(&normalized, &mut context).root;
    polish_inspector(&mut root, view_model, theme);
    root
}

fn register_inspector_widget_ids(
    view_model: &InspectorViewModel,
    context: &mut UiDefinitionContext,
) {
    for (path, widget_id) in [
        ("root", INSPECTOR_PANEL_WIDGET_ID),
        ("root/body", INSPECTOR_BODY_WIDGET_ID),
        ("root/body/title", INSPECTOR_TITLE_WIDGET_ID),
        ("root/body/scroll", INSPECTOR_SCROLL_WIDGET_ID),
        ("root/body/scroll/list", INSPECTOR_LIST_WIDGET_ID),
        ("root/body/scroll/list/target", INSPECTOR_TARGET_WIDGET_ID),
    ] {
        context
            .widget_ids_by_path
            .insert(AuthoredUiNodePath(path.to_string()), widget_id);
    }
    for (index, _) in view_model.fields.iter().enumerate() {
        let key = index.to_string();
        for control_id in [
            "value_bool",
            "value_number",
            "value_text",
            "value_enum",
            "value_readonly",
        ] {
            context.widget_ids_by_path.insert(
                AuthoredUiNodePath(format!(
                    "root/body/scroll/list/fields[{key}]/field_row/{control_id}"
                )),
                inspector_field_widget_id(index),
            );
        }
        context.widget_ids_by_path.insert(
            AuthoredUiNodePath(format!(
                "root/body/scroll/list/fields[{key}]/field_row/name"
            )),
            inspector_field_focus_widget_id(index),
        );
    }
}

fn populate_inspector_context(view_model: &InspectorViewModel, context: &mut UiDefinitionContext) {
    context.values.insert(
        "inspector.target".into(),
        UiValue::Text(inspector_target_label(&view_model.target)),
    );
    context.collections.insert(
        "inspector.fields".into(),
        view_model
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let mut item = UiCollectionItem::new(index.to_string(), field.label.clone());
                item.enabled = true;
                item.values.insert(
                    "field.label".into(),
                    UiValue::Text(format!("{}:", field.label)),
                );
                item.values.insert(
                    "field.display_value".into(),
                    UiValue::Text(field.value_summary.clone()),
                );
                install_inspector_control_values(field, &mut item);
                item.values.insert(
                    "inspector.field.editable".into(),
                    UiValue::Bool(field.editable),
                );
                item.values
                    .insert("field.focused".into(), UiValue::Bool(field.is_focused));
                item
            })
            .collect(),
    );
}

fn inspector_target_label(target: &InspectorTargetViewModel) -> String {
    match target {
        InspectorTargetViewModel::Empty => "Nothing selected".to_string(),
        InspectorTargetViewModel::Entity { display_name } => format!("Entity: {display_name}"),
        InspectorTargetViewModel::Component {
            entity_display_name,
            component_display_name,
        } => format!("{entity_display_name} / {component_display_name}"),
        InspectorTargetViewModel::Resource { display_name } => format!("Resource: {display_name}"),
        InspectorTargetViewModel::Unsupported { label } => format!("Unsupported: {label}"),
        InspectorTargetViewModel::Error { message } => format!("Error: {message}"),
    }
}

fn polish_inspector(root: &mut UiNode, _view_model: &InspectorViewModel, theme: &ThemeTokens) {
    if let UiNodeKind::Panel(panel) = &mut root.kind {
        panel.theme.background_panel = UiColor::new(
            (theme.background_panel.r + 0.02).clamp(0.0, 1.0),
            (theme.background_panel.g + 0.02).clamp(0.0, 1.0),
            (theme.background_panel.b + 0.02).clamp(0.0, 1.0),
            0.94,
        );
    }
    if let Some(body) = find_node_mut(root, INSPECTOR_BODY_WIDGET_ID)
        && let UiNodeKind::Stack(stack) = &mut body.kind
    {
        stack.child_main_policies = vec![SizePolicy::Auto, SizePolicy::flex(1.0)];
    }
    if let Some(title) = find_node_mut(root, INSPECTOR_TITLE_WIDGET_ID)
        && let UiNodeKind::Label(label) = &mut title.kind
    {
        label.text_style = theme.heading_text_style(FontId(1));
    }
    if let Some(target) = find_node_mut(root, INSPECTOR_TARGET_WIDGET_ID)
        && let UiNodeKind::Label(label) = &mut target.kind
    {
        label.text_style = theme.body_text_style(FontId(1));
        label.text_style.overflow = TextOverflow::Ellipsis;
    }
    polish_inspector_fields(root, theme);
}

fn polish_inspector_fields(node: &mut UiNode, theme: &ThemeTokens) {
    match &mut node.kind {
        UiNodeKind::Button(_) => {
            apply_compact_surface_control_polish(node, theme);
            if let UiNodeKind::Button(button) = &mut node.kind {
                button.theme.border_width = 0.0;
                button.theme.background_panel = UiColor::new(0.0, 0.0, 0.0, 0.0);
                button.theme.border = UiColor::new(0.0, 0.0, 0.0, 0.0);
            }
        }
        UiNodeKind::TextInput(_) => {
            apply_compact_surface_control_polish(node, theme);
        }
        UiNodeKind::Toggle(_) => {
            apply_compact_surface_control_polish(node, theme);
        }
        UiNodeKind::NumericInput(_) => {
            apply_compact_surface_control_polish(node, theme);
        }
        UiNodeKind::Select(_) => {
            apply_compact_surface_control_polish(node, theme);
        }
        UiNodeKind::Label(label) => {
            label.text_style = theme.body_small_text_style(FontId(1));
            label.text_style.overflow = TextOverflow::Ellipsis;
        }
        UiNodeKind::Stack(stack) if matches!(stack.axis, ui_math::Axis::Horizontal) => {
            stack.child_main_policies = vec![SizePolicy::Auto, SizePolicy::flex(1.0)];
            stack.gap = theme.spacing.xs;
        }
        _ => {}
    }
    for child in &mut node.children {
        polish_inspector_fields(child, theme);
    }
}

fn install_inspector_control_values(
    field: &crate::InspectorFieldViewModel,
    item: &mut UiCollectionItem,
) {
    let unavailable = |name: &str| {
        UiValue::Availability(UiAvailability::Unavailable {
            reason: format!("{name} is not the field control"),
        })
    };
    item.values
        .insert("field.control.bool".into(), unavailable("bool"));
    item.values
        .insert("field.control.number".into(), unavailable("number"));
    item.values
        .insert("field.control.text".into(), unavailable("text"));
    item.values
        .insert("field.control.enum".into(), unavailable("enum"));
    item.values
        .insert("field.control.readonly".into(), unavailable("readonly"));
    item.values
        .insert("field.bool_value".into(), UiValue::Bool(false));
    item.values
        .insert("field.number_value".into(), UiValue::Number(0.0));

    match &field.control {
        InspectorFieldControlKind::BoolToggle { checked } => {
            item.values.insert(
                "field.control.bool".into(),
                UiValue::Availability(UiAvailability::Available),
            );
            item.values
                .insert("field.bool_value".into(), UiValue::Bool(*checked));
        }
        InspectorFieldControlKind::IntegerInput { value } => {
            item.values.insert(
                "field.control.number".into(),
                UiValue::Availability(UiAvailability::Available),
            );
            item.values
                .insert("field.number_value".into(), UiValue::Number(*value as f64));
        }
        InspectorFieldControlKind::FloatInput { value } => {
            item.values.insert(
                "field.control.number".into(),
                UiValue::Availability(UiAvailability::Available),
            );
            item.values
                .insert("field.number_value".into(), UiValue::Number(*value));
        }
        InspectorFieldControlKind::TextInput => {
            item.values.insert(
                "field.control.text".into(),
                UiValue::Availability(UiAvailability::Available),
            );
        }
        InspectorFieldControlKind::EnumSelect {
            current,
            options,
            selected_index,
        } => {
            let enum_availability = if options.is_empty() {
                UiAvailability::Disabled {
                    reason: "enum field has no selectable options".to_string(),
                }
            } else {
                UiAvailability::Available
            };
            item.values.insert(
                "field.control.enum".into(),
                UiValue::Availability(enum_availability),
            );
            let enum_items = if options.is_empty() {
                vec![UiCollectionItem::new("current", current.clone())]
            } else {
                options
                    .iter()
                    .enumerate()
                    .map(|(index, option)| UiCollectionItem::new(index.to_string(), option.clone()))
                    .collect()
            };
            let selected_key = selected_index
                .map(|index| index.to_string())
                .unwrap_or_else(|| "current".to_string());
            item.collections
                .insert("inspector.field.enum_options".into(), enum_items);
            item.selections
                .insert("inspector.field.enum_selected".into(), selected_key);
        }
        InspectorFieldControlKind::ReadOnly
        | InspectorFieldControlKind::Group
        | InspectorFieldControlKind::Unsupported => {
            item.values.insert(
                "field.control.readonly".into(),
                UiValue::Availability(UiAvailability::Available),
            );
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
