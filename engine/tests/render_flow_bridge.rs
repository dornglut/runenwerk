use engine::plugins::render::domain::{RegisteredPassKind, RegisteredPipelineRef, RenderGraphRegistryResource};
use engine::plugins::render::{RenderFlow, RenderFlowContribution, RenderFlowRegistryResource};

#[test]
fn sync_bridges_render_flow_into_legacy_graph_registry() {
    let mut flow_registry = RenderFlowRegistryResource::default();
    flow_registry.upsert_flow(
        RenderFlow::new("bridge.main")
            .color_target("bridge.output")
            .compute_pass("bridge.compute")
            .shader("assets/shaders/bridge_compute.wgsl")
            .writes("bridge.output")
            .finish(),
    );

    let mut graph_registry = RenderGraphRegistryResource::default();
    flow_registry.sync_into_graph_registry(&mut graph_registry);

    let owners = graph_registry.owners();
    let owner = owners
        .iter()
        .find(|owner| owner.owner == "flow::bridge.main")
        .expect("flow owner registration must exist");

    assert_eq!(owner.pipelines.len(), 1);
    assert_eq!(owner.passes.len(), 1);

    let pass = &owner.passes[0];
    assert_eq!(pass.id, "bridge.compute");
    assert_eq!(pass.kind, RegisteredPassKind::Compute);
    assert_eq!(pass.executor.as_deref(), Some("bridge.compute"));
    assert!(matches!(
        pass.pipeline,
        Some(RegisteredPipelineRef::Named(ref id)) if id == "bridge.compute.pipeline"
    ));
}

#[test]
fn sync_removes_stale_flow_owners_after_deletion() {
    let mut flow_registry = RenderFlowRegistryResource::default();
    flow_registry.upsert_flow(
        RenderFlow::new("bridge.main")
            .color_target("bridge.output")
            .compute_pass("bridge.compute")
            .writes("bridge.output")
            .finish(),
    );

    let mut graph_registry = RenderGraphRegistryResource::default();
    flow_registry.sync_into_graph_registry(&mut graph_registry);
    assert!(graph_registry
        .owners()
        .iter()
        .any(|owner| owner.owner == "flow::bridge.main"));

    assert!(flow_registry.remove_flow("bridge.main"));
    flow_registry.sync_into_graph_registry(&mut graph_registry);
    assert!(!graph_registry
        .owners()
        .iter()
        .any(|owner| owner.owner == "flow::bridge.main"));
}

#[test]
fn sync_merges_registered_contributions_into_flow_owner() {
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

    let mut graph_registry = RenderGraphRegistryResource::default();
    flow_registry.sync_into_graph_registry(&mut graph_registry);

    let owners = graph_registry.owners();
    let owner = owners
        .iter()
        .find(|owner| owner.owner == "flow::bridge.main")
        .expect("flow owner registration must exist");

    assert_eq!(owner.passes.len(), 2);
    assert!(owner.passes.iter().any(|pass| pass.id == "post.tonemap"));
    assert!(owner.passes.iter().any(|pass| pass.id == "post.present"));
}
