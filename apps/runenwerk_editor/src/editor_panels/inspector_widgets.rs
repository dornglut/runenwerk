use editor_inspector::{InspectorEditValue, InspectorPath, InspectorSection, InspectorValue};

use crate::editor_panels::InspectorEditableField;
use crate::editor_runtime::InspectorFieldDraft;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InspectorWidgetKind {
	Checkbox,
	IntegerInput,
	FloatInput,
	TextInput,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InspectorWidgetField {
	pub display_name: String,
	pub path: InspectorPath,
	pub kind: InspectorWidgetKind,
	pub value: InspectorValue,
	pub draft_value: Option<InspectorEditValue>,
	pub is_focused: bool,
}

impl InspectorWidgetField {
	/// File: apps/runenwerk_editor/src/editor_panels/inspector_widgets.rs
	/// Method: from_editable_field
	pub fn from_editable_field(
		field: InspectorEditableField,
		active_draft: Option<&InspectorFieldDraft>,
		focused_field: Option<&InspectorPath>,
	) -> Self {
		let draft_value = active_draft
			.filter(|draft| draft.path == field.path)
			.map(|draft| draft.value.clone());

		let is_focused = focused_field
			.map(|path| *path == field.path)
			.unwrap_or(false);

		Self {
			display_name: field.display_name,
			path: field.path,
			kind: widget_kind_from_value(&field.value),
			value: field.value,
			draft_value,
			is_focused,
		}
	}
}

/// File: apps/runenwerk_editor/src/editor_panels/inspector_widgets.rs
/// Method: build_widget_fields
pub fn build_widget_fields(
	sections: &[InspectorSection],
	active_draft: Option<&InspectorFieldDraft>,
	focused_field: Option<&InspectorPath>,
) -> Vec<InspectorWidgetField> {
	super::flatten_editable_fields(sections)
		.into_iter()
		.map(|field| InspectorWidgetField::from_editable_field(field, active_draft, focused_field))
		.collect()
}

/// File: apps/runenwerk_editor/src/editor_panels/inspector_widgets.rs
/// Method: widget_kind_from_value
fn widget_kind_from_value(value: &InspectorValue) -> InspectorWidgetKind {
	match value {
		InspectorValue::Bool(_) => InspectorWidgetKind::Checkbox,
		InspectorValue::Integer(_) => InspectorWidgetKind::IntegerInput,
		InspectorValue::Float(_) => InspectorWidgetKind::FloatInput,
		InspectorValue::Text(_) => InspectorWidgetKind::TextInput,
		_ => InspectorWidgetKind::TextInput,
	}
}