//! File: domain/editor/editor_shell/src/expression/mod.rs
//! Purpose: Expression-frame contracts for shell renderer consumers.

use editor_core::{ComponentTypeId, EntityId, RealityVersion};
use ui_render_data::UiFrame;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExpressionConsumerKind {
    UiRenderer,
    ViewportInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExpressionSourceReality {
    ObservedShell,
    ObservedPicking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExpressionFrameId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExpressionFrameVersion(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExpressionFrameMetadata {
    pub frame_id: ExpressionFrameId,
    pub source_reality: ExpressionSourceReality,
    pub consumer_kind: ExpressionConsumerKind,
    pub source_version: RealityVersion,
    pub expression_version: ExpressionFrameVersion,
}

impl ExpressionFrameMetadata {
    pub fn new(
        source_reality: ExpressionSourceReality,
        consumer_kind: ExpressionConsumerKind,
        source_version: RealityVersion,
    ) -> Self {
        Self {
            frame_id: ExpressionFrameId(source_version.0),
            source_reality,
            consumer_kind,
            source_version,
            expression_version: ExpressionFrameVersion(source_version.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShellUiExpressionFrame {
    pub metadata: ExpressionFrameMetadata,
    pub frame: UiFrame,
}

impl ShellUiExpressionFrame {
    pub fn new(source_version: RealityVersion, frame: UiFrame) -> Self {
        Self {
            metadata: ExpressionFrameMetadata::new(
                ExpressionSourceReality::ObservedShell,
                ExpressionConsumerKind::UiRenderer,
                source_version,
            ),
            frame,
        }
    }

    pub fn into_ui_frame(self) -> UiFrame {
        self.frame
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PickingExpressionAxis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PickingExpressionTarget {
    None,
    Grid,
    Entity(EntityId),
    ComponentHandle {
        entity: EntityId,
        component_type: ComponentTypeId,
    },
    GizmoAxis(PickingExpressionAxis),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PickingExpressionFrame {
    pub metadata: ExpressionFrameMetadata,
    pub target: PickingExpressionTarget,
    pub distance: f32,
}

impl PickingExpressionFrame {
    pub fn new(
        source_version: RealityVersion,
        target: PickingExpressionTarget,
        distance: f32,
    ) -> Self {
        Self {
            metadata: ExpressionFrameMetadata::new(
                ExpressionSourceReality::ObservedPicking,
                ExpressionConsumerKind::ViewportInput,
                source_version,
            ),
            target,
            distance: if distance.is_finite() { distance } else { 0.0 },
        }
    }
}
