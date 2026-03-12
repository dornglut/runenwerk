use engine::plugins::render::RenderFlow;

#[test]
fn pass_order_validation_returns_topological_order() {
    let flow = RenderFlow::new("main.flow")
        .sampled_texture("main.input")
        .color_target("main.compute_out")
        .color_target("main.output")
        .compute_pass("main.compute")
        .reads("main.input")
        .writes("main.compute_out")
        .finish()
        .fullscreen_pass("main.compose")
        .reads("main.compute_out")
        .writes("main.output")
        .depends_on("main.compute")
        .finish();

    let report = flow.validate().expect("flow should validate");
    assert_eq!(report.pass_order, vec!["main.compute", "main.compose"]);
}

#[test]
fn resource_reference_validation_reports_missing_resources() {
    let flow = RenderFlow::new("main.flow")
        .color_target("main.output")
        .compute_pass("main.compute")
        .reads("main.missing")
        .writes("main.output")
        .finish();

    let err = flow
        .validate()
        .expect_err("missing resource reference must fail validation");
    assert!(
        err.to_string().contains("unknown resource 'main.missing'"),
        "unexpected error: {err}"
    );
}

#[test]
fn cycle_detection_reports_dependency_cycles() {
    let flow = RenderFlow::new("main.flow")
        .color_target("main.a")
        .color_target("main.b")
        .compute_pass("main.a_pass")
        .writes("main.a")
        .depends_on("main.b_pass")
        .finish()
        .fullscreen_pass("main.b_pass")
        .reads("main.a")
        .writes("main.b")
        .depends_on("main.a_pass")
        .finish();

    let err = flow.validate().expect_err("cycle must fail validation");
    assert!(err.to_string().contains("dependency cycle"));
}

#[test]
fn duplicate_namespace_conflicts_are_reported() {
    let flow = RenderFlow::new("main.flow")
        .color_target("post.output")
        .color_target("post.output")
        .compute_pass("post.compute")
        .writes("post.output")
        .finish()
        .fullscreen_pass("post.compute")
        .reads("post.output")
        .writes("post.output")
        .finish();

    let err = flow
        .validate()
        .expect_err("duplicate ids must fail validation");
    let message = err.to_string();
    assert!(message.contains("duplicate resource id 'post.output'"));
    assert!(message.contains("duplicate pass id 'post.compute'"));
}

#[test]
fn copy_pass_shape_validation_rejects_invalid_declaration() {
    let flow = RenderFlow::new("copy.flow")
        .color_target("copy.a")
        .color_target("copy.b")
        .copy_pass("copy.pass")
        .reads("copy.a")
        .reads("copy.b")
        .writes("copy.b")
        .finish();

    let err = flow
        .validate()
        .expect_err("copy pass must enforce single read/write shape");
    assert!(
        err.to_string()
            .contains("must declare exactly one reads(...) and one writes(...)"),
        "unexpected error: {err}"
    );
}

#[test]
fn present_pass_requires_single_read() {
    let flow = RenderFlow::new("present.flow")
        .import_texture("surface.color")
        .color_target("post.output")
        .present_pass("post.present")
        .finish();

    let err = flow
        .validate()
        .expect_err("present pass must enforce exactly one read");
    assert!(
        err.to_string()
            .contains("must declare exactly one reads(...) resource"),
        "unexpected error: {err}"
    );
}

#[test]
fn graphics_buffer_role_validation_rejects_texture_as_vertex_buffer() {
    let flow = RenderFlow::new("graphics.flow")
        .color_target("surface.color")
        .sampled_texture("gfx.texture")
        .graphics_pass("gfx.draw")
        .vertex_buffer("gfx.texture")
        .writes("surface.color")
        .finish();

    let err = flow
        .validate()
        .expect_err("texture must not validate as vertex buffer");
    assert!(
        err.to_string()
            .contains("vertex_buffer(...) but kind 'sampled_texture' is not buffer-like"),
        "unexpected error: {err}"
    );
}

#[test]
fn imported_texture_write_is_restricted_by_pass_kind() {
    let flow = RenderFlow::new("import.flow")
        .import_texture("surface.color")
        .compute_pass("import.compute")
        .writes("surface.color")
        .finish();

    let err = flow
        .validate()
        .expect_err("compute writes to imported texture must fail");
    assert!(
        err.to_string()
            .contains("writes imported texture 'surface.color'"),
        "unexpected error: {err}"
    );
}
