use editor_core::RealityVersion;
use editor_shell::{
    PanelInstanceId, StructuralCommandTarget, StructuralWidgetRoutingContext, TabStackId,
    ToolSurfaceInstanceId, VIEWPORT_SURFACE_DEFINITION_ID, WidgetId, viewport_embed_slot_for,
};
use editor_viewport::{
    ExpressionDimensions, ExpressionProductDescriptor, ExpressionProductId, ExpressionProductKind,
    ExpressionSourceRealityClass, ViewportId, ViewportPresentationState,
    ViewportSurfacePresentationSlot,
};
use engine::plugins::render::{
    EditorPickingHit, EditorPickingTarget, RenderFlowRegistryResource, UiFrameProducerId,
    UiFrameSubmissionRegistryResource,
};
use runenwerk_editor::runtime::viewport::{
    EDITOR_MAIN_FLOW_ID, MAIN_VIEWPORT_ID, MountedSurfaceRegistryResource, PRODUCT_ID_PICKING_IDS,
    PRODUCT_ID_SCENE_COLOR, SurfaceDefinitionRegistryResource, ToolSurfaceRuntimeBindingRecord,
    ToolSurfaceRuntimeBindingRegistryResource, VIEWPORT_RESOURCE_OVERLAY,
    VIEWPORT_RESOURCE_PICKING_IDS, VIEWPORT_RESOURCE_SCENE_COLOR,
    ViewportArtifactObservationResource, ViewportLayoutEntry, ViewportLayoutMapResource,
    ViewportPickingResultsResource, ViewportPresentationStateResource,
    ViewportProductRegistryResource, ViewportSurfaceHandle, ViewportSurfaceSetResource,
    ViewportSurfaceSlot, build_surface_binding_registry, initial_presentation_state,
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
    let descriptors =
        initial_product_descriptors(ExpressionDimensions::new(1280, 720), RealityVersion(1));
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
fn runtime_viewport_resources_start_empty_before_bootstrap() {
    assert!(
        ViewportSurfaceSetResource::default()
            .viewport_ids()
            .next()
            .is_none()
    );
    assert!(ViewportProductRegistryResource::default().is_empty());
    assert!(ViewportPresentationStateResource::default().is_empty());
    assert!(ViewportArtifactObservationResource::default().is_empty());
    assert!(ViewportPickingResultsResource::default().is_empty());
}

#[test]
fn bootstrap_seeding_is_explicit_and_main_viewport_scoped() {
    let app = runenwerk_editor::runtime::build_headless_app()
        .run_for_frames(1)
        .expect("headless editor app should run");
    let surface_sets = app
        .world()
        .resource::<ViewportSurfaceSetResource>()
        .expect("surface set resource should exist");
    let presentations = app
        .world()
        .resource::<ViewportPresentationStateResource>()
        .expect("presentation state resource should exist");

    assert!(
        surface_sets.surface_set(MAIN_VIEWPORT_ID).is_some(),
        "startup seeding should register the main viewport surface set explicitly",
    );
    assert!(
        presentations.state_for(MAIN_VIEWPORT_ID).is_some(),
        "startup seeding should register explicit presentation state for the main viewport",
    );
}

#[test]
fn runtime_mounts_tool_surfaces_through_ui_surface_contracts() {
    let app = runenwerk_editor::runtime::build_headless_app()
        .run_for_frames(1)
        .expect("headless editor app should run");
    let definitions = app
        .world()
        .resource::<SurfaceDefinitionRegistryResource>()
        .expect("surface definition registry should exist");
    let mounted = app
        .world()
        .resource::<MountedSurfaceRegistryResource>()
        .expect("mounted surface registry should exist");

    assert!(
        definitions
            .definition(VIEWPORT_SURFACE_DEFINITION_ID)
            .is_some(),
        "viewport surface definition must be registered in ui_surface definition registry",
    );
    assert!(
        mounted.mounted_surfaces().next().is_some(),
        "at least one production tool surface should be mounted through ui_surface contracts",
    );
}

#[test]
fn derived_bindings_support_multiple_viewports_without_main_fallback() {
    let viewport_a = ViewportId(2);
    let viewport_b = ViewportId(3);

    let mut surface_sets = ViewportSurfaceSetResource::default();
    for viewport_id in [viewport_a, viewport_b] {
        surface_sets.set_surface(
            viewport_id,
            ViewportSurfaceSlot::PrimaryColor,
            ViewportSurfaceHandle::new(EDITOR_MAIN_FLOW_ID, VIEWPORT_RESOURCE_SCENE_COLOR),
        );
        surface_sets.set_surface(
            viewport_id,
            ViewportSurfaceSlot::PickingIds,
            ViewportSurfaceHandle::new(EDITOR_MAIN_FLOW_ID, VIEWPORT_RESOURCE_PICKING_IDS),
        );
        surface_sets.set_surface(
            viewport_id,
            ViewportSurfaceSlot::Overlay,
            ViewportSurfaceHandle::new(EDITOR_MAIN_FLOW_ID, VIEWPORT_RESOURCE_OVERLAY),
        );
    }

    let mut presentations = ViewportPresentationStateResource::default();
    presentations.upsert_state(initial_presentation_state(viewport_a));
    let mut state_b = initial_presentation_state(viewport_b);
    state_b.select_primary_product(PRODUCT_ID_PICKING_IDS);
    presentations.upsert_state(state_b);

    let registry = build_surface_binding_registry(&surface_sets, &presentations);
    let primary_a = registry
        .get(
            viewport_a.0,
            viewport_embed_slot_for(ViewportSurfacePresentationSlot::Primary),
        )
        .expect("viewport A should retain its primary binding");
    let primary_b = registry
        .get(
            viewport_b.0,
            viewport_embed_slot_for(ViewportSurfacePresentationSlot::Primary),
        )
        .expect("viewport B should retain its primary binding");

    assert_eq!(
        primary_a.resource_id.as_str(),
        VIEWPORT_RESOURCE_SCENE_COLOR
    );
    assert_eq!(
        primary_b.resource_id.as_str(),
        VIEWPORT_RESOURCE_PICKING_IDS
    );
    assert!(
        registry
            .get(
                MAIN_VIEWPORT_ID.0,
                viewport_embed_slot_for(ViewportSurfacePresentationSlot::Primary),
            )
            .is_none(),
        "derived registry must not synthesize an implicit main viewport fallback",
    );
}

#[test]
fn viewport_picking_results_do_not_overwrite_across_viewports() {
    let mut picking_results = ViewportPickingResultsResource::default();
    let viewport_a = ViewportId(11);
    let viewport_b = ViewportId(12);
    picking_results.set_viewport_result(
        viewport_a,
        (100.0, 120.0),
        (0.0, 0.0, 300.0, 200.0),
        EditorPickingHit {
            target: EditorPickingTarget::Entity(42),
            distance: 1.0,
        },
    );
    picking_results.set_viewport_result(
        viewport_b,
        (600.0, 180.0),
        (400.0, 0.0, 300.0, 200.0),
        EditorPickingHit {
            target: EditorPickingTarget::Grid,
            distance: 2.0,
        },
    );

    assert_eq!(
        picking_results
            .result_for(viewport_a)
            .map(|value| value.hit.target),
        Some(EditorPickingTarget::Entity(42)),
        "viewport A picking result should remain intact after viewport B update",
    );
    assert_eq!(
        picking_results
            .result_for(viewport_b)
            .map(|value| value.hit.target),
        Some(EditorPickingTarget::Grid),
        "viewport B picking result should be stored independently",
    );
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
        .filter(|resource| {
            matches!(
                resource,
                engine::plugins::render::RenderResourceDescriptor::ColorTarget(_)
            )
        })
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

#[test]
fn runtime_tool_surface_binding_tracks_rebind_without_mutating_structural_identity() {
    let tool_surface_id = ToolSurfaceInstanceId::new(91);
    let panel_instance_id = PanelInstanceId::new(41);
    let tab_stack_id = TabStackId::new(51);
    let mut layout = ViewportLayoutMapResource::default();
    layout.upsert_entry(ViewportLayoutEntry {
        viewport_id: ViewportId(1),
        host_widget_id: WidgetId(1001),
        structural_context: StructuralWidgetRoutingContext {
            panel_instance_id,
            active_tool_surface: Some(tool_surface_id),
            tab_stack_id,
        },
        bounds: ui_math::UiRect::new(0.0, 0.0, 640.0, 360.0),
    });

    let mut bindings = ToolSurfaceRuntimeBindingRegistryResource::default();
    bindings.rebuild_from_layout_map(&layout);

    layout.clear();
    layout.upsert_entry(ViewportLayoutEntry {
        viewport_id: ViewportId(2),
        host_widget_id: WidgetId(1002),
        structural_context: StructuralWidgetRoutingContext {
            panel_instance_id,
            active_tool_surface: Some(tool_surface_id),
            tab_stack_id,
        },
        bounds: ui_math::UiRect::new(0.0, 0.0, 640.0, 360.0),
    });
    bindings.rebuild_from_layout_map(&layout);

    let binding = bindings
        .binding_for_tool_surface(tool_surface_id)
        .expect("binding should exist after rebind");
    assert_eq!(binding.viewport_id, ViewportId(2));
    assert_eq!(binding.panel_instance_id, panel_instance_id);
    assert_eq!(binding.tab_stack_id, tab_stack_id);
    assert_eq!(
        bindings
            .latest_rebind_for_tool_surface(tool_surface_id)
            .expect("rebind should be tracked")
            .from_viewport_id,
        ViewportId(1),
    );
}

#[test]
fn runtime_binding_resolution_rejects_structural_mismatch_even_when_viewport_matches() {
    let mut bindings = ToolSurfaceRuntimeBindingRegistryResource::default();
    bindings.upsert_binding(ToolSurfaceRuntimeBindingRecord {
        tool_surface_id: ToolSurfaceInstanceId::new(7),
        panel_instance_id: PanelInstanceId::new(11),
        tab_stack_id: TabStackId::new(21),
        viewport_id: ViewportId(1),
        host_widget_id: WidgetId(301),
        bounds: ui_math::UiRect::new(0.0, 0.0, 320.0, 200.0),
        generation: 1,
    });

    let error = bindings
        .resolve_command_target(
            StructuralCommandTarget {
                panel_instance_id: PanelInstanceId::new(99),
                active_tool_surface: Some(ToolSurfaceInstanceId::new(7)),
                tab_stack_id: TabStackId::new(199),
            },
            ViewportId(1),
        )
        .expect_err("structural mismatch should fail closed");

    assert!(matches!(
        error,
        runenwerk_editor::runtime::viewport::ToolSurfaceRuntimeBindingResolveError::StructuralBindingMismatch { .. }
    ));
}

#[test]
fn runtime_systems_do_not_route_viewports_through_first_frame_fallback() {
    let frame_submit = include_str!("../src/runtime/systems/frame_submit.rs");
    let input_bridge = include_str!("../src/runtime/systems/input_bridge.rs");
    let product_registry = include_str!("../src/runtime/viewport/product_registry.rs");

    assert!(
        !frame_submit.contains("first_frame("),
        "frame_submit routing must not depend on first_frame fallback selection",
    );
    assert!(
        !input_bridge.contains("first_frame("),
        "input bridge routing must not depend on first_frame fallback selection",
    );
    assert!(
        !product_registry.contains("pub fn first_frame"),
        "viewport observation resource must not expose first_frame fallback helpers",
    );
}

#[test]
fn runtime_systems_share_single_viewport_bootstrap_routing_seam() {
    let frame_submit = include_str!("../src/runtime/systems/frame_submit.rs");
    let input_bridge = include_str!("../src/runtime/systems/input_bridge.rs");
    let routing = include_str!("../src/runtime/viewport/routing.rs");

    assert!(
        !frame_submit.contains("fn resolve_structural_viewport_products"),
        "frame_submit should consume shared viewport routing policy instead of declaring a local copy",
    );
    assert!(
        !input_bridge.contains("fn resolve_structural_viewport_products"),
        "input_bridge should consume shared viewport routing policy instead of declaring a local copy",
    );
    assert!(
        routing.contains("projection_artifacts_available"),
        "shared routing seam must explicitly gate bootstrap fallback on projection-artifact availability",
    );
}

#[test]
fn viewport_adapter_does_not_use_viewport_id_zero_fallback() {
    let viewport_adapter = include_str!("../src/shell/viewport_adapter.rs");
    assert!(
        !viewport_adapter.contains("ViewportId(0)"),
        "viewport adapter must not synthesize ViewportId(0) fallback identities",
    );
}

#[test]
fn viewport_product_selection_routes_through_surface_presentation_and_ratification() {
    let dispatch_shell_command = include_str!("../src/shell/dispatch_shell_command.rs");

    assert!(
        dispatch_shell_command.contains("SurfacePresentationModel"),
        "viewport product command path should build from a prepared surface presentation model",
    );
    assert!(
        dispatch_shell_command.contains("ratify_surface_intent"),
        "viewport product command path should route mutation through a ratification adapter",
    );
}

#[test]
fn outliner_selection_routes_through_surface_presentation_and_ratification() {
    let dispatch_shell_command = include_str!("../src/shell/dispatch_shell_command.rs");

    assert!(
        dispatch_shell_command.contains("build_outliner_surface_presentation_model"),
        "outliner selection should build from a prepared surface presentation model",
    );
    assert!(
        dispatch_shell_command.contains("SurfaceIntent::select_entity"),
        "outliner selection should route through typed surface intent emission",
    );
    assert!(
        dispatch_shell_command.contains("OutlinerEntitySelectionRatificationAdapter"),
        "outliner selection should ratify through adapter boundary, not direct mutation shortcut",
    );
}

#[test]
fn inspector_activation_routes_through_surface_presentation_and_ratification() {
    let dispatch_shell_command = include_str!("../src/shell/dispatch_shell_command.rs");

    assert!(
        dispatch_shell_command.contains("build_inspector_surface_presentation_model"),
        "inspector activation should build from a prepared surface presentation model",
    );
    assert!(
        dispatch_shell_command.contains("SurfaceIntent::activate_field"),
        "inspector activation should route through typed surface intent emission",
    );
    assert!(
        dispatch_shell_command.contains("InspectorFieldActivationRatificationAdapter"),
        "inspector activation should ratify through adapter boundary, not direct mutation shortcut",
    );
}

#[test]
fn viewport_slot_semantics_stay_in_editor_viewport_and_payload_slots_stay_opaque() {
    let viewport_surface_embed =
        include_str!("../../../domain/ui/ui_render_data/src/primitives/viewport_surface_embed.rs");
    let viewport_surface_semantics =
        include_str!("../../../domain/editor/editor_viewport/src/expression/surface_set.rs");

    assert!(
        viewport_surface_embed.contains("struct ViewportSurfaceEmbedSlotId"),
        "ui_render_data should expose an opaque renderer-facing embed slot id",
    );
    assert!(
        !viewport_surface_embed.contains("enum ViewportSurfaceSlot"),
        "ui_render_data must not own viewport semantic slot taxonomy",
    );
    assert!(
        viewport_surface_semantics.contains("enum ViewportSurfaceSlot"),
        "editor_viewport remains the semantic owner for viewport slot taxonomy",
    );
}

#[test]
fn viewport_slot_mapping_happens_at_integration_edge() {
    let presentation_resolver = include_str!("../src/runtime/viewport/presentation_resolver.rs");
    let viewport_embed_mapping =
        include_str!("../../../domain/editor/editor_shell/src/workspace/viewport_embed_slot.rs");

    assert!(
        viewport_embed_mapping.contains("viewport_embed_slot_for"),
        "editor_shell should own integration-edge mapping from semantic slots to payload slot IDs",
    );
    assert!(
        presentation_resolver.contains("viewport_embed_slot_for"),
        "runenwerk_editor viewport resolver should consume adapter mapping rather than defining parallel slot taxonomy",
    );
    assert!(
        !presentation_resolver.contains("UiViewportSurfaceSlot::"),
        "presentation resolver must not define a second canonical semantic slot enum in runenwerk_editor",
    );
}
