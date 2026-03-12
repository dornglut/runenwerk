use anyhow::Result;
use engine::plugins::render::RenderFlow;

fn main() -> Result<()> {
    let flow = RenderFlow::new("minimal.flow")
        .import_texture("surface.color")
        .import_texture("scene.color")
        .fullscreen_pass("minimal.compose")
        .reads("scene.color")
        .writes("surface.color")
        .finish();

    let order = flow.pass_order()?;
    println!("minimal flow order: {}", order.join(" -> "));
    Ok(())
}
