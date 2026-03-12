use anyhow::Result;
use engine::plugins::render::RenderFlow;

fn main() -> Result<()> {
    let flow = RenderFlow::new("sdf.flow")
        .import_texture("surface.color")
        .history_texture("sdf.history")
        .color_target("sdf.output")
        .compute_pass("sdf.field_update")
        .write_texture("sdf.history")
        .finish()
        .fullscreen_pass("sdf.raymarch")
        .sample_texture("sdf.history")
        .writes("sdf.output")
        .depends_on("sdf.field_update")
        .finish()
        .copy_pass("sdf.copy_to_surface")
        .reads("sdf.output")
        .writes("surface.color")
        .depends_on("sdf.raymarch")
        .finish()
        .present_pass("sdf.present")
        .reads("surface.color")
        .depends_on("sdf.copy_to_surface")
        .finish();

    let order = flow.pass_order()?;
    println!("sdf flow order: {}", order.join(" -> "));
    Ok(())
}
