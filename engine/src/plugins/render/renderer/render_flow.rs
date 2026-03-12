use super::*;

impl Renderer {
    // Owner: Engine Renderer - Render Packet Execute and Submit

    pub(crate) fn render_packet(
        &mut self,
        device: &Device,
        queue: &Queue,
        frame_view: &TextureView,
        frame_data: &RenderFrameDataRegistry<'_>,
        packet: RendererPreparedPacket,
        render_graph_registry: &RenderGraphRegistryResource,
        render_executor_registry: &RenderPassExecutorRegistryResource,
    ) -> RendererFrameTimings {
        let world_scene = packet.world_scene_label.as_str();
        let overlay_scene = packet.overlay_scene_label.as_str();
        let frame_graph_output = self.build_frame_graph(render_graph_registry);
        self.log_frame_graph_diagnostics(
            world_scene,
            overlay_scene,
            render_graph_registry.revision(),
            &frame_graph_output.diagnostics,
        );
        let graph = frame_graph_output.graph;
        let fallback_order = frame_graph_output.handles;
        let pass_executor_bindings = frame_graph_output.pass_executor_bindings;
        let order = match graph.execution_order() {
            Ok(order) => {
                self.clear_execution_order_error();
                order
            }
            Err(err) => {
                self.log_execution_order_error_once(&err);
                fallback_order
            }
        };
        let mut active_executors = BTreeSet::new();
        for handle in &order {
            if let Some(node) = graph.node(*handle) {
                let executor_name = pass_executor_bindings
                    .get(&node.name)
                    .map(String::as_str)
                    .unwrap_or(node.name.as_str());
                active_executors.insert(executor_name.to_string());
            }
        }

        let mut timings = packet.prepare_timings;
        self.prepare_registered_passes(
            device,
            queue,
            frame_data,
            &packet,
            &active_executors,
            render_executor_registry,
            &mut timings,
        );

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("engine_render_encoder"),
        });

        let mut missing_executors = Vec::<(String, String)>::new();
        for handle in order {
            let Some(node) = graph.node(handle) else {
                continue;
            };
            let executor_name = pass_executor_bindings
                .get(&node.name)
                .map(String::as_str)
                .unwrap_or(node.name.as_str());
            if let Some(builtin) = render_executor_registry.resolve_builtin(executor_name) {
                let executor = Self::builtin_pass_executor(builtin);
                executor.encode(
                    self,
                    device,
                    &mut encoder,
                    frame_view,
                    &packet,
                    node.pipeline.clone(),
                );
                continue;
            }
            if let Some(custom) = render_executor_registry.resolve_custom(executor_name) {
                let uses_ui_dispatch = executor_name.eq_ignore_ascii_case("builtin_ui_composite")
                    || node.name.eq_ignore_ascii_case("ui_composite");
                if uses_ui_dispatch {
                    let mut dispatch_ui = |encoder: &mut CommandEncoder| -> Result<()> {
                        self.encode_ui_pass(encoder, frame_view, &packet.prepared_ui);
                        Ok(())
                    };
                    let mut ctx = RenderPassEncodeContext::new(
                        device,
                        &mut encoder,
                        frame_view,
                        frame_data,
                        packet.surface_format,
                        packet.surface_size,
                        node.pipeline.clone(),
                    )
                    .with_ui_dispatch(&mut dispatch_ui);
                    if let Err(err) = custom.encode(&mut ctx) {
                        tracing::error!(
                            pass = node.name.as_str(),
                            executor = executor_name,
                            ?err,
                            "custom render pass executor encode failed"
                        );
                    }
                } else {
                    let mut dispatch_builtin = |encoder: &mut CommandEncoder,
                                                builtin: BuiltinRenderPassExecutor|
                     -> Result<()> {
                        let executor = Self::builtin_pass_executor(builtin);
                        executor.encode(
                            self,
                            device,
                            encoder,
                            frame_view,
                            &packet,
                            node.pipeline.clone(),
                        );
                        Ok(())
                    };
                    let mut ctx = RenderPassEncodeContext::new(
                        device,
                        &mut encoder,
                        frame_view,
                        frame_data,
                        packet.surface_format,
                        packet.surface_size,
                        node.pipeline.clone(),
                    )
                    .with_builtin_dispatch(&mut dispatch_builtin);
                    if let Err(err) = custom.encode(&mut ctx) {
                        tracing::error!(
                            pass = node.name.as_str(),
                            executor = executor_name,
                            ?err,
                            "custom render pass executor encode failed"
                        );
                    }
                }
                continue;
            }
            missing_executors.push((node.name.clone(), executor_name.to_string()));
        }
        self.log_missing_executors_once(&missing_executors);

        let encode_submit_start = Instant::now();
        {
            let _span = tracing::info_span!("renderer.encode_submit").entered();
            queue.submit(std::iter::once(encoder.finish()));
        }
        timings.encode_submit_ms = encode_submit_start.elapsed().as_secs_f32() * 1000.0;
        timings
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        frame_view: &TextureView,
        frame_data: &RenderFrameDataRegistry<'_>,
        draw_list: &UiDrawList,
        shader_registry: &mut ShaderRegistryResource,
        render_graph_registry: &RenderGraphRegistryResource,
        render_executor_registry: &RenderPassExecutorRegistryResource,
        ui_rect_shader: Option<ShaderHandle>,
        surface_format: TextureFormat,
        surface_width: f32,
        surface_height: f32,
    ) -> RendererFrameTimings {
        let packet = self.prepare_packet(
            device,
            queue,
            frame_data,
            draw_list,
            shader_registry,
            ui_rect_shader,
            surface_format,
            surface_width,
            surface_height,
        );
        self.render_packet(
            device,
            queue,
            frame_view,
            frame_data,
            packet,
            render_graph_registry,
            render_executor_registry,
        )
    }
}
