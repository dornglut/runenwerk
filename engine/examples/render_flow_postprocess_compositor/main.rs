use anyhow::Result;
use engine::plugins::render::RenderFlow;

fn main() -> Result<()> {
    let flow = RenderFlow::new("post.flow")
        .import_texture("surface.color")
        .color_target("post.bloom.extract")
        .color_target("post.bloom.blur_x")
        .color_target("post.bloom.blur_y")
        .fullscreen_pass("post.bloom_extract")
        .shader("assets/shaders/bloom_extract.wgsl")
        .reads("surface.color")
        .writes("post.bloom.extract")
        .finish()
        .fullscreen_pass("post.bloom_blur_x")
        .shader("assets/shaders/blur_x.wgsl")
        .reads("post.bloom.extract")
        .writes("post.bloom.blur_x")
        .depends_on("post.bloom_extract")
        .finish()
        .fullscreen_pass("post.bloom_blur_y")
        .shader("assets/shaders/blur_y.wgsl")
        .reads("post.bloom.blur_x")
        .writes("post.bloom.blur_y")
        .depends_on("post.bloom_blur_x")
        .finish()
        .copy_pass("post.copy_to_surface")
        .reads("post.bloom.blur_y")
        .writes("surface.color")
        .depends_on("post.bloom_blur_y")
        .finish()
        .present_pass("post.present")
        .reads("surface.color")
        .depends_on("post.copy_to_surface")
        .finish();

    let order = flow.pass_order()?;
    println!("postprocess flow order: {}", order.join(" -> "));
    Ok(())
}
