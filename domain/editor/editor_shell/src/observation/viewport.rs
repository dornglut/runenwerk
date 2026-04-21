//! File: domain/editor/editor_shell/src/observation/viewport.rs
//! Purpose: Viewport observation frame contracts.

use editor_core::EntityId;
use editor_viewport::{
    ExpressionFreshness, ExpressionProductId, ExpressionProductKind, ProducerHealth,
    ProductAvailabilityState, ViewportId,
};

use crate::ObservationFrameMetadata;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewportProductObservation {
    pub viewport_id: ViewportId,
    pub product_id: ExpressionProductId,
    pub product_kind: ExpressionProductKind,
    pub label: String,
    pub freshness: ExpressionFreshness,
    pub availability: ProductAvailabilityState,
    pub producer_health: ProducerHealth,
    pub is_selected_primary: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewportObservationFrame {
    pub metadata: ObservationFrameMetadata,
    pub viewport_id: ViewportId,
    pub selected_primary_product_id: Option<ExpressionProductId>,
    pub products: Vec<ViewportProductObservation>,
    pub details_visible: bool,
    pub selected_entity: Option<EntityId>,
    pub hovered_entity: Option<EntityId>,
    pub drag_in_progress: bool,
    pub preview_active: bool,
}
