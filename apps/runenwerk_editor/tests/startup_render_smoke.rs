use engine::plugins::render::{
    CompiledPassExecutionPlan, RenderFlowRegistryResource, RenderShaderReference,
    ShaderRegistryResource, UiFrameProducerId, UiFrameSubmissionRegistryResource,
};
use runenwerk_editor::editor_runtime::EditorPrimitiveKind;
use runenwerk_editor::runtime::app::EDITOR_VIEWPORT_SDF_SHADER_ID;
use runenwerk_editor::runtime::resources::{EditorViewportDebugStage, EditorViewportRenderState};

const EDITOR_VIEWPORT_SDF_PASS_ID: &str = "runenwerk.editor.viewport.sdf";

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
    let shader_registry = app
        .world()
        .resource::<ShaderRegistryResource>()
        .expect("shader registry should exist");
    let viewport_state = app
        .world()
        .resource::<EditorViewportRenderState>()
        .expect("viewport render state should exist");

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
    let viewport_pass_shader_ref = flow_registry
        .compiled_flows()
        .iter()
        .flat_map(|flow| flow.execution.passes.iter())
        .find_map(|pass| match pass {
            CompiledPassExecutionPlan::Fullscreen(plan)
                if plan.pass_id == EDITOR_VIEWPORT_SDF_PASS_ID =>
            {
                plan.shader.as_ref().and_then(|shader| match shader {
                    RenderShaderReference::AssetPath(path) => Some(path.as_str()),
                    RenderShaderReference::RegistryHandle(_) => None,
                })
            }
            _ => None,
        });
    assert!(
        viewport_pass_shader_ref.is_some(),
        "editor render flows should include the viewport SDF pass",
    );
    assert_eq!(
        viewport_pass_shader_ref,
        Some(EDITOR_VIEWPORT_SDF_SHADER_ID),
        "viewport SDF pass should reference the shader id used for stable runtime lookup",
    );

    let submission = submissions
        .get(&UiFrameProducerId::new("editor.shell"))
        .expect("editor shell submission should exist");
    let scene_overlay_submission = submissions.get(&UiFrameProducerId::new("scene.overlay"));

    assert!(
        !submission.frame.is_empty(),
        "editor shell frame should not be empty"
    );
    assert!(
        submission.primitive_count_hint() > 0,
        "editor shell frame should contain renderable primitives"
    );
    assert!(
        scene_overlay_submission
            .map(|submission| submission.is_empty())
            .unwrap_or(true),
        "startup path should not include a non-empty scene.overlay submission that could overwrite viewport output",
    );
    let shader_revision = shader_registry.revision_for(EDITOR_VIEWPORT_SDF_SHADER_ID);
    assert!(
        shader_revision > 0,
        "viewport SDF shader id should resolve to a loaded shader revision (>0); got {shader_revision}",
    );
    assert!(
        viewport_state.shader_loaded,
        "viewport render diagnostics should report viewport shader as loaded",
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
    assert!(
        matches!(
            viewport_state.primitive_kind,
            EditorPrimitiveKind::Box | EditorPrimitiveKind::Sphere
        ),
        "viewport verification primitive must stay in box/sphere scope",
    );
}
