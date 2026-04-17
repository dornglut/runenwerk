//! File: domain/editor/editor_shell/src/observation/frame.rs
//! Purpose: Shared observation frame metadata contracts.

use editor_core::RealityVersion;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservationSourceReality {
    ObservedScene,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservationConsumerKind {
    Outliner,
    Inspector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservationStalenessTolerance {
    StrictCurrentVersion,
    Eventual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObservationFrameMetadata {
    pub source_reality: ObservationSourceReality,
    pub consumer_kind: ObservationConsumerKind,
    pub source_version: RealityVersion,
    pub staleness_tolerance: ObservationStalenessTolerance,
}

impl ObservationFrameMetadata {
    pub fn strict_current(
        source_reality: ObservationSourceReality,
        consumer_kind: ObservationConsumerKind,
        source_version: RealityVersion,
    ) -> Self {
        Self {
            source_reality,
            consumer_kind,
            source_version,
            staleness_tolerance: ObservationStalenessTolerance::StrictCurrentVersion,
        }
    }
}
