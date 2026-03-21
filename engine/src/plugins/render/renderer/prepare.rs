use super::*;
use crate::plugins::render::frame_packet::PreparedUiInput;

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
        let (surface_width_u32, surface_height_u32) = prepared_frame.surface.target_size_px;
        let surface_width = surface_width_u32.max(1) as f32;
        let surface_height = surface_height_u32.max(1) as f32;
        let draw_list = match &prepared_frame.ui {
            PreparedUiInput::RawDrawList(value) => value,
        };

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
        let world_scene_label = prepared_frame.scene.world_scene_label.clone();
        let overlay_scene_label = prepared_frame.scene.overlay_scene_label.clone();
        let prepare_ui_start = Instant::now();
        let prepared_ui = {
            let _span = tracing::info_span!("renderer.prepare_ui_draws").entered();
            self.prepare_ui_draws(device, queue, draw_list, surface_width, surface_height)
        };
        prepare_timings.prepare_ui_ms = prepare_ui_start.elapsed().as_secs_f32() * 1000.0;
        prepare_timings.prepare_mesh_ms = 0.0;
        prepare_timings.mesh_hot_path = MeshPrepareHotPath::default();

        RendererPreparedPacket {
            surface_format,
            surface_size,
            world_scene_label,
            overlay_scene_label,
            prepared_ui,
            prepare_timings,
        }
    }
}
