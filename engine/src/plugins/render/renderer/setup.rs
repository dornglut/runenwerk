use super::resource_descriptors::{
    buffer_descriptor, gpu_texture_format, linear_sampler_descriptor, texture_descriptor,
    whole_texture_view_descriptor,
};
use super::*;
use crate::plugins::gpu::{
    CurrentRenderPipelineBindGroupsTerminal, CurrentRenderRenderPipelinesTerminal,
    CurrentRenderTimestampWritesTerminal, CurrentRenderVertexBufferTerminal,
    GpuBindGroupLayoutDescriptor, GpuBindingDeclaration, GpuBindingKey, GpuBindingKind,
    GpuBindingProvenance, GpuBlendMode, GpuBufferUsage, GpuCapabilityRequirements,
    GpuColorTargetStateDescriptor, GpuColorWriteMask, GpuEntryPointDescriptor, GpuEntryPointName,
    GpuFragmentOutputStateDescriptor, GpuMemoryIntent, GpuMultisampleStateDescriptor,
    GpuPipelineLayoutDescriptor, GpuPrimitiveStateDescriptor, GpuProgramDescriptor,
    GpuProgramInterfaceDescriptor, GpuProgramSourceKey, GpuProgramSourceProvenance,
    GpuRealizedBindGroup, GpuRealizedBindGroupLayout, GpuRealizedRenderPipeline,
    GpuRenderEntryPoints, GpuRenderPipelineDescriptor, GpuRenderPipelineStateDescriptor,
    GpuResourceLifetime, GpuRuntimeBindingResource, GpuRuntimeBindingValue,
    GpuRuntimeBufferBinding, GpuRuntimeTextureViewBinding, GpuSamplerClass, GpuShaderStage,
    GpuShaderStages, GpuSpecializationSchema, GpuSpecializationValueSet, GpuTextureFormat,
    GpuTextureSampleClass, GpuTextureUsage, GpuTextureViewDimension, GpuVertexAttribute,
    GpuVertexBufferLayoutDescriptor, GpuVertexFormat, GpuVertexInputStateDescriptor,
    GpuVertexStepMode,
};
use std::num::NonZeroU64;

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer {
    pub(super) fn realize_buffer_resource(
        &mut self,
        context: &GpuContext,
        label: &str,
        size: u64,
        usages: impl IntoIterator<Item = GpuBufferUsage>,
        lifetime: GpuResourceLifetime,
        memory_intent: GpuMemoryIntent,
    ) -> Result<RendererBufferResource> {
        let handle = self.resource_ids.allocate_buffer_handle(buffer_descriptor(
            label,
            size,
            usages,
            lifetime,
            memory_intent,
        )?)?;
        let realized = context.realize_buffer(&handle)?;
        Ok(RendererBufferResource {
            _handle: handle,
            realized,
        })
    }

    fn realize_linear_sampler_resource(
        &mut self,
        context: &GpuContext,
        label: &str,
        lifetime: GpuResourceLifetime,
    ) -> Result<RendererSamplerResource> {
        let handle = self
            .resource_ids
            .allocate_sampler_handle(linear_sampler_descriptor(label, lifetime)?)?;
        let realized = context.realize_sampler(&handle)?;
        Ok(RendererSamplerResource {
            _handle: handle,
            _realized: realized,
        })
    }

    // Owner: Engine Renderer - UI Pipeline Setup and Encoding
    pub fn new() -> Self {
        Self {
            resource_ids: GpuWorkResourceIdAllocator::new(),
            rect_pass: None,
            rect_pass_format: None,
            rect_pass_shader_revision: 0,
            stroke_pass: None,
            stroke_pass_format: None,
            glyph_pass: None,
            glyph_pass_format: None,
            viewport_embed_pass: None,
            viewport_embed_pass_format: None,
            product_surface_pass: None,
            product_surface_pass_format: None,
            glyph_atlas_gpu: std::collections::BTreeMap::new(),
            dynamic_texture_targets:
                super::dynamic_targets::RendererDynamicTextureTargetCache::default(),
            flow_runtime_cache: std::collections::BTreeMap::new(),
            flow_pipeline_cache: super::pipeline_cache::FlowPipelineArtifactCache::default(),
            preflight_cache: None,
            last_good_ui_prepared: None,
            last_pass_timings: Vec::new(),
            last_gpu_pass_timing_evidence: Vec::new(),
            last_runtime_resources: Vec::new(),
            last_pass_provenance: Vec::new(),
            last_preflight_report:
                crate::plugins::render::graph::RenderExecutionGraphPreparedReport::default(),
            last_preflight_cache_state:
                crate::plugins::render::graph::RenderPreparedFramePreflightCacheState::default(),
            last_capture_plan: crate::plugins::render::inspect::ResolvedRenderCapturePlan::default(
            ),
            last_capture_selector_results: Vec::new(),
            last_captured_textures: Vec::new(),
        }
    }

    pub fn last_pass_timings(&self) -> &[crate::plugins::render::inspect::PassTimingSample] {
        &self.last_pass_timings
    }

    pub fn last_gpu_pass_timing_evidence(
        &self,
    ) -> &[crate::plugins::render::inspect::RenderPassTimingEvidence] {
        &self.last_gpu_pass_timing_evidence
    }

    pub fn last_runtime_resources(
        &self,
    ) -> &[crate::plugins::render::inspect::RuntimeResourceInspectionEntry] {
        &self.last_runtime_resources
    }

    pub fn last_pass_provenance(
        &self,
    ) -> &[crate::plugins::render::inspect::RenderPassProvenanceRecord] {
        &self.last_pass_provenance
    }

    pub fn last_preflight_report(
        &self,
    ) -> &crate::plugins::render::graph::RenderExecutionGraphPreparedReport {
        &self.last_preflight_report
    }

    pub fn last_preflight_cache_state(
        &self,
    ) -> &crate::plugins::render::graph::RenderPreparedFramePreflightCacheState {
        &self.last_preflight_cache_state
    }

    pub fn last_capture_plan(&self) -> &crate::plugins::render::inspect::ResolvedRenderCapturePlan {
        &self.last_capture_plan
    }

    pub fn last_capture_selector_results(
        &self,
    ) -> &[crate::plugins::render::inspect::RenderCaptureSelectorResult] {
        &self.last_capture_selector_results
    }

    pub fn last_captured_textures(
        &self,
    ) -> &[crate::plugins::render::inspect::RenderCapturedTexture] {
        &self.last_captured_textures
    }

    pub fn flow_pipeline_cache_stats(&self) -> super::pipeline_cache::RendererPipelineCacheStats {
        self.flow_pipeline_cache.stats()
    }

    #[allow(clippy::too_many_arguments)]
    fn realize_ui_program_binding_artifacts(
        &mut self,
        context: &GpuContext,
        kind: UiPipelineKind,
        format: TextureFormat,
        canonical_wgsl: &str,
        source_key: &'static str,
        source_revision: u64,
        screen_label: &'static str,
        texture_sampler_label: Option<&'static str>,
    ) -> Result<UiProgramBindingArtifacts> {
        // The renderer realizes the full G4C1/G4C2/G4C3 UI dependency chain before G5 begins.
        let screen_buffer = self.realize_buffer_resource(
            context,
            screen_label,
            std::mem::size_of::<ScreenUniformRaw>() as u64,
            [GpuBufferUsage::Uniform, GpuBufferUsage::CopyDestination],
            GpuResourceLifetime::Retained,
            GpuMemoryIntent::Device,
        )?;
        let texture_sampler = texture_sampler_label
            .map(|label| {
                self.realize_linear_sampler_resource(context, label, GpuResourceLifetime::Retained)
            })
            .transpose()?;

        let screen_layout_descriptor = ui_screen_bind_group_layout()?;
        let texture_layout_descriptor = texture_sampler_label
            .map(|_| ui_texture_bind_group_layout())
            .transpose()?;
        let pipeline_layout_descriptor = GpuPipelineLayoutDescriptor::new(
            std::iter::once(screen_layout_descriptor.clone())
                .chain(texture_layout_descriptor.iter().cloned()),
        )?;
        let interface = GpuProgramInterfaceDescriptor::new(
            pipeline_layout_descriptor
                .groups()
                .flat_map(|group| group.bindings().cloned()),
        )?;
        let admitted_source = self.flow_pipeline_cache.admit_program_source(
            GpuProgramSourceKey::new(source_key)?,
            source_revision,
            canonical_wgsl,
            GpuProgramSourceProvenance::new("renderer-ui-pipeline", Some(source_key.to_owned()))?,
        )?;
        let vertex_entry = GpuEntryPointName::new("vs_main")?;
        let fragment_entry = GpuEntryPointName::new("fs_main")?;
        let program_descriptor = GpuProgramDescriptor::new(
            admitted_source,
            interface.clone(),
            [
                GpuEntryPointDescriptor::new(
                    vertex_entry.clone(),
                    GpuShaderStage::Vertex,
                    interface.clone(),
                ),
                GpuEntryPointDescriptor::new(
                    fragment_entry.clone(),
                    GpuShaderStage::Fragment,
                    interface,
                ),
            ],
        )?;
        let pipeline_descriptor = ui_render_pipeline_descriptor(
            kind,
            format,
            program_descriptor.clone(),
            pipeline_layout_descriptor.clone(),
            vertex_entry,
            fragment_entry,
        )?;

        let program = pollster::block_on(context.realize_program(&program_descriptor))?;
        let screen_layout =
            pollster::block_on(context.realize_bind_group_layout(&screen_layout_descriptor))?;
        let texture_layout = texture_layout_descriptor
            .as_ref()
            .map(|layout| pollster::block_on(context.realize_bind_group_layout(layout)))
            .transpose()?;
        let pipeline_layout =
            pollster::block_on(context.realize_pipeline_layout(&pipeline_layout_descriptor))?;
        let pipeline = pollster::block_on(context.realize_render_pipeline(
            &pipeline_descriptor,
            &program,
            &pipeline_layout,
        ))?;
        let screen_size = NonZeroU64::new(screen_buffer._handle.descriptor().size_bytes())
            .expect("screen uniform buffers are nonempty");
        let screen_binding = GpuRuntimeBindingValue::new(
            GpuBindingKey::try_new(0, 0)?,
            [GpuRuntimeBindingResource::Buffer(
                GpuRuntimeBufferBinding::new(screen_buffer._handle.clone(), 0, screen_size, None),
            )],
        )?;
        let screen_bind_group = pollster::block_on(
            context.realize_bind_group(&screen_layout, [screen_binding.clone()]),
        )?;

        Ok(UiProgramBindingArtifacts {
            pipeline,
            screen_buffer,
            screen_binding,
            screen_bind_group,
            texture_bind_group_layout: texture_layout,
            texture_sampler,
        })
    }

    pub(super) fn ensure_rect_pass(
        &mut self,
        context: &GpuContext,
        format: TextureFormat,
        shader_source: &str,
        shader_revision: u64,
    ) -> Result<()> {
        if self.rect_pass.is_some()
            && self.rect_pass_format == Some(format)
            && self.rect_pass_shader_revision == shader_revision
        {
            return Ok(());
        }
        let artifacts = self.realize_ui_program_binding_artifacts(
            context,
            UiPipelineKind::Rect,
            format,
            shader_source,
            "ui:rect",
            shader_revision,
            "engine_ui_screen_uniform",
            None,
        )?;

        self.rect_pass = Some(RectPass {
            pipeline: artifacts.pipeline,
            screen_buffer: artifacts.screen_buffer,
            screen_binding: artifacts.screen_binding,
            screen_bind_group: artifacts.screen_bind_group,
        });
        self.rect_pass_format = Some(format);
        self.rect_pass_shader_revision = shader_revision;
        Ok(())
    }

    pub(super) fn ensure_stroke_pass(
        &mut self,
        context: &GpuContext,
        format: TextureFormat,
    ) -> Result<()> {
        if self.stroke_pass.is_some() && self.stroke_pass_format == Some(format) {
            return Ok(());
        }
        let artifacts = self.realize_ui_program_binding_artifacts(
            context,
            UiPipelineKind::Stroke,
            format,
            DEFAULT_UI_STROKE_SHADER,
            "ui:stroke",
            0,
            "engine_ui_stroke_screen_uniform",
            None,
        )?;

        self.stroke_pass = Some(StrokePass {
            pipeline: artifacts.pipeline,
            screen_buffer: artifacts.screen_buffer,
            screen_binding: artifacts.screen_binding,
            screen_bind_group: artifacts.screen_bind_group,
        });
        self.stroke_pass_format = Some(format);
        Ok(())
    }

    pub(super) fn ensure_glyph_pass(
        &mut self,
        context: &GpuContext,
        format: TextureFormat,
    ) -> Result<()> {
        if self.glyph_pass.is_some() && self.glyph_pass_format == Some(format) {
            return Ok(());
        }
        let artifacts = self.realize_ui_program_binding_artifacts(
            context,
            UiPipelineKind::Glyph,
            format,
            DEFAULT_UI_GLYPH_SHADER,
            "ui:glyph",
            0,
            "engine_ui_glyph_screen_uniform",
            Some("engine_ui_glyph_sampler"),
        )?;

        self.glyph_pass = Some(GlyphPass {
            pipeline: artifacts.pipeline,
            screen_buffer: artifacts.screen_buffer,
            screen_binding: artifacts.screen_binding,
            screen_bind_group: artifacts.screen_bind_group,
            texture_bind_group_layout: artifacts
                .texture_bind_group_layout
                .expect("glyph pipeline should realize a texture bind-group layout"),
            texture_sampler: artifacts
                .texture_sampler
                .expect("glyph pipeline should realize a texture sampler"),
        });
        self.glyph_pass_format = Some(format);
        self.glyph_atlas_gpu.clear();
        Ok(())
    }

    pub(super) fn ensure_viewport_embed_pass(
        &mut self,
        context: &GpuContext,
        format: TextureFormat,
    ) -> Result<()> {
        if self.viewport_embed_pass.is_some() && self.viewport_embed_pass_format == Some(format) {
            return Ok(());
        }
        let artifacts = self.realize_ui_program_binding_artifacts(
            context,
            UiPipelineKind::ViewportEmbed,
            format,
            DEFAULT_UI_VIEWPORT_EMBED_SHADER,
            "ui:viewport-embed",
            0,
            "engine_ui_viewport_embed_screen_uniform",
            Some("engine_ui_viewport_embed_sampler"),
        )?;

        self.viewport_embed_pass = Some(ViewportEmbedPass {
            pipeline: artifacts.pipeline,
            screen_buffer: artifacts.screen_buffer,
            screen_binding: artifacts.screen_binding,
            screen_bind_group: artifacts.screen_bind_group,
            texture_bind_group_layout: artifacts
                .texture_bind_group_layout
                .expect("viewport pipeline should realize a texture bind-group layout"),
            texture_sampler: artifacts
                .texture_sampler
                .expect("viewport pipeline should realize a texture sampler"),
        });
        self.viewport_embed_pass_format = Some(format);
        Ok(())
    }

    pub(super) fn ensure_product_surface_pass(
        &mut self,
        context: &GpuContext,
        format: TextureFormat,
    ) -> Result<()> {
        if self.product_surface_pass.is_some() && self.product_surface_pass_format == Some(format) {
            return Ok(());
        }
        let artifacts = self.realize_ui_program_binding_artifacts(
            context,
            UiPipelineKind::ProductSurface,
            format,
            DEFAULT_UI_PRODUCT_SURFACE_SHADER,
            "ui:product-surface",
            0,
            "engine_ui_product_surface_screen_uniform",
            Some("engine_ui_product_surface_sampler"),
        )?;

        self.product_surface_pass = Some(ProductSurfacePass {
            pipeline: artifacts.pipeline,
            screen_buffer: artifacts.screen_buffer,
            screen_binding: artifacts.screen_binding,
            screen_bind_group: artifacts.screen_bind_group,
            texture_bind_group_layout: artifacts
                .texture_bind_group_layout
                .expect("product-surface pipeline should realize a texture bind-group layout"),
            texture_sampler: artifacts
                .texture_sampler
                .expect("product-surface pipeline should realize a texture sampler"),
        });
        self.product_surface_pass_format = Some(format);
        Ok(())
    }

    pub(super) fn ensure_glyph_atlas_gpu(
        &mut self,
        context: &GpuContext,
        atlas: &crate::plugins::render::features::UiFontAtlasResource,
        texture_id: u64,
        pending_operations: &mut RendererPendingOperations,
    ) -> Result<bool> {
        if self.glyph_atlas_gpu.contains_key(&texture_id) {
            return Ok(true);
        }

        let Some(glyph_pass) = self.glyph_pass.as_ref() else {
            return Ok(false);
        };
        let texture_bind_group_layout = glyph_pass.texture_bind_group_layout.clone();
        let texture_sampler = glyph_pass.texture_sampler.clone();
        let Some((_, atlas_image)) = atlas.atlas_for_texture_id(texture_id) else {
            return Ok(false);
        };
        let texture_handle = self
            .resource_ids
            .allocate_texture_handle(texture_descriptor(
                "engine_ui_glyph_atlas_texture",
                (atlas_image.width.max(1), atlas_image.height.max(1)),
                GpuTextureFormat::R8Unorm,
                [GpuTextureUsage::Sampled, GpuTextureUsage::CopyDestination],
                GpuResourceLifetime::Retained,
            )?)?;
        let texture = RendererTextureResource {
            realized: context.realize_texture(&texture_handle)?,
            _handle: texture_handle,
        };

        let view_handle =
            self.resource_ids
                .allocate_texture_view_handle(whole_texture_view_descriptor(
                    "engine_ui_glyph_atlas_view",
                    &texture._handle,
                )?)?;
        let view = RendererTextureViewResource {
            _realized: context.realize_texture_view(&view_handle, &texture.realized)?,
            _handle: view_handle,
        };
        let logical_values =
            ui_texture_bind_group_values(view._handle.clone(), texture_sampler._handle.clone())?;
        let bind_group = pollster::block_on(
            context.realize_bind_group(&texture_bind_group_layout, logical_values.clone()),
        )?;

        // The glyph texture/view/sampler resources and bind group are authoritative before G5.
        // Retain the canonical upload operation plus only the temporary realized-object sidecar
        // required until the live frame moves to G5 submission.
        pending_operations.queue_texture(&texture, &atlas_image.pixels)?;

        self.glyph_atlas_gpu.insert(
            texture_id,
            UiGlyphAtlasGpu {
                _texture: texture,
                _view: view,
                texture_bindings: UiTextureBindings {
                    logical_values,
                    realized: bind_group,
                },
            },
        );
        Ok(true)
    }

    pub(super) fn full_scissor(surface_width: u32, surface_height: u32) -> (u32, u32, u32, u32) {
        (0, 0, surface_width.max(1), surface_height.max(1))
    }

    pub(super) fn clip_to_scissor(
        clip: [f32; 4],
        surface_width: u32,
        surface_height: u32,
    ) -> Option<(u32, u32, u32, u32)> {
        let max_x = surface_width.max(1) as i32;
        let max_y = surface_height.max(1) as i32;

        let x0 = (clip[0].floor() as i32).clamp(0, max_x);
        let y0 = (clip[1].floor() as i32).clamp(0, max_y);
        let x1 = ((clip[0] + clip[2]).ceil() as i32).clamp(0, max_x);
        let y1 = ((clip[1] + clip[3]).ceil() as i32).clamp(0, max_y);

        if x1 <= x0 || y1 <= y0 {
            return None;
        }

        Some((x0 as u32, y0 as u32, (x1 - x0) as u32, (y1 - y0) as u32))
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn encode_ui_pass(
        &self,
        context: &GpuContext,
        encoder: &mut CommandEncoder,
        frame_view: &TextureView,
        prepared: &UiPreparedDraws,
        viewport_surface_bindings: &ViewportSurfaceBindingRegistry,
        viewport_bind_groups: &UiViewportBindGroups,
        product_surface_bind_groups: &UiProductSurfaceBindGroups,
        gpu_timestamp_writes: Option<super::render_flow::GpuPassTimestampWrites>,
    ) -> Result<()> {
        let Some(rect_pass) = self.rect_pass.as_ref() else {
            return Ok(());
        };

        let mut realized = vec![&rect_pass.pipeline];
        let mut slots = UiPipelineSlots {
            rect: 0,
            ..UiPipelineSlots::default()
        };
        if let Some(pass) = self.stroke_pass.as_ref() {
            slots.stroke = Some(realized.len());
            realized.push(&pass.pipeline);
        }
        if let Some(pass) = self.glyph_pass.as_ref() {
            slots.glyph = Some(realized.len());
            realized.push(&pass.pipeline);
        }
        if let Some(pass) = self.viewport_embed_pass.as_ref() {
            slots.viewport_embed = Some(realized.len());
            realized.push(&pass.pipeline);
        }
        if let Some(pass) = self.product_surface_pass.as_ref() {
            slots.product_surface = Some(realized.len());
            realized.push(&pass.pipeline);
        }

        let mut output = Ok(());
        context
            .current_render_execution_bridge()
            .for_render_pipelines(
                &realized,
                EncodeUiPipelines {
                    renderer: self,
                    context,
                    encoder,
                    frame_view,
                    prepared,
                    viewport_surface_bindings,
                    viewport_bind_groups,
                    product_surface_bind_groups,
                    slots,
                    gpu_timestamp_writes,
                    output: &mut output,
                },
            )?;
        output
    }

    #[allow(clippy::too_many_arguments)]
    fn encode_ui_pass_current(
        &self,
        context: &GpuContext,
        encoder: &mut CommandEncoder,
        frame_view: &TextureView,
        prepared: &UiPreparedDraws,
        viewport_surface_bindings: &ViewportSurfaceBindingRegistry,
        viewport_bind_groups: &UiViewportBindGroups,
        product_surface_bind_groups: &UiProductSurfaceBindGroups,
        pipelines: &[&RenderPipeline],
        slots: UiPipelineSlots,
        timestamp: Option<(&QuerySet, super::render_flow::GpuPassTimestampIndices)>,
    ) -> Result<()> {
        let Some(rect_pass) = self.rect_pass.as_ref() else {
            return Ok(());
        };

        let timestamp_writes = timestamp.map(|(query_set, indices)| RenderPassTimestampWrites {
            query_set,
            beginning_of_pass_write_index: Some(indices.begin),
            end_of_pass_write_index: Some(indices.end),
        });
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("engine_ui_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: frame_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        for command in &prepared.draw_plan {
            match *command {
                UiPreparedDrawCommand::Rect(index) => {
                    let Some(batch) = prepared.rect_batches.get(index) else {
                        continue;
                    };
                    pass.set_pipeline(pipelines[slots.rect]);
                    set_ui_bind_group(
                        context,
                        &mut pass,
                        0,
                        std::slice::from_ref(&rect_pass.screen_binding),
                        &rect_pass.screen_bind_group,
                    )?;
                    pass.set_scissor_rect(
                        batch.scissor.0,
                        batch.scissor.1,
                        batch.scissor.2,
                        batch.scissor.3,
                    );
                    context
                        .current_render_execution_bridge()
                        .for_vertex_buffer(
                            &batch.instance_buffer.realized,
                            DrawUiInstances {
                                pass: &mut pass,
                                instance_count: batch.instance_count,
                            },
                        )?;
                }
                UiPreparedDrawCommand::Stroke(index) => {
                    let Some(stroke_pass) = self.stroke_pass.as_ref() else {
                        continue;
                    };
                    let Some(slot) = slots.stroke else {
                        continue;
                    };
                    let Some(batch) = prepared.stroke_batches.get(index) else {
                        continue;
                    };
                    pass.set_pipeline(pipelines[slot]);
                    set_ui_bind_group(
                        context,
                        &mut pass,
                        0,
                        std::slice::from_ref(&stroke_pass.screen_binding),
                        &stroke_pass.screen_bind_group,
                    )?;
                    pass.set_scissor_rect(
                        batch.scissor.0,
                        batch.scissor.1,
                        batch.scissor.2,
                        batch.scissor.3,
                    );
                    context
                        .current_render_execution_bridge()
                        .for_vertex_buffer(
                            &batch.instance_buffer.realized,
                            DrawUiInstances {
                                pass: &mut pass,
                                instance_count: batch.instance_count,
                            },
                        )?;
                }
                UiPreparedDrawCommand::ViewportEmbed(index) => {
                    let Some(viewport_embed_pass) = self.viewport_embed_pass.as_ref() else {
                        continue;
                    };
                    let Some(slot) = slots.viewport_embed else {
                        continue;
                    };
                    let Some(batch) = prepared.viewport_embed_batches.get(index) else {
                        continue;
                    };
                    pass.set_pipeline(pipelines[slot]);
                    set_ui_bind_group(
                        context,
                        &mut pass,
                        0,
                        std::slice::from_ref(&viewport_embed_pass.screen_binding),
                        &viewport_embed_pass.screen_bind_group,
                    )?;
                    let Some(binding) =
                        viewport_surface_bindings.get(batch.viewport_id, batch.slot)
                    else {
                        continue;
                    };

                    let Some(bindings) = viewport_bind_groups.get(&binding.source) else {
                        continue;
                    };
                    set_ui_bind_group(
                        context,
                        &mut pass,
                        1,
                        &bindings.logical_values,
                        &bindings.realized,
                    )?;
                    pass.set_scissor_rect(
                        batch.scissor.0,
                        batch.scissor.1,
                        batch.scissor.2,
                        batch.scissor.3,
                    );
                    context
                        .current_render_execution_bridge()
                        .for_vertex_buffer(
                            &batch.instance_buffer.realized,
                            DrawUiInstances {
                                pass: &mut pass,
                                instance_count: batch.instance_count,
                            },
                        )?;
                }
                UiPreparedDrawCommand::ProductSurface(index) => {
                    let Some(product_surface_pass) = self.product_surface_pass.as_ref() else {
                        continue;
                    };
                    let Some(slot) = slots.product_surface else {
                        continue;
                    };
                    let Some(batch) = prepared.product_surface_batches.get(index) else {
                        continue;
                    };
                    pass.set_pipeline(pipelines[slot]);
                    set_ui_bind_group(
                        context,
                        &mut pass,
                        0,
                        std::slice::from_ref(&product_surface_pass.screen_binding),
                        &product_surface_pass.screen_bind_group,
                    )?;

                    let Some(bindings) = product_surface_bind_groups.get(&batch.source) else {
                        continue;
                    };
                    set_ui_bind_group(
                        context,
                        &mut pass,
                        1,
                        &bindings.logical_values,
                        &bindings.realized,
                    )?;
                    pass.set_scissor_rect(
                        batch.scissor.0,
                        batch.scissor.1,
                        batch.scissor.2,
                        batch.scissor.3,
                    );
                    context
                        .current_render_execution_bridge()
                        .for_vertex_buffer(
                            &batch.instance_buffer.realized,
                            DrawUiInstances {
                                pass: &mut pass,
                                instance_count: batch.instance_count,
                            },
                        )?;
                }
                UiPreparedDrawCommand::Glyph(index) => {
                    let Some(glyph_pass) = self.glyph_pass.as_ref() else {
                        continue;
                    };
                    let Some(slot) = slots.glyph else {
                        continue;
                    };
                    let Some(batch) = prepared.glyph_batches.get(index) else {
                        continue;
                    };
                    pass.set_pipeline(pipelines[slot]);
                    set_ui_bind_group(
                        context,
                        &mut pass,
                        0,
                        std::slice::from_ref(&glyph_pass.screen_binding),
                        &glyph_pass.screen_bind_group,
                    )?;
                    let Some(atlas_gpu) = self.glyph_atlas_gpu.get(&batch.texture_id) else {
                        continue;
                    };
                    set_ui_bind_group(
                        context,
                        &mut pass,
                        1,
                        &atlas_gpu.texture_bindings.logical_values,
                        &atlas_gpu.texture_bindings.realized,
                    )?;
                    pass.set_scissor_rect(
                        batch.scissor.0,
                        batch.scissor.1,
                        batch.scissor.2,
                        batch.scissor.3,
                    );
                    context
                        .current_render_execution_bridge()
                        .for_vertex_buffer(
                            &batch.instance_buffer.realized,
                            DrawUiInstances {
                                pass: &mut pass,
                                instance_count: batch.instance_count,
                            },
                        )?;
                }
            }
        }
        Ok(())
    }

    pub(super) fn realize_ui_dynamic_bind_groups(
        &self,
        context: &GpuContext,
        prepared: &UiPreparedDraws,
        viewport_surface_bindings: &ViewportSurfaceBindingRegistry,
    ) -> Result<(UiViewportBindGroups, UiProductSurfaceBindGroups)> {
        let mut viewport_bind_groups = UiViewportBindGroups::new();
        let mut product_surface_bind_groups = UiProductSurfaceBindGroups::new();

        for command in &prepared.draw_plan {
            match *command {
                UiPreparedDrawCommand::ViewportEmbed(index) => {
                    let Some(viewport_embed_pass) = self.viewport_embed_pass.as_ref() else {
                        continue;
                    };
                    let Some(batch) = prepared.viewport_embed_batches.get(index) else {
                        continue;
                    };
                    let Some(binding) =
                        viewport_surface_bindings.get(batch.viewport_id, batch.slot)
                    else {
                        continue;
                    };
                    if viewport_bind_groups.contains_key(&binding.source) {
                        continue;
                    }
                    let ViewportSurfaceBindingSource::DynamicTexture {
                        namespace,
                        target_id,
                    } = &binding.source;
                    let key = crate::plugins::render::RenderDynamicTextureTargetKey::new(
                        namespace.clone(),
                        target_id.clone(),
                    );
                    let Ok(view_handle) = self.dynamic_texture_targets.ui_texture_view_handle(&key)
                    else {
                        continue;
                    };
                    let logical_values = ui_texture_bind_group_values(
                        view_handle,
                        viewport_embed_pass.texture_sampler._handle.clone(),
                    )?;
                    let bind_group = pollster::block_on(context.realize_bind_group(
                        &viewport_embed_pass.texture_bind_group_layout,
                        logical_values.clone(),
                    ))?;
                    viewport_bind_groups.insert(
                        binding.source.clone(),
                        UiTextureBindings {
                            logical_values,
                            realized: bind_group,
                        },
                    );
                }
                UiPreparedDrawCommand::ProductSurface(index) => {
                    let Some(product_surface_pass) = self.product_surface_pass.as_ref() else {
                        continue;
                    };
                    let Some(batch) = prepared.product_surface_batches.get(index) else {
                        continue;
                    };
                    if product_surface_bind_groups.contains_key(&batch.source) {
                        continue;
                    }
                    let ProductSurfaceTextureBindingSource::DynamicTexture {
                        namespace,
                        target_id,
                    } = &batch.source;
                    let key = crate::plugins::render::RenderDynamicTextureTargetKey::new(
                        namespace.clone(),
                        target_id.clone(),
                    );
                    let Ok(view_handle) = self.dynamic_texture_targets.ui_texture_view_handle(&key)
                    else {
                        continue;
                    };
                    let logical_values = ui_texture_bind_group_values(
                        view_handle,
                        product_surface_pass.texture_sampler._handle.clone(),
                    )?;
                    let bind_group = pollster::block_on(context.realize_bind_group(
                        &product_surface_pass.texture_bind_group_layout,
                        logical_values.clone(),
                    ))?;
                    product_surface_bind_groups.insert(
                        batch.source.clone(),
                        UiTextureBindings {
                            logical_values,
                            realized: bind_group,
                        },
                    );
                }
                UiPreparedDrawCommand::Rect(_)
                | UiPreparedDrawCommand::Stroke(_)
                | UiPreparedDrawCommand::Glyph(_) => {}
            }
        }

        Ok((viewport_bind_groups, product_surface_bind_groups))
    }
}

#[derive(Debug)]
struct UiProgramBindingArtifacts {
    pipeline: GpuRealizedRenderPipeline,
    screen_buffer: RendererBufferResource,
    screen_binding: GpuRuntimeBindingValue,
    screen_bind_group: GpuRealizedBindGroup,
    texture_bind_group_layout: Option<GpuRealizedBindGroupLayout>,
    texture_sampler: Option<RendererSamplerResource>,
}

fn ui_screen_bind_group_layout() -> Result<GpuBindGroupLayoutDescriptor> {
    Ok(GpuBindGroupLayoutDescriptor::new(
        0,
        [GpuBindingDeclaration::new(
            GpuBindingKey::try_new(0, 0)?,
            GpuShaderStages::one(GpuShaderStage::Vertex),
            GpuBindingKind::uniform_buffer(false, None),
            None,
            "ui-screen-uniform",
            GpuBindingProvenance::new("renderer-ui-pipeline", Some("screen".to_owned()))?,
        )?],
    )?)
}

fn ui_texture_bind_group_layout() -> Result<GpuBindGroupLayoutDescriptor> {
    Ok(GpuBindGroupLayoutDescriptor::new(
        1,
        [
            GpuBindingDeclaration::new(
                GpuBindingKey::try_new(1, 0)?,
                GpuShaderStages::one(GpuShaderStage::Fragment),
                GpuBindingKind::sampled_texture(
                    GpuTextureSampleClass::FloatFilterable,
                    GpuTextureViewDimension::D2,
                    false,
                )?,
                None,
                "ui-sampled-texture",
                GpuBindingProvenance::new(
                    "renderer-ui-pipeline",
                    Some("sampled texture".to_owned()),
                )?,
            )?,
            GpuBindingDeclaration::new(
                GpuBindingKey::try_new(1, 1)?,
                GpuShaderStages::one(GpuShaderStage::Fragment),
                GpuBindingKind::sampler(GpuSamplerClass::Filtering),
                None,
                "ui-texture-sampler",
                GpuBindingProvenance::new(
                    "renderer-ui-pipeline",
                    Some("texture sampler".to_owned()),
                )?,
            )?,
        ],
    )?)
}

fn ui_render_pipeline_descriptor(
    kind: UiPipelineKind,
    format: TextureFormat,
    program: GpuProgramDescriptor,
    layout: GpuPipelineLayoutDescriptor,
    vertex_entry: GpuEntryPointName,
    fragment_entry: GpuEntryPointName,
) -> Result<GpuRenderPipelineDescriptor> {
    let vertex_layout = GpuVertexBufferLayoutDescriptor::new(
        0,
        ui_pipeline_stride(kind),
        GpuVertexStepMode::Instance,
        ui_pipeline_attributes(kind),
    )?;
    let vertex_input = GpuVertexInputStateDescriptor::new([vertex_layout])?;
    let color_target = GpuColorTargetStateDescriptor::new(
        gpu_texture_format(format)?,
        GpuBlendMode::Alpha,
        GpuColorWriteMask::ALL,
    )?;
    let state = GpuRenderPipelineStateDescriptor::new(
        vertex_input,
        Some(GpuFragmentOutputStateDescriptor::new([color_target])),
        GpuPrimitiveStateDescriptor::default(),
        None,
        GpuMultisampleStateDescriptor::default(),
    )?;
    let specialization = GpuSpecializationValueSet::new(GpuSpecializationSchema::new([])?, [])?;
    Ok(GpuRenderPipelineDescriptor::new(
        program,
        GpuRenderEntryPoints::new(vertex_entry, Some(fragment_entry)),
        state,
        layout,
        specialization,
        GpuCapabilityRequirements::new(),
    )?)
}

fn ui_pipeline_stride(kind: UiPipelineKind) -> u64 {
    match kind {
        UiPipelineKind::Rect => std::mem::size_of::<RectInstanceRaw>() as u64,
        UiPipelineKind::Stroke => std::mem::size_of::<StrokeSegmentInstanceRaw>() as u64,
        UiPipelineKind::Glyph => std::mem::size_of::<GlyphInstanceRaw>() as u64,
        UiPipelineKind::ViewportEmbed | UiPipelineKind::ProductSurface => {
            std::mem::size_of::<ViewportEmbedInstanceRaw>() as u64
        }
    }
}

fn ui_pipeline_attributes(kind: UiPipelineKind) -> Vec<GpuVertexAttribute> {
    match kind {
        UiPipelineKind::Rect => vec![
            GpuVertexAttribute::new(0, 0, GpuVertexFormat::Float32x4),
            GpuVertexAttribute::new(1, 16, GpuVertexFormat::Float32x4),
            GpuVertexAttribute::new(2, 32, GpuVertexFormat::Float32),
            GpuVertexAttribute::new(3, 36, GpuVertexFormat::Float32),
        ],
        UiPipelineKind::Stroke => vec![
            GpuVertexAttribute::new(0, 0, GpuVertexFormat::Float32x2),
            GpuVertexAttribute::new(1, 8, GpuVertexFormat::Float32x2),
            GpuVertexAttribute::new(2, 16, GpuVertexFormat::Float32x4),
            GpuVertexAttribute::new(3, 32, GpuVertexFormat::Float32),
        ],
        UiPipelineKind::Glyph | UiPipelineKind::ViewportEmbed | UiPipelineKind::ProductSurface => {
            vec![
                GpuVertexAttribute::new(0, 0, GpuVertexFormat::Float32x4),
                GpuVertexAttribute::new(1, 16, GpuVertexFormat::Float32x4),
                GpuVertexAttribute::new(2, 32, GpuVertexFormat::Float32x4),
            ]
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct UiPipelineSlots {
    rect: usize,
    stroke: Option<usize>,
    glyph: Option<usize>,
    viewport_embed: Option<usize>,
    product_surface: Option<usize>,
}

struct EncodeUiPipelines<'a, 'bindings> {
    renderer: &'a Renderer,
    context: &'a GpuContext,
    encoder: &'a mut CommandEncoder,
    frame_view: &'a TextureView,
    prepared: &'a UiPreparedDraws,
    viewport_surface_bindings: &'a ViewportSurfaceBindingRegistry,
    viewport_bind_groups: &'bindings UiViewportBindGroups,
    product_surface_bind_groups: &'bindings UiProductSurfaceBindGroups,
    slots: UiPipelineSlots,
    gpu_timestamp_writes: Option<super::render_flow::GpuPassTimestampWrites>,
    output: &'a mut Result<()>,
}

impl CurrentRenderRenderPipelinesTerminal for EncodeUiPipelines<'_, '_> {
    fn use_render_pipelines(self, pipelines: &[&RenderPipeline]) {
        if let Some(writes) = self.gpu_timestamp_writes {
            let mut nested_result = Ok(());
            let bridge_result = self
                .context
                .current_render_execution_bridge()
                .for_timestamp_writes(
                    &writes.query_set,
                    EncodeTimestampedUiPass {
                        renderer: self.renderer,
                        context: self.context,
                        encoder: self.encoder,
                        frame_view: self.frame_view,
                        prepared: self.prepared,
                        viewport_surface_bindings: self.viewport_surface_bindings,
                        viewport_bind_groups: self.viewport_bind_groups,
                        product_surface_bind_groups: self.product_surface_bind_groups,
                        pipelines,
                        slots: self.slots,
                        indices: writes.indices,
                        output: &mut nested_result,
                    },
                );
            *self.output = match bridge_result {
                Ok(()) => nested_result,
                Err(error) => Err(error.into()),
            };
        } else {
            *self.output = self.renderer.encode_ui_pass_current(
                self.context,
                self.encoder,
                self.frame_view,
                self.prepared,
                self.viewport_surface_bindings,
                self.viewport_bind_groups,
                self.product_surface_bind_groups,
                pipelines,
                self.slots,
                None,
            );
        }
    }
}

struct EncodeTimestampedUiPass<'a, 'bindings, 'pipelines> {
    renderer: &'a Renderer,
    context: &'a GpuContext,
    encoder: &'a mut CommandEncoder,
    frame_view: &'a TextureView,
    prepared: &'a UiPreparedDraws,
    viewport_surface_bindings: &'a ViewportSurfaceBindingRegistry,
    viewport_bind_groups: &'bindings UiViewportBindGroups,
    product_surface_bind_groups: &'bindings UiProductSurfaceBindGroups,
    pipelines: &'pipelines [&'pipelines RenderPipeline],
    slots: UiPipelineSlots,
    indices: super::render_flow::GpuPassTimestampIndices,
    output: &'a mut Result<()>,
}

impl CurrentRenderTimestampWritesTerminal for EncodeTimestampedUiPass<'_, '_, '_> {
    fn write_timestamps(self, query_set: &QuerySet) {
        *self.output = self.renderer.encode_ui_pass_current(
            self.context,
            self.encoder,
            self.frame_view,
            self.prepared,
            self.viewport_surface_bindings,
            self.viewport_bind_groups,
            self.product_surface_bind_groups,
            self.pipelines,
            self.slots,
            Some((query_set, self.indices)),
        );
    }
}

struct DrawUiInstances<'a, 'pass> {
    pass: &'a mut RenderPass<'pass>,
    instance_count: u32,
}

impl CurrentRenderVertexBufferTerminal for DrawUiInstances<'_, '_> {
    fn use_vertex_buffer(self, buffer: &Buffer) {
        self.pass.set_vertex_buffer(0, buffer.slice(..));
        self.pass.draw(0..6, 0..self.instance_count);
    }
}

fn ui_texture_bind_group_values(
    view: GpuTextureViewHandle,
    sampler: GpuSamplerHandle,
) -> Result<[GpuRuntimeBindingValue; 2]> {
    Ok([
        GpuRuntimeBindingValue::new(
            GpuBindingKey::try_new(1, 0)?,
            [GpuRuntimeBindingResource::TextureView(
                GpuRuntimeTextureViewBinding::new(view, GpuTextureViewDimension::D2),
            )],
        )?,
        GpuRuntimeBindingValue::new(
            GpuBindingKey::try_new(1, 1)?,
            [GpuRuntimeBindingResource::Sampler(sampler)],
        )?,
    ])
}

fn set_ui_bind_group(
    context: &GpuContext,
    pass: &mut RenderPass<'_>,
    index: u32,
    logical_values: &[GpuRuntimeBindingValue],
    bind_group: &GpuRealizedBindGroup,
) -> Result<()> {
    if bind_group.layout_descriptor().group() != index {
        return Err(anyhow::anyhow!(
            "UI bind-group slot {index} disagrees with realized layout group {}",
            bind_group.layout_descriptor().group()
        ));
    }
    let expected_keys = bind_group
        .layout_descriptor()
        .bindings()
        .map(|binding| binding.key())
        .collect::<Vec<_>>();
    let logical_keys = logical_values
        .iter()
        .map(GpuRuntimeBindingValue::key)
        .collect::<Vec<_>>();
    if logical_keys != expected_keys {
        return Err(anyhow::anyhow!(
            "UI logical binding keys {logical_keys:?} disagree with realized group-{index} layout keys {expected_keys:?}"
        ));
    }
    context
        .current_render_execution_bridge()
        .for_pipeline_bind_groups(&[bind_group], SetUiBindGroup { pass, index })?;
    Ok(())
}

struct SetUiBindGroup<'a, 'pass> {
    pass: &'a mut RenderPass<'pass>,
    index: u32,
}

impl CurrentRenderPipelineBindGroupsTerminal for SetUiBindGroup<'_, '_> {
    fn bind_groups(self, groups: &[&BindGroup]) {
        let bind_group = groups
            .first()
            .expect("G4C2 UI bind-group bridge should receive one bind group");
        self.pass.set_bind_group(self.index, *bind_group, &[]);
    }
}
