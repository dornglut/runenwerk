//! File: domain/drawing/src/composition/adjustment.rs
//! Purpose: Cheap deterministic adjustment descriptors.

use graph::NodeId;

#[derive(Debug, Clone, PartialEq)]
pub enum AdjustmentDescriptor {
    Opacity(f32),
    BrightnessContrast {
        brightness: f32,
        contrast: f32,
    },
    Hsv {
        hue_degrees: f32,
        saturation: f32,
        value: f32,
    },
    Threshold {
        threshold: f32,
    },
    ChannelRemap {
        red_from: u8,
        green_from: u8,
        blue_from: u8,
        alpha_from: u8,
    },
    SimpleGradientMap {
        dark: crate::ColorRgba,
        light: crate::ColorRgba,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdjustmentNode {
    pub source_node: Option<NodeId>,
    pub descriptor: AdjustmentDescriptor,
}

impl AdjustmentNode {
    pub const fn new(source_node: NodeId, descriptor: AdjustmentDescriptor) -> Self {
        Self {
            source_node: Some(source_node),
            descriptor,
        }
    }
}
