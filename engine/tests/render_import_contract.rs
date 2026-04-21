use engine::plugins::render::{
    RenderFlowGraph, RenderFlowId, RenderFlowValidationIssue, RenderPassId, RenderPassKind,
    RenderPassNode, RenderResourceDescriptor, RenderResourceId, validate_flow_graph,
};

#[test]
fn external_imported_texture_is_rejected_in_active_runtime_path() {
    let mut graph = RenderFlowGraph::new(RenderFlowId::new(1), "import.contract.external");
    let external_id = RenderResourceId::new(1);
    graph.add_resource(RenderResourceDescriptor::imported_texture(external_id));

    let mut pass = RenderPassNode::new(
        RenderPassId::new(1),
        "post.present",
        RenderPassKind::Present,
    );
    pass.reads.push(external_id);
    graph.add_pass(pass);

    let err = validate_flow_graph(&graph).expect_err("flow must reject external imports");
    assert!(
        err.issues
            .iter()
            .any(|issue| matches!(issue, RenderFlowValidationIssue::UnsupportedExternalImportedTexture { .. })),
        "expected external import rejection issue, got {:?}",
        err.issues
    );
}

#[test]
fn builtin_ui_composite_requires_canonical_read_write_contract() {
    let mut graph = RenderFlowGraph::new(RenderFlowId::new(2), "import.contract.ui");
    let surface_color = RenderResourceId::new(10);
    let ui_output = RenderResourceId::new(11);
    graph.add_resource(RenderResourceDescriptor::imported_surface_color(surface_color));
    graph.add_resource(RenderResourceDescriptor::color_target(ui_output));

    let mut pass = RenderPassNode::new(
        RenderPassId::new(2),
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
    let mut graph = RenderFlowGraph::new(RenderFlowId::new(3), "import.contract.canonical");
    graph.add_resource(RenderResourceDescriptor::imported_surface_color(
        RenderResourceId::new(20),
    ));
    graph.add_resource(RenderResourceDescriptor::imported_surface_color(
        RenderResourceId::new(21),
    ));

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
