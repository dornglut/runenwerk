use super::descriptors::{
    ProceduralPassDescriptor, ProceduralShader, ProceduralTargetDescriptor,
    ProceduralVisualDescriptor,
};
use super::validation::{ProceduralValidationError, validate_procedural_pass};
use crate::plugins::render::RenderFlow;

pub fn build_procedural_pass(
    flow: RenderFlow,
    descriptor: ProceduralPassDescriptor,
) -> Result<RenderFlow, ProceduralValidationError> {
    validate_procedural_pass(&descriptor)?;

    let ProceduralPassDescriptor {
        label,
        shader,
        visual,
        instance_buffer,
        instance_count,
        index_buffer,
        target,
        policy,
        dependencies,
    } = descriptor;

    let mut builder = flow.graphics_pass(label);
    builder = match shader.expect("validated procedural pass has shader") {
        ProceduralShader::AssetPath(path) => builder.shader_asset(path),
        ProceduralShader::RegistryHandle(handle) => builder.shader(handle),
    };
    builder = builder.raster_state(policy.into());

    if let ProceduralVisualDescriptor::MeshSprite { vertex_buffer, .. } = &visual {
        builder = builder
            .push_vertex_buffer_resource(vertex_buffer.resource_id, vertex_buffer.layout.clone());
    }

    builder =
        builder.push_instance_buffer_resource(instance_buffer.resource_id, instance_buffer.layout);

    if let Some(index_buffer) = index_buffer {
        builder = builder.push_index_buffer_resource(index_buffer.resource_id);
    }

    let target = target.expect("validated procedural pass has target");
    builder = apply_target(builder, target);

    builder = builder.draw(visual.vertex_count(), instance_count);
    for dependency in dependencies {
        builder = builder.depends_on(dependency);
    }

    Ok(builder.finish())
}

fn apply_target(
    mut builder: crate::plugins::render::api::GraphicsPassBuilder,
    target: ProceduralTargetDescriptor,
) -> crate::plugins::render::api::GraphicsPassBuilder {
    builder = builder.write_color_target(target.color_target);
    if let Some(depth_target) = target.depth_target {
        builder = builder.depth_target(depth_target);
    }
    if let Some(clear_color) = target.clear_color {
        builder = builder.clear_color(clear_color);
    }
    builder
}
