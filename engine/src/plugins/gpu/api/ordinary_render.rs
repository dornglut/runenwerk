use super::{
    GpuBlendConstant, GpuBlendMode, GpuColorWriteMask, GpuDrawIntent,
    GpuMultisampleStateDescriptor, GpuPrimitiveStateDescriptor, GpuRenderColorAttachment,
    GpuRenderDraw, GpuRenderOperation, GpuRenderPassSignature, GpuRenderPipelineDescriptor,
    GpuRuntimeBindingSet, GpuScissorRect, GpuViewport, GpuWorkOperationCause,
    GpuWorkOperationError,
};

impl GpuRenderOperation {
    /// Constructs one ordinary single-color draw covering the complete effective target extent.
    ///
    /// The color attachment remains explicit, including load/store/clear semantics. The pipeline,
    /// runtime bindings, and draw intent also remain explicit workload choices. This constructor is
    /// intentionally limited to the state produced by [`GpuRenderPipelineDescriptor::ordinary_color`]:
    /// no host vertex buffers, one replacement-blended full-write color target, default primitive
    /// state, no depth/stencil, and single-sample rendering.
    ///
    /// Within that boundary, the attachment already determines the effective render extent, so the
    /// full-target viewport/scissor and otherwise irrelevant empty vertex/index bindings, blend
    /// constant, stencil reference, depth attachment, and timestamp writes are framework
    /// administration. Use [`GpuRenderOperation::new`] plus [`GpuRenderDraw::new`] when any of those
    /// choices are material.
    pub fn ordinary_color_full_target(
        attachment: GpuRenderColorAttachment,
        pipeline: GpuRenderPipelineDescriptor,
        bindings: GpuRuntimeBindingSet,
        draw: GpuDrawIntent,
    ) -> Result<Self, GpuWorkOperationError> {
        validate_ordinary_color_pipeline(&pipeline)?;
        let signature =
            GpuRenderPassSignature::from_attachments(std::slice::from_ref(&attachment), None)?;
        let extent = signature.extent();
        let draw = GpuRenderDraw::new(
            pipeline,
            bindings,
            [],
            None,
            draw,
            GpuViewport::new(
                0.0,
                0.0,
                extent.width() as f32,
                extent.height() as f32,
                0.0,
                1.0,
            )?,
            GpuScissorRect::new(0, 0, extent.width(), extent.height())?,
            GpuBlendConstant::new(0.0, 0.0, 0.0, 0.0)?,
            0,
        )?;
        Self::new([attachment], None, [draw], None)
    }
}

fn validate_ordinary_color_pipeline(
    pipeline: &GpuRenderPipelineDescriptor,
) -> Result<(), GpuWorkOperationError> {
    let state = pipeline.state();
    let targets = state
        .fragment_output()
        .map(|output| output.color_targets().collect::<Vec<_>>())
        .unwrap_or_default();
    let ordinary_target = targets.len() == 1
        && targets[0].blend() == GpuBlendMode::Replace
        && targets[0].write_mask() == GpuColorWriteMask::ALL;
    let ordinary = state.vertex_input().layouts().len() == 0
        && ordinary_target
        && state.primitive() == GpuPrimitiveStateDescriptor::default()
        && state.depth_stencil().is_none()
        && state.multisample() == GpuMultisampleStateDescriptor::default();
    if !ordinary {
        return Err(GpuWorkOperationError::invalid(
            "construct ordinary full-target GPU color render",
            "render pipeline",
            None,
            GpuWorkOperationCause::InvalidDraw,
            "use an ordinary-color pipeline or construct the render draw and operation explicitly",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::gpu::{
        GpuAdmittedProgramSource, GpuAttachmentStore, GpuBindingLayoutRefinement,
        GpuBufferInitialization, GpuColorAttachmentLoad, GpuColorClearValue,
        GpuColorTargetStateDescriptor, GpuEntryPointName, GpuFragmentOutputStateDescriptor,
        GpuPipelineConfiguration, GpuProgramDescriptor, GpuReconstruction, GpuRenderEntryPoints,
        GpuRenderPipelineStateDescriptor, GpuResourceLifetime, GpuResourceScope,
        GpuRuntimeBindingValue, GpuTextureDescriptor, GpuTextureFormat, GpuTextureInitialization,
        GpuTextureUsage, GpuTextureViewDescriptor, GpuVertexInputStateDescriptor, GpuDrawRange,
        admit_static_wgsl_sources,
    };

    const RENDER_WGSL: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
"#;

    fn source() -> GpuAdmittedProgramSource {
        let [source] = admit_static_wgsl_sources([("proof.render.full-target", 1, RENDER_WGSL)])
            .unwrap();
        source
    }

    fn attachment(
        resources: &mut GpuResourceScope,
        format: GpuTextureFormat,
        width: u32,
        height: u32,
    ) -> GpuRenderColorAttachment {
        let texture = resources
            .texture(
                GpuTextureDescriptor::ordinary_owned_2d(
                    "full target",
                    GpuResourceLifetime::Transient,
                    GpuReconstruction::SourceBacked,
                    width,
                    height,
                    format,
                    [GpuTextureUsage::ColorAttachment],
                    GpuTextureInitialization::Uninitialized,
                )
                .unwrap(),
            )
            .unwrap();
        let view = resources
            .texture_view(
                GpuTextureViewDescriptor::ordinary_full_owned("full target view", &texture)
                    .unwrap(),
            )
            .unwrap();
        GpuRenderColorAttachment::new(
            view,
            GpuColorAttachmentLoad::Clear(
                GpuColorClearValue::new(0.0, 0.0, 0.0, 1.0).unwrap(),
            ),
            GpuAttachmentStore::Store,
            None,
        )
        .unwrap()
    }

    fn draw_intent() -> GpuDrawIntent {
        GpuDrawIntent::direct(
            GpuDrawRange::new(0, 3).unwrap(),
            GpuDrawRange::new(0, 1).unwrap(),
        )
    }

    #[test]
    fn ordinary_color_full_target_derives_only_documented_render_administration() {
        let pipeline =
            GpuRenderPipelineDescriptor::ordinary_color(source(), "vs_main", "fs_main", GpuTextureFormat::Rgba8Unorm)
                .unwrap();
        let bindings = pipeline
            .runtime_bindings(std::iter::empty::<GpuRuntimeBindingValue>())
            .unwrap();
        let mut resources = GpuResourceScope::new();
        let operation = GpuRenderOperation::ordinary_color_full_target(
            attachment(&mut resources, GpuTextureFormat::Rgba8Unorm, 32, 16),
            pipeline,
            bindings,
            draw_intent(),
        )
        .unwrap();

        assert_eq!(operation.color_attachments().len(), 1);
        assert_eq!(operation.depth_stencil_attachment(), None);
        assert_eq!(operation.timestamp_writes(), None);
        assert_eq!(operation.signature().extent().width(), 32);
        assert_eq!(operation.signature().extent().height(), 16);
        let draw = &operation.draws()[0];
        assert!(draw.vertex_buffers().is_empty());
        assert!(draw.index_buffer().is_none());
        assert_eq!(draw.viewport().values(), [0.0, 0.0, 32.0, 16.0, 0.0, 1.0]);
        assert_eq!(draw.scissor().x(), 0);
        assert_eq!(draw.scissor().y(), 0);
        assert_eq!(draw.scissor().width(), 32);
        assert_eq!(draw.scissor().height(), 16);
        assert_eq!(draw.blend_constant().components(), [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(draw.stencil_reference(), 0);
    }

    #[test]
    fn ordinary_color_full_target_rejects_nonordinary_pipeline_state() {
        let source = source();
        let vertex = GpuEntryPointName::new("vs_main").unwrap();
        let fragment = GpuEntryPointName::new("fs_main").unwrap();
        let program = GpuProgramDescriptor::new(
            source,
            [vertex.clone(), fragment.clone()],
            std::iter::empty::<GpuBindingLayoutRefinement>(),
        )
        .unwrap();
        let state = GpuRenderPipelineStateDescriptor::new(
            GpuVertexInputStateDescriptor::new([]).unwrap(),
            Some(GpuFragmentOutputStateDescriptor::new([
                GpuColorTargetStateDescriptor::new(
                    GpuTextureFormat::Rgba8Unorm,
                    GpuBlendMode::Alpha,
                    GpuColorWriteMask::ALL,
                )
                .unwrap(),
            ])),
            GpuPrimitiveStateDescriptor::default(),
            None,
            GpuMultisampleStateDescriptor::default(),
        )
        .unwrap();
        let pipeline = GpuRenderPipelineDescriptor::new(
            program,
            GpuRenderEntryPoints::new(vertex, Some(fragment)),
            state,
            GpuPipelineConfiguration::default(),
        )
        .unwrap();
        let bindings = pipeline
            .runtime_bindings(std::iter::empty::<GpuRuntimeBindingValue>())
            .unwrap();
        let mut resources = GpuResourceScope::new();
        let error = GpuRenderOperation::ordinary_color_full_target(
            attachment(&mut resources, GpuTextureFormat::Rgba8Unorm, 8, 8),
            pipeline,
            bindings,
            draw_intent(),
        )
        .unwrap_err();

        assert_eq!(error.cause(), GpuWorkOperationCause::InvalidDraw);
    }
}
