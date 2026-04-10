use engine::plugins::render::{
    CompiledPassExecutionPlan, RenderFlowRegistryResource, UiFrameProducerId,
    UiFrameSubmissionRegistryResource,
};

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
    let has_editor_viewport_sdf_pass = flow_registry
        .compiled_flows()
        .iter()
        .flat_map(|flow| flow.execution.passes.iter())
        .any(|pass| {
            matches!(
                pass,
                CompiledPassExecutionPlan::Fullscreen(plan)
                if plan.pass_id == EDITOR_VIEWPORT_SDF_PASS_ID
            )
        });
    assert!(
        has_editor_viewport_sdf_pass,
        "editor render flows should include the viewport SDF pass",
    );

    let submission = submissions
        .get(&UiFrameProducerId::new("editor.shell"))
        .expect("editor shell submission should exist");

    assert!(
        !submission.frame.is_empty(),
        "editor shell frame should not be empty"
    );
    assert!(
        submission.primitive_count_hint() > 0,
        "editor shell frame should contain renderable primitives"
    );
}
