//! File: domain/ecs/src/reflect/field_info.rs
//! Purpose: Field metadata and field accessor function pointers.

use crate::reflect::{ReflectTypeId, ReflectValueRef, ReflectValueMut};

pub type FieldGetRef = for<'a> fn(&'a dyn std::any::Any) -> Option<ReflectValueRef<'a>>;
pub type FieldGetMut = for<'a> fn(&'a mut dyn std::any::Any) -> Option<ReflectValueMut<'a>>;

#[derive(Debug, Clone, Copy)]
pub struct FieldInfo {
	pub name: &'static str,
	pub display_name: &'static str,
	pub type_id: ReflectTypeId,
	pub get_ref: FieldGetRef,
	pub get_mut: FieldGetMut,
}

impl FieldInfo {
	/// File: domain/ecs/src/reflect/field_info.rs
	/// Method: new
	pub const fn new(
		name: &'static str,
		display_name: &'static str,
		type_id: ReflectTypeId,
		get_ref: FieldGetRef,
		get_mut: FieldGetMut,
	) -> Self {
		Self {
			name,
			display_name,
			type_id,
			get_ref,
			get_mut,
		}
	}
}