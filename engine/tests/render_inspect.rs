use engine::plugins::render::inspect::{
    PassTimingSample, dump_flow_graph, inspect_resources, inspect_texture_resources,
    summarize_pass_timings,
};
use engine::plugins::render::{RenderFlow, ResourceLifetime};

#[test]
fn graph_dump_contains_resources_passes_and_execution_order() {
    let flow = RenderFlow::new("inspect.flow")
        .import_texture("surface.color")
        .color_target("post.output")
        .fullscreen_pass("post.compose")
        .reads("surface.color")
        .writes("post.output")
        .finish();

    let dump = dump_flow_graph(&flow).expect("dump should validate");
    let text = dump.lines.join("\n");
    assert!(text.contains("flow: inspect.flow"));
    assert!(text.contains("resources:"));
    assert!(text.contains("passes:"));
    assert!(text.contains("execution_order: post.compose"));
}

#[test]
fn resource_and_texture_inspection_reports_lifetimes() {
    let flow = RenderFlow::new("inspect.flow")
        .import_texture("surface.color")
        .history_texture("taa.history")
        .transient_color_target("post.temp")
        .storage_buffer::<DummyStorage>("sim.buffer");

    let resources = inspect_resources(&flow);
    assert!(resources.iter().any(|entry| {
        entry.id == "surface.color" && entry.lifetime == ResourceLifetime::Imported
    }));
    assert!(resources.iter().any(|entry| {
        entry.id == "taa.history" && entry.kind == "history_texture"
    }));

    let textures = inspect_texture_resources(&flow);
    assert!(textures.iter().any(|entry| {
        entry.id == "post.temp" && entry.lifetime == ResourceLifetime::Transient
    }));
}

#[test]
fn timing_summary_reports_total_and_slowest_pass() {
    let snapshot = summarize_pass_timings(&[
        PassTimingSample {
            pass_id: "a".to_string(),
            millis: 0.4,
        },
        PassTimingSample {
            pass_id: "b".to_string(),
            millis: 1.2,
        },
    ]);

    assert!((snapshot.total_millis - 1.6).abs() < 0.0001);
    assert_eq!(snapshot.slowest_pass_id.as_deref(), Some("b"));
    assert!((snapshot.slowest_pass_millis - 1.2).abs() < 0.0001);
}

#[derive(Debug, Clone, Copy, engine::plugins::render::GpuStorage)]
struct DummyStorage {
    value: u32,
}
