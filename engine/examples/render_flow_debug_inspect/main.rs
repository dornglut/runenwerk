use anyhow::Result;
use engine::plugins::render::RenderFlow;
use engine::plugins::render::inspect::{
    PassTimingSample, dump_flow_graph, inspect_resources, inspect_texture_resources,
    summarize_pass_timings,
};

fn main() -> Result<()> {
    let flow = RenderFlow::new("inspect.flow")
        .import_texture("surface.color")
        .history_texture("taa.history")
        .transient_color_target("post.temp")
        .fullscreen_pass("post.compose")
        .reads("taa.history")
        .writes("post.temp")
        .finish()
        .copy_pass("post.copy_to_surface")
        .reads("post.temp")
        .writes("surface.color")
        .depends_on("post.compose")
        .finish()
        .present_pass("post.present")
        .reads("surface.color")
        .depends_on("post.copy_to_surface")
        .finish();

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
            pass_id: "post.compose".to_string(),
            millis: 0.5,
        },
        PassTimingSample {
            pass_id: "post.copy_to_surface".to_string(),
            millis: 0.2,
        },
    ]);
    println!(
        "frame {:.2}ms, slowest {:?}",
        timings.total_millis, timings.slowest_pass_id
    );

    Ok(())
}
