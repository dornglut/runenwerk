//! File: domain/ecs/src/reflect/type_info.rs
//! Purpose: Core reflected type metadata.

use crate::reflect::EnumInfo;
use crate::reflect::ReflectTypeId;
use crate::reflect::StructInfo;

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
    Enum(&'static EnumInfo),
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

    pub fn struct_info(&self) -> Option<&'static StructInfo> {
        match self.shape {
            ReflectShape::Opaque => None,
            ReflectShape::Struct(info) => Some(info),
            ReflectShape::Enum(_) => None,
        }
    }

    pub fn enum_info(&self) -> Option<&'static EnumInfo> {
        match self.shape {
            ReflectShape::Opaque => None,
            ReflectShape::Struct(_) => None,
            ReflectShape::Enum(info) => Some(info),
        }
    }

    pub fn is_component(&self) -> bool {
        matches!(self.classification, ReflectClassification::Component)
    }

    pub fn is_resource(&self) -> bool {
        matches!(self.classification, ReflectClassification::Resource)
    }
}
