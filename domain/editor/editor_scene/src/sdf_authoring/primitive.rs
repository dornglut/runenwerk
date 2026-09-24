//! File: domain/editor/editor_scene/src/sdf_authoring/primitive.rs
//! Purpose: Primitive SDF authoring vocabulary shared by scene and operation workflows.

use editor_core::EntityId;

use crate::SceneTransform;

pub const DEFAULT_SDF_SMOOTH_RADIUS_METERS: f32 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdfPrimitiveKind {
    Box,
    Sphere,
    Capsule,
    Cylinder,
    Torus,
    Plane,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdfBooleanIntent {
    Add,
    Subtract,
    Intersect,
    SmoothAdd,
    SmoothSubtract,
    SmoothIntersect,
}

impl SdfBooleanIntent {
    pub const fn requires_smooth_radius(self) -> bool {
        matches!(
            self,
            Self::SmoothAdd | Self::SmoothSubtract | Self::SmoothIntersect
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SdfPrimitiveSpec {
    pub kind: SdfPrimitiveKind,
    pub boolean: SdfBooleanIntent,
    pub transform: SceneTransform,
    pub smooth_radius_meters: Option<f32>,
}

impl SdfPrimitiveSpec {
    pub const fn new(kind: SdfPrimitiveKind, boolean: SdfBooleanIntent) -> Self {
        Self {
            kind,
            boolean,
            transform: SceneTransform::identity(),
            smooth_radius_meters: None,
        }
    }

    pub const fn with_transform(mut self, transform: SceneTransform) -> Self {
        self.transform = transform;
        self
    }

    pub const fn with_smooth_radius_meters(mut self, radius: f32) -> Self {
        self.smooth_radius_meters = Some(radius);
        self
    }

    pub const fn with_default_smooth_radius(mut self) -> Self {
        self.smooth_radius_meters = Some(DEFAULT_SDF_SMOOTH_RADIUS_METERS);
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
