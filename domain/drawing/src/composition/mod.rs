//! File: domain/drawing/src/composition/mod.rs
//! Purpose: Drawing semantic composition graph contracts over domain/graph.

mod adjustment;
mod clip;
mod graph_model;
mod group;
mod mask;
mod node;
mod output;
mod port;
mod source;
mod stack;
mod transform;

pub use adjustment::{AdjustmentDescriptor, AdjustmentNode};
pub use clip::{ClipMode, ClipNode};
pub use graph_model::DrawingCompositeGraph;
pub use group::{GroupClipPolicy, GroupIsolationPolicy, GroupNode};
pub use mask::{MaskMode, MaskNode};
pub use node::{DrawingCompositeNode, EffectFamily, EffectMaturityTier, EffectNode};
pub use output::{CompositeOutput, CompositeOutputId, CompositeOutputSemantics};
pub use port::{CompositePort, CompositePortSemantic};
pub use source::{PaintLayerSource, PaperSource, ReferenceImageSource};
pub use stack::{BlendMode, LayerStackEntry, LayerStackEntryContent, LayerStackNode};
pub use transform::TransformNode;
