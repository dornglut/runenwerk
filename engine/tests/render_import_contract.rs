use engine::plugins::gpu::GpuWorkResourceId;
use engine::plugins::render::{
    RenderFlow, RenderFlowGraph, RenderFlowId, RenderFlowValidationIssue, RenderPassId,
    RenderPassKind, RenderPassNode, RenderResourceDescriptor, validate_flow_graph,
};

fn test_resource_ids(count: usize) -> Vec<GpuWorkResourceId> {
    let labels = (0..count)
        .map(|index| format!("test.import.resource.{index}"))
        .collect::<Vec<_>>();
    let flow = labels
        .iter()
        .fold(RenderFlow::new("test.import.resources"), |flow, label| {
            flow.with_color_target(label.clone())
                .expect("render flow authoring should succeed")
        });
    labels
        .iter()
        .map(|label| flow.resource_id(label).expect("test resource should exist"))
        .collect()
}

#[test]
fn external_imported_texture_is_rejected_in_active_runtime_path() {
    let mut graph = RenderFlowGraph::new(
        RenderFlowId::try_from_raw(1).unwrap(),
        "import.contract.external",
    );
    let external_id = test_resource_ids(1)[0];
    graph.add_resource(RenderResourceDescriptor::imported_texture(external_id));

    let mut pass = RenderPassNode::new(
        RenderPassId::try_from_raw(1).unwrap(),
        "post.present",
        RenderPassKind::Present,
    );
    pass.reads.push(external_id);
    graph.add_pass(pass);

    let err = validate_flow_graph(&graph).expect_err("flow must reject external imports");
    assert!(
        err.issues.iter().any(|issue| matches!(
            issue,
            RenderFlowValidationIssue::UnsupportedExternalImportedTexture { .. }
        )),
        "expected external import rejection issue, got {:?}",
        err.issues
    );
}

#[test]
fn builtin_ui_composite_requires_canonical_read_write_contract() {
    let mut graph =
        RenderFlowGraph::new(RenderFlowId::try_from_raw(2).unwrap(), "import.contract.ui");
    let ids = test_resource_ids(2);
    let surface_color = ids[0];
    let ui_output = ids[1];
    graph.add_resource(RenderResourceDescriptor::imported_surface_color(
        surface_color,
    ));
    graph.add_resource(RenderResourceDescriptor::color_target(ui_output));

    let mut pass = RenderPassNode::new(
        RenderPassId::try_from_raw(2).unwrap(),
        "ui.composite",
        RenderPassKind::BuiltinUiComposite,
    );
    pass.reads.push(surface_color);
    pass.writes.push(ui_output);
    graph.add_pass(pass);

    let err = validate_flow_graph(&graph).expect_err("flow must enforce UI composite contract");
    assert!(
        err.issues
            .iter()
            .any(|issue| matches!(issue, RenderFlowValidationIssue::BuiltinUiHasReads { .. })),
        "expected UI reads contract issue, got {:?}",
        err.issues
    );
}

#[test]
fn typed_surface_imports_require_canonical_resource_ids() {
    let mut graph = RenderFlowGraph::new(
        RenderFlowId::try_from_raw(3).unwrap(),
        "import.contract.canonical",
    );
    let ids = test_resource_ids(2);
    graph.add_resource(RenderResourceDescriptor::imported_surface_color(ids[0]));
    graph.add_resource(RenderResourceDescriptor::imported_surface_color(ids[1]));

    let err = validate_flow_graph(&graph).expect_err("flow must enforce unique surface imports");
    assert!(
        err.issues.iter().any(|issue| matches!(
            issue,
            RenderFlowValidationIssue::MultipleSurfaceColorImports { .. }
        )),
        "expected duplicate surface color import issue, got {:?}",
        err.issues
    );
}
