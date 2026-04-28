use super::*;
use crate::plugins::render::features::{UI_RENDER_FEATURE_ID, UiFontAtlasResource};
use crate::plugins::{PreparedUiFrameContribution, RenderFeatureId};
use std::hash::{Hash, Hasher};

type ScissoredRectBatch = ((u32, u32, u32, u32), Vec<RectInstanceRaw>);
type ScissoredViewportEmbedBatch = (
    (u32, u32, u32, u32),
    u64,
    ViewportSurfaceEmbedSlotId,
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

        let rect_batches = group_rect_batches_ordered(
            flattened_rect_instances,
            surface_width_u32,
            surface_height_u32,
        )
        .into_iter()
        .filter_map(|(scissor, instances)| {
            if instances.is_empty() {
                return None;
            }
            let instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
                label: Some("engine_ui_rect_batch_instances"),
                contents: bytemuck::cast_slice(&instances),
                usage: BufferUsages::VERTEX,
            });
            Some(UiRectBatch {
                scissor,
                instance_count: instances.len() as u32,
                instance_buffer,
            })
        })
        .collect::<Vec<_>>();

        let mut glyph_batches_by_scissor =
            Vec::<((u32, u32, u32, u32), u64, Vec<GlyphInstanceRaw>)>::new();
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
            if let Some((last_scissor, last_texture, instances)) =
                glyph_batches_by_scissor.last_mut()
                && *last_scissor == scissor
                && *last_texture == instance.texture_id
            {
                instances.push(instance.raw);
            } else {
                glyph_batches_by_scissor.push((scissor, instance.texture_id, vec![instance.raw]));
            }
        }
        let glyph_batches = glyph_batches_by_scissor
            .into_iter()
            .filter_map(|(scissor, texture_id, instances)| {
                if instances.is_empty() {
                    return None;
                }
                let instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
                    label: Some("engine_ui_glyph_batch_instances"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: BufferUsages::VERTEX,
                });
                Some(UiGlyphBatch {
                    scissor,
                    instance_count: instances.len() as u32,
                    instance_buffer,
                    texture_id,
                })
            })
            .collect::<Vec<_>>();
        let viewport_embed_batches = group_viewport_embed_batches_ordered(
            flattened_viewport_embed_instances,
            surface_width_u32,
            surface_height_u32,
        )
        .into_iter()
        .filter_map(|(scissor, viewport_id, slot, instances)| {
            if instances.is_empty() {
                return None;
            }
            let instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
                label: Some("engine_ui_viewport_embed_batch_instances"),
                contents: bytemuck::cast_slice(&instances),
                usage: BufferUsages::VERTEX,
            });
            Some(UiViewportEmbedBatch {
                scissor,
                instance_count: instances.len() as u32,
                instance_buffer,
                viewport_id,
                slot,
            })
        })
        .collect::<Vec<_>>();

        if let Some(rect_pass) = self.rect_pass.as_ref() {
            let screen = ScreenUniformRaw {
                size: [surface_width.max(1.0), surface_height.max(1.0)],
                _pad: [0.0; 2],
            };
            queue.write_buffer(&rect_pass.screen_buffer, 0, bytemuck::bytes_of(&screen));
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

        UiPreparedDraws {
            rect_batches,
            glyph_batches,
            viewport_embed_batches,
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

        let mut prepare_timings = RendererFrameTimings::default();
        let ui_rect_shader = ui_rect_shader_handle
            .map(|handle| shader_registry.source_or_handle(handle, DEFAULT_UI_RECT_SHADER))
            .unwrap_or(DEFAULT_UI_RECT_SHADER)
            .to_string();
        let ui_rect_revision = ui_rect_shader_handle
            .map(|handle| shader_registry.revision_for_handle(handle))
            .unwrap_or(0);

        self.ensure_rect_pass(device, surface_format, &ui_rect_shader, ui_rect_revision);
        self.ensure_glyph_pass(device, surface_format);
        self.ensure_viewport_embed_pass(device, surface_format);
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
            view_count: prepared_frame.views.len(),
            feature_gates,
            feature_runtime_signatures,
            prepared_ui,
            viewport_surface_bindings: viewport_surface_bindings.clone(),
            prepare_timings,
        }
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
        if let Some((last_scissor, instances)) = grouped.last_mut()
            && *last_scissor == scissor
        {
            instances.push(instance.raw);
        } else {
            grouped.push((scissor, vec![instance.raw]));
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
        if let Some((last_scissor, last_viewport_id, last_slot, instances)) = grouped.last_mut()
            && *last_scissor == scissor
            && *last_viewport_id == instance.viewport_id
            && *last_slot == instance.slot
        {
            instances.push(instance.raw);
        } else {
            grouped.push((
                scissor,
                instance.viewport_id,
                instance.slot,
                vec![instance.raw],
            ));
        }
    }
    grouped
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
            value.instances.len().hash(&mut hasher);
            for instance in &value.instances {
                instance.material_instance_id.hash(&mut hasher);
                instance.specialization_key_fragment.hash(&mut hasher);
                instance.parameter_blob.hash(&mut hasher);
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
        };
        let b = FlattenedUiRectInstance {
            raw: RectInstanceRaw {
                rect: [20.0, 0.0, 10.0, 10.0],
                color: [1.0, 1.0, 1.0, 1.0],
                radius: 0.0,
                _pad: [0.0; 3],
            },
            clip: Some([20.0, 0.0, 10.0, 10.0]),
        };
        let c = FlattenedUiRectInstance {
            raw: RectInstanceRaw {
                rect: [1.0, 1.0, 8.0, 8.0],
                color: [1.0, 1.0, 1.0, 1.0],
                radius: 0.0,
                _pad: [0.0; 3],
            },
            clip: Some([0.0, 0.0, 10.0, 10.0]),
        };
        let grouped = group_rect_batches_ordered(vec![a, b, c], 100, 100);
        assert_eq!(grouped.len(), 3);
        assert_eq!(grouped[0].1.len(), 1);
        assert_eq!(grouped[1].1.len(), 1);
        assert_eq!(grouped[2].1.len(), 1);
    }
}
