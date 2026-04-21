use anyhow::Result;
use engine::plugins::render::RenderFlow;

fn main() -> Result<()> {
    let flow = RenderFlow::new("minimal.flow")
        .with_surface_color()
        .fullscreen_pass("minimal.compose")
        .write_surface_color()
        .finish()
        .validate()?;

    let order = flow.pass_order()?;
    let order = order
        .into_iter()
        .map(|pass_id| pass_id.to_string())
        .collect::<Vec<_>>();
    println!("minimal flow order: {}", order.join(" -> "));
    Ok(())
}
