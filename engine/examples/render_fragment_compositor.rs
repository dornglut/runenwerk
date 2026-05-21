use anyhow::Result;
use engine::plugins::render::{
    RenderBackendCapabilityProfile, RenderFlow, RenderFragmentDescriptor,
    RenderFragmentPackageDescriptor, RenderFragmentPassDescriptor,
    RenderFragmentResourceDescriptor, RenderTextureTargetFormat, merge_fragment_package_into_flow,
};

fn compositor_package() -> RenderFragmentPackageDescriptor {
    RenderFragmentPackageDescriptor::new(
        "example.fragment_compositor",
        "example_compositor",
        "examples/render_fragment_compositor.ron",
        1,
    )
    .with_fragment(
        RenderFragmentDescriptor::new("compose", "example_compositor")
            .with_resource(RenderFragmentResourceDescriptor::color_target_exact(
                "scene",
                RenderTextureTargetFormat::Rgba8Unorm,
            ))
            .with_pass(
                RenderFragmentPassDescriptor::fullscreen("compose")
                    .shader_asset("assets/shaders/fullscreen_composite.wgsl")
                    .write_local_color_target("scene"),
            ),
    )
}

fn build_flow() -> Result<RenderFlow> {
    let merged = merge_fragment_package_into_flow(
        RenderFlow::new("example.fragment_compositor"),
        &compositor_package(),
        &RenderBackendCapabilityProfile::runtime_default(),
    )?;
    Ok(merged.flow)
}

fn main() -> Result<()> {
    let flow = build_flow()?;
    let order = flow
        .pass_order()?
        .into_iter()
        .map(|pass_id| pass_id.to_string())
        .collect::<Vec<_>>()
        .join(" -> ");
    println!("fragment compositor pass order: {order}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_fragment_compositor_builds_normal_render_flow() {
        let flow = build_flow().expect("example fragment compositor flow should build");

        assert!(flow.pass_id("example_compositor::compose").is_some());
        assert_eq!(flow.pass_order().unwrap().len(), 1);
    }
}
