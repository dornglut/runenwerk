use editor_shell::viewport_embed_slot_for;
use editor_viewport::ViewportSurfacePresentationSlot;
use engine::plugins::render::{
    CompiledPassExecutionPlan, RenderFlowRegistryResource, UiFrameProducerId,
    UiFrameSubmissionRegistryResource, ViewportSurfaceBindingRegistryResource,
};
use runenwerk_editor::runtime::resources::{EditorViewportDebugStage, EditorViewportRenderState};
use runenwerk_editor::runtime::viewport::{
    EDITOR_MAIN_FLOW_ID, VIEWPORT_RESOURCE_SCENE_COLOR, ViewportRenderStateResource,
};
use ui_render_data::UiPrimitive;

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
    let viewport_state = app
        .world()
        .resource::<EditorViewportRenderState>()
        .expect("viewport render state should exist");
    let viewport_render_states = app
        .world()
        .resource::<ViewportRenderStateResource>()
        .expect("viewport render state registry should exist");

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
    let color_target_count = editor_flow
        .resources
        .resources
        .iter()
        .filter(|resource| {
            matches!(
                resource,
                engine::plugins::render::RenderResourceDescriptor::ColorTarget(_)
            )
        })
        .count();
    assert!(
        color_target_count >= 3,
        "editor flow resources should include the three viewport product color targets",
    );

    let submission = submissions
        .get(&EDITOR_SHELL_UI_PRODUCER_ID)
        .expect("editor shell submission should exist");
    let scene_overlay_submission = submissions.get(&SCENE_OVERLAY_UI_PRODUCER_ID);

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
        .expect("viewport primary surface binding should exist");
    assert_eq!(primary_binding.flow_id.as_str(), EDITOR_MAIN_FLOW_ID);
    assert_eq!(
        primary_binding.resource_id.as_str(),
        VIEWPORT_RESOURCE_SCENE_COLOR,
    );
    assert!(
        viewport_render_states
            .state_for(editor_viewport::ViewportId(viewport_embed.viewport_id))
            .is_some(),
        "embedded viewport should have viewport-owned render state",
    );

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
