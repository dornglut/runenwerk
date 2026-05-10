//! File: domain/drawing/src/composition/transform.rs
//! Purpose: Non-destructive 2D transform intent.

use graph::NodeId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformNode {
    pub source_node: Option<NodeId>,
    pub translation: (f64, f64),
    pub scale: (f64, f64),
    pub rotation_radians: f64,
}

impl TransformNode {
    pub const fn identity(source_node: NodeId) -> Self {
        Self {
            source_node: Some(source_node),
            translation: (0.0, 0.0),
            scale: (1.0, 1.0),
            rotation_radians: 0.0,
        }
    }

    pub fn is_valid(self) -> bool {
        self.translation.0.is_finite()
            && self.translation.1.is_finite()
            && self.scale.0.is_finite()
            && self.scale.1.is_finite()
            && self.scale.0 != 0.0
            && self.scale.1 != 0.0
            && self.rotation_radians.is_finite()
    }
}
