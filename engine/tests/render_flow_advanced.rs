use engine::plugins::render::RenderFlow;

#[test]
fn storage_texture_write_read_chain_validates() {
    let flow = RenderFlow::new("sim.flow")
        .storage_texture("sim.field")
        .color_target("surface.color")
        .compute_pass("sim.update")
        .write_texture("sim.field")
        .finish()
        .fullscreen_pass("sim.compose")
        .sample_texture("sim.field")
        .writes("surface.color")
        .depends_on("sim.update")
        .finish();

    let report = flow.validate().expect("flow should validate");
    assert_eq!(report.pass_order, vec!["sim.update", "sim.compose"]);
}

#[test]
fn copy_and_present_passes_validate() {
    let flow = RenderFlow::new("present.flow")
        .import_texture("surface.color")
        .color_target("post.output")
        .copy_pass("post.copy")
        .reads("post.output")
        .writes("surface.color")
        .finish()
        .present_pass("post.present")
        .reads("surface.color")
        .depends_on("post.copy")
        .finish();

    let report = flow.validate().expect("flow should validate");
    assert_eq!(report.pass_order, vec!["post.copy", "post.present"]);
}

#[test]
fn history_texture_chain_validates() {
    let flow = RenderFlow::new("taa.flow")
        .import_texture("surface.color")
        .history_texture("taa.history")
        .color_target("taa.output")
        .fullscreen_pass("taa.resolve")
        .reads("surface.color")
        .sample_texture("taa.history")
        .writes("taa.output")
        .finish()
        .copy_pass("taa.history_update")
        .reads("taa.output")
        .writes("taa.history")
        .depends_on("taa.resolve")
        .finish()
        .present_pass("taa.present")
        .reads("taa.output")
        .depends_on("taa.history_update")
        .finish();

    let report = flow.validate().expect("history flow should validate");
    assert_eq!(
        report.pass_order,
        vec!["taa.resolve", "taa.history_update", "taa.present"]
    );
}
