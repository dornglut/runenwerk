use super::*;
use crate::plugins::render::features::{
    MATERIAL_RENDER_FEATURE_ID, UI_RENDER_FEATURE_ID, UiFontAtlasResource,
};
use crate::plugins::render::texture_upload::load_material_ktx2_upload;
use crate::plugins::{PreparedUiFrameContribution, RenderFeatureId};
use std::hash::{Hash, Hasher};

type ScissoredRectBatch = (u32, u32, u32, (u32, u32, u32, u32), Vec<RectInstanceRaw>);
type ScissoredStrokeBatch = (
    u32,
    u32,
    u32,
    (u32, u32, u32, u32),
    Vec<StrokeSegmentInstanceRaw>,
);
type ScissoredViewportEmbedBatch = (
    u32,
    u32,
    u32,
    (u32, u32, u32, u32),
    u64,
    ViewportSurfaceEmbedSlotId,
    Vec<ViewportEmbedInstanceRaw>,
);
type ScissoredProductSurfaceBatch = (
    u32,
    u32,
    u32,
    (u32, u32, u32, u32),
    ProductSurfaceTextureBindingSource,
    Vec<ViewportEmbedInstanceRaw>,
);

impl Renderer {
    pub(super) fn prepare_ui_draws(
        &mut self,
        device: &Device,
        queue: &Queue,
        contribution: &PreparedUiFrameContribution,
        atlas_resource: &UiFontAtlasResource,
        surface_width: f32,
        surface_height: f32,
    ) -> UiPreparedDraws {
        let surface_width_u32 = surface_width.max(1.0).round() as u32;
        let surface_height_u32 = surface_height.max(1.0).round() as u32;
        let flattened_rect_instances = contribution
            .submissions
            .iter()
            .flat_map(|submission| Self::extract_rect_instances(&submission.frame))
            .collect::<Vec<_>>();
        let flattened_stroke_instances = contribution
            .submissions
            .iter()
            .flat_map(|submission| Self::extract_stroke_instances(&submission.frame))
            .collect::<Vec<_>>();
        let flattened_glyph_instances = contribution
            .submissions
            .iter()
            .flat_map(|submission| Self::extract_glyph_instances(&submission.frame, atlas_resource))
            .collect::<Vec<_>>();
        let flattened_viewport_embed_instances = contribution
            .submissions
            .iter()
            .flat_map(|submission| Self::extract_viewport_embed_instances(&submission.frame))
            .collect::<Vec<_>>();
        let flattened_product_surface_instances = contribution
            .submissions
            .iter()
            .flat_map(|submission| Self::extract_product_surface_instances(&submission.frame))
            .collect::<Vec<_>>();

        let rect_batches = group_rect_batches_ordered(
            flattened_rect_instances,
            surface_width_u32,
            surface_height_u32,
        )
        .into_iter()
        .filter_map(
            |(layer_order, first_order, last_order, scissor, instances)| {
                if instances.is_empty() {
                    return None;
                }
                let instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
                    label: Some("engine_ui_rect_batch_instances"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: BufferUsages::VERTEX,
                });
                Some(UiRectBatch {
                    layer_order,
                    first_primitive_order: first_order,
                    last_primitive_order: last_order,
                    scissor,
                    instance_count: instances.len() as u32,
                    instance_buffer,
                })
            },
        )
        .collect::<Vec<_>>();

        let stroke_batches = group_stroke_batches_ordered(
            flattened_stroke_instances,
            surface_width_u32,
            surface_height_u32,
        )
        .into_iter()
        .filter_map(
            |(layer_order, first_order, last_order, scissor, instances)| {
                if instances.is_empty() {
                    return None;
                }
                let instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
                    label: Some("engine_ui_stroke_batch_instances"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: BufferUsages::VERTEX,
                });
                Some(UiStrokeBatch {
                    layer_order,
                    first_primitive_order: first_order,
                    last_primitive_order: last_order,
                    scissor,
                    instance_count: instances.len() as u32,
                    instance_buffer,
                })
            },
        )
        .collect::<Vec<_>>();

        let mut glyph_batches_by_scissor = Vec::<(
            u32,
            u32,
            u32,
            (u32, u32, u32, u32),
            u64,
            Vec<GlyphInstanceRaw>,
        )>::new();
        for instance in flattened_glyph_instances {
            let scissor = instance
                .clip
                .map(|clip| Self::clip_to_scissor(clip, surface_width_u32, surface_height_u32))
                .unwrap_or_else(|| Some(Self::full_scissor(surface_width_u32, surface_height_u32)));
            let Some(scissor) = scissor else {
                continue;
            };
            if self
                .ensure_glyph_atlas_gpu(device, queue, atlas_resource, instance.texture_id)
                .is_none()
            {
                continue;
            }
            if let Some((
                last_layer_order,
                _first_order,
                last_order,
                last_scissor,
                last_texture,
                instances,
            )) = glyph_batches_by_scissor.last_mut()
                && *last_layer_order == instance.layer_order
                && *last_scissor == scissor
                && *last_texture == instance.texture_id
                && (*last_order == instance.primitive_order
                    || last_order.saturating_add(1) == instance.primitive_order)
            {
                instances.push(instance.raw);
                *last_order = instance.primitive_order;
            } else {
                glyph_batches_by_scissor.push((
                    instance.layer_order,
                    instance.primitive_order,
                    instance.primitive_order,
                    scissor,
                    instance.texture_id,
                    vec![instance.raw],
                ));
            }
        }
        let glyph_batches = glyph_batches_by_scissor
            .into_iter()
            .filter_map(
                |(layer_order, first_order, last_order, scissor, texture_id, instances)| {
                    if instances.is_empty() {
                        return None;
                    }
                    let instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
                        label: Some("engine_ui_glyph_batch_instances"),
                        contents: bytemuck::cast_slice(&instances),
                        usage: BufferUsages::VERTEX,
                    });
                    Some(UiGlyphBatch {
                        layer_order,
                        first_primitive_order: first_order,
                        last_primitive_order: last_order,
                        scissor,
                        instance_count: instances.len() as u32,
                        instance_buffer,
                        texture_id,
                    })
                },
            )
            .collect::<Vec<_>>();
        let viewport_embed_batches = group_viewport_embed_batches_ordered(
            flattened_viewport_embed_instances,
            surface_width_u32,
            surface_height_u32,
        )
        .into_iter()
        .filter_map(
            |(layer_order, first_order, last_order, scissor, viewport_id, slot, instances)| {
                if instances.is_empty() {
                    return None;
                }
                let instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
                    label: Some("engine_ui_viewport_embed_batch_instances"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: BufferUsages::VERTEX,
                });
                Some(UiViewportEmbedBatch {
                    layer_order,
                    first_primitive_order: first_order,
                    last_primitive_order: last_order,
                    scissor,
                    instance_count: instances.len() as u32,
                    instance_buffer,
                    viewport_id,
                    slot,
                })
            },
        )
        .collect::<Vec<_>>();
        let product_surface_batches = group_product_surface_batches_ordered(
            flattened_product_surface_instances,
            surface_width_u32,
            surface_height_u32,
        )
        .into_iter()
        .filter_map(
            |(layer_order, first_order, last_order, scissor, source, instances)| {
                if instances.is_empty() {
                    return None;
                }
                let instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
                    label: Some("engine_ui_product_surface_batch_instances"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: BufferUsages::VERTEX,
                });
                Some(UiProductSurfaceBatch {
                    layer_order,
                    first_primitive_order: first_order,
                    last_primitive_order: last_order,
                    scissor,
                    instance_count: instances.len() as u32,
                    instance_buffer,
                    source,
                })
            },
        )
        .collect::<Vec<_>>();

        if let Some(rect_pass) = self.rect_pass.as_ref() {
            let screen = ScreenUniformRaw {
                size: [surface_width.max(1.0), surface_height.max(1.0)],
                _pad: [0.0; 2],
            };
            queue.write_buffer(&rect_pass.screen_buffer, 0, bytemuck::bytes_of(&screen));
        }
        if let Some(stroke_pass) = self.stroke_pass.as_ref() {
            let screen = ScreenUniformRaw {
                size: [surface_width.max(1.0), surface_height.max(1.0)],
                _pad: [0.0; 2],
            };
            queue.write_buffer(&stroke_pass.screen_buffer, 0, bytemuck::bytes_of(&screen));
        }
        if let Some(glyph_pass) = self.glyph_pass.as_ref() {
            let screen = ScreenUniformRaw {
                size: [surface_width.max(1.0), surface_height.max(1.0)],
                _pad: [0.0; 2],
            };
            queue.write_buffer(&glyph_pass.screen_buffer, 0, bytemuck::bytes_of(&screen));
        }
        if let Some(viewport_embed_pass) = self.viewport_embed_pass.as_ref() {
            let screen = ScreenUniformRaw {
                size: [surface_width.max(1.0), surface_height.max(1.0)],
                _pad: [0.0; 2],
            };
            queue.write_buffer(
                &viewport_embed_pass.screen_buffer,
                0,
                bytemuck::bytes_of(&screen),
            );
        }
        if let Some(product_surface_pass) = self.product_surface_pass.as_ref() {
            let screen = ScreenUniformRaw {
                size: [surface_width.max(1.0), surface_height.max(1.0)],
                _pad: [0.0; 2],
            };
            queue.write_buffer(
                &product_surface_pass.screen_buffer,
                0,
                bytemuck::bytes_of(&screen),
            );
        }

        let draw_plan = build_ui_draw_plan(
            &rect_batches,
            &stroke_batches,
            &glyph_batches,
            &viewport_embed_batches,
            &product_surface_batches,
        );

        UiPreparedDraws {
            rect_batches,
            stroke_batches,
            glyph_batches,
            viewport_embed_batches,
            product_surface_batches,
            draw_plan,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare_packet(
        &mut self,
        device: &Device,
        queue: &Queue,
        prepared_frame: &PreparedRenderFrame,
        shader_registry: &mut ShaderRegistryResource,
        ui_rect_shader_handle: Option<ShaderHandle>,
        ui_font_atlas: &UiFontAtlasResource,
        viewport_surface_bindings: &ViewportSurfaceBindingRegistry,
        surface_format: TextureFormat,
    ) -> RendererPreparedPacket {
        let view = prepared_frame.main_view().cloned().unwrap_or_else(|| {
            crate::plugins::render::PreparedViewFrame::main(prepared_frame.surface.target_size_px)
        });
        let (surface_width_u32, surface_height_u32) = view.target_size_px;
        let surface_width = surface_width_u32.max(1) as f32;
        let surface_height = surface_height_u32.max(1) as f32;
        let empty_ui = PreparedUiFrameContribution::default();
        let ui = prepared_frame.ui().unwrap_or(&empty_ui);

        let mut feature_gates = BTreeMap::<RenderFeatureId, FeatureExecutionGate>::new();
        let mut feature_runtime_signatures = BTreeMap::<RenderFeatureId, u64>::new();

        for (feature_id, contribution) in &prepared_frame.contributions.by_feature {
            feature_gates.insert(
                *feature_id,
                FeatureExecutionGate {
                    status: contribution.status,
                    fallback_policy: contribution.fallback_policy,
                },
            );
            feature_runtime_signatures.insert(
                *feature_id,
                hash_prepared_feature_contribution(contribution),
            );
        }

        let ui_gate = feature_gates
            .get(&UI_RENDER_FEATURE_ID)
            .copied()
            .unwrap_or_default();
        let prepared_material = prepared_frame
            .contributions
            .feature(&MATERIAL_RENDER_FEATURE_ID)
            .and_then(|contribution| match &contribution.payload {
                crate::plugins::render::PreparedFeaturePayload::Material(payload) => {
                    Some(payload.clone())
                }
                _ => None,
            });
        let prepared_material_gpu_resources =
            self.prepare_material_gpu_resources(device, queue, prepared_material.as_ref());

        let mut prepare_timings = RendererFrameTimings::default();
        let ui_rect_shader = ui_rect_shader_handle
            .map(|handle| shader_registry.source_or_handle(handle, DEFAULT_UI_RECT_SHADER))
            .unwrap_or(DEFAULT_UI_RECT_SHADER)
            .to_string();
        let ui_rect_revision = ui_rect_shader_handle
            .map(|handle| shader_registry.revision_for_handle(handle))
            .unwrap_or(0);

        self.ensure_rect_pass(device, surface_format, &ui_rect_shader, ui_rect_revision);
        self.ensure_stroke_pass(device, surface_format);
        self.ensure_glyph_pass(device, surface_format);
        self.ensure_viewport_embed_pass(device, surface_format);
        self.ensure_product_surface_pass(device, surface_format);
        let surface_size = (surface_width_u32.max(1), surface_height_u32.max(1));
        let prepare_ui_start = Instant::now();
        let prepared_ui_current = {
            let _span = tracing::info_span!("renderer.prepare_ui_draws").entered();
            self.prepare_ui_draws(
                device,
                queue,
                ui,
                ui_font_atlas,
                surface_width,
                surface_height,
            )
        };
        let prepared_ui = self.resolve_ui_prepared_with_gate(prepared_ui_current, ui_gate);
        prepare_timings.prepare_ui_ms = prepare_ui_start.elapsed().as_secs_f32() * 1000.0;
        prepare_timings.prepare_mesh_ms = 0.0;
        prepare_timings.mesh_hot_path = MeshPrepareHotPath::default();

        RendererPreparedPacket {
            surface_format,
            surface_size,
            view_id: view.view_id,
            feature_gates,
            feature_runtime_signatures,
            prepared_material,
            prepared_material_gpu_resources,
            prepared_ui,
            viewport_surface_bindings: viewport_surface_bindings.clone(),
            prepare_timings,
        }
    }

    fn prepare_material_gpu_resources(
        &mut self,
        device: &Device,
        queue: &Queue,
        material: Option<&crate::plugins::render::PreparedMaterialFeatureContribution>,
    ) -> Option<PreparedMaterialGpuResources> {
        let material = material?;
        if material.validate_portable_limits().is_err() {
            return None;
        }
        let mut bindings = material
            .instances
            .iter()
            .flat_map(|instance| instance.texture_bindings.iter())
            .collect::<Vec<_>>();
        if bindings.is_empty() {
            return None;
        }
        bindings.sort_by_key(|binding| binding.resource_slot_index);

        let mut layout_entries = Vec::with_capacity(bindings.len() * 2);
        for binding in &bindings {
            let texture_binding = binding.resource_slot_index.saturating_mul(2);
            layout_entries.push(BindGroupLayoutEntry {
                binding: texture_binding,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: match binding.texture_kind {
                        crate::plugins::render::PreparedMaterialTextureKind::Texture2D => {
                            TextureViewDimension::D2
                        }
                        crate::plugins::render::PreparedMaterialTextureKind::Texture3D => {
                            TextureViewDimension::D3
                        }
                    },
                    multisampled: false,
                },
                count: None,
            });
            layout_entries.push(BindGroupLayoutEntry {
                binding: texture_binding + 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            });
        }

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("engine_material_resource_bind_group_layout"),
            entries: &layout_entries,
        });

        let mut textures = Vec::with_capacity(bindings.len());
        let mut texture_views = Vec::with_capacity(bindings.len());
        let mut samplers = Vec::with_capacity(bindings.len());
        let mut bind_group_bindings = Vec::with_capacity(bindings.len());

        for binding in bindings {
            let upload = match load_material_ktx2_upload(binding) {
                Ok(upload) => upload,
                Err(error) => {
                    tracing::warn!(
                        texture_artifact = binding.artifact_id.as_str(),
                        path = binding.artifact_path.as_str(),
                        error = %error,
                        "material texture residency rejected KTX2 artifact"
                    );
                    return None;
                }
            };
            let texture = device.create_texture(&TextureDescriptor {
                label: Some("engine_material_resident_texture"),
                size: upload.size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: upload.dimension,
                format: upload.format,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            });
            queue.write_texture(
                TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                &upload.bytes,
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(upload.bytes_per_row),
                    rows_per_image: Some(upload.rows_per_image),
                },
                upload.size,
            );

            let view = texture.create_view(&TextureViewDescriptor::default());
            let sampler = device.create_sampler(&SamplerDescriptor {
                label: Some("engine_material_resident_sampler"),
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
                address_mode_w: AddressMode::Repeat,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                mipmap_filter: FilterMode::Nearest,
                ..Default::default()
            });
            let texture_binding = binding.resource_slot_index.saturating_mul(2);
            texture_views.push(view);
            samplers.push(sampler);
            bind_group_bindings.push(texture_binding);
            textures.push(texture);
        }

        let mut bind_group_entries = Vec::with_capacity(bind_group_bindings.len() * 2);
        for (index, texture_binding) in bind_group_bindings.iter().copied().enumerate() {
            bind_group_entries.push(BindGroupEntry {
                binding: texture_binding,
                resource: BindingResource::TextureView(&texture_views[index]),
            });
            bind_group_entries.push(BindGroupEntry {
                binding: texture_binding + 1,
                resource: BindingResource::Sampler(&samplers[index]),
            });
        }

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("engine_material_resource_bind_group"),
            layout: &layout,
            entries: &bind_group_entries,
        });
        drop(bind_group_entries);

        Some(PreparedMaterialGpuResources {
            layout,
            bind_group,
            _textures: textures,
            _texture_views: texture_views,
            _samplers: samplers,
        })
    }

    fn resolve_ui_prepared_with_gate(
        &mut self,
        prepared_ui_current: UiPreparedDraws,
        gate: FeatureExecutionGate,
    ) -> UiPreparedDraws {
        match gate.status {
            FeatureContributionStatus::Ready => {
                self.last_good_ui_prepared = Some(prepared_ui_current.clone());
                prepared_ui_current
            }
            FeatureContributionStatus::Stale => {
                if matches!(gate.fallback_policy, FeatureFallbackPolicy::ReuseLastGood)
                    && let Some(cached) = self.last_good_ui_prepared.clone()
                {
                    return cached;
                }
                self.last_good_ui_prepared = Some(prepared_ui_current.clone());
                prepared_ui_current
            }
            FeatureContributionStatus::Disabled | FeatureContributionStatus::Missing => {
                match gate.fallback_policy {
                    FeatureFallbackPolicy::ReuseLastGood => self
                        .last_good_ui_prepared
                        .clone()
                        .unwrap_or(prepared_ui_current),
                    FeatureFallbackPolicy::EmptyContribution
                    | FeatureFallbackPolicy::SkipFeaturePasses
                    | FeatureFallbackPolicy::FailFrame => prepared_ui_current,
                }
            }
        }
    }
}

fn group_rect_batches_ordered(
    flattened_rect_instances: Vec<FlattenedUiRectInstance>,
    surface_width_u32: u32,
    surface_height_u32: u32,
) -> Vec<ScissoredRectBatch> {
    let mut grouped = Vec::<ScissoredRectBatch>::new();
    for instance in flattened_rect_instances {
        let scissor = instance
            .clip
            .map(|clip| Renderer::clip_to_scissor(clip, surface_width_u32, surface_height_u32))
            .unwrap_or_else(|| {
                Some(Renderer::full_scissor(
                    surface_width_u32,
                    surface_height_u32,
                ))
            });
        let Some(scissor) = scissor else {
            continue;
        };
        if let Some((last_layer_order, _first_order, last_order, last_scissor, instances)) =
            grouped.last_mut()
            && *last_layer_order == instance.layer_order
            && *last_scissor == scissor
            && last_order.saturating_add(1) == instance.primitive_order
        {
            instances.push(instance.raw);
            *last_order = instance.primitive_order;
        } else {
            grouped.push((
                instance.layer_order,
                instance.primitive_order,
                instance.primitive_order,
                scissor,
                vec![instance.raw],
            ));
        }
    }
    grouped
}

fn group_stroke_batches_ordered(
    flattened_instances: Vec<FlattenedUiStrokeSegmentInstance>,
    surface_width_u32: u32,
    surface_height_u32: u32,
) -> Vec<ScissoredStrokeBatch> {
    let mut grouped = Vec::<ScissoredStrokeBatch>::new();
    for instance in flattened_instances {
        let scissor = instance
            .clip
            .map(|clip| Renderer::clip_to_scissor(clip, surface_width_u32, surface_height_u32))
            .unwrap_or_else(|| {
                Some(Renderer::full_scissor(
                    surface_width_u32,
                    surface_height_u32,
                ))
            });
        let Some(scissor) = scissor else {
            continue;
        };
        if let Some((last_layer_order, _first_order, last_order, last_scissor, instances)) =
            grouped.last_mut()
            && *last_layer_order == instance.layer_order
            && *last_scissor == scissor
            && (*last_order == instance.primitive_order
                || last_order.saturating_add(1) == instance.primitive_order)
        {
            instances.push(instance.raw);
            *last_order = instance.primitive_order;
        } else {
            grouped.push((
                instance.layer_order,
                instance.primitive_order,
                instance.primitive_order,
                scissor,
                vec![instance.raw],
            ));
        }
    }
    grouped
}

fn group_viewport_embed_batches_ordered(
    flattened_instances: Vec<FlattenedUiViewportEmbedInstance>,
    surface_width_u32: u32,
    surface_height_u32: u32,
) -> Vec<ScissoredViewportEmbedBatch> {
    let mut grouped = Vec::<ScissoredViewportEmbedBatch>::new();
    for instance in flattened_instances {
        let scissor = instance
            .clip
            .map(|clip| Renderer::clip_to_scissor(clip, surface_width_u32, surface_height_u32))
            .unwrap_or_else(|| {
                Some(Renderer::full_scissor(
                    surface_width_u32,
                    surface_height_u32,
                ))
            });
        let Some(scissor) = scissor else {
            continue;
        };
        if let Some((
            last_layer_order,
            _first_order,
            last_order,
            last_scissor,
            last_viewport_id,
            last_slot,
            instances,
        )) = grouped.last_mut()
            && *last_layer_order == instance.layer_order
            && *last_scissor == scissor
            && *last_viewport_id == instance.viewport_id
            && *last_slot == instance.slot
            && last_order.saturating_add(1) == instance.primitive_order
        {
            instances.push(instance.raw);
            *last_order = instance.primitive_order;
        } else {
            grouped.push((
                instance.layer_order,
                instance.primitive_order,
                instance.primitive_order,
                scissor,
                instance.viewport_id,
                instance.slot,
                vec![instance.raw],
            ));
        }
    }
    grouped
}

fn group_product_surface_batches_ordered(
    flattened_instances: Vec<FlattenedUiProductSurfaceInstance>,
    surface_width_u32: u32,
    surface_height_u32: u32,
) -> Vec<ScissoredProductSurfaceBatch> {
    let mut grouped = Vec::<ScissoredProductSurfaceBatch>::new();
    for instance in flattened_instances {
        let scissor = instance
            .clip
            .map(|clip| Renderer::clip_to_scissor(clip, surface_width_u32, surface_height_u32))
            .unwrap_or_else(|| {
                Some(Renderer::full_scissor(
                    surface_width_u32,
                    surface_height_u32,
                ))
            });
        let Some(scissor) = scissor else {
            continue;
        };
        if let Some((
            last_layer_order,
            _first_order,
            last_order,
            last_scissor,
            last_source,
            instances,
        )) = grouped.last_mut()
            && *last_layer_order == instance.layer_order
            && *last_scissor == scissor
            && *last_source == instance.source
            && last_order.saturating_add(1) == instance.primitive_order
        {
            instances.push(instance.raw);
            *last_order = instance.primitive_order;
        } else {
            grouped.push((
                instance.layer_order,
                instance.primitive_order,
                instance.primitive_order,
                scissor,
                instance.source,
                vec![instance.raw],
            ));
        }
    }
    grouped
}

fn build_ui_draw_plan(
    rect_batches: &[UiRectBatch],
    stroke_batches: &[UiStrokeBatch],
    glyph_batches: &[UiGlyphBatch],
    viewport_embed_batches: &[UiViewportEmbedBatch],
    product_surface_batches: &[UiProductSurfaceBatch],
) -> Vec<UiPreparedDrawCommand> {
    let mut commands = Vec::with_capacity(
        rect_batches.len()
            + stroke_batches.len()
            + glyph_batches.len()
            + viewport_embed_batches.len()
            + product_surface_batches.len(),
    );
    commands.extend(
        rect_batches
            .iter()
            .enumerate()
            .map(|(index, _)| UiPreparedDrawCommand::Rect(index)),
    );
    commands.extend(
        stroke_batches
            .iter()
            .enumerate()
            .map(|(index, _)| UiPreparedDrawCommand::Stroke(index)),
    );
    commands.extend(
        viewport_embed_batches
            .iter()
            .enumerate()
            .map(|(index, _)| UiPreparedDrawCommand::ViewportEmbed(index)),
    );
    commands.extend(
        product_surface_batches
            .iter()
            .enumerate()
            .map(|(index, _)| UiPreparedDrawCommand::ProductSurface(index)),
    );
    commands.extend(
        glyph_batches
            .iter()
            .enumerate()
            .map(|(index, _)| UiPreparedDrawCommand::Glyph(index)),
    );
    commands.sort_by(|left, right| {
        let left_key = draw_command_sort_key(
            *left,
            rect_batches,
            stroke_batches,
            glyph_batches,
            viewport_embed_batches,
            product_surface_batches,
        );
        let right_key = draw_command_sort_key(
            *right,
            rect_batches,
            stroke_batches,
            glyph_batches,
            viewport_embed_batches,
            product_surface_batches,
        );
        left_key.cmp(&right_key)
    });
    commands
}

fn draw_command_sort_key(
    command: UiPreparedDrawCommand,
    rect_batches: &[UiRectBatch],
    stroke_batches: &[UiStrokeBatch],
    glyph_batches: &[UiGlyphBatch],
    viewport_embed_batches: &[UiViewportEmbedBatch],
    product_surface_batches: &[UiProductSurfaceBatch],
) -> (u32, u32, u32, u8) {
    match command {
        UiPreparedDrawCommand::Rect(index) => {
            let batch = &rect_batches[index];
            (
                batch.layer_order,
                batch.first_primitive_order,
                batch.last_primitive_order,
                0,
            )
        }
        UiPreparedDrawCommand::Stroke(index) => {
            let batch = &stroke_batches[index];
            (
                batch.layer_order,
                batch.first_primitive_order,
                batch.last_primitive_order,
                1,
            )
        }
        UiPreparedDrawCommand::ViewportEmbed(index) => {
            let batch = &viewport_embed_batches[index];
            (
                batch.layer_order,
                batch.first_primitive_order,
                batch.last_primitive_order,
                2,
            )
        }
        UiPreparedDrawCommand::ProductSurface(index) => {
            let batch = &product_surface_batches[index];
            (
                batch.layer_order,
                batch.first_primitive_order,
                batch.last_primitive_order,
                2,
            )
        }
        UiPreparedDrawCommand::Glyph(index) => {
            let batch = &glyph_batches[index];
            (
                batch.layer_order,
                batch.first_primitive_order,
                batch.last_primitive_order,
                3,
            )
        }
    }
}

fn hash_prepared_feature_contribution(
    contribution: &crate::plugins::render::PreparedFeatureContribution,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    contribution.status.hash(&mut hasher);
    contribution.fallback_policy.hash(&mut hasher);
    match &contribution.payload {
        crate::plugins::render::PreparedFeaturePayload::Empty => {
            "empty".hash(&mut hasher);
        }
        crate::plugins::render::PreparedFeaturePayload::Ui(value) => {
            "ui".hash(&mut hasher);
            value.submissions.len().hash(&mut hasher);

            for submission in &value.submissions {
                submission.producer_id.hash(&mut hasher);
                submission.route.hash(&mut hasher);
                submission.layer.hash(&mut hasher);
                submission.priority.hash(&mut hasher);
                submission.rect_shader_asset_id.hash(&mut hasher);
                submission.primitive_count_hint().hash(&mut hasher);
            }
        }
        crate::plugins::render::PreparedFeaturePayload::SceneRoute(value) => {
            "scene_route".hash(&mut hasher);
            value.world_scene_label.hash(&mut hasher);
            value.overlay_scene_label.hash(&mut hasher);
        }
        crate::plugins::render::PreparedFeaturePayload::Draw(value) => {
            "draw".hash(&mut hasher);
            value.batches.len().hash(&mut hasher);
            for batch in &value.batches {
                batch.batch_id.hash(&mut hasher);
                batch.mesh_ref.hash(&mut hasher);
                batch.material_ref.hash(&mut hasher);
                batch.instance_count.hash(&mut hasher);
            }
        }
        crate::plugins::render::PreparedFeaturePayload::World(value) => {
            "world".hash(&mut hasher);
            value.visible_chunks.len().hash(&mut hasher);
            value.residency_intents.len().hash(&mut hasher);
            for chunk in &value.visible_chunks {
                chunk.chunk_id.hash(&mut hasher);
                chunk.chunk_revision.hash(&mut hasher);
                chunk.chunk_generation.hash(&mut hasher);
                chunk.draw_batch_ref.hash(&mut hasher);
            }
            for intent in &value.residency_intents {
                intent.chunk_id.hash(&mut hasher);
                intent.priority.hash(&mut hasher);
                intent.hard_pin.hash(&mut hasher);
            }
        }
        crate::plugins::render::PreparedFeaturePayload::Caves(value) => {
            "caves".hash(&mut hasher);
            value.visible_sector_ids.hash(&mut hasher);
            value.scoped_light_volume_count.hash(&mut hasher);
        }
        crate::plugins::render::PreparedFeaturePayload::Detail(value) => {
            "detail".hash(&mut hasher);
            value.cells.len().hash(&mut hasher);
            for cell in &value.cells {
                cell.cell_id.hash(&mut hasher);
                cell.chunk_id.hash(&mut hasher);
                cell.instance_count.hash(&mut hasher);
            }
        }
        crate::plugins::render::PreparedFeaturePayload::ProceduralWorld(value) => {
            "procedural_world".hash(&mut hasher);
            value.overlays.len().hash(&mut hasher);
            for overlay in &value.overlays {
                overlay.overlay_id.hash(&mut hasher);
                overlay.source_revision.hash(&mut hasher);
            }
        }
        crate::plugins::render::PreparedFeaturePayload::WindFields(value) => {
            "wind_fields".hash(&mut hasher);
            value.fields.len().hash(&mut hasher);
            for field in &value.fields {
                field.field_id.hash(&mut hasher);
                field.strength.to_bits().hash(&mut hasher);
            }
        }
        crate::plugins::render::PreparedFeaturePayload::Material(value) => {
            "material".hash(&mut hasher);
            match value.binding_table.backend {
                crate::plugins::render::PreparedMaterialBindingTableBackend::FixedCapacityArray {
                    capacity,
                } => {
                    "fixed_capacity_array".hash(&mut hasher);
                    capacity.hash(&mut hasher);
                }
            }
            value.binding_table.slots.len().hash(&mut hasher);
            for slot in &value.binding_table.slots {
                slot.slot_index.hash(&mut hasher);
                slot.material_instance_id.hash(&mut hasher);
                slot.formed_material_artifact_id.hash(&mut hasher);
                slot.shader_artifact_id.hash(&mut hasher);
                slot.material_cache_key.hash(&mut hasher);
                slot.shader_cache_key.hash(&mut hasher);
                slot.prior_valid.hash(&mut hasher);
            }
            if let Some(scene_bundle) = &value.scene_bundle {
                "scene_bundle".hash(&mut hasher);
                scene_bundle.shader_artifact_id.hash(&mut hasher);
                scene_bundle.shader_cache_key.hash(&mut hasher);
                scene_bundle.shader_path.hash(&mut hasher);
                scene_bundle.shader_identity.hash(&mut hasher);
                scene_bundle.material_table_identity.hash(&mut hasher);
                scene_bundle.resource_layout_identity.hash(&mut hasher);
            }
            value.model_mesh_material_selections.len().hash(&mut hasher);
            for selection in &value.model_mesh_material_selections {
                selection.surface.source.asset_id.hash(&mut hasher);
                selection.surface.source.source_id.hash(&mut hasher);
                selection
                    .surface
                    .source
                    .source_revision_id
                    .hash(&mut hasher);
                selection.surface.source.source_revision.hash(&mut hasher);
                selection.surface.region_key.hash(&mut hasher);
                selection.requested_material_slot_id.hash(&mut hasher);
                selection.resolved_material_slot_id.hash(&mut hasher);
                selection.material_table_index.hash(&mut hasher);
                selection.used_default_fallback.hash(&mut hasher);
            }
            value.instances.len().hash(&mut hasher);
            for instance in &value.instances {
                instance.material_instance_id.hash(&mut hasher);
                instance.specialization_key_fragment.hash(&mut hasher);
                instance.parameter_payload.encode_v1().hash(&mut hasher);
                instance.texture_bindings.len().hash(&mut hasher);
                for binding in &instance.texture_bindings {
                    binding.node_id.hash(&mut hasher);
                    binding.binding_key.hash(&mut hasher);
                    binding.resource_slot_index.hash(&mut hasher);
                    binding.artifact_id.hash(&mut hasher);
                    binding.artifact_path.hash(&mut hasher);
                    binding.texture_kind.hash(&mut hasher);
                    binding.extent_width.hash(&mut hasher);
                    binding.extent_height.hash(&mut hasher);
                    binding.extent_depth.hash(&mut hasher);
                    binding.cache_key.hash(&mut hasher);
                    binding.sampler_policy.hash(&mut hasher);
                    binding.texture_dimension.hash(&mut hasher);
                    binding.residency_identity.hash(&mut hasher);
                    binding.artifact_revision.hash(&mut hasher);
                    binding.descriptor_hash.hash(&mut hasher);
                    binding.pixel_format.hash(&mut hasher);
                    binding.supercompression.hash(&mut hasher);
                    binding.container_byte_length.hash(&mut hasher);
                }
            }
        }
        crate::plugins::render::PreparedFeaturePayload::Deformation(value) => {
            "deformation".hash(&mut hasher);
            value.streams.len().hash(&mut hasher);
            for stream in &value.streams {
                stream.stream_id.hash(&mut hasher);
                stream.input_pose_ref.hash(&mut hasher);
                stream.output_buffer_ref.hash(&mut hasher);
            }
        }
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::{
        FeatureContributionStatus, FeatureFallbackPolicy, PreparedFeatureContribution,
        PreparedFeaturePayload, PreparedMaterialBindingTable, PreparedMaterialFeatureContribution,
        PreparedMaterialInstanceInput, PreparedMaterialOutputTarget,
        PreparedMaterialParameterInput, PreparedMaterialParameterKind,
        PreparedMaterialParameterPayloadV1, PreparedMaterialParameterProfile,
        PreparedMaterialTextureBinding, PreparedMaterialTextureKind,
    };

    #[test]
    fn material_contribution_hash_uses_encoded_typed_parameter_payload() {
        fn contribution_with_parameter(key: &str) -> PreparedFeatureContribution {
            PreparedFeatureContribution {
                status: FeatureContributionStatus::Ready,
                fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
                payload: PreparedFeaturePayload::Material(PreparedMaterialFeatureContribution {
                    instances: vec![PreparedMaterialInstanceInput {
                        material_instance_id: "material.product.1".to_string(),
                        specialization_key_fragment: "material.first_slice".to_string(),
                        parameter_payload: PreparedMaterialParameterPayloadV1::new(
                            PreparedMaterialParameterProfile::RenderMaterial,
                            PreparedMaterialOutputTarget::RenderMaterial,
                            [PreparedMaterialParameterInput::new(
                                key,
                                PreparedMaterialParameterKind::Scalar,
                            )],
                        ),
                        texture_bindings: Vec::new(),
                    }],
                    binding_table: PreparedMaterialBindingTable::default(),
                    scene_bundle: None,
                    model_mesh_material_selections: Vec::new(),
                }),
            }
        }

        let roughness = contribution_with_parameter("roughness");
        let metallic = contribution_with_parameter("metallic");

        assert_ne!(
            hash_prepared_feature_contribution(&roughness),
            hash_prepared_feature_contribution(&metallic)
        );
        let PreparedFeaturePayload::Material(payload) = &roughness.payload else {
            panic!("expected material payload");
        };
        assert!(
            String::from_utf8_lossy(&payload.instances[0].parameter_payload.encode_v1())
                .contains("roughness")
        );
    }

    #[test]
    fn rect_batch_grouping_preserves_non_consecutive_order() {
        let a = FlattenedUiRectInstance {
            raw: RectInstanceRaw {
                rect: [0.0, 0.0, 10.0, 10.0],
                color: [1.0, 1.0, 1.0, 1.0],
                radius: 0.0,
                _pad: [0.0; 3],
            },
            clip: Some([0.0, 0.0, 10.0, 10.0]),
            layer_order: 0,
            primitive_order: 1,
        };
        let b = FlattenedUiRectInstance {
            raw: RectInstanceRaw {
                rect: [20.0, 0.0, 10.0, 10.0],
                color: [1.0, 1.0, 1.0, 1.0],
                radius: 0.0,
                _pad: [0.0; 3],
            },
            clip: Some([20.0, 0.0, 10.0, 10.0]),
            layer_order: 0,
            primitive_order: 3,
        };
        let c = FlattenedUiRectInstance {
            raw: RectInstanceRaw {
                rect: [1.0, 1.0, 8.0, 8.0],
                color: [1.0, 1.0, 1.0, 1.0],
                radius: 0.0,
                _pad: [0.0; 3],
            },
            clip: Some([0.0, 0.0, 10.0, 10.0]),
            layer_order: 0,
            primitive_order: 5,
        };
        let grouped = group_rect_batches_ordered(vec![a, b, c], 100, 100);
        assert_eq!(grouped.len(), 3);
        assert_eq!(grouped[0].4.len(), 1);
        assert_eq!(grouped[1].4.len(), 1);
        assert_eq!(grouped[2].4.len(), 1);
    }

    #[test]
    fn material_ktx2_upload_reads_exact_base_level_bytes() {
        let bytes = build_rgba8_ktx2(2, 2, 1, [12, 34, 56, 255]);
        let path = std::env::temp_dir().join(format!(
            "runenwerk-material-ktx2-upload-{}.ktx2",
            std::process::id()
        ));
        std::fs::write(&path, &bytes).expect("test ktx2 should write");
        let binding = PreparedMaterialTextureBinding::new(
            7,
            "albedo",
            "artifact.7",
            path.to_string_lossy(),
            PreparedMaterialTextureKind::Texture2D,
            "cache",
        )
        .with_extent(2, 2, 1)
        .with_descriptor_hash("descriptor-hash")
        .with_ktx2_contract("Rgba8Unorm", "None", Some(bytes.len() as u64));

        let upload = load_material_ktx2_upload(&binding).expect("ktx2 upload should load");

        assert_eq!(upload.size.width, 2);
        assert_eq!(upload.size.height, 2);
        assert_eq!(upload.size.depth_or_array_layers, 1);
        assert_eq!(upload.format, TextureFormat::Rgba8Unorm);
        assert_eq!(&upload.bytes[0..4], &[12, 34, 56, 255]);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn material_ktx2_upload_rejects_byte_length_mismatch() {
        let bytes = build_rgba8_ktx2(1, 1, 1, [1, 2, 3, 255]);
        let path = std::env::temp_dir().join(format!(
            "runenwerk-material-ktx2-mismatch-{}.ktx2",
            std::process::id()
        ));
        std::fs::write(&path, &bytes).expect("test ktx2 should write");
        let binding = PreparedMaterialTextureBinding::new(
            7,
            "albedo",
            "artifact.7",
            path.to_string_lossy(),
            PreparedMaterialTextureKind::Texture2D,
            "cache",
        )
        .with_extent(1, 1, 1)
        .with_descriptor_hash("descriptor-hash")
        .with_ktx2_contract("Rgba8Unorm", "None", Some(bytes.len() as u64 + 1));

        let error = load_material_ktx2_upload(&binding).expect_err("length mismatch should fail");

        assert!(error.to_string().contains("byte length"));
        let _ = std::fs::remove_file(path);
    }

    fn build_rgba8_ktx2(width: u32, height: u32, depth: u32, texel: [u8; 4]) -> Vec<u8> {
        let format = ktx2::Format::R8G8B8A8_UNORM;
        let (basic, type_size) =
            ktx2::dfd::Basic::from_format(format).expect("rgba8 dfd should build");
        let dfd_block = ktx2::dfd::Block::Basic(basic);
        let dfd_block_bytes = dfd_block.to_vec();
        let dfd_total_size = 4 + dfd_block_bytes.len();
        let level_index_offset = ktx2::Header::LENGTH;
        let dfd_offset = level_index_offset + ktx2::LevelIndex::LENGTH;
        let after_dfd = dfd_offset + dfd_total_size;
        let level_data_offset = (after_dfd + 3) / 4 * 4;
        let texel_count = width as usize * height as usize * depth.max(1) as usize;
        let level_data_size = texel_count * 4;
        let mut bytes = vec![0u8; level_data_offset + level_data_size];

        let header = ktx2::Header {
            format: Some(format),
            type_size,
            pixel_width: width,
            pixel_height: height,
            pixel_depth: if depth > 1 { depth } else { 0 },
            layer_count: 0,
            face_count: 1,
            level_count: 1,
            supercompression_scheme: None,
            index: ktx2::Index {
                dfd_byte_offset: dfd_offset as u32,
                dfd_byte_length: dfd_total_size as u32,
                kvd_byte_offset: 0,
                kvd_byte_length: 0,
                sgd_byte_offset: 0,
                sgd_byte_length: 0,
            },
        };
        bytes[..ktx2::Header::LENGTH].copy_from_slice(&header.as_bytes());
        let level_index = ktx2::LevelIndex {
            byte_offset: level_data_offset as u64,
            byte_length: level_data_size as u64,
            uncompressed_byte_length: level_data_size as u64,
        };
        bytes[level_index_offset..level_index_offset + ktx2::LevelIndex::LENGTH]
            .copy_from_slice(&level_index.as_bytes());
        bytes[dfd_offset..dfd_offset + 4].copy_from_slice(&(dfd_total_size as u32).to_le_bytes());
        bytes[dfd_offset + 4..dfd_offset + 4 + dfd_block_bytes.len()]
            .copy_from_slice(&dfd_block_bytes);
        for index in 0..texel_count {
            let start = level_data_offset + index * 4;
            bytes[start..start + 4].copy_from_slice(&texel);
        }
        bytes
    }
}
