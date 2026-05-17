//! File: apps/runenwerk_editor/src/runtime/viewport/render_jobs.rs
//! Purpose: Per-viewport render jobs bound to dynamic product targets.

use std::collections::BTreeMap;

use editor_viewport::{ExpressionDimensions, ViewportId, ViewportSurfacePresentationSlot};
use engine::plugins::render::{
    PreparedFlowInvocationId, PreparedFlowInvocationRequest, PreparedRenderFrameRequestResource,
    PreparedTargetBinding, PreparedViewFrame, RenderDynamicTextureTargetKey, RenderFlowId,
    RenderFlowRegistryResource, RenderResourceId,
};
use engine::runtime::{Res, ResMut};
use ui_math::UiRect;

use crate::runtime::viewport::{
    EDITOR_MAIN_FLOW_ID, EDITOR_VIEWPORT_RENDER_PRODUCT_PRODUCER_ID,
    EDITOR_VIEWPORT_SCENE_PRODUCT_UNIFORM_ID, OVERLAY_PRODUCT_ID, PICKING_IDS_PRODUCT_ID,
    SCENE_COLOR_PRODUCT_ID, VIEWPORT_TARGET_ALIAS_OVERLAY, VIEWPORT_TARGET_ALIAS_PICKING_IDS,
    VIEWPORT_TARGET_ALIAS_SCENE_COLOR, ViewportProductTargetRegistryResource,
    ViewportRenderStateResource, expression_dimensions_for_bounds,
};

#[derive(Debug)]
pub struct ViewportRenderJob {
    pub viewport_id: ViewportId,
    pub bounds: UiRect,
    pub dimensions: ExpressionDimensions,
    pub scene_color_target: RenderDynamicTextureTargetKey,
    pub picking_ids_target: RenderDynamicTextureTargetKey,
    pub overlay_target: RenderDynamicTextureTargetKey,
    pub prepared_view: PreparedViewFrame,
    pub prepared_flow_invocation: PreparedFlowInvocationRequest,
}

#[derive(Debug, Default, ecs::Component, ecs::Resource)]
pub struct ViewportRenderJobResource {
    jobs_by_viewport: BTreeMap<ViewportId, ViewportRenderJob>,
}

impl ViewportRenderJobResource {
    pub fn replace_jobs(&mut self, jobs: impl IntoIterator<Item = ViewportRenderJob>) {
        self.jobs_by_viewport = jobs.into_iter().map(|job| (job.viewport_id, job)).collect();
    }

    pub fn job_for(&self, viewport_id: ViewportId) -> Option<&ViewportRenderJob> {
        self.jobs_by_viewport.get(&viewport_id)
    }

    pub fn jobs(&self) -> impl Iterator<Item = &ViewportRenderJob> {
        self.jobs_by_viewport.values()
    }

    pub fn viewport_ids(&self) -> impl Iterator<Item = ViewportId> + '_ {
        self.jobs_by_viewport.keys().copied()
    }

    pub fn is_empty(&self) -> bool {
        self.jobs_by_viewport.is_empty()
    }
}

pub fn sync_viewport_render_jobs_system(
    flow_registry: Res<RenderFlowRegistryResource>,
    viewport_render_states: Res<ViewportRenderStateResource>,
    viewport_product_targets: Res<ViewportProductTargetRegistryResource>,
    mut viewport_render_jobs: ResMut<ViewportRenderJobResource>,
    mut prepared_frame_requests: ResMut<PreparedRenderFrameRequestResource>,
) {
    let Some((flow_id, scene_uniform_id)) = editor_main_flow_ids(&flow_registry) else {
        viewport_render_jobs.replace_jobs(Vec::new());
        let _ =
            prepared_frame_requests.remove_contribution(EDITOR_VIEWPORT_RENDER_PRODUCT_PRODUCER_ID);
        return;
    };

    let jobs = viewport_render_states
        .entries()
        .filter_map(|state| {
            build_viewport_render_job(
                flow_id,
                scene_uniform_id,
                &state.render_state,
                state.viewport_id,
                state.bounds,
                &viewport_product_targets,
            )
        })
        .collect::<Vec<_>>();
    let views = jobs.iter().map(|job| job.prepared_view.clone());
    let invocations = jobs.iter().map(|job| job.prepared_flow_invocation.clone());
    prepared_frame_requests
        .replace_contribution(
            EDITOR_VIEWPORT_RENDER_PRODUCT_PRODUCER_ID,
            views,
            invocations,
        )
        .expect("editor viewport prepared frame contribution must be unique");
    viewport_render_jobs.replace_jobs(jobs);
}

fn build_viewport_render_job(
    flow_id: RenderFlowId,
    scene_uniform_id: RenderResourceId,
    viewport_render: &crate::runtime::resources::EditorViewportRenderState,
    viewport_id: ViewportId,
    bounds: UiRect,
    product_targets: &ViewportProductTargetRegistryResource,
) -> Option<ViewportRenderJob> {
    let dimensions = expression_dimensions_for_bounds(bounds);
    let scene_color_target = product_targets
        .record_for_product(
            viewport_id,
            ViewportSurfacePresentationSlot::Primary,
            SCENE_COLOR_PRODUCT_ID,
        )?
        .dynamic_key();
    let picking_ids_target = product_targets
        .record_for_product(
            viewport_id,
            ViewportSurfacePresentationSlot::Picking,
            PICKING_IDS_PRODUCT_ID,
        )?
        .dynamic_key();
    let overlay_target = product_targets
        .record_for_product(
            viewport_id,
            ViewportSurfacePresentationSlot::Overlay,
            OVERLAY_PRODUCT_ID,
        )?
        .dynamic_key();

    let view_id = prepared_view_id(viewport_id);
    let prepared_view = PreparedViewFrame::offscreen_product(
        view_id.clone(),
        (dimensions.width, dimensions.height),
    );
    let prepared_flow_invocation = PreparedFlowInvocationRequest {
        invocation_id: PreparedFlowInvocationId::new(format!(
            "editor.viewport.{}.{}",
            viewport_id.0, EDITOR_MAIN_FLOW_ID
        )),
        flow_id,
        view_id,
        target_alias_bindings: target_alias_bindings(
            scene_color_target.clone(),
            picking_ids_target.clone(),
            overlay_target.clone(),
        ),
        uniform_overrides: BTreeMap::new(),
        history_signature: None,
    }
    .with_uniform_override(
        scene_uniform_id,
        viewport_render.compose_scene_product_uniform_bytes((dimensions.width, dimensions.height)),
    );

    Some(ViewportRenderJob {
        viewport_id,
        bounds,
        dimensions,
        scene_color_target,
        picking_ids_target,
        overlay_target,
        prepared_view,
        prepared_flow_invocation,
    })
}

pub fn prepared_view_id(viewport_id: ViewportId) -> String {
    format!("editor.viewport.{}.view", viewport_id.0)
}

fn target_alias_bindings(
    scene_color_target: RenderDynamicTextureTargetKey,
    picking_ids_target: RenderDynamicTextureTargetKey,
    overlay_target: RenderDynamicTextureTargetKey,
) -> BTreeMap<String, PreparedTargetBinding> {
    BTreeMap::from([
        (
            VIEWPORT_TARGET_ALIAS_SCENE_COLOR.to_string(),
            PreparedTargetBinding::DynamicTexture(scene_color_target),
        ),
        (
            VIEWPORT_TARGET_ALIAS_PICKING_IDS.to_string(),
            PreparedTargetBinding::DynamicTexture(picking_ids_target),
        ),
        (
            VIEWPORT_TARGET_ALIAS_OVERLAY.to_string(),
            PreparedTargetBinding::DynamicTexture(overlay_target),
        ),
    ])
}

fn editor_main_flow_ids(
    flow_registry: &RenderFlowRegistryResource,
) -> Option<(RenderFlowId, RenderResourceId)> {
    let flow = flow_registry
        .compiled_flows()
        .iter()
        .find(|flow| flow.flow_label == EDITOR_MAIN_FLOW_ID)?;
    let uniform_id = flow
        .resource_ids_by_label
        .get(EDITOR_VIEWPORT_SCENE_PRODUCT_UNIFORM_ID)
        .copied()?;
    Some((flow.flow_id, uniform_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::resources::EditorViewportRenderState;
    use crate::runtime::viewport::{
        ViewportProductRegistryResource, ViewportProductTargetRegistryResource,
        material_preview_descriptor,
    };
    use editor_core::RealityVersion;
    use editor_viewport::{
        ExpressionFormat, ExpressionFreshness, ExpressionPresentationHints,
        ExpressionProductDescriptor, ExpressionProductKind, ExpressionSourceRealityClass,
    };

    fn descriptor(
        id: editor_viewport::ExpressionProductId,
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
    fn prepared_view_ids_are_stable_per_viewport() {
        assert_eq!(prepared_view_id(ViewportId(5)), "editor.viewport.5.view");
    }

    #[test]
    fn render_jobs_require_all_locked_product_targets() {
        let viewport_id = ViewportId(9);
        let mut product_registry = ViewportProductRegistryResource::default();
        product_registry.update_viewport_descriptors(
            viewport_id,
            vec![
                descriptor(
                    SCENE_COLOR_PRODUCT_ID,
                    ExpressionProductKind::SceneColor2D,
                    ExpressionFormat::Rgba8Unorm,
                ),
                descriptor(
                    PICKING_IDS_PRODUCT_ID,
                    ExpressionProductKind::PickingIds2D,
                    ExpressionFormat::R32Uint,
                ),
                descriptor(
                    OVERLAY_PRODUCT_ID,
                    ExpressionProductKind::Overlay2D,
                    ExpressionFormat::Rgba8Unorm,
                ),
            ],
        );
        let target_registry = ViewportProductTargetRegistryResource::from_descriptors_for_viewport(
            viewport_id,
            product_registry
                .descriptors_for(viewport_id)
                .expect("descriptors should exist"),
        );
        let flow_id = RenderFlowId::try_from_raw(1).expect("test flow id should be valid");
        let scene_uniform_id =
            RenderResourceId::try_from_raw(9).expect("test uniform id should be valid");
        let viewport_render = EditorViewportRenderState::default();
        let job = build_viewport_render_job(
            flow_id,
            scene_uniform_id,
            &viewport_render,
            viewport_id,
            UiRect::new(0.0, 0.0, 320.0, 200.0),
            &target_registry,
        )
        .expect("job should exist when all product targets exist");

        assert_eq!(job.viewport_id, viewport_id);
        assert_eq!(job.dimensions, ExpressionDimensions::new(320, 200));
        let scene_target = target_registry
            .record_for_product(
                viewport_id,
                ViewportSurfacePresentationSlot::Primary,
                SCENE_COLOR_PRODUCT_ID,
            )
            .expect("scene target should exist")
            .dynamic_key();
        assert_eq!(job.scene_color_target, scene_target);
        assert_eq!(
            job.prepared_flow_invocation
                .target_alias_bindings
                .get(VIEWPORT_TARGET_ALIAS_SCENE_COLOR),
            Some(&PreparedTargetBinding::DynamicTexture(scene_target)),
        );
        assert_eq!(
            job.prepared_flow_invocation
                .uniform_overrides
                .get(&scene_uniform_id),
            Some(&viewport_render.compose_scene_product_uniform_bytes((320, 200))),
            "viewport render jobs must carry target-local scene uniforms in the prepared invocation"
        );
    }

    #[test]
    fn selected_material_preview_does_not_retarget_scene_render_alias() {
        let viewport_id = ViewportId(9);
        let material_product_id = editor_viewport::ExpressionProductId(42);
        let mut descriptors = vec![
            descriptor(
                SCENE_COLOR_PRODUCT_ID,
                ExpressionProductKind::SceneColor2D,
                ExpressionFormat::Rgba8Unorm,
            ),
            descriptor(
                PICKING_IDS_PRODUCT_ID,
                ExpressionProductKind::PickingIds2D,
                ExpressionFormat::R32Uint,
            ),
            descriptor(
                OVERLAY_PRODUCT_ID,
                ExpressionProductKind::Overlay2D,
                ExpressionFormat::Rgba8Unorm,
            ),
        ];
        descriptors.push(material_preview_descriptor(
            material_product_id,
            ExpressionDimensions::new(320, 200),
            RealityVersion(1),
            "material.first_slice.render_material".to_string(),
        ));
        let target_registry = ViewportProductTargetRegistryResource::from_descriptors_for_viewport(
            viewport_id,
            &descriptors,
        );
        let flow_id = RenderFlowId::try_from_raw(1).expect("test flow id should be valid");
        let scene_uniform_id =
            RenderResourceId::try_from_raw(9).expect("test uniform id should be valid");
        let viewport_render = EditorViewportRenderState::default();

        let job = build_viewport_render_job(
            flow_id,
            scene_uniform_id,
            &viewport_render,
            viewport_id,
            UiRect::new(0.0, 0.0, 320.0, 200.0),
            &target_registry,
        )
        .expect("job should exist when scene/support targets exist");

        let material_target = target_registry
            .record_for_product(
                viewport_id,
                ViewportSurfacePresentationSlot::Primary,
                material_product_id,
            )
            .expect("material target should exist")
            .dynamic_key();
        let scene_target = target_registry
            .record_for_product(
                viewport_id,
                ViewportSurfacePresentationSlot::Primary,
                SCENE_COLOR_PRODUCT_ID,
            )
            .expect("scene target should exist")
            .dynamic_key();
        assert_eq!(job.scene_color_target, scene_target);
        assert_ne!(job.scene_color_target, material_target);
        assert_eq!(
            job.prepared_flow_invocation
                .target_alias_bindings
                .get(VIEWPORT_TARGET_ALIAS_SCENE_COLOR),
            Some(&PreparedTargetBinding::DynamicTexture(scene_target)),
        );
    }
}
