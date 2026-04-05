//! File: domain/editor/editor_inspector/src/field.rs
//! Purpose: Generic inspector field surface for generated and custom inspectors.

use crate::ValidationMessage;

#[derive(Debug, Clone, PartialEq)]
pub enum InspectorValue {
	Bool(bool),
	Integer(i64),
	Float(f64),
	Text(String),
	Enum {
		current: String,
		options: Vec<String>,
	},
	ReadOnlyText(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InspectorField {
	pub stable_name: String,
	pub display_name: String,
	pub value: InspectorValue,
	pub is_read_only: bool,
	pub validation: Vec<ValidationMessage>,
}

impl InspectorField {
	pub fn new(
		stable_name: impl Into<String>,
		display_name: impl Into<String>,
		value: InspectorValue,
	) -> Self {
		Self {
			stable_name: stable_name.into(),
			display_name: display_name.into(),
			value,
			is_read_only: false,
			validation: Vec::new(),
		}
	}

	pub fn read_only(mut self, is_read_only: bool) -> Self {
		self.is_read_only = is_read_only;
		self
	}

	pub fn with_validation(mut self, message: ValidationMessage) -> Self {
		self.validation.push(message);
		self
	}
}