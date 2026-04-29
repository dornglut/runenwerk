//! File: domain/editor/editor_shell/src/composition/build_inspector_panel.rs
//! Purpose: Compose inspector panel widgets.

use crate::{
    UiNode, UiNodeKind, button, hstack_with_policies, label, panel, text_input, vscroll, vstack,
    vstack_with_policies,
};
use ui_layout::SizePolicy;
use ui_math::{UiInsets, UiSize};
use ui_text::{FontId, TextOverflow};
use ui_theme::{ThemeTokens, UiColor};

use crate::{
    INSPECTOR_BODY_WIDGET_ID, INSPECTOR_LIST_WIDGET_ID, INSPECTOR_PANEL_WIDGET_ID,
    INSPECTOR_SCROLL_WIDGET_ID, INSPECTOR_TARGET_WIDGET_ID, INSPECTOR_TITLE_WIDGET_ID,
    InspectorTargetViewModel, InspectorViewModel, PanelInstanceId, ToolSurfaceInstanceId,
    inspector_field_focus_widget_id, inspector_field_widget_id,
};

type InspectorVectorField = (usize, char, crate::InspectorFieldViewModel);
type InspectorVectorGroup = (String, Vec<InspectorVectorField>);

pub fn build_inspector_panel(
    view_model: &InspectorViewModel,
    theme: &ThemeTokens,
    _panel_instance_id: PanelInstanceId,
    _active_tool_surface: Option<ToolSurfaceInstanceId>,
) -> UiNode {
    let title = label(
        INSPECTOR_TITLE_WIDGET_ID,
        "Inspector",
        theme.heading_text_style(FontId(1)),
    );

    let target_label = match &view_model.target {
        InspectorTargetViewModel::Empty => "Nothing selected".to_string(),
        InspectorTargetViewModel::Entity { display_name } => {
            format!("Entity: {display_name}")
        }
        InspectorTargetViewModel::Component {
            entity_display_name,
            component_display_name,
        } => format!("{entity_display_name} / {component_display_name}"),
        InspectorTargetViewModel::Resource { display_name } => {
            format!("Resource: {display_name}")
        }
        InspectorTargetViewModel::Unsupported { label } => {
            format!("Unsupported: {label}")
        }
        InspectorTargetViewModel::Error { message } => {
            format!("Error: {message}")
        }
    };

    let mut target_style = theme.body_text_style(FontId(1));
    target_style.overflow = TextOverflow::Ellipsis;
    let mut row_style = theme.body_small_text_style(FontId(1));
    row_style.overflow = TextOverflow::Ellipsis;

    let mut rows = vec![label(
        INSPECTOR_TARGET_WIDGET_ID,
        target_label,
        target_style,
    )];

    let mut index = 0usize;
    while index < view_model.fields.len() {
        if let Some((group_label, grouped_fields)) = editable_vector_group(view_model, index) {
            rows.push(compact_inspector_vector_row(
                &group_label,
                &grouped_fields,
                row_style.clone(),
                theme.clone(),
            ));
            index += grouped_fields.len();
            continue;
        }

        let field = &view_model.fields[index];
        if field.editable {
            rows.push(compact_inspector_input_row(
                index,
                inspector_field_widget_id(index),
                field,
                row_style.clone(),
                theme.clone(),
            ));
        } else {
            rows.push(compact_inspector_row(button(
                inspector_field_widget_id(index),
                format!("{}: {}", field.label, field.value_summary),
                row_style.clone(),
                theme.clone(),
            )));
        }
        index += 1;
    }

    let list = vstack(
        INSPECTOR_LIST_WIDGET_ID,
        (theme.spacing.xs * 0.85).max(2.0),
        rows,
    );
    let scroll = vscroll(INSPECTOR_SCROLL_WIDGET_ID, theme.clone(), vec![list]);
    let body = vstack_with_policies(
        INSPECTOR_BODY_WIDGET_ID,
        theme.spacing.xs,
        vec![SizePolicy::Auto, SizePolicy::flex(1.0)],
        vec![title, scroll],
    );
    let mut panel_theme = theme.clone();
    panel_theme.background_panel = UiColor::new(
        (theme.background_panel.r + 0.02).clamp(0.0, 1.0),
        (theme.background_panel.g + 0.02).clamp(0.0, 1.0),
        (theme.background_panel.b + 0.02).clamp(0.0, 1.0),
        0.94,
    );
    panel(INSPECTOR_PANEL_WIDGET_ID, panel_theme, vec![body])
}

fn editable_vector_group(
    view_model: &InspectorViewModel,
    start_index: usize,
) -> Option<InspectorVectorGroup> {
    let first = view_model.fields.get(start_index)?;
    if !first.editable {
        return None;
    }
    let first_path = first.path_key.as_deref()?;
    let (group_label, axis) = parse_axis_suffix(first_path)?;
    if axis != 'x' {
        return None;
    }

    let mut grouped = Vec::new();
    let mut next_axis = 'x';
    let mut cursor = start_index;
    while let Some(field) = view_model.fields.get(cursor) {
        if !field.editable {
            break;
        }
        let Some(path_key) = field.path_key.as_deref() else {
            break;
        };
        let Some((candidate_label, candidate_axis)) = parse_axis_suffix(path_key) else {
            break;
        };
        if candidate_label != group_label || candidate_axis != next_axis {
            break;
        }
        grouped.push((cursor, candidate_axis, field.clone()));
        cursor += 1;
        next_axis = match next_axis {
            'x' => 'y',
            'y' => 'z',
            'z' => 'w',
            _ => break,
        };
    }

    match grouped.len() {
        3 | 4 => Some((format_vector_group_label(&group_label), grouped)),
        _ => None,
    }
}

fn parse_axis_suffix(label: &str) -> Option<(String, char)> {
    let (prefix, suffix) = label.rsplit_once('.')?;
    let axis = suffix.chars().next()?.to_ascii_lowercase();
    if !matches!(axis, 'x' | 'y' | 'z' | 'w') {
        return None;
    }
    if suffix.len() != 1 {
        return None;
    }
    Some((prefix.to_string(), axis))
}

fn format_vector_group_label(label: &str) -> String {
    match label {
        "translation" => "Position".to_string(),
        "rotation" => "Rotation".to_string(),
        "scale" => "Scale".to_string(),
        _ => label.replace('_', " "),
    }
}

fn compact_inspector_vector_row(
    group_label: &str,
    fields: &[InspectorVectorField],
    row_style: ui_text::TextStyle,
    theme: ThemeTokens,
) -> UiNode {
    let base_id = inspector_field_widget_id(fields[0].0);
    let group_widget_id = crate::WidgetId(base_id.0 + 710_000);
    let header_widget_id = crate::WidgetId(base_id.0 + 710_001);
    let axes_row_widget_id = crate::WidgetId(base_id.0 + 710_002);

    let mut axis_children = Vec::with_capacity(fields.len());
    let mut axis_policies = Vec::with_capacity(fields.len());
    for (index, axis, field) in fields {
        let cell_widget_id = crate::WidgetId(base_id.0 + 711_000 + *index as u64);
        let axis_label_id = inspector_field_focus_widget_id(*index);
        let input_widget_id = inspector_field_widget_id(*index);

        let axis_label = compact_focus_label_button(
            axis_label_id,
            format!("{axis}:"),
            row_style.clone(),
            &theme,
        );
        let mut input_node = text_input(
            input_widget_id,
            field.value_summary.clone(),
            "",
            row_style.clone(),
            theme.clone(),
        );
        if let UiNodeKind::TextInput(text_input) = &mut input_node.kind {
            let vertical = (theme.spacing.xs * 0.60).max(1.0);
            let horizontal = (theme.spacing.sm * 0.90).max(2.0);
            text_input.padding = UiInsets::new(horizontal, vertical, horizontal, vertical);
            let line_height = text_input
                .text_style
                .line_height_or_default(text_input.text_style.font_size * 1.2);
            text_input.min_size =
                UiSize::new(0.0, (line_height + text_input.padding.vertical()).max(13.0));
            text_input.fill_width = true;
        }
        let cell = hstack_with_policies(
            cell_widget_id,
            theme.spacing.xs * 0.5,
            vec![SizePolicy::Auto, SizePolicy::flex(1.0)],
            vec![axis_label, input_node],
        );
        axis_children.push(cell);
        axis_policies.push(SizePolicy::flex(1.0));
    }

    let header = label(
        header_widget_id,
        format!("{group_label}:"),
        row_style.clone(),
    );
    let axes_row = hstack_with_policies(
        axes_row_widget_id,
        theme.spacing.xs,
        axis_policies,
        axis_children,
    );
    vstack_with_policies(
        group_widget_id,
        (theme.spacing.xs * 0.35).max(1.0),
        vec![SizePolicy::Auto, SizePolicy::Auto],
        vec![header, axes_row],
    )
}

fn compact_inspector_input_row(
    index: usize,
    input_widget_id: crate::WidgetId,
    field: &crate::InspectorFieldViewModel,
    row_style: ui_text::TextStyle,
    theme: ThemeTokens,
) -> UiNode {
    let label_widget_id = inspector_field_focus_widget_id(index);
    let row_widget_id = crate::WidgetId(input_widget_id.0 + 700_000);
    let label_node = compact_focus_label_button(
        label_widget_id,
        format!("{}:", field.label),
        row_style.clone(),
        &theme,
    );

    let mut input_node = text_input(
        input_widget_id,
        field.value_summary.clone(),
        "",
        row_style,
        theme.clone(),
    );
    if let UiNodeKind::TextInput(text_input) = &mut input_node.kind {
        let vertical = (theme.spacing.xs * 0.60).max(1.0);
        let horizontal = (theme.spacing.sm * 0.90).max(2.0);
        text_input.padding = UiInsets::new(horizontal, vertical, horizontal, vertical);
        let line_height = text_input
            .text_style
            .line_height_or_default(text_input.text_style.font_size * 1.2);
        text_input.min_size =
            UiSize::new(0.0, (line_height + text_input.padding.vertical()).max(13.0));
        text_input.fill_width = true;
    }

    hstack_with_policies(
        row_widget_id,
        theme.spacing.xs,
        vec![SizePolicy::Auto, SizePolicy::flex(1.0)],
        vec![label_node, input_node],
    )
}

fn compact_focus_label_button(
    widget_id: crate::WidgetId,
    text: String,
    text_style: ui_text::TextStyle,
    theme: &ThemeTokens,
) -> UiNode {
    let mut node = button(widget_id, text, text_style, theme.clone());
    if let UiNodeKind::Button(button) = &mut node.kind {
        button.padding = UiInsets::new(
            (theme.spacing.xs * 0.35).max(1.0),
            (theme.spacing.xs * 0.35).max(1.0),
            (theme.spacing.xs * 0.35).max(1.0),
            (theme.spacing.xs * 0.35).max(1.0),
        );
        button.min_size = UiSize::new(0.0, 0.0);
        button.theme.border_width = 0.0;
        button.theme.background_panel = UiColor::new(0.0, 0.0, 0.0, 0.0);
        button.theme.border = UiColor::new(0.0, 0.0, 0.0, 0.0);
    }
    node
}

fn compact_inspector_row(mut node: UiNode) -> UiNode {
    if let UiNodeKind::Button(button) = &mut node.kind {
        let vertical = (button.theme.spacing.xs * 0.60).max(1.0);
        let horizontal = (button.theme.spacing.sm * 0.90).max(2.0);
        button.padding = UiInsets::new(horizontal, vertical, horizontal, vertical);
        let line_height = button
            .text_style
            .line_height_or_default(button.text_style.font_size * 1.2);
        button.min_size = UiSize::new(0.0, (line_height + button.padding.vertical()).max(13.0));
    }
    node
}
