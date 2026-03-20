use anyhow::Result;
use engine::plugins::render::inspect::{
    PassTimingSample, dump_flow_graph, inspect_resources, inspect_texture_resources,
    summarize_pass_timings,
};
use engine::plugins::render::RenderFlow;

const FLOW_ID: &str = "inspect.flow";

#[derive(Debug, Clone, Copy, engine::plugins::render::GpuStorage)]
struct InspectCell {
    value: u32,
}

fn main() -> Result<()> {
    let flow = RenderFlow::new(FLOW_ID)
        .with_surface_color()
        .with_builtin_ui()
        .double_buffer_storage_array::<InspectCell>("inspect.cells", 16)
        .compute_pass("inspect.sim")
        .shader_asset("assets/shaders/world_compute_basic.wgsl")
        .bind_ping_pong_storage("inspect.cells")
        .dispatch([1, 1, 1])
        .finish()
        .fullscreen_pass("inspect.compose")
        .shader_asset("assets/shaders/tonemap.wgsl")
        .bind_ping_pong_storage("inspect.cells")
        .write_surface_color()
        .depends_on("inspect.sim")
        .finish()
        .builtin_ui_composite_pass("inspect.ui")
        .depends_on("inspect.compose")
        .finish()
        .validate()?;

    let dump = dump_flow_graph(&flow)?;
    println!("graph dump:");
    for line in &dump.lines {
        println!("  {line}");
    }

    let resources = inspect_resources(&flow);
    println!("resource count: {}", resources.len());
    let textures = inspect_texture_resources(&flow);
    println!("texture count: {}", textures.len());

    let timings = summarize_pass_timings(&[
        PassTimingSample {
            flow_id: FLOW_ID.to_string(),
            pass_id: "inspect.sim".to_string(),
            pass_kind: "compute".to_string(),
            millis: 0.5,
            dispatch_workgroups: Some([1, 1, 1]),
        },
        PassTimingSample {
            flow_id: FLOW_ID.to_string(),
            pass_id: "inspect.compose".to_string(),
            pass_kind: "fullscreen".to_string(),
            millis: 0.2,
            dispatch_workgroups: None,
        },
    ]);
    println!(
        "frame {:.2}ms, slowest {:?}",
        timings.total_millis, timings.slowest_pass_id
    );

    Ok(())
}
