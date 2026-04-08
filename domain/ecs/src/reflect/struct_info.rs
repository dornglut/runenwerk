//! File: domain/ecs/src/reflect/struct_info.rs
//! Purpose: Reflected struct shape metadata.

use crate::reflect::FieldInfo;

#[derive(Debug, Clone, Copy)]
pub struct StructInfo {
    pub fields: &'static [FieldInfo],
}

impl StructInfo {
    /// File: domain/ecs/src/reflect/struct_info.rs
    /// Method: new
    pub const fn new(fields: &'static [FieldInfo]) -> Self {
        Self { fields }
    }

    /// File: domain/ecs/src/reflect/struct_info.rs
    /// Method: field_count
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    /// File: domain/ecs/src/reflect/struct_info.rs
    /// Method: field_at
    pub fn field_at(&self, index: usize) -> Option<&'static FieldInfo> {
        self.fields.get(index)
    }

    /// File: domain/ecs/src/reflect/struct_info.rs
    /// Method: field_named
    pub fn field_named(&self, name: &str) -> Option<&'static FieldInfo> {
        self.fields.iter().find(|field| field.name == name)
    }
}
