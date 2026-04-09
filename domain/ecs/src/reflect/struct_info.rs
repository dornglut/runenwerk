//! File: domain/ecs/src/reflect/struct_info.rs
//! Purpose: Reflected struct shape metadata.

use crate::reflect::FieldInfo;

#[derive(Debug, Clone, Copy)]
pub struct StructInfo {
    pub fields: &'static [FieldInfo],
}

impl StructInfo {
    pub const fn new(fields: &'static [FieldInfo]) -> Self {
        Self { fields }
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    pub fn field_at(&self, index: usize) -> Option<&'static FieldInfo> {
        self.fields.get(index)
    }

    pub fn field_named(&self, name: &str) -> Option<&'static FieldInfo> {
        self.fields.iter().find(|field| field.name == name)
    }
}
