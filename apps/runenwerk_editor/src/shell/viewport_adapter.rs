use crate::editor_panels::ViewportToolState;
use editor_core::EntityId;
use editor_shell::{
    ObservationConsumerKind, ObservationFrameMetadata, ObservationSourceReality,
    ViewportObservationFrame, ViewportProductObservation, ViewportProductChoiceViewModel,
    ViewportViewModel,
};
use editor_viewport::{ArtifactObservationFrame, ProductAvailabilityState, ProducerHealth};

pub fn build_viewport_observation_frame(
    products: Option<&ArtifactObservationFrame>,
    selected_entity: Option<EntityId>,
    drag_in_progress: bool,
    tool_state: ViewportToolState,
    source_version: editor_core::RealityVersion,
) -> ViewportObservationFrame {
    let viewport_id = products
        .map(|value| value.viewport_id)
        .unwrap_or(editor_viewport::ViewportId(1));
    let selected_primary_product_id = products.and_then(|value| value.selected_primary_product_id);
    let products = products
        .map(|value| {
            value
                .available_products
                .iter()
                .map(|descriptor| {
                    let availability = value
                        .availability_by_product
                        .get(&descriptor.id)
                        .copied()
                        .unwrap_or(ProductAvailabilityState::Unavailable);
                    let producer_health = value
                        .producer_health_by_product
                        .get(&descriptor.id)
                        .copied()
                        .unwrap_or(ProducerHealth::Unavailable);
                    ViewportProductObservation {
                        viewport_id: value.viewport_id,
                        product_id: descriptor.id,
                        product_kind: descriptor.kind,
                        label: format!("{:?}", descriptor.kind),
                        freshness: descriptor.freshness,
                        availability,
                        producer_health,
                        is_selected_primary: value.selected_primary_product_id
                            == Some(descriptor.id),
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    ViewportObservationFrame {
        metadata: ObservationFrameMetadata::strict_current(
            ObservationSourceReality::ObservedScene,
            ObservationConsumerKind::Viewport,
            source_version,
        ),
        viewport_id,
        selected_primary_product_id,
        products,
        selected_entity,
        hovered_entity: tool_state.hovered_entity,
        drag_in_progress,
        preview_active: tool_state.active_preview.is_some(),
    }
}

pub fn build_viewport_view_model(frame: &ViewportObservationFrame) -> ViewportViewModel {
    ViewportViewModel {
        viewport_id: Some(frame.viewport_id),
        selected_primary_product_id: frame.selected_primary_product_id,
        product_choices: frame
            .products
            .iter()
            .map(|product| ViewportProductChoiceViewModel {
                viewport_id: product.viewport_id,
                product_id: product.product_id,
                label: format!(
                    "{:?} [{:?}/{:?}]",
                    product.product_kind, product.availability, product.producer_health
                ),
                selected: product.is_selected_primary,
                enabled: product.availability == ProductAvailabilityState::Available,
            })
            .collect::<Vec<_>>(),
        selected_entity: frame.selected_entity,
        hovered_entity: frame.hovered_entity,
        drag_in_progress: frame.drag_in_progress,
        preview_active: frame.preview_active,
    }
}
