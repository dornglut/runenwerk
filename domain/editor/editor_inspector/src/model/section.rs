//! File: domain/editor/editor_inspector/src/model/section.rs
//! Purpose: Inspector section/grouping model.

use crate::InspectorField;

#[derive(Debug, Clone, PartialEq)]
pub struct InspectorSection {
	pub stable_name: String,
	pub display_name: String,
	pub fields: Vec<InspectorField>,
}

impl InspectorSection {
	/// File: domain/editor/editor_inspector/src/model/section.rs
	/// Method: new
	pub fn new(
		stable_name: impl Into<String>,
		display_name: impl Into<String>,
	) -> Self {
		Self {
			stable_name: stable_name.into(),
			display_name: display_name.into(),
			fields: Vec::new(),
		}
	}

	/// File: domain/editor/editor_inspector/src/model/section.rs
	/// Method: with_field
	pub fn with_field(mut self, field: InspectorField) -> Self {
		self.fields.push(field);
		self
	}
}