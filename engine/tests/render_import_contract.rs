use engine::plugins::render::{
    RenderFlowGraph, RenderPassKind, RenderPassNode, RenderResourceDescriptor, RenderResourceId,
    validate_flow_graph,
};

#[test]
fn external_imported_texture_is_rejected_in_active_runtime_path() {
    let mut graph = RenderFlowGraph::new("import.contract.external");
    graph.add_resource(RenderResourceDescriptor::imported_texture(
        "post.external_input",
    ));

    let mut pass = RenderPassNode::new("post.present", RenderPassKind::Present);
    pass.reads
        .push(RenderResourceId::new("post.external_input"));
    graph.add_pass(pass);

    let err = validate_flow_graph(&graph).expect_err("flow must reject external imports");
    assert!(
        err.issues
            .iter()
            .any(|issue| issue.contains("external imported texture semantics")),
        "expected external import rejection issue, got {:?}",
        err.issues
    );
}

#[test]
fn builtin_ui_composite_requires_canonical_read_write_contract() {
    let mut graph = RenderFlowGraph::new("import.contract.ui");
    graph.add_resource(RenderResourceDescriptor::imported_ui_draw_list(
        "ui.draw_list",
    ));
    graph.add_resource(RenderResourceDescriptor::imported_surface_color(
        "surface.color",
    ));

    let mut pass = RenderPassNode::new("ui.composite", RenderPassKind::BuiltinUiComposite);
    pass.reads.push(RenderResourceId::new("surface.color"));
    pass.writes.push(RenderResourceId::new("ui.draw_list"));
    graph.add_pass(pass);

    let err = validate_flow_graph(&graph).expect_err("flow must enforce UI composite contract");
    assert!(
        err.issues
            .iter()
            .any(|issue| issue.contains("must read exactly 'ui.draw_list'")),
        "expected canonical UI read contract issue, got {:?}",
        err.issues
    );
    assert!(
        err.issues
            .iter()
            .any(|issue| issue.contains("must write exactly 'surface.color'")),
        "expected canonical UI write contract issue, got {:?}",
        err.issues
    );
}

#[test]
fn typed_surface_imports_require_canonical_resource_ids() {
    let mut graph = RenderFlowGraph::new("import.contract.canonical");
    graph.add_resource(RenderResourceDescriptor::imported_surface_color(
        "post.surface_color_alias",
    ));

    let mut pass = RenderPassNode::new("post.compose", RenderPassKind::Fullscreen);
    pass.writes
        .push(RenderResourceId::new("post.surface_color_alias"));
    graph.add_pass(pass);

    let err = validate_flow_graph(&graph).expect_err("flow must enforce canonical import ids");
    assert!(
        err.issues.iter().any(|issue| {
            issue.contains("imported surface-color texture must use canonical id")
        }),
        "expected canonical id issue, got {:?}",
        err.issues
    );
}
