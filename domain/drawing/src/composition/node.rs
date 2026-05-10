//! File: domain/drawing/src/composition/node.rs
//! Purpose: Drawing-owned semantic node variants.

use crate::{
    AdjustmentNode, ClipNode, CompositeOutput, GroupNode, LayerStackNode, MaskNode,
    PaintLayerSource, PaperSource, ReferenceImageSource, TransformNode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectMaturityTier {
    Declared,
    Descriptor,
    Preview,
    Final,
    Shippable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectFamily {
    Painterly,
    Watercolor,
    DecorativeFinish,
    PaperMaterial,
    TechnicalDrawing,
    ComicReader,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectNode {
    pub family: EffectFamily,
    pub stable_name: String,
    pub maturity: EffectMaturityTier,
}

impl EffectNode {
    pub fn new(
        family: EffectFamily,
        stable_name: impl Into<String>,
        maturity: EffectMaturityTier,
    ) -> Self {
        Self {
            family,
            stable_name: stable_name.into(),
            maturity,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DrawingCompositeNode {
    LayerStack(LayerStackNode),
    Group(GroupNode),
    PaintLayerSource(PaintLayerSource),
    PaperSource(PaperSource),
    ReferenceImageSource(ReferenceImageSource),
    Mask(MaskNode),
    Clip(ClipNode),
    Transform(TransformNode),
    Adjustment(AdjustmentNode),
    CompositeOutput(CompositeOutput),
    Effect(EffectNode),
}
