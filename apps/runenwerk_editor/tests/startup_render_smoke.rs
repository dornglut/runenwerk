use editor_shell::viewport_embed_slot_for;
use editor_viewport::{ViewportId, ViewportSurfacePresentationSlot};
use engine::plugins::render::backend::RenderSurfaceId;
use engine::plugins::render::{
    CompiledPassExecutionPlan, RenderFlowRegistryResource, UiFrameProducerId,
    UiFrameSubmissionRegistryResource, ViewportSurfaceBindingRegistryResource,
};
use runenwerk_editor::runtime::resources::{EditorHostResource, EditorViewportDebugStage};
use runenwerk_editor::runtime::viewport::{
    EDITOR_MAIN_FLOW_ID, EDITOR_VIEWPORT_SCENE_PRODUCT_UNIFORM_ID, SCENE_COLOR_PRODUCT_ID,
    VIEWPORT_DYNAMIC_TARGET_NAMESPACE, ViewportProductTargetRegistryResource,
    ViewportRenderJobResource, ViewportRenderStateResource,
};
use ui_render_data::{UiPrimitive, ViewportSurfaceBindingSource, ViewportSurfaceEmbedPrimitive};

const LEGACY_FULLSCREEN_MASK_PASS_ID: &str = "runenwerk.editor.viewport.sdf";
const SURFACE_CLEAR_PASS_ID: &str = "runenwerk.editor.surface.clear";
const SCENE_PRODUCT_PASS_ID: &str = "runenwerk.editor.viewport.product.scene";
const PICKING_PRODUCT_PASS_ID: &str = "runenwerk.editor.viewport.product.picking";
const OVERLAY_PRODUCT_PASS_ID: &str = "runenwerk.editor.viewport.product.overlay";
const VIEWPORT_BOUNDS_EPSILON: f32 = 0.75;
const EDITOR_SHELL_UI_PRODUCER_ID: UiFrameProducerId = ui_frame_producer_id(1001);
const SCENE_OVERLAY_UI_PRODUCER_ID: UiFrameProducerId = ui_frame_producer_id(1);

const fn ui_frame_producer_id(raw: u64) -> UiFrameProducerId {
    match UiFrameProducerId::try_from_raw(raw) {
        Ok(id) => id,
        Err(_) => panic!("ui frame producer id constants must be non-zero"),
    }
}

#[test]
fn startup_render_smoke_publishes_editor_shell_submission() {
    let app = runenwerk_editor::runtime::build_headless_app()
        .run_for_frames(2)
        .expect("headless editor app should run");

    let submissions = app
        .world()
        .resource::<UiFrameSubmissionRegistryResource>()
        .expect("ui submission registry should exist");
    let flow_registry = app
        .world()
        .resource::<RenderFlowRegistryResource>()
        .expect("render flow registry should exist");
    let viewport_bindings = app
        .world()
        .resource::<ViewportSurfaceBindingRegistryResource>()
        .expect("viewport surface binding registry should exist");
    let viewport_render_states = app
        .world()
        .resource::<ViewportRenderStateResource>()
        .expect("viewport render state registry should exist");
    let viewport_product_targets = app
        .world()
        .resource::<ViewportProductTargetRegistryResource>()
        .expect("viewport product target registry should exist");
    let viewport_render_jobs = app
        .world()
        .resource::<ViewportRenderJobResource>()
        .expect("viewport render job registry should exist");

    assert!(
        flow_registry.flow_count() > 0,
        "editor app should register at least one render flow",
    );
    let has_builtin_ui_pass = flow_registry
        .compiled_flows()
        .iter()
        .flat_map(|flow| flow.execution.passes.iter())
        .any(|pass| matches!(pass, CompiledPassExecutionPlan::BuiltinUiComposite(_)));
    assert!(
        has_builtin_ui_pass,
        "editor render flows should include a builtin UI composite pass",
    );

    let pass_ids = flow_registry
        .compiled_flows()
        .iter()
        .flat_map(|flow| {
            flow.pass_order
                .iter()
                .map(|pass| pass.pass_label().to_string())
        })
        .collect::<Vec<_>>();
    assert!(
        pass_ids.iter().any(|id| id == SURFACE_CLEAR_PASS_ID),
        "render flow should include surface clear pass",
    );
    assert!(
        pass_ids.iter().any(|id| id == SCENE_PRODUCT_PASS_ID),
        "render flow should include scene product pass",
    );
    assert!(
        pass_ids.iter().any(|id| id == PICKING_PRODUCT_PASS_ID),
        "render flow should include picking product pass",
    );
    assert!(
        pass_ids.iter().any(|id| id == OVERLAY_PRODUCT_PASS_ID),
        "render flow should include overlay product pass",
    );
    assert!(
        !pass_ids
            .iter()
            .any(|id| id == LEGACY_FULLSCREEN_MASK_PASS_ID),
        "legacy fullscreen-mask viewport pass must not be present in active render flow",
    );

    let editor_flow = flow_registry
        .compiled_flows()
        .iter()
        .find(|flow| flow.flow_label == EDITOR_MAIN_FLOW_ID)
        .expect("editor main flow should exist");
    let target_alias_count = editor_flow
        .resources
        .resources
        .iter()
        .filter(|resource| {
            matches!(
                resource,
                engine::plugins::render::RenderResourceDescriptor::TargetAlias(_)
            )
        })
        .count();
    assert!(
        target_alias_count >= 3,
        "editor flow resources should include the three viewport product target aliases",
    );

    let submission = submissions
        .get_for_surface(&EDITOR_SHELL_UI_PRODUCER_ID, RenderSurfaceId::primary())
        .expect("editor shell primary surface submission should exist");
    let scene_overlay_submission =
        submissions.get_for_surface(&SCENE_OVERLAY_UI_PRODUCER_ID, RenderSurfaceId::primary());

    assert!(
        !submission.frame.is_empty(),
        "editor shell frame should not be empty"
    );
    assert!(
        submission.primitive_count_hint() > 0,
        "editor shell frame should contain renderable primitives"
    );
    assert!(
        submission
            .frame
            .surfaces
            .iter()
            .flat_map(|surface| surface.layers.iter())
            .flat_map(|layer| layer.primitives.iter())
            .any(|primitive| matches!(primitive, UiPrimitive::ViewportSurfaceEmbed(_))),
        "editor shell submission must embed viewport surface through dedicated embed primitive",
    );
    assert!(
        scene_overlay_submission
            .map(|submission| submission.is_empty())
            .unwrap_or(true),
        "startup path should not include a non-empty scene.overlay submission that could overwrite viewport output",
    );

    let viewport_embed = submission
        .frame
        .surfaces
        .iter()
        .flat_map(|surface| surface.layers.iter())
        .flat_map(|layer| layer.primitives.iter())
        .find_map(|primitive| {
            let UiPrimitive::ViewportSurfaceEmbed(embed) = primitive else {
                return None;
            };
            if embed.slot == viewport_embed_slot_for(ViewportSurfacePresentationSlot::Primary) {
                Some(embed)
            } else {
                None
            }
        })
        .expect("viewport embed primitive for primary slot should exist");
    let primary_binding = viewport_bindings
        .registry()
        .get(
            viewport_embed.viewport_id,
            viewport_embed_slot_for(ViewportSurfacePresentationSlot::Primary),
        )
        .unwrap_or_else(|| {
            let available_bindings = viewport_bindings
                .registry()
                .iter()
                .map(|((viewport_id, slot), binding)| {
                    let ViewportSurfaceBindingSource::DynamicTexture {
                        namespace,
                        target_id,
                    } = &binding.source;
                    format!("{viewport_id}:{}={namespace}/{target_id}", slot.raw())
                })
                .collect::<Vec<_>>();
            panic!(
                "viewport primary surface binding should exist for viewport {} slot {}; available bindings: {:?}",
                viewport_embed.viewport_id,
                viewport_embed_slot_for(ViewportSurfacePresentationSlot::Primary).raw(),
                available_bindings
            );
        });
    let ViewportSurfaceBindingSource::DynamicTexture {
        namespace,
        target_id,
    } = &primary_binding.source;
    assert_eq!(namespace.as_str(), VIEWPORT_DYNAMIC_TARGET_NAMESPACE);
    assert_eq!(
        viewport_embed.uv_rect,
        ui_math::UiRect::new(0.0, 0.0, 1.0, 1.0),
        "viewport-local dynamic products must be sampled with full-product UVs, not a screen-subrect crop",
    );
    assert!(
        target_id.starts_with(format!("editor.viewport.{}.", viewport_embed.viewport_id).as_str()),
        "dynamic target id should be scoped to the owning viewport, got {target_id}",
    );
    assert!(
        viewport_product_targets
            .records()
            .any(|record| record.target_id == *target_id),
        "dynamic primary binding should be backed by a product target record",
    );
    assert!(
        viewport_render_jobs
            .job_for(editor_viewport::ViewportId(viewport_embed.viewport_id))
            .is_some(),
        "embedded viewport should have a prepared render job; jobs={:?} targets={:?}",
        viewport_render_jobs.viewport_ids().collect::<Vec<_>>(),
        viewport_product_targets
            .records()
            .map(|record| {
                format!(
                    "{}:{:?}:{:?}:{}",
                    record.key.viewport_id.0,
                    record.key.presentation_slot,
                    record.key.product_id,
                    record.target_id
                )
            })
            .collect::<Vec<_>>(),
    );
    let viewport_state = &viewport_render_states
        .state_for(editor_viewport::ViewportId(viewport_embed.viewport_id))
        .expect("embedded viewport should have viewport-owned render state")
        .render_state;

    assert!(
        (viewport_state.viewport_bounds_px.0 - viewport_embed.rect.x).abs()
            <= VIEWPORT_BOUNDS_EPSILON
            && (viewport_state.viewport_bounds_px.1 - viewport_embed.rect.y).abs()
                <= VIEWPORT_BOUNDS_EPSILON
            && (viewport_state.viewport_bounds_px.2 - viewport_embed.rect.width).abs()
                <= VIEWPORT_BOUNDS_EPSILON
            && (viewport_state.viewport_bounds_px.3 - viewport_embed.rect.height).abs()
                <= VIEWPORT_BOUNDS_EPSILON,
        "viewport render bounds must match shell embed rect; state={:?} embed=({:.2},{:.2},{:.2},{:.2})",
        viewport_state.viewport_bounds_px,
        viewport_embed.rect.x,
        viewport_embed.rect.y,
        viewport_embed.rect.width,
        viewport_embed.rect.height,
    );

    assert!(
        viewport_state.viewport_valid,
        "viewport render diagnostics should mark viewport as valid",
    );
    assert!(
        viewport_state.has_primitive,
        "viewport render diagnostics should include a primitive",
    );
    assert!(
        viewport_state.viewport_bounds_px.2 > f32::EPSILON
            && viewport_state.viewport_bounds_px.3 > f32::EPSILON,
        "viewport bounds should be non-zero",
    );
    assert_eq!(
        viewport_state.debug_stage,
        EditorViewportDebugStage::Scene,
        "headless startup should default to scene debug stage",
    );
    assert!(
        !viewport_state.root_background_opaque,
        "root background should default to non-occluding mode",
    );
}

#[test]
fn static_composition_keeps_viewport_product_sized_without_structural_mutation() {
    let mut app = runenwerk_editor::runtime::build_headless_app()
        .run_for_frames(1)
        .expect("headless editor app should run");
    let definition_before = {
        let host = app
            .world()
            .resource::<EditorHostResource>()
            .expect("editor host should exist");
        host.shell_state
            .composition_runtime()
            .composition()
            .definition()
            .clone()
    };

    app = app
        .run_for_frames(3)
        .expect("static composition app should run");
    assert_viewport_products_match_embeds(&app);
    let host = app
        .world()
        .resource::<EditorHostResource>()
        .expect("editor host should exist");
    assert_eq!(
        host.shell_state
            .composition_runtime()
            .composition()
            .definition(),
        &definition_before,
        "static frame submission must not mutate composition structure",
    );
}

fn primary_viewport_embeds(app: &engine::App) -> Vec<ViewportSurfaceEmbedPrimitive> {
    let submissions = app
        .world()
        .resource::<UiFrameSubmissionRegistryResource>()
        .expect("ui submission registry should exist");
    let submission = submissions
        .get_for_surface(&EDITOR_SHELL_UI_PRODUCER_ID, RenderSurfaceId::primary())
        .expect("editor shell primary surface submission should exist");
    submission
        .frame
        .surfaces
        .iter()
        .flat_map(|surface| surface.layers.iter())
        .flat_map(|layer| layer.primitives.iter())
        .filter_map(|primitive| {
            let UiPrimitive::ViewportSurfaceEmbed(embed) = primitive else {
                return None;
            };
            (embed.slot == viewport_embed_slot_for(ViewportSurfacePresentationSlot::Primary))
                .then(|| embed.clone())
        })
        .collect()
}

fn assert_viewport_products_match_embeds(app: &engine::App) {
    let embeds = primary_viewport_embeds(app);
    assert!(
        !embeds.is_empty(),
        "composition should render a viewport embed"
    );

    let viewport_render_states = app
        .world()
        .resource::<ViewportRenderStateResource>()
        .expect("viewport render state registry should exist");
    let viewport_product_targets = app
        .world()
        .resource::<ViewportProductTargetRegistryResource>()
        .expect("viewport product target registry should exist");
    let viewport_render_jobs = app
        .world()
        .resource::<ViewportRenderJobResource>()
        .expect("viewport render job registry should exist");
    let flow_registry = app
        .world()
        .resource::<RenderFlowRegistryResource>()
        .expect("render flow registry should exist");
    let scene_uniform_id = flow_registry
        .compiled_flows()
        .iter()
        .find(|flow| flow.flow_label == EDITOR_MAIN_FLOW_ID)
        .and_then(|flow| {
            flow.resource_ids_by_label
                .get(EDITOR_VIEWPORT_SCENE_PRODUCT_UNIFORM_ID)
        })
        .copied()
        .expect("editor flow should expose the scene product uniform id");
    for embed in embeds {
        let viewport_id = ViewportId(embed.viewport_id);
        let state = viewport_render_states
            .state_for(viewport_id)
            .expect("embedded viewport should have per-viewport render state");
        assert!(
            (state.bounds.x - embed.rect.x).abs() <= VIEWPORT_BOUNDS_EPSILON
                && (state.bounds.y - embed.rect.y).abs() <= VIEWPORT_BOUNDS_EPSILON
                && (state.bounds.width - embed.rect.width).abs() <= VIEWPORT_BOUNDS_EPSILON
                && (state.bounds.height - embed.rect.height).abs() <= VIEWPORT_BOUNDS_EPSILON,
            "viewport state bounds must track the matching embed rect; viewport={} state={:?} embed={:?}",
            viewport_id.0,
            state.bounds,
            embed.rect,
        );
        let expected_size = (
            embed.rect.width.max(1.0).round() as u32,
            embed.rect.height.max(1.0).round() as u32,
        );
        let job = viewport_render_jobs
            .job_for(viewport_id)
            .expect("embedded viewport should have a render job");
        assert_eq!(
            (job.dimensions.width, job.dimensions.height),
            expected_size,
            "render job dimensions must follow viewport-local embed bounds"
        );
        let target = viewport_product_targets
            .record_for_product(
                viewport_id,
                ViewportSurfacePresentationSlot::Primary,
                SCENE_COLOR_PRODUCT_ID,
            )
            .expect("embedded viewport should have a scene-color target");
        assert_eq!(
            (target.width, target.height),
            expected_size,
            "scene-color dynamic target dimensions must follow viewport-local embed bounds"
        );
        assert_eq!(
            job.prepared_flow_invocation()
                .uniform_overrides
                .get(&scene_uniform_id),
            Some(
                &state
                    .render_state
                    .compose_scene_product_uniform_bytes(expected_size)
            ),
            "prepared viewport invocation must carry target-local scene uniforms for viewport={}",
            viewport_id.0,
        );
    }
}
