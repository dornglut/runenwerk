//! File: apps/runenwerk_editor/src/runtime/viewport/product_targets.rs
//! Purpose: App-owned viewport product target registry and dynamic target requests.

use std::collections::{BTreeMap, BTreeSet};

use editor_core::RealityVersion;
use editor_viewport::{
    ArtifactObservationFrame, ExpressionDimensions, ExpressionFormat, ExpressionProductDescriptor,
    ExpressionProductId, ExpressionProductKind, ProducerHealth, ProductAvailabilityState,
    ViewportId, ViewportPresentationState, ViewportSurfacePresentationSlot,
};
use engine::plugins::render::{
    RenderDynamicTextureRetention, RenderDynamicTextureTargetDescriptor,
    RenderDynamicTextureTargetKey, RenderDynamicTextureTargetRequestRegistryResource,
    RenderTextureSampleMode, RenderTextureTargetFormat, RenderTextureTargetUsage,
    ViewportSurfaceBindingRegistryResource,
};
use engine::runtime::{Res, ResMut};

use crate::runtime::resources::EditorHostResource;
use crate::runtime::viewport::{
    EDITOR_VIEWPORT_RENDER_PRODUCT_PRODUCER_ID, MAIN_VIEWPORT_ID,
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
    ViewportPresentationStateResource, ViewportProductRegistryResource,
    ViewportRenderStateResource, ViewportSurfaceHandle, ViewportSurfaceSetResource,
    ViewportSurfaceSlot, build_surface_binding_registry, ensure_editor_main_surface_set,
    expression_dimensions_for_bounds, initial_presentation_state, initial_product_descriptors,
};

pub const VIEWPORT_DYNAMIC_TARGET_NAMESPACE: &str = "runenwerk.editor.viewport";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ViewportProductTargetKey {
    pub viewport_id: ViewportId,
    pub presentation_slot: ViewportSurfacePresentationSlot,
    pub product_id: ExpressionProductId,
}

impl ViewportProductTargetKey {
    pub const fn new(
        viewport_id: ViewportId,
        presentation_slot: ViewportSurfacePresentationSlot,
        product_id: ExpressionProductId,
    ) -> Self {
        Self {
            viewport_id,
            presentation_slot,
            product_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportProductTargetStatus {
    Requested,
    Unavailable,
}

#[derive(Debug, Clone)]
pub struct ViewportProductTargetRecord {
    pub key: ViewportProductTargetKey,
    pub surface_slot: ViewportSurfaceSlot,
    pub namespace: String,
    pub target_id: String,
    pub width: u32,
    pub height: u32,
    pub format: ExpressionFormat,
    pub target_format: Option<RenderTextureTargetFormat>,
    pub usage: RenderTextureTargetUsage,
    pub sample_mode: RenderTextureSampleMode,
    pub ui_sampleable: bool,
    pub status: ViewportProductTargetStatus,
    pub generation: u64,
}

impl ViewportProductTargetRecord {
    pub fn dynamic_key(&self) -> RenderDynamicTextureTargetKey {
        RenderDynamicTextureTargetKey {
            namespace: self.namespace.clone(),
            target_id: self.target_id.clone(),
        }
    }

    pub fn dynamic_descriptor(&self) -> Option<RenderDynamicTextureTargetDescriptor> {
        if self.status != ViewportProductTargetStatus::Requested {
            return None;
        }
        let target_format = self.target_format?;
        Some(RenderDynamicTextureTargetDescriptor {
            key: self.dynamic_key(),
            width: self.width,
            height: self.height,
            format: target_format,
            usage: self.usage,
            sample_mode: self.sample_mode,
            retention: RenderDynamicTextureRetention::RetainWhileRequested,
        })
    }

    pub fn surface_handle(&self) -> Option<ViewportSurfaceHandle> {
        if self.status != ViewportProductTargetStatus::Requested {
            return None;
        }
        Some(ViewportSurfaceHandle::dynamic_texture(
            self.namespace.clone(),
            self.target_id.clone(),
            self.ui_sampleable,
        ))
    }
}

#[derive(Debug, Default, ecs::Component, ecs::Resource)]
pub struct ViewportProductTargetRegistryResource {
    records: BTreeMap<ViewportProductTargetKey, ViewportProductTargetRecord>,
    generation: u64,
}

impl ViewportProductTargetRegistryResource {
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn record(&self, key: ViewportProductTargetKey) -> Option<&ViewportProductTargetRecord> {
        self.records.get(&key)
    }

    pub fn record_for_product(
        &self,
        viewport_id: ViewportId,
        presentation_slot: ViewportSurfacePresentationSlot,
        product_id: ExpressionProductId,
    ) -> Option<&ViewportProductTargetRecord> {
        self.record(ViewportProductTargetKey::new(
            viewport_id,
            presentation_slot,
            product_id,
        ))
    }

    pub fn records(&self) -> impl Iterator<Item = &ViewportProductTargetRecord> {
        self.records.values()
    }

    pub fn viewport_ids(&self) -> impl Iterator<Item = ViewportId> + '_ {
        self.records
            .keys()
            .map(|key| key.viewport_id)
            .collect::<BTreeSet<_>>()
            .into_iter()
    }

    pub fn requested_dynamic_descriptors(&self) -> Vec<RenderDynamicTextureTargetDescriptor> {
        self.records
            .values()
            .filter_map(ViewportProductTargetRecord::dynamic_descriptor)
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn replace_records(&mut self, records: Vec<ViewportProductTargetRecord>) {
        let previous = std::mem::take(&mut self.records);
        self.generation = self.generation.saturating_add(1);
        let generation = self.generation;

        for mut record in records {
            if let Some(previous_record) = previous.get(&record.key)
                && target_signature_matches(previous_record, &record)
            {
                record.generation = previous_record.generation;
            } else {
                record.generation = generation;
            }
            self.records.insert(record.key, record);
        }
    }
}

pub fn sync_viewport_product_targets_system(
    viewport_products: Res<ViewportProductRegistryResource>,
    viewport_presentations: Res<ViewportPresentationStateResource>,
    mut viewport_product_targets: ResMut<ViewportProductTargetRegistryResource>,
    mut dynamic_target_requests: ResMut<RenderDynamicTextureTargetRequestRegistryResource>,
    mut viewport_surface_sets: ResMut<ViewportSurfaceSetResource>,
    mut viewport_surface_bindings: ResMut<ViewportSurfaceBindingRegistryResource>,
) {
    let mut records = Vec::new();
    for viewport_id in viewport_products.viewport_ids() {
        let Some(descriptors) = viewport_products.descriptors_for(viewport_id) else {
            continue;
        };
        records.extend(descriptors.iter().filter_map(|descriptor| {
            product_target_record_for_descriptor(viewport_id, descriptor)
        }));
    }

    viewport_product_targets.replace_records(records);
    dynamic_target_requests
        .replace_contribution(
            EDITOR_VIEWPORT_RENDER_PRODUCT_PRODUCER_ID,
            viewport_product_targets.requested_dynamic_descriptors(),
        )
        .expect("editor viewport dynamic target contribution must be valid and uniquely owned");
    sync_surface_sets_from_product_targets(&viewport_product_targets, &mut viewport_surface_sets);
    viewport_surface_bindings.replace_registry(build_surface_binding_registry(
        &viewport_surface_sets,
        &viewport_presentations,
    ));
}

#[allow(clippy::too_many_arguments)]
pub fn sync_viewport_presentation_products_system(
    host: Res<EditorHostResource>,
    viewport_render_states: Res<ViewportRenderStateResource>,
    mut viewport_surface_sets: ResMut<ViewportSurfaceSetResource>,
    tool_surface_bindings: Res<ToolSurfaceRuntimeBindingRegistryResource>,
    mut viewport_products_registry: ResMut<ViewportProductRegistryResource>,
    mut viewport_presentations: ResMut<ViewportPresentationStateResource>,
    mut viewport_observations: ResMut<ViewportArtifactObservationResource>,
) {
    let canonical_viewport_ids =
        canonical_viewport_ids_for_sync(&viewport_surface_sets, &tool_surface_bindings);
    for viewport_id in &canonical_viewport_ids {
        ensure_editor_main_surface_set(&mut viewport_surface_sets, *viewport_id);
    }

    let source_version = host.app.runtime().current_scene_reality_version();
    for viewport_id in &canonical_viewport_ids {
        let product_dimensions =
            product_dimensions_for_viewport(*viewport_id, &viewport_render_states);
        let descriptors = initial_product_descriptors(product_dimensions, source_version);
        viewport_products_registry.update_viewport_descriptors(*viewport_id, descriptors.clone());

        let mut presentation_state = viewport_presentations
            .state_for(*viewport_id)
            .cloned()
            .unwrap_or_else(|| initial_presentation_state(*viewport_id));
        if !descriptors
            .iter()
            .any(|descriptor| descriptor.id == presentation_state.selected_primary_product_id)
        {
            presentation_state.select_primary_product(
                initial_presentation_state(*viewport_id).selected_primary_product_id,
            );
        }
        viewport_presentations.upsert_state(presentation_state.clone());
        viewport_observations.upsert_frame(build_artifact_observation_frame(
            &descriptors,
            &presentation_state,
            source_version,
        ));
    }

    let viewport_id_set = canonical_viewport_ids
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    viewport_products_registry.retain_viewports(|viewport_id| {
        viewport_id == MAIN_VIEWPORT_ID || viewport_id_set.contains(&viewport_id)
    });
    viewport_presentations.retain_viewports(|viewport_id| {
        viewport_id == MAIN_VIEWPORT_ID || viewport_id_set.contains(&viewport_id)
    });
    viewport_observations.retain_viewports(|viewport_id| {
        viewport_id == MAIN_VIEWPORT_ID || viewport_id_set.contains(&viewport_id)
    });
    viewport_surface_sets.retain_viewports(|viewport_id| {
        viewport_id == MAIN_VIEWPORT_ID || viewport_id_set.contains(&viewport_id)
    });
}

fn canonical_viewport_ids_for_sync(
    viewport_surface_sets: &ViewportSurfaceSetResource,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
) -> Vec<ViewportId> {
    let mut viewport_ids = tool_surface_bindings
        .bindings()
        .map(|binding| binding.viewport_id)
        .collect::<std::collections::BTreeSet<_>>();
    if viewport_ids.is_empty() {
        viewport_ids.extend(viewport_surface_sets.viewport_ids());
    }
    viewport_ids.into_iter().collect()
}

fn product_dimensions_for_viewport(
    viewport_id: ViewportId,
    viewport_render_states: &ViewportRenderStateResource,
) -> ExpressionDimensions {
    viewport_render_states
        .state_for(viewport_id)
        .map(|state| expression_dimensions_for_bounds(state.bounds))
        .unwrap_or_else(|| ExpressionDimensions::new(1, 1))
}

fn build_artifact_observation_frame(
    descriptors: &[ExpressionProductDescriptor],
    presentation_state: &ViewportPresentationState,
    source_version: RealityVersion,
) -> ArtifactObservationFrame {
    let mut frame = ArtifactObservationFrame::new(presentation_state.viewport_id, source_version);
    frame.available_products = descriptors.to_vec();
    frame.selected_primary_product_id = Some(presentation_state.selected_primary_product_id);
    frame.selected_overlay_product_ids = presentation_state.selected_overlay_product_ids.clone();

    for descriptor in descriptors {
        let target_record =
            product_target_record_for_descriptor(presentation_state.viewport_id, descriptor);
        let available = target_record
            .as_ref()
            .is_some_and(|record| record.status == ViewportProductTargetStatus::Requested);
        frame.availability_by_product.insert(
            descriptor.id,
            if available {
                ProductAvailabilityState::Available
            } else {
                ProductAvailabilityState::Unavailable
            },
        );
        frame.producer_health_by_product.insert(
            descriptor.id,
            if available {
                ProducerHealth::Healthy
            } else {
                ProducerHealth::Unavailable
            },
        );
    }

    if let std::collections::btree_map::Entry::Vacant(e) = frame
        .availability_by_product
        .entry(presentation_state.selected_primary_product_id)
    {
        e.insert(ProductAvailabilityState::Unavailable);
        frame.producer_health_by_product.insert(
            presentation_state.selected_primary_product_id,
            ProducerHealth::Unavailable,
        );
    }

    frame
}

fn sync_surface_sets_from_product_targets(
    viewport_product_targets: &ViewportProductTargetRegistryResource,
    viewport_surface_sets: &mut ViewportSurfaceSetResource,
) {
    let active_viewports = viewport_product_targets
        .viewport_ids()
        .collect::<BTreeSet<_>>();
    viewport_surface_sets.retain_viewports(|viewport_id| active_viewports.contains(&viewport_id));
    for viewport_id in &active_viewports {
        viewport_surface_sets.clear_viewport_surfaces(*viewport_id);
    }

    for record in viewport_product_targets.records() {
        let Some(handle) = record.surface_handle() else {
            continue;
        };
        viewport_surface_sets.set_surface(record.key.viewport_id, record.surface_slot, handle);
    }
}

pub(crate) fn product_target_record_for_descriptor(
    viewport_id: ViewportId,
    descriptor: &ExpressionProductDescriptor,
) -> Option<ViewportProductTargetRecord> {
    let (presentation_slot, surface_slot) = product_target_slots(descriptor.kind)?;
    let key = ViewportProductTargetKey::new(viewport_id, presentation_slot, descriptor.id);
    let target_format = target_format_for_descriptor(descriptor);
    let status = if descriptor.dimensions.width == 0
        || descriptor.dimensions.height == 0
        || target_format.is_none()
    {
        ViewportProductTargetStatus::Unavailable
    } else {
        ViewportProductTargetStatus::Requested
    };
    let sample_mode = sample_mode_for_descriptor(descriptor);
    let ui_sampleable = matches!(
        sample_mode,
        RenderTextureSampleMode::FilterableFloat | RenderTextureSampleMode::NonFilterableFloat
    );

    Some(ViewportProductTargetRecord {
        key,
        surface_slot,
        namespace: VIEWPORT_DYNAMIC_TARGET_NAMESPACE.to_string(),
        target_id: dynamic_target_id_for(key),
        width: descriptor.dimensions.width,
        height: descriptor.dimensions.height,
        format: descriptor.format.clone(),
        target_format,
        usage: usage_for_descriptor(descriptor),
        sample_mode,
        ui_sampleable,
        status,
        generation: 0,
    })
}

fn product_target_slots(
    kind: ExpressionProductKind,
) -> Option<(ViewportSurfacePresentationSlot, ViewportSurfaceSlot)> {
    match kind {
        ExpressionProductKind::SceneColor2D => Some((
            ViewportSurfacePresentationSlot::Primary,
            ViewportSurfaceSlot::PrimaryColor,
        )),
        ExpressionProductKind::PickingIds2D => Some((
            ViewportSurfacePresentationSlot::Picking,
            ViewportSurfaceSlot::PickingIds,
        )),
        ExpressionProductKind::Overlay2D => Some((
            ViewportSurfacePresentationSlot::Overlay,
            ViewportSurfaceSlot::Overlay,
        )),
        ExpressionProductKind::ScalarField2D => Some((
            ViewportSurfacePresentationSlot::Primary,
            ViewportSurfaceSlot::ScalarField,
        )),
        ExpressionProductKind::VectorField2D => Some((
            ViewportSurfacePresentationSlot::Primary,
            ViewportSurfaceSlot::VectorField,
        )),
        ExpressionProductKind::Atlas2D => Some((
            ViewportSurfacePresentationSlot::Primary,
            ViewportSurfaceSlot::Atlas,
        )),
        ExpressionProductKind::VolumeSlice2D => Some((
            ViewportSurfacePresentationSlot::Primary,
            ViewportSurfaceSlot::VolumeSlice,
        )),
        ExpressionProductKind::BrickmapDebug2D => Some((
            ViewportSurfacePresentationSlot::Primary,
            ViewportSurfaceSlot::BrickmapDebug,
        )),
        ExpressionProductKind::HistoryColor2D => Some((
            ViewportSurfacePresentationSlot::Primary,
            ViewportSurfaceSlot::HistoryColor,
        )),
        ExpressionProductKind::Depth2D | ExpressionProductKind::Diagnostics2D => None,
    }
}

pub fn dynamic_target_id_for(key: ViewportProductTargetKey) -> String {
    format!(
        "editor.viewport.{}.{}.{}",
        key.viewport_id.0,
        key.product_id.0,
        presentation_slot_label(key.presentation_slot)
    )
}

fn presentation_slot_label(slot: ViewportSurfacePresentationSlot) -> &'static str {
    match slot {
        ViewportSurfacePresentationSlot::Primary => "primary",
        ViewportSurfacePresentationSlot::Picking => "picking",
        ViewportSurfacePresentationSlot::Overlay => "overlay",
    }
}

fn target_format_for_descriptor(
    descriptor: &ExpressionProductDescriptor,
) -> Option<RenderTextureTargetFormat> {
    match &descriptor.format {
        ExpressionFormat::Rgba8Unorm if descriptor.presentation_hints.srgb => {
            Some(RenderTextureTargetFormat::Rgba8UnormSrgb)
        }
        ExpressionFormat::Rgba8Unorm => Some(RenderTextureTargetFormat::Rgba8Unorm),
        ExpressionFormat::R32Uint => Some(RenderTextureTargetFormat::R32Uint),
        ExpressionFormat::Depth32Float => Some(RenderTextureTargetFormat::Depth32Float),
        ExpressionFormat::Other(_) => None,
    }
}

fn usage_for_descriptor(descriptor: &ExpressionProductDescriptor) -> RenderTextureTargetUsage {
    match descriptor.kind {
        ExpressionProductKind::SceneColor2D | ExpressionProductKind::Overlay2D => {
            RenderTextureTargetUsage {
                color_attachment: true,
                depth_attachment: false,
                sampled: true,
                storage: false,
                copy_src: true,
                copy_dst: false,
            }
        }
        ExpressionProductKind::PickingIds2D => RenderTextureTargetUsage {
            color_attachment: true,
            depth_attachment: false,
            sampled: false,
            storage: false,
            copy_src: true,
            copy_dst: false,
        },
        ExpressionProductKind::Depth2D => RenderTextureTargetUsage {
            color_attachment: false,
            depth_attachment: true,
            sampled: false,
            storage: false,
            copy_src: true,
            copy_dst: false,
        },
        ExpressionProductKind::Diagnostics2D => RenderTextureTargetUsage {
            color_attachment: true,
            depth_attachment: false,
            sampled: true,
            storage: false,
            copy_src: true,
            copy_dst: false,
        },
        ExpressionProductKind::ScalarField2D
        | ExpressionProductKind::VectorField2D
        | ExpressionProductKind::Atlas2D
        | ExpressionProductKind::VolumeSlice2D
        | ExpressionProductKind::BrickmapDebug2D
        | ExpressionProductKind::HistoryColor2D => RenderTextureTargetUsage {
            color_attachment: false,
            depth_attachment: false,
            sampled: true,
            storage: false,
            copy_src: true,
            copy_dst: true,
        },
    }
}

fn sample_mode_for_descriptor(descriptor: &ExpressionProductDescriptor) -> RenderTextureSampleMode {
    match &descriptor.format {
        ExpressionFormat::Rgba8Unorm => RenderTextureSampleMode::FilterableFloat,
        ExpressionFormat::R32Uint => RenderTextureSampleMode::NotSampled,
        ExpressionFormat::Depth32Float => RenderTextureSampleMode::NotSampled,
        ExpressionFormat::Other(_) => RenderTextureSampleMode::NotSampled,
    }
}

fn target_signature_matches(
    previous: &ViewportProductTargetRecord,
    next: &ViewportProductTargetRecord,
) -> bool {
    previous.namespace == next.namespace
        && previous.target_id == next.target_id
        && previous.width == next.width
        && previous.height == next.height
        && previous.format == next.format
        && previous.ui_sampleable == next.ui_sampleable
        && previous.status == next.status
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::viewport::{
        HISTORY_COLOR_PRODUCT_ID, OVERLAY_PRODUCT_ID, PICKING_IDS_PRODUCT_ID,
        SCALAR_FIELD_PRODUCT_ID, SCENE_COLOR_PRODUCT_ID, ToolSurfaceRuntimeBindingRecord,
    };
    use editor_core::RealityVersion;
    use editor_shell::{PanelInstanceId, TabStackId, ToolSurfaceInstanceId, WidgetId};
    use editor_viewport::{
        ExpressionDimensions, ExpressionFreshness, ExpressionPresentationHints,
        ExpressionSourceRealityClass,
    };
    use ui_math::UiRect;

    fn binding(
        surface: u64,
        panel: u64,
        stack: u64,
        viewport_id: ViewportId,
        bounds: UiRect,
    ) -> ToolSurfaceRuntimeBindingRecord {
        ToolSurfaceRuntimeBindingRecord {
            tool_surface_id: ToolSurfaceInstanceId::try_from_raw(surface).unwrap(),
            panel_instance_id: PanelInstanceId::try_from_raw(panel).unwrap(),
            tab_stack_id: TabStackId::try_from_raw(stack).unwrap(),
            viewport_id,
            host_widget_id: WidgetId(10_000 + surface),
            bounds,
            generation: 1,
        }
    }

    fn descriptor(
        id: ExpressionProductId,
        kind: ExpressionProductKind,
        format: ExpressionFormat,
    ) -> ExpressionProductDescriptor {
        ExpressionProductDescriptor::new(
            id,
            kind,
            ExpressionDimensions::new(320, 200),
            format,
            "test.producer",
            ExpressionSourceRealityClass::ObservedScene,
            RealityVersion(1),
            ExpressionFreshness::Current,
            ExpressionPresentationHints::default(),
            None,
        )
    }

    #[test]
    fn dynamic_target_ids_are_stable_per_viewport_product_slot() {
        let key = ViewportProductTargetKey::new(
            ViewportId(42),
            ViewportSurfacePresentationSlot::Primary,
            SCENE_COLOR_PRODUCT_ID,
        );

        assert_eq!(dynamic_target_id_for(key), "editor.viewport.42.1.primary");
    }

    #[test]
    fn scene_targets_are_sampleable_and_picking_targets_are_not() {
        let scene = product_target_record_for_descriptor(
            ViewportId(1),
            &descriptor(
                SCENE_COLOR_PRODUCT_ID,
                ExpressionProductKind::SceneColor2D,
                ExpressionFormat::Rgba8Unorm,
            ),
        )
        .expect("scene color should map to a product target");
        let picking = product_target_record_for_descriptor(
            ViewportId(1),
            &descriptor(
                PICKING_IDS_PRODUCT_ID,
                ExpressionProductKind::PickingIds2D,
                ExpressionFormat::R32Uint,
            ),
        )
        .expect("picking ids should map to a product target");

        assert!(scene.ui_sampleable);
        assert!(!picking.ui_sampleable);
    }

    #[test]
    fn overlay_targets_use_the_overlay_presentation_slot() {
        let overlay = product_target_record_for_descriptor(
            ViewportId(7),
            &descriptor(
                OVERLAY_PRODUCT_ID,
                ExpressionProductKind::Overlay2D,
                ExpressionFormat::Rgba8Unorm,
            ),
        )
        .expect("overlay should map to a product target");

        assert_eq!(
            overlay.key.presentation_slot,
            ViewportSurfacePresentationSlot::Overlay
        );
        assert_eq!(overlay.surface_slot, ViewportSurfaceSlot::Overlay);
    }

    #[test]
    fn field_and_volume_debug_products_are_displayable_rgba_targets() {
        let descriptors =
            initial_product_descriptors(ExpressionDimensions::new(320, 200), RealityVersion(1));
        let presentation_state = initial_presentation_state(ViewportId(1));

        let frame =
            build_artifact_observation_frame(&descriptors, &presentation_state, RealityVersion(1));

        assert_eq!(
            frame
                .availability_by_product
                .get(&SCENE_COLOR_PRODUCT_ID)
                .copied(),
            Some(ProductAvailabilityState::Available)
        );
        assert_eq!(
            frame
                .availability_by_product
                .get(&SCALAR_FIELD_PRODUCT_ID)
                .copied(),
            Some(ProductAvailabilityState::Available)
        );
        assert_eq!(
            frame
                .producer_health_by_product
                .get(&HISTORY_COLOR_PRODUCT_ID)
                .copied(),
            Some(ProducerHealth::Healthy)
        );
    }

    #[test]
    fn presentation_sync_prefers_runtime_bindings_over_bootstrap_surface_set() {
        let mut surface_sets = ViewportSurfaceSetResource::default();
        ensure_editor_main_surface_set(&mut surface_sets, MAIN_VIEWPORT_ID);
        let mut bindings = ToolSurfaceRuntimeBindingRegistryResource::default();
        let first = ViewportId(2);
        let second = ViewportId(3);
        bindings.upsert_binding(binding(1, 1, 1, first, UiRect::new(0.0, 0.0, 320.0, 240.0)));
        bindings.upsert_binding(binding(
            2,
            2,
            2,
            second,
            UiRect::new(320.0, 0.0, 480.0, 240.0),
        ));

        assert_eq!(
            canonical_viewport_ids_for_sync(&surface_sets, &bindings),
            vec![first, second],
        );
    }
}
