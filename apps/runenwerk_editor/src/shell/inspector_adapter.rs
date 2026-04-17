use editor_inspector::InspectorValue;
use editor_shell::{
    InspectorFieldViewModel, InspectorObservationFrame, InspectorObservedField,
    InspectorObservedTarget, InspectorTargetViewModel, InspectorViewModel, ObservationConsumerKind,
    ObservationFrameMetadata, ObservationSourceReality,
};

use crate::editor_panels::{InspectorPanelViewModel, InspectorWidgetField};

pub fn build_inspector_observation_frame(
    view_model: &InspectorPanelViewModel,
    source_version: editor_core::RealityVersion,
) -> InspectorObservationFrame {
    let metadata = ObservationFrameMetadata::strict_current(
        ObservationSourceReality::ObservedScene,
        ObservationConsumerKind::Inspector,
        source_version,
    );

    match view_model {
        InspectorPanelViewModel::Empty => InspectorObservationFrame {
            metadata,
            target: InspectorObservedTarget::Empty,
            fields: Vec::new(),
        },
        InspectorPanelViewModel::Entity {
            display_name,
            components,
            available_component_types,
            ..
        } => {
            InspectorObservationFrame {
                metadata,
                target: InspectorObservedTarget::Entity {
                    display_name: display_name.clone(),
                },
                fields: components
                    .iter()
                    .map(|component| InspectorObservedField {
                        label: component.display_name.clone(),
                        value_summary: if component.is_selected {
                            "selected".to_string()
                        } else {
                            "attached".to_string()
                        },
                        is_focused: false,
                    })
                    .chain(available_component_types.iter().map(|component| {
                        InspectorObservedField {
                            label: format!("+ {}", component.display_name),
                            value_summary: if component.already_attached {
                                "already attached".to_string()
                            } else {
                                "available".to_string()
                            },
                            is_focused: false,
                        }
                    }))
                    .collect(),
            }
        }
        InspectorPanelViewModel::Component {
            entity_display_name,
            component_display_name,
            widget_fields,
            ..
        } => InspectorObservationFrame {
            metadata,
            target: InspectorObservedTarget::Component {
                entity_display_name: entity_display_name.clone(),
                component_display_name: component_display_name.clone(),
            },
            fields: widget_fields
                .iter()
                .map(build_inspector_observed_field)
                .collect(),
        },
        InspectorPanelViewModel::Resource { resource_type } => InspectorObservationFrame {
            metadata,
            target: InspectorObservedTarget::Resource {
                display_name: format!("Resource {}", resource_type.0),
            },
            fields: Vec::new(),
        },
        InspectorPanelViewModel::Unsupported { target } => InspectorObservationFrame {
            metadata,
            target: InspectorObservedTarget::Unsupported {
                label: target.clone(),
            },
            fields: Vec::new(),
        },
        InspectorPanelViewModel::Error { message } => InspectorObservationFrame {
            metadata,
            target: InspectorObservedTarget::Error {
                message: message.clone(),
            },
            fields: Vec::new(),
        },
    }
}

pub fn build_inspector_view_model(frame: &InspectorObservationFrame) -> InspectorViewModel {
    let target = match &frame.target {
        InspectorObservedTarget::Empty => InspectorTargetViewModel::Empty,
        InspectorObservedTarget::Entity { display_name } => InspectorTargetViewModel::Entity {
            display_name: display_name.clone(),
        },
        InspectorObservedTarget::Component {
            entity_display_name,
            component_display_name,
        } => InspectorTargetViewModel::Component {
            entity_display_name: entity_display_name.clone(),
            component_display_name: component_display_name.clone(),
        },
        InspectorObservedTarget::Resource { display_name } => InspectorTargetViewModel::Resource {
            display_name: display_name.clone(),
        },
        InspectorObservedTarget::Unsupported { label } => InspectorTargetViewModel::Unsupported {
            label: label.clone(),
        },
        InspectorObservedTarget::Error { message } => InspectorTargetViewModel::Error {
            message: message.clone(),
        },
    };

    InspectorViewModel {
        target,
        fields: frame
            .fields
            .iter()
            .map(|field| InspectorFieldViewModel {
                label: field.label.clone(),
                value_summary: field.value_summary.clone(),
                is_focused: field.is_focused,
            })
            .collect(),
    }
}

fn build_inspector_observed_field(field: &InspectorWidgetField) -> InspectorObservedField {
    let draft_prefix = field
        .draft_value
        .as_ref()
        .map(|draft| format!("draft={draft:?} | "))
        .unwrap_or_default();

    InspectorObservedField {
        label: field.display_name.clone(),
        value_summary: format!("{draft_prefix}{}", inspector_value_summary(&field.value)),
        is_focused: field.is_focused,
    }
}

fn inspector_value_summary(value: &InspectorValue) -> String {
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
