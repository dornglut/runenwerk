use engine::plugins::render::{
    CompiledPassDescriptor, RenderFlow, RenderFlowContribution, RenderFlowRegistryResource,
    backend::ensure_compiled_pass_is_supported, compile_flow_plan,
};

#[test]
fn sync_compiles_render_flow_into_compiled_plans() {
    let mut flow_registry = RenderFlowRegistryResource::default();
    flow_registry.upsert_flow(
        RenderFlow::new("bridge.main")
            .color_target("bridge.output")
            .compute_pass("bridge.compute")
            .shader("assets/shaders/world_compute_basic.wgsl")
            .writes("bridge.output")
            .finish(),
    );

    flow_registry.sync_compiled_flows();
    let compiled = flow_registry.compiled_flows();
    assert_eq!(compiled.len(), 1);
    assert_eq!(compiled[0].flow_id, "bridge.main");
    assert_eq!(compiled[0].pass_order.len(), 1);
    match &compiled[0].pass_order[0] {
        CompiledPassDescriptor::Compute(pass) => {
            assert_eq!(pass.node.id.as_str(), "bridge.compute");
        }
        other => panic!("expected compute pass, got {:?}", other),
    }
}

#[test]
fn sync_removes_stale_compiled_flows_after_deletion() {
    let mut flow_registry = RenderFlowRegistryResource::default();
    flow_registry.upsert_flow(
        RenderFlow::new("bridge.main")
            .import_texture("surface.color")
            .fullscreen_pass("bridge.compose")
            .writes("surface.color")
            .finish(),
    );

    flow_registry.sync_compiled_flows();
    assert_eq!(flow_registry.compiled_flows().len(), 1);

    assert!(flow_registry.remove_flow("bridge.main"));
    flow_registry.sync_compiled_flows();
    assert!(flow_registry.compiled_flows().is_empty());
}

#[test]
fn sync_merges_registered_contributions_into_compiled_flow() {
    let mut flow_registry = RenderFlowRegistryResource::default();
    flow_registry.upsert_flow(RenderFlow::new("bridge.main").import_texture("surface.color"));
    flow_registry.upsert_contribution(
        RenderFlowContribution::new("post")
            .color_target("post.output")
            .fullscreen_pass("post.tonemap")
            .writes("post.output")
            .finish()
            .fullscreen_pass("post.present")
            .reads("post.output")
            .writes("surface.color")
            .depends_on("post.tonemap")
            .finish(),
    );

    flow_registry.sync_compiled_flows();
    let compiled = flow_registry.compiled_flows();
    assert_eq!(compiled.len(), 1);
    let pass_ids = compiled[0]
        .pass_order
        .iter()
        .map(|pass| pass.pass_id().to_string())
        .collect::<Vec<_>>();
    assert!(pass_ids.iter().any(|id| id == "post.tonemap"));
    assert!(pass_ids.iter().any(|id| id == "post.present"));
}

#[test]
fn backend_support_checks_fail_loudly_for_unimplemented_builtin_passes() {
    let flow = RenderFlow::new("bridge.main")
        .import_texture("surface.color")
        .color_target("bridge.post")
        .fullscreen_pass("bridge.compose")
        .writes("bridge.post")
        .finish()
        .copy_pass("bridge.copy_to_surface")
        .reads("bridge.post")
        .writes("surface.color")
        .depends_on("bridge.compose")
        .finish()
        .present_pass("bridge.present")
        .reads("surface.color")
        .depends_on("bridge.copy_to_surface")
        .finish();

    let compiled = compile_flow_plan(&flow).expect("flow should compile into a pass plan");
    let mut saw_copy = false;
    let mut saw_present = false;
    for pass in &compiled.pass_order {
        match pass {
            CompiledPassDescriptor::Fullscreen(_) => {
                ensure_compiled_pass_is_supported(pass)
                    .expect("fullscreen pass should be supported by builtin backend execution");
            }
            CompiledPassDescriptor::Copy(_) => {
                saw_copy = true;
                let err = ensure_compiled_pass_is_supported(pass)
                    .expect_err("copy pass must fail loudly until backend copy execution exists");
                assert!(
                    err.to_string().contains("copy pass"),
                    "unexpected copy support error: {err}"
                );
            }
            CompiledPassDescriptor::Present(_) => {
                saw_present = true;
                let err = ensure_compiled_pass_is_supported(pass).expect_err(
                    "present pass must fail loudly until backend present execution exists",
                );
                assert!(
                    err.to_string().contains("present pass"),
                    "unexpected present support error: {err}"
                );
            }
            _ => {}
        }
    }

    assert!(saw_copy, "compiled plan should include a copy pass");
    assert!(saw_present, "compiled plan should include a present pass");
}
