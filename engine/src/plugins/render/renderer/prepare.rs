use super::*;
use crate::plugins::render::features::UI_RENDER_FEATURE_ID;
use std::hash::{Hash, Hasher};

impl Renderer {
    pub(super) fn prepare_ui_draws(
        &self,
        device: &Device,
        queue: &Queue,
        draw_list: &UiDrawList,
        surface_width: f32,
        surface_height: f32,
    ) -> UiPreparedDraws {
        let surface_width_u32 = surface_width.max(1.0).round() as u32;
        let surface_height_u32 = surface_height.max(1.0).round() as u32;
        let instances = Self::extract_rect_instances(draw_list);
        let rect_instance_buffer = if instances.is_empty() {
            None
        } else {
            Some(device.create_buffer_init(&util::BufferInitDescriptor {
                label: Some("engine_ui_rect_instances"),
                contents: bytemuck::cast_slice(&instances),
                usage: BufferUsages::VERTEX,
            }))
        };

        if let Some(rect_pass) = self.rect_pass.as_ref() {
            let screen = ScreenUniformRaw {
                size: [surface_width.max(1.0), surface_height.max(1.0)],
                _pad: [0.0; 2],
            };
            queue.write_buffer(&rect_pass.screen_buffer, 0, bytemuck::bytes_of(&screen));
        }

        if let Some(text_renderer) = self.text_renderer.as_ref() {
            text_renderer.write_screen_uniform(queue, surface_width, surface_height);
        }

        let text_draws = if let Some(text_renderer) = self.text_renderer.as_ref() {
            let full_scissor = Self::full_scissor(surface_width_u32, surface_height_u32);
            let mut draws = Vec::new();
            for cmd in &draw_list.commands {
                let UiDrawCmd::Text { clip, .. } = cmd else {
                    continue;
                };
                let scissor = clip
                    .and_then(|clip| {
                        Self::clip_to_scissor(clip, surface_width_u32, surface_height_u32)
                    })
                    .unwrap_or(full_scissor);
                let single = UiDrawList {
                    commands: vec![cmd.clone()],
                };
                if let Some((buffer, count)) = text_renderer.build_instance_buffer(device, &single)
                {
                    draws.push((buffer, count, scissor));
                }
            }
            draws
        } else {
            Vec::new()
        };

        UiPreparedDraws {
            rect_instances: instances.len(),
            rect_instance_buffer,
            text_draws,
            surface_size: (surface_width_u32, surface_height_u32),
        }
    }

    pub(crate) fn prepare_packet(
        &mut self,
        device: &Device,
        queue: &Queue,
        prepared_frame: &PreparedRenderFrame,
        shader_registry: &mut ShaderRegistryResource,
        ui_rect_shader_handle: Option<ShaderHandle>,
        surface_format: TextureFormat,
    ) -> RendererPreparedPacket {
        let view = prepared_frame.main_view().cloned().unwrap_or_else(|| {
            crate::plugins::render::PreparedViewFrame::main(prepared_frame.surface.target_size_px)
        });
        let (surface_width_u32, surface_height_u32) = view.target_size_px;
        let surface_width = surface_width_u32.max(1) as f32;
        let surface_height = surface_height_u32.max(1) as f32;
        let empty_draw_list = UiDrawList::default();
        let draw_list = prepared_frame.ui_draw_list().unwrap_or(&empty_draw_list);

        let mut feature_gates = BTreeMap::<String, FeatureExecutionGate>::new();
        let mut feature_runtime_signatures = BTreeMap::<String, u64>::new();
        for (feature_id, contribution) in &prepared_frame.contributions.by_feature {
            feature_gates.insert(
                feature_id.as_str().to_string(),
                FeatureExecutionGate {
                    status: contribution.status,
                    fallback_policy: contribution.fallback_policy,
                },
            );
            feature_runtime_signatures.insert(
                feature_id.as_str().to_string(),
                hash_prepared_feature_contribution(contribution),
            );
        }
        let ui_gate = feature_gates
            .get(UI_RENDER_FEATURE_ID)
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
        self.ensure_text_renderer(device, queue, surface_format);
        let surface_size = (surface_width_u32.max(1), surface_height_u32.max(1));
        let prepare_ui_start = Instant::now();
        let prepared_ui_current = {
            let _span = tracing::info_span!("renderer.prepare_ui_draws").entered();
            self.prepare_ui_draws(device, queue, draw_list, surface_width, surface_height)
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
            value.draw_list.commands.len().hash(&mut hasher);
            value.rect_shader_asset_id.hash(&mut hasher);
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
