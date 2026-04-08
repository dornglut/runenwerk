use editor_inspector::InspectorValue;
use editor_shell::{
	InspectorFieldViewModel, InspectorTargetViewModel, InspectorViewModel,
};

use crate::editor_panels::{InspectorPanelViewModel, InspectorWidgetField};

pub fn build_inspector_view_model(
	view_model: &InspectorPanelViewModel,
) -> InspectorViewModel {
	match view_model {
		InspectorPanelViewModel::Empty => InspectorViewModel::default(),
		InspectorPanelViewModel::Entity {
			display_name,
			components,
			available_component_types,
			..
		} => InspectorViewModel {
			target: InspectorTargetViewModel::Entity {
				display_name: display_name.clone(),
			},
			fields: components
				.iter()
				.map(|component| InspectorFieldViewModel {
					label: component.display_name.clone(),
					value_summary: if component.is_selected {
						"selected".to_string()
					} else {
						"attached".to_string()
					},
					is_focused: false,
				})
				.chain(available_component_types.iter().map(|component| InspectorFieldViewModel {
					label: format!("+ {}", component.display_name),
					value_summary: if component.already_attached {
						"already attached".to_string()
					} else {
						"available".to_string()
					},
					is_focused: false,
				}))
				.collect(),
		},
		InspectorPanelViewModel::Component {
			entity_display_name,
			component_display_name,
			widget_fields,
			..
		} => InspectorViewModel {
			target: InspectorTargetViewModel::Component {
				entity_display_name: entity_display_name.clone(),
				component_display_name: component_display_name.clone(),
			},
			fields: widget_fields
				.iter()
				.map(build_inspector_field_view_model)
				.collect(),
		},
		InspectorPanelViewModel::Resource { resource_type } => InspectorViewModel {
			target: InspectorTargetViewModel::Resource {
				display_name: format!("Resource {}", resource_type.0),
			},
			fields: Vec::new(),
		},
		InspectorPanelViewModel::Unsupported { target } => InspectorViewModel {
			target: InspectorTargetViewModel::Unsupported {
				label: target.clone(),
			},
			fields: Vec::new(),
		},
		InspectorPanelViewModel::Error { message } => InspectorViewModel {
			target: InspectorTargetViewModel::Error {
				message: message.clone(),
			},
			fields: Vec::new(),
		},
	}
}

fn build_inspector_field_view_model(
	field: &InspectorWidgetField,
) -> InspectorFieldViewModel {
	let draft_prefix = field
		.draft_value
		.as_ref()
		.map(|draft| format!("draft={draft:?} | "))
		.unwrap_or_default();

	InspectorFieldViewModel {
		label: field.display_name.clone(),
		value_summary: format!("{draft_prefix}{}", inspector_value_summary(&field.value)),
		is_focused: field.is_focused,
	}
}

fn inspector_value_summary(
	value: &InspectorValue,
) -> String {
	match value {
		InspectorValue::Bool(v) => v.to_string(),
		InspectorValue::Integer(v) => v.to_string(),
		InspectorValue::Float(v) => v.to_string(),
		InspectorValue::Text(v) => v.clone(),
		InspectorValue::Enum { current, .. } => current.clone(),
		InspectorValue::ReadOnlyText(v) => v.clone(),
		InspectorValue::Group => "group".to_string(),
		InspectorValue::Unsupported { type_name } => format!("unsupported<{type_name}>"),
	}
}