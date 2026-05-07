use std::fs;
use std::path::Path;

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
    EDITOR_MAIN_FLOW_ID, MAIN_VIEWPORT_ID, MountedSurfaceRegistryResource,
    SurfaceDefinitionRegistryResource, ToolSurfaceRuntimeBindingRecord,
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
    ViewportLayoutEntry, ViewportLayoutMapResource, ViewportPickingResultsResource,
    ViewportPresentationStateResource, ViewportProductRegistryResource, ViewportSurfaceHandle,
    ViewportSurfaceSetResource, ViewportSurfaceSlot, build_surface_binding_registry,
    initial_product_descriptors,
};
use ui_render_data::{UiPrimitive, ViewportSurfaceBindingSource};

const EDITOR_SHELL_UI_PRODUCER_ID: UiFrameProducerId = ui_frame_producer_id(1001);
const TEST_RESOURCE_SCENE_COLOR: &str = "test.viewport.scene_color";
const TEST_RESOURCE_SCENE_COLOR_B: &str = "test.viewport.scene_color.b";

const fn ui_frame_producer_id(raw: u64) -> UiFrameProducerId {
    match UiFrameProducerId::try_from_raw(raw) {
        Ok(id) => id,
        Err(_) => panic!("ui frame producer id constants must be non-zero"),
    }
}

#[test]
fn viewport_presentation_state_is_product_addressed() {
    let viewport_id = ViewportId(77);
    let product_id = ExpressionProductId(177);
    let state = ViewportPresentationState::new(viewport_id, product_id);

    assert_eq!(state.viewport_id, viewport_id);
    assert_eq!(state.selected_primary_product_id, product_id);
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
    surface_sets.set_surface(
        viewport_a,
        ViewportSurfaceSlot::PrimaryColor,
        ViewportSurfaceHandle::dynamic_texture("test.viewport", TEST_RESOURCE_SCENE_COLOR, true),
    );
    surface_sets.set_surface(
        viewport_b,
        ViewportSurfaceSlot::PrimaryColor,
        ViewportSurfaceHandle::dynamic_texture("test.viewport", TEST_RESOURCE_SCENE_COLOR_B, true),
    );

    let mut presentations = ViewportPresentationStateResource::default();
    presentations.upsert_state(ViewportPresentationState::new(
        viewport_a,
        ExpressionProductId(1),
    ));
    presentations.upsert_state(ViewportPresentationState::new(
        viewport_b,
        ExpressionProductId(1),
    ));

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

    assert!(matches!(
        &primary_a.source,
        ViewportSurfaceBindingSource::DynamicTexture { target_id, .. }
            if target_id == TEST_RESOURCE_SCENE_COLOR
    ));
    assert!(matches!(
        &primary_b.source,
        ViewportSurfaceBindingSource::DynamicTexture { target_id, .. }
            if target_id == TEST_RESOURCE_SCENE_COLOR_B
    ));
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
    let color_target_alias_count = editor_flow
        .resources
        .resources
        .iter()
        .filter(|resource| {
            matches!(
                resource,
                engine::plugins::render::RenderResourceDescriptor::TargetAlias(alias)
                    if alias.kind == engine::plugins::render::RenderTargetAliasKind::Color
            )
        })
        .count();
    assert!(
        color_target_alias_count >= 3,
        "editor main flow must declare the three viewport product color target aliases",
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
fn final_shell_frame_body_path_does_not_fan_out_by_panel_kind() {
    let shell_builder =
        include_str!("../../../domain/editor/editor_shell/src/composition/build_editor_shell.rs");
    let body_start = shell_builder
        .find("fn build_tab_stack_host_from_frame")
        .expect("final frame tab-stack host function should exist");
    let body_end = shell_builder[body_start..]
        .find("fn build_tab_strip")
        .map(|offset| body_start + offset)
        .expect("next function after final frame tab-stack host should exist");
    let final_body_path = &shell_builder[body_start..body_end];

    assert!(
        final_body_path.contains("frame_model.surface(surface_id)"),
        "final shell body path must resolve panel content through mounted ToolSurfaceInstanceId",
    );
    assert!(
        !final_body_path.contains("panel.panel_kind"),
        "final shell body path must not switch on legacy PanelKind",
    );
    for concrete_builder in [
        "build_outliner_panel(",
        "build_entity_table_panel(",
        "build_viewport_panel(",
        "build_inspector_panel(",
        "build_console_panel(",
    ] {
        assert!(
            !final_body_path.contains(concrete_builder),
            "final shell body path must not directly fan out to {concrete_builder}",
        );
    }
}

#[test]
fn production_shell_paths_use_app_owned_registry_and_surface_sessions() {
    let controller = include_str!("../src/shell/controller.rs");
    let dispatcher = include_str!("../src/shell/dispatch_shell_command.rs");
    let app_state = include_str!("../src/editor_app/state.rs");
    let shell_mod = include_str!("../src/shell/mod.rs");
    let providers = provider_sources();

    assert!(
        !controller.contains("EditorSurfaceProviderRegistry::runenwerk_default()"),
        "production controller paths must use the app/plugin-host-owned provider registry",
    );
    for forbidden in [
        concat!("EditorShell", "ViewModel"),
        concat!("build_editor_shell", "_view_model"),
        concat!("rebuild", "_view_model"),
    ] {
        assert!(
            !controller.contains(forbidden)
                && !dispatcher.contains(forbidden)
                && !shell_mod.contains(forbidden),
            "production shell modules must not expose or call old aggregate API {forbidden}",
        );
    }
    for forbidden in [
        "inspector_ui_state:",
        "entity_table_ui_state:",
        "viewport_interaction_state:",
        "console_follow_enabled:",
        "viewport_details_visible:",
        "fn inspector_ui_state",
        "fn entity_table_ui_state",
        "fn viewport_interaction_state",
        "fn set_console_follow_enabled",
        "fn set_viewport_details_visible",
        "fn toggle_viewport_details_visible",
    ] {
        assert!(
            !app_state.contains(forbidden),
            "RunenwerkEditorApp must not expose app-global provider-local state through {forbidden}",
        );
    }
    assert!(
        !providers.contains(concat!("SurfaceCommandProposal::", "ShellCommand"))
            && !providers.contains(concat!("SurfaceCommandProposal::", "shell")),
        "providers must return typed SurfaceCommandProposal variants, not boxed shell commands",
    );
    assert!(
        !providers.contains(concat!("SurfaceCommandProposal::", "NoOp")),
        "providers must represent no proposal as None, not a provider-side NoOp proposal",
    );
    for deleted_adapter_module in [
        "console_adapter",
        "entity_table_adapter",
        "inspector_adapter",
        "outliner_adapter",
        "viewport_adapter",
    ] {
        assert!(
            !shell_mod.contains(deleted_adapter_module)
                && !providers.contains(deleted_adapter_module),
            "old shell adapter module {deleted_adapter_module} must not remain wired into the final provider path",
        );
    }
    for forbidden in [
        "inspector_ui_state_mut",
        "entity_table_ui_state_mut",
        "viewport_interaction_state_mut",
        "set_console_follow_enabled",
        "set_viewport_details_visible",
        "toggle_viewport_details_visible",
    ] {
        assert!(
            !controller.contains(forbidden) && !dispatcher.contains(forbidden),
            "final shell controller/dispatcher paths must not mutate app-global provider-local state through {forbidden}",
        );
    }
}

#[test]
fn production_sources_do_not_keep_deleted_shell_adapter_modules() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for deleted_path in [
        "src/shell/console_adapter.rs",
        "src/shell/entity_table_adapter.rs",
        "src/shell/inspector_adapter.rs",
        "src/shell/outliner_adapter.rs",
        "src/shell/viewport_adapter.rs",
    ] {
        assert!(
            !manifest_dir.join(deleted_path).exists(),
            "obsolete shell adapter file must be deleted: {deleted_path}",
        );
    }
}

#[test]
fn provider_seam_has_no_legacy_aggregate_or_boxed_shell_proposal_escape_hatch() {
    let surface_provider =
        include_str!("../../../domain/editor/editor_shell/src/surface_provider.rs");
    let shell_command =
        include_str!("../../../domain/editor/editor_shell/src/commands/shell_command.rs");
    let providers = provider_sources();
    let controller = include_str!("../src/shell/controller.rs");

    for source in [
        surface_provider.to_string(),
        shell_command.to_string(),
        providers,
        controller.to_string(),
    ] {
        for forbidden in [
            concat!("EditorShell", "ViewModel"),
            concat!("build_editor_shell", "_view_model"),
            concat!("rebuild", "_view_model"),
            concat!("ShellCommand", "(Box<ShellCommand>)"),
            concat!("SurfaceCommandProposal::", "NoOp"),
        ] {
            assert!(
                !source.contains(forbidden),
                "final provider/shell source must not contain obsolete path marker {forbidden}",
            );
        }
    }
}

#[test]
fn production_sources_do_not_implement_deferred_ui_execution_strategies() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("runenwerk_editor lives under apps/");
    let production_roots = [
        workspace_root.join("domain/ui"),
        workspace_root.join("domain/editor/editor_shell/src"),
        manifest_dir.join("src"),
    ];
    let mut offenders = Vec::new();
    for root in &production_roots {
        collect_source_files(root, &mut offenders);
    }

    let forbidden_terms = [
        concat!("compiled ", "reactive"),
        concat!("Sv", "elte"),
        concat!("ECS-driven ", "UI"),
        concat!("multiple execution ", "strategies"),
    ];
    let offenders = offenders
        .into_iter()
        .filter_map(|path| {
            let contents = fs::read_to_string(&path).ok()?;
            forbidden_terms
                .iter()
                .any(|term| contents.contains(term))
                .then_some(path)
        })
        .collect::<Vec<_>>();

    assert!(
        offenders.is_empty(),
        "deferred UI execution strategies must not be implemented in production sources: {offenders:?}",
    );
}

#[test]
fn production_input_bridge_routes_viewport_interaction_by_tool_surface_session() {
    let input_bridge = include_str!("../src/runtime/systems/input_bridge.rs");

    for forbidden in [
        "viewport_interaction_state()",
        "viewport_interaction_state_mut()",
        concat!("dispatch_viewport_interaction", "_command("),
    ] {
        assert!(
            !input_bridge.contains(forbidden),
            "production input bridge must not use app-global viewport interaction state through {forbidden}",
        );
    }
    assert!(
        input_bridge.contains("tool_surface_id"),
        "viewport input route must carry ToolSurfaceInstanceId",
    );
    assert!(
        input_bridge.contains("dispatch_viewport_interaction_for_surface"),
        "viewport input must dispatch interaction commands to a targeted surface session",
    );
    assert!(
        input_bridge.contains("active_viewport_drag_surface"),
        "viewport drag continuation must resolve captured surface session state",
    );
    assert!(
        input_bridge.contains("viewport_scene_binding_for_widget"),
        "viewport input must route through an explicit scene-region binding helper",
    );
    assert!(
        !input_bridge.contains("resolve_structural_context(structural_context)\n        && binding.bounds.contains(position)"),
        "viewport input must not treat arbitrary viewport-surface chrome as scene interaction",
    );
}

#[test]
fn production_picking_routes_only_through_viewport_scene_region() {
    let picking = include_str!("../src/runtime/systems/picking.rs");

    assert!(
        picking.contains("viewport_scene_binding_for_widget"),
        "picking must distinguish viewport scene region widgets from provider chrome",
    );
    assert!(
        picking.contains("binding_containing_cursor(cursor)"),
        "picking fallback must use explicit viewport scene-region bounds",
    );
    assert!(
        !picking.contains("runtime_state.hovered_widget.and_then"),
        "picking must not resolve arbitrary hovered viewport-surface widgets as scene input",
    );
}

#[test]
fn editor_frame_submission_runs_after_input_bridge() {
    let plugin = include_str!("../src/runtime/plugin.rs");

    assert!(
        plugin.contains("submit_editor_frame_system")
            && plugin.contains(".after(EditorRuntimeSet::InputBridge)"),
        "editor frame submission must consume the same-frame split/input layout mutations before product targets are prepared",
    );
}

#[test]
fn viewport_details_dispatch_has_no_first_active_viewport_fallback() {
    let dispatcher = include_str!("../src/shell/dispatch_shell_command.rs");
    let providers = provider_sources();
    let shell_command =
        include_str!("../../../domain/editor/editor_shell/src/commands/shell_command.rs");

    assert!(
        !dispatcher.contains("active_surface_by_kind"),
        "viewport details dispatch must not pick the first active viewport surface",
    );
    assert!(
        !dispatcher.contains("ToolSurfaceKind::Viewport)\n            .map"),
        "viewport details dispatch must not map from viewport kind to an arbitrary active surface",
    );
    assert!(
        shell_command.contains("ToggleViewportDetails {\n        target: StructuralCommandTarget"),
        "viewport details command must carry structural ToolSurfaceInstanceId context",
    );
    assert!(
        dispatcher.contains("target.active_tool_surface"),
        "viewport details mutation must use the routed ToolSurfaceInstanceId",
    );
    assert!(
        dispatcher.contains("session.viewport_details_visible = !session.viewport_details_visible"),
        "viewport details mutation must update the routed surface session",
    );
    assert!(
        providers.contains("SurfaceLocalAction::ToggleViewportDetails"),
        "viewport provider must emit a provider-local details-toggle action",
    );
    assert!(
        providers.contains("VIEWPORT_DETAILS_TOGGLE_WIDGET_ID")
            && !providers.contains("VIEWPORT_PANEL_WIDGET_ID,\n            SurfaceLocalRoute::new(SurfaceLocalAction::ToggleViewportDetails)"),
        "viewport details route must be attached to the dedicated details-toggle widget id",
    );
}

#[test]
fn runtime_tool_surface_binding_tracks_rebind_without_mutating_structural_identity() {
    let tool_surface_id = ToolSurfaceInstanceId::try_from_raw(91).unwrap();
    let panel_instance_id = PanelInstanceId::try_from_raw(41).unwrap();
    let tab_stack_id = TabStackId::try_from_raw(51).unwrap();
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
        tool_surface_id: ToolSurfaceInstanceId::try_from_raw(7).unwrap(),
        panel_instance_id: PanelInstanceId::try_from_raw(11).unwrap(),
        tab_stack_id: TabStackId::try_from_raw(21).unwrap(),
        viewport_id: ViewportId(1),
        host_widget_id: WidgetId(301),
        bounds: ui_math::UiRect::new(0.0, 0.0, 320.0, 200.0),
        generation: 1,
    });

    let error = bindings
        .resolve_command_target(
            StructuralCommandTarget {
                panel_instance_id: PanelInstanceId::try_from_raw(99).unwrap(),
                active_tool_surface: Some(ToolSurfaceInstanceId::try_from_raw(7).unwrap()),
                tab_stack_id: TabStackId::try_from_raw(199).unwrap(),
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
fn viewport_provider_does_not_use_viewport_id_zero_fallback() {
    let viewport_provider = provider_sources();
    assert!(
        !viewport_provider.contains("ViewportId(0)"),
        "viewport provider must not synthesize ViewportId(0) fallback identities",
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
fn rb0_rb7_runtime_sources_do_not_depend_on_static_shared_viewport_product_ids() {
    let sources = read_workspace_sources(&[
        "apps/runenwerk_editor/src/runtime/app.rs",
        "apps/runenwerk_editor/src/runtime/systems/bootstrap.rs",
        "apps/runenwerk_editor/src/runtime/systems/frame_submit.rs",
        "apps/runenwerk_editor/src/runtime/viewport/product_registry.rs",
        "apps/runenwerk_editor/src/runtime/viewport/producer_scene.rs",
        "apps/runenwerk_editor/src/runtime/viewport/surface_set.rs",
        "apps/runenwerk_editor/src/runtime/viewport/presentation_resolver.rs",
    ]);
    let forbidden_terms = [
        "PRODUCT_ID_SCENE_COLOR",
        "PRODUCT_ID_PICKING_IDS",
        "PRODUCT_ID_OVERLAY",
        "VIEWPORT_RESOURCE_SCENE_COLOR",
        "VIEWPORT_RESOURCE_PICKING_IDS",
        "VIEWPORT_RESOURCE_OVERLAY",
        "\"editor.viewport.v1.scene_color\"",
        "\"editor.viewport.v1.picking_ids\"",
        "\"editor.viewport.v1.overlay\"",
    ];
    let offenders = forbidden_source_markers(&sources, &forbidden_terms);

    assert!(
        offenders.is_empty(),
        "RB0/RB7 viewport products must be viewport-scoped dynamic targets, not shared static ids: {offenders:?}",
    );
}

#[test]
fn rb0_runtime_sources_do_not_build_render_flows_per_viewport() {
    let app_source = read_workspace_source("apps/runenwerk_editor/src/runtime/app.rs");
    for forbidden in [
        "RenderFlow::new(format!",
        "RenderFlow::new(&format!",
        "format!(\"runenwerk.editor.viewport",
        "format!(\"runenwerk.editor.{}",
    ] {
        assert!(
            !app_source.contains(forbidden),
            "editor app render setup must not synthesize one RenderFlow per viewport through marker '{forbidden}'",
        );
    }

    let viewport_sources = read_workspace_source_tree("apps/runenwerk_editor/src/runtime/viewport");
    let offenders = forbidden_source_markers(
        &viewport_sources,
        &[
            "RenderFlow::new(",
            "add_render_flow(",
            "CompiledRenderFlowPlan",
        ],
    );
    assert!(
        offenders.is_empty(),
        "viewport runtime modules must not work around product surfaces by owning render-flow construction: {offenders:?}",
    );
}

#[test]
fn rb7_viewport_scene_product_shader_has_no_multi_rect_containment() {
    let sources = read_workspace_sources(&[
        "apps/runenwerk_editor/src/runtime/resources.rs",
        "assets/shaders/editor_viewport_scene_product.wgsl",
    ]);
    let forbidden_terms = [
        "viewport_b :",
        "viewport_c :",
        "viewport_d :",
        "pub viewport_b:",
        "pub viewport_c:",
        "pub viewport_d:",
        "u.viewport_b",
        "u.viewport_c",
        "u.viewport_d",
        "additional_viewport_bounds_px",
        "set_viewport_bounds_list",
        "viewport_contains_rect",
    ];
    let offenders = forbidden_source_markers(&sources, &forbidden_terms);

    assert!(
        offenders.is_empty(),
        "RB7 viewport scene products must render one viewport-local job instead of shader-side multi-rect containment: {offenders:?}",
    );
}

#[test]
fn rb7_editor_frame_submit_does_not_seed_or_extract_viewport_product_lifecycle() {
    let frame_submit =
        read_workspace_source("apps/runenwerk_editor/src/runtime/systems/frame_submit.rs");
    for forbidden in [
        "seed_viewport_binding_for_active_workspace",
        "sync_viewport_presentation_products_system",
        "build_surface_binding_registry",
        "viewport_products_registry.update_viewport_descriptors",
        "viewport_surface_bindings.replace_registry",
        "ensure_editor_main_surface_set",
        "initial_product_descriptors(",
        "viewport_surface_sets.retain_viewports",
        "viewport_presentations.retain_viewports",
        "ViewportSurfaceBindingRegistryResource",
        "ViewportProductRegistryResource",
    ] {
        assert!(
            !frame_submit.contains(forbidden),
            "editor frame submit must not own viewport identity/product binding lifecycle through marker '{forbidden}'",
        );
    }
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
fn viewport_embed_shader_uses_full_prepared_product_rect() {
    let shader = read_workspace_source("engine/src/plugins/render/renderer/mod.rs");

    assert!(
        shader.contains("input.uv_rect.x + input.uv_rect.z * input.local.x")
            && shader.contains("input.uv_rect.y + input.uv_rect.w * input.local.y"),
        "viewport embed shader must sample the full prepared product rect; product sizing owns aspect correctness",
    );
    assert!(
        !shader.contains("textureDimensions(viewport_texture)")
            && !shader.contains("fit_origin")
            && !shader.contains("fit_size"),
        "viewport embed shader must not apply a second implicit fit that diverges from picking bounds",
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

fn workspace_root() -> std::path::PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("runenwerk_editor lives under apps/")
        .to_path_buf()
}

fn read_workspace_source(path: &str) -> String {
    let full_path = workspace_root().join(path);
    fs::read_to_string(&full_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", full_path.display()))
}

fn read_workspace_sources(paths: &[&'static str]) -> Vec<(String, String)> {
    paths
        .iter()
        .map(|path| ((*path).to_string(), read_workspace_source(path)))
        .collect()
}

fn read_workspace_source_tree(path: &str) -> Vec<(String, String)> {
    let root = workspace_root();
    let mut files = Vec::new();
    collect_source_files(&root.join(path), &mut files);
    files.sort();
    files
        .into_iter()
        .filter_map(|file| {
            let display_path = file
                .strip_prefix(&root)
                .unwrap_or(&file)
                .display()
                .to_string();
            fs::read_to_string(&file)
                .ok()
                .map(|source| (display_path, source))
        })
        .collect()
}

fn forbidden_source_markers(sources: &[(String, String)], forbidden_terms: &[&str]) -> Vec<String> {
    sources
        .iter()
        .flat_map(|(file, source)| {
            forbidden_terms
                .iter()
                .filter(move |term| source.contains(**term))
                .map(move |term| format!("{file}: {term}"))
        })
        .collect()
}

fn collect_source_files(root: &Path, files: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_source_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn provider_sources() -> String {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let provider_root = manifest_dir.join("src/shell/providers");
    let mut files = Vec::new();
    collect_source_files(&provider_root, &mut files);
    files.sort();
    files
        .into_iter()
        .filter_map(|path| fs::read_to_string(path).ok())
        .collect::<Vec<_>>()
        .join("\n")
}
