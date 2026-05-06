//! File: domain/editor/editor_scene/src/sdf_authoring/mod.rs
//! Purpose: Scene-authoring contracts for SDF primitive and brush workflows.

use editor_core::EntityId;

use crate::SceneTransform;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdfPrimitiveKind {
    Box,
    Sphere,
    Capsule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdfBooleanIntent {
    Add,
    Subtract,
    Intersect,
    SmoothAdd,
    SmoothSubtract,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SdfPrimitiveSpec {
    pub kind: SdfPrimitiveKind,
    pub boolean: SdfBooleanIntent,
    pub transform: SceneTransform,
}

impl SdfPrimitiveSpec {
    pub const fn new(kind: SdfPrimitiveKind, boolean: SdfBooleanIntent) -> Self {
        Self {
            kind,
            boolean,
            transform: SceneTransform::identity(),
        }
    }

    pub const fn with_transform(mut self, transform: SceneTransform) -> Self {
        self.transform = transform;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdfBrushLayerMetadata {
    pub stable_name: String,
    pub display_name: String,
    pub enabled: bool,
}

impl SdfBrushLayerMetadata {
    pub fn new(stable_name: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            stable_name: stable_name.into(),
            display_name: display_name.into(),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SdfPreviewDiagnostic {
    pub code: &'static str,
    pub message: String,
    pub entity: Option<EntityId>,
}

impl SdfPreviewDiagnostic {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            entity: None,
        }
    }

    pub fn with_entity(mut self, entity: EntityId) -> Self {
        self.entity = Some(entity);
        self
    }
}
