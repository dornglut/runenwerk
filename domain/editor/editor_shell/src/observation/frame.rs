//! File: domain/editor/editor_shell/src/observation/frame.rs
//! Purpose: Shared observation frame metadata contracts.

use editor_core::RealityVersion;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObservationFrameId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservationSourceReality {
    ObservedScene,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservationConsumerKind {
    Outliner,
    Inspector,
    Toolbar,
    Viewport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservationFreshness {
    Current,
    PotentiallyStale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObservationFrameMetadata {
    pub frame_id: ObservationFrameId,
    pub source_reality: ObservationSourceReality,
    pub consumer_kind: ObservationConsumerKind,
    pub source_version: RealityVersion,
    pub freshness: ObservationFreshness,
}

impl ObservationFrameMetadata {
    pub fn strict_current(
        source_reality: ObservationSourceReality,
        consumer_kind: ObservationConsumerKind,
        source_version: RealityVersion,
    ) -> Self {
        Self {
            frame_id: ObservationFrameId(source_version.0),
            source_reality,
            consumer_kind,
            source_version,
            freshness: ObservationFreshness::Current,
        }
    }
}
