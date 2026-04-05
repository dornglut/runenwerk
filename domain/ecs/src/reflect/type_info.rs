//! File: domain/ecs/src/reflect/type_info.rs
//! Purpose: Core reflected type metadata.

use crate::reflect::StructInfo;
use crate::reflect::ReflectTypeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReflectClassification {
	Plain,
	Component,
	Resource,
}

#[derive(Debug, Clone, Copy)]
pub enum ReflectShape {
	Opaque,
	Struct(&'static StructInfo),
}

#[derive(Debug, Clone, Copy)]
pub struct TypeInfo {
	pub id: ReflectTypeId,
	pub rust_name: &'static str,
	pub stable_name: &'static str,
	pub classification: ReflectClassification,
	pub shape: ReflectShape,
}

impl TypeInfo {
	/// File: domain/ecs/src/reflect/type_info.rs
	/// Method: new
	pub const fn new(
		id: ReflectTypeId,
		rust_name: &'static str,
		stable_name: &'static str,
		classification: ReflectClassification,
		shape: ReflectShape,
	) -> Self {
		Self {
			id,
			rust_name,
			stable_name,
			classification,
			shape,
		}
	}

	/// File: domain/ecs/src/reflect/type_info.rs
	/// Method: struct_info
	pub fn struct_info(&self) -> Option<&'static StructInfo> {
		match self.shape {
			ReflectShape::Opaque => None,
			ReflectShape::Struct(info) => Some(info),
		}
	}

	/// File: domain/ecs/src/reflect/type_info.rs
	/// Method: is_component
	pub fn is_component(&self) -> bool {
		matches!(self.classification, ReflectClassification::Component)
	}

	/// File: domain/ecs/src/reflect/type_info.rs
	/// Method: is_resource
	pub fn is_resource(&self) -> bool {
		matches!(self.classification, ReflectClassification::Resource)
	}
}