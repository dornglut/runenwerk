use editor_core::RealityVersion;
use editor_viewport::{
    ExpressionDimensions, ExpressionProductDescriptor, ExpressionProductId, ExpressionProductKind,
    ExpressionSourceRealityClass, ViewportPresentationState,
};
use engine::plugins::render::{RenderFlowRegistryResource, UiFrameProducerId, UiFrameSubmissionRegistryResource};
use runenwerk_editor::runtime::viewport::{
    EDITOR_MAIN_FLOW_ID, MAIN_VIEWPORT_ID, PRODUCT_ID_SCENE_COLOR,
    default_presentation_state,
    initial_product_descriptors,
};
use ui_render_data::UiPrimitive;

const EDITOR_SHELL_UI_PRODUCER_ID: UiFrameProducerId = UiFrameProducerId::new(1001);

#[test]
fn viewport_presentation_state_is_product_addressed() {
    let state = ViewportPresentationState::new(MAIN_VIEWPORT_ID, PRODUCT_ID_SCENE_COLOR);

    assert_eq!(state.viewport_id, MAIN_VIEWPORT_ID);
    assert_eq!(state.selected_primary_product_id, PRODUCT_ID_SCENE_COLOR);
}

#[test]
fn phase_one_product_kind_subset_is_locked() {
    let descriptors = initial_product_descriptors(ExpressionDimensions::new(1280, 720), RealityVersion(1));
    let kinds = descriptors
        .iter()
        .map(|descriptor| descriptor.kind)
        .collect::<Vec<_>>();

    assert!(kinds.contains(&ExpressionProductKind::SceneColor2D));
    assert!(kinds.contains(&ExpressionProductKind::PickingIds2D));
    assert!(kinds.contains(&ExpressionProductKind::Overlay2D));
}

#[test]
fn viewport_product_descriptor_requires_explicit_product_identity() {
    let descriptor = ExpressionProductDescriptor::new(
        ExpressionProductId(77),
        ExpressionProductKind::SceneColor2D,
        ExpressionDimensions::new(64, 64),
        editor_viewport::ExpressionFormat::Rgba8Unorm,
        "test.producer",
        ExpressionSourceRealityClass::ObservedScene,
        RealityVersion(2),
        editor_viewport::ExpressionFreshness::Current,
        editor_viewport::ExpressionPresentationHints::default(),
        None,
    );

    assert_eq!(descriptor.id, ExpressionProductId(77));
}

#[test]
fn default_presentation_state_is_stable_for_single_session_viewport() {
    let state = default_presentation_state();

    assert_eq!(state.viewport_id, MAIN_VIEWPORT_ID);
    assert_eq!(state.selected_primary_product_id, PRODUCT_ID_SCENE_COLOR);
}

#[test]
fn active_flow_excludes_legacy_fullscreen_mask_architecture() {
    let app = runenwerk_editor::runtime::build_headless_app()
        .run_for_frames(1)
        .expect("headless editor app should run");
    let flow_registry = app
        .world()
        .resource::<RenderFlowRegistryResource>()
        .expect("render flow registry should exist");

    let editor_flow = flow_registry
        .compiled_flows()
        .iter()
        .find(|flow| flow.flow_label == EDITOR_MAIN_FLOW_ID)
        .expect("editor main flow should exist");
    let pass_ids = editor_flow
        .pass_order
        .iter()
        .map(|pass| pass.pass_label().to_string())
        .collect::<Vec<_>>();

    assert!(
        !pass_ids
            .iter()
            .any(|id| id == "runenwerk.editor.viewport.sdf"),
        "legacy fullscreen viewport-mask pass must not remain active",
    );
    let color_target_count = editor_flow
        .resources
        .resources
        .iter()
        .filter(|resource| matches!(resource, engine::plugins::render::RenderResourceDescriptor::ColorTarget(_)))
        .count();
    assert!(
        color_target_count >= 3,
        "editor main flow must declare the three viewport product color targets",
    );
}

#[test]
fn shell_frame_uses_viewport_embed_primitive_instead_of_raw_image_path() {
    let app = runenwerk_editor::runtime::build_headless_app()
        .run_for_frames(1)
        .expect("headless editor app should run");
    let submissions = app
        .world()
        .resource::<UiFrameSubmissionRegistryResource>()
        .expect("ui submission registry should exist");
    let submission = submissions
        .get(&EDITOR_SHELL_UI_PRODUCER_ID)
        .expect("editor shell submission should exist");
    let has_embed = submission
        .frame
        .surfaces
        .iter()
        .flat_map(|surface| surface.layers.iter())
        .flat_map(|layer| layer.primitives.iter())
        .any(|primitive| matches!(primitive, UiPrimitive::ViewportSurfaceEmbed(_)));
    let has_image = submission
        .frame
        .surfaces
        .iter()
        .flat_map(|surface| surface.layers.iter())
        .flat_map(|layer| layer.primitives.iter())
        .any(|primitive| matches!(primitive, UiPrimitive::Image(_)));

    assert!(
        has_embed,
        "viewport panel must render through ViewportSurfaceEmbed primitive",
    );
    assert!(
        !has_image,
        "viewport panel must not use generic raw image texture path",
    );
}
