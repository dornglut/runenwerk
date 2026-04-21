//! File: domain/editor/editor_viewport/src/expression/observation.rs
//! Purpose: Observation frame exposed for viewport product tooling.

use std::collections::BTreeMap;

use editor_core::RealityVersion;

use crate::{ExpressionProductDescriptor, ExpressionProductId, ViewportId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProductAvailabilityState {
    Available,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProducerHealth {
    Healthy,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactObservationFrame {
    pub viewport_id: ViewportId,
    pub source_version: RealityVersion,
    pub available_products: Vec<ExpressionProductDescriptor>,
    pub selected_primary_product_id: Option<ExpressionProductId>,
    pub selected_overlay_product_ids: Vec<ExpressionProductId>,
    pub availability_by_product: BTreeMap<ExpressionProductId, ProductAvailabilityState>,
    pub producer_health_by_product: BTreeMap<ExpressionProductId, ProducerHealth>,
}

impl ArtifactObservationFrame {
    pub fn new(viewport_id: ViewportId, source_version: RealityVersion) -> Self {
        Self {
            viewport_id,
            source_version,
            available_products: Vec::new(),
            selected_primary_product_id: None,
            selected_overlay_product_ids: Vec::new(),
            availability_by_product: BTreeMap::new(),
            producer_health_by_product: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observation_frame_tracks_selected_product_identity() {
        let mut frame = ArtifactObservationFrame::new(ViewportId(1), RealityVersion(4));
        frame.selected_primary_product_id = Some(ExpressionProductId(12));

        assert_eq!(frame.viewport_id, ViewportId(1));
        assert_eq!(frame.source_version, RealityVersion(4));
        assert_eq!(
            frame.selected_primary_product_id,
            Some(ExpressionProductId(12))
        );
    }
}
