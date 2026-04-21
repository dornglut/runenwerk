use anyhow::Result;
use engine::plugins::render::{GpuStorage, RenderFlow};

#[derive(Debug, Clone, Copy, GpuStorage)]
struct BloomCell {
    value: u32,
}

fn main() -> Result<()> {
    let flow = RenderFlow::new("post.flow")
        .with_surface_color()
        .double_buffer_storage_array::<BloomCell>("post.bloom", 64)
        .compute_pass("post.bloom_extract")
        .shader_asset("assets/shaders/bloom_extract.wgsl")
        .bind_ping_pong_storage("post.bloom")
        .dispatch([1, 1, 1])
        .finish()
        .fullscreen_pass("post.compose")
        .shader_asset("assets/shaders/blur_y.wgsl")
        .bind_ping_pong_storage("post.bloom")
        .write_surface_color()
        .depends_on("post.bloom_extract")
        .finish()
        .validate()?;

    let order = flow.pass_order()?;
    let order = order
        .into_iter()
        .map(|pass_id| pass_id.to_string())
        .collect::<Vec<_>>();
    println!("postprocess flow order: {}", order.join(" -> "));
    Ok(())
}
