use engine::plugins::render::graph::merge_flow_with_contributions;
use engine::plugins::render::{RenderFlow, RenderFlowContribution};

#[test]
fn merge_order_resolves_cross_contribution_dependencies() {
    let base = RenderFlow::new("main.flow")
        .import_texture("surface.color")
        .import_texture("ui.draw_list");

    let boids = RenderFlowContribution::new("boids")
        .color_target("boids.out")
        .compute_pass("boids.simulate")
        .writes("boids.out")
        .finish();

    let post = RenderFlowContribution::new("post")
        .color_target("post.tonemap.out")
        .fullscreen_pass("post.tonemap")
        .reads("boids.out")
        .writes("post.tonemap.out")
        .depends_on("boids.simulate")
        .finish();

    let ui = RenderFlowContribution::new("ui")
        .builtin_ui_composite_pass("ui.composite")
        .reads("ui.draw_list")
        .writes("surface.color")
        .depends_on("post.tonemap")
        .finish();

    let merged = merge_flow_with_contributions(&base, &[boids, post, ui])
        .expect("flow contributions should merge");
    let order = merged.pass_order().expect("merged flow should validate");
    assert_eq!(
        order,
        vec!["boids.simulate", "post.tonemap", "ui.composite"]
    );
}

#[test]
fn namespace_validation_reports_duplicate_ids_across_contributions() {
    let base = RenderFlow::new("main.flow").import_texture("surface.color");

    let a = RenderFlowContribution::new("post")
        .color_target("post.output")
        .fullscreen_pass("post.tonemap")
        .writes("post.output")
        .finish();
    let b = RenderFlowContribution::new("debug")
        .color_target("post.output")
        .fullscreen_pass("post.tonemap")
        .writes("post.output")
        .finish();

    let err = merge_flow_with_contributions(&base, &[a, b])
        .expect_err("duplicate ids must fail contribution merge");
    let message = err.to_string();
    assert!(message.contains("duplicate pass id 'post.tonemap'"));
    assert!(message.contains("duplicate resource id 'post.output'"));
}

#[test]
fn namespace_validation_reports_unknown_cross_plugin_dependencies() {
    let base = RenderFlow::new("main.flow").import_texture("surface.color");

    let contribution = RenderFlowContribution::new("post")
        .color_target("post.output")
        .fullscreen_pass("post.tonemap")
        .writes("post.output")
        .depends_on("boids.simulate")
        .finish();

    let err = merge_flow_with_contributions(&base, &[contribution])
        .expect_err("unknown cross-plugin dependency must fail merge");
    assert!(err.to_string().contains("invalid cross-plugin dependency"));
}

#[test]
fn contribution_copy_and_present_passes_merge_and_validate() {
    let base = RenderFlow::new("main.flow")
        .import_texture("surface.color")
        .color_target("post.output");

    let post = RenderFlowContribution::new("post")
        .copy_pass("post.copy")
        .reads("post.output")
        .writes("surface.color")
        .finish()
        .present_pass("post.present")
        .reads("surface.color")
        .depends_on("post.copy")
        .finish();

    let merged =
        merge_flow_with_contributions(&base, &[post]).expect("contribution merge should work");
    let order = merged.pass_order().expect("merged flow should validate");
    assert_eq!(order, vec!["post.copy", "post.present"]);
}
