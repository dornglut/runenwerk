//! Domain-owned stroke visualization contracts.

use crate::{CanvasCoordinate, StrokeSample};

#[derive(Debug, Clone, PartialEq)]
pub struct StrokeSampleTimeline {
    samples: Vec<StrokeSample>,
}

impl StrokeSampleTimeline {
    pub fn from_samples(samples: impl IntoIterator<Item = StrokeSample>) -> Self {
        Self {
            samples: samples.into_iter().collect(),
        }
    }

    pub fn samples(&self) -> &[StrokeSample] {
        &self.samples
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrokeReconstructionPolicy {
    pub version: u32,
}

impl StrokeReconstructionPolicy {
    pub const fn identity() -> Self {
        Self { version: 1 }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrokeReconstructedPath {
    samples: Vec<StrokeSample>,
    policy: StrokeReconstructionPolicy,
}

impl StrokeReconstructedPath {
    pub fn from_timeline(
        timeline: &StrokeSampleTimeline,
        policy: StrokeReconstructionPolicy,
    ) -> Self {
        Self {
            samples: timeline.samples().to_vec(),
            policy,
        }
    }

    pub fn samples(&self) -> &[StrokeSample] {
        &self.samples
    }

    pub fn policy(&self) -> StrokeReconstructionPolicy {
        self.policy
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrushDab {
    pub center: CanvasCoordinate,
    pub radius: f64,
    pub opacity: f32,
    pub flow: f32,
    pub edge_softness: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrushDabStream {
    dabs: Vec<BrushDab>,
}

impl BrushDabStream {
    pub fn new(dabs: impl IntoIterator<Item = BrushDab>) -> Self {
        Self {
            dabs: dabs.into_iter().collect(),
        }
    }

    pub fn dabs(&self) -> &[BrushDab] {
        &self.dabs
    }

    pub fn len(&self) -> usize {
        self.dabs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dabs.is_empty()
    }
}
