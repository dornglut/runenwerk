use super::*;
use crate::plugins::render::RenderPassId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeaturePassAction {
    Execute,
    Skip,
}

impl Renderer {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_packet(
        &mut self,
        device: &Device,
        queue: &Queue,
        frame_texture: &Texture,
        frame_view: &TextureView,
        prepared_frame: &PreparedRenderFrame,
        packet: RendererPreparedPacket,
        compiled_flows: &[CompiledRenderFlowPlan],
        shader_registry: &ShaderRegistryResource,
        preflight_config: crate::plugins::render::graph::RenderPreflightValidationConfigResource,
        debug_control: &RenderDebugControlResource,
        debug_config: &RenderDebugConfigResource,
        gpu_timing_capability: RenderGpuTimingCapability,
    ) -> Result<RendererFrameTimings> {
        let mut timings = packet.prepare_timings;
        self.last_pass_timings.clear();
        self.last_gpu_pass_timing_evidence.clear();
        self.last_runtime_resources.clear();
        self.last_pass_provenance.clear();
        let preflight_start = Instant::now();
        self.last_preflight_report =
            self.preflight_prepared_frame(prepared_frame, compiled_flows, preflight_config)?;
        timings.preflight_ms = preflight_start.elapsed().as_secs_f32() * 1000.0;
        self.last_capture_plan = ResolvedRenderCapturePlan::default();
        self.last_capture_selector_results.clear();
        self.last_captured_textures.clear();

        let flow_encode_start = Instant::now();
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("engine_render_encoder"),
        });
        let frame_index = prepared_frame.context.frame_index;
        let mut capture_runtime =
            FrameCaptureRuntime::new(frame_index, debug_control, &debug_config.capture_selectors);
        let mut pending_capture_readbacks = Vec::<PendingCaptureReadback>::new();
        let mut gpu_pass_timing_frame =
            if gpu_timing_capability == RenderGpuTimingCapability::Supported {
                GpuPassTimingFrame::new(
                    device,
                    queue,
                    gpu_timing_pass_capacity(prepared_frame, compiled_flows),
                )
            } else {
                None
            };

        let dynamic_target_history_signatures =
            prepared_frame.dynamic_target_history_signatures()?;
        self.dynamic_texture_targets.realize_for_frame(
            device,
            &prepared_frame.dynamic_texture_targets,
            &dynamic_target_history_signatures,
        )?;
        let upload_report = self
            .dynamic_texture_targets
            .apply_uploads(queue, &prepared_frame.dynamic_texture_uploads);
        for diagnostic in &upload_report.diagnostics {
            tracing::warn!(
                target = "renderer.dynamic_texture_upload",
                target_key = %diagnostic.target_key,
                message = %diagnostic.message,
                "dynamic texture upload rejected"
            );
        }

        let mut flow_runtime_cache = std::mem::take(&mut self.flow_runtime_cache);
        let render_result = (|| -> Result<()> {
            let active_flow_ids = compiled_flows
                .iter()
                .map(|flow| flow.flow_id)
                .collect::<Vec<_>>();

            flow_runtime_cache.retain(|flow_id, _| active_flow_ids.contains(flow_id));
            self.flow_pipeline_cache.retain_flows(&active_flow_ids);

            for flow in compiled_flows {
                let runtime_resources = flow_runtime_cache.entry(flow.flow_id).or_default();
                runtime_resources.realize_for_frame(
                    device,
                    flow,
                    packet.surface_size,
                    packet.surface_format,
                );
                let invocation_ids = prepared_frame
                    .flow_invocations_for_flow(flow.flow_id)
                    .map(|invocation| invocation.invocation_id.0.as_str())
                    .collect::<Vec<_>>();
                runtime_resources.retain_invocation_uniform_scopes(invocation_ids);
                for invocation in prepared_frame.flow_invocations_for_flow(flow.flow_id) {
                    let Some(view) = prepared_frame.view(invocation.view_id.as_str()) else {
                        bail!(
                            "prepared flow invocation '{}' references missing view '{}'",
                            invocation.invocation_id.0,
                            invocation.view_id
                        );
                    };
                    let mut invocation_packet = packet.clone();
                    invocation_packet.view_id = view.view_id.clone();
                    invocation_packet.surface_size = view.target_size_px;
                    runtime_resources.target_alias_bindings =
                        invocation.target_alias_bindings.clone();
                    runtime_resources
                        .set_active_invocation_uniform_scope(invocation.invocation_id.0.clone());
                    let effective_history_signature = invocation
                        .history_signature
                        .as_deref()
                        .or(view.history_signature.as_deref());
                    let invocation_result = (|| -> Result<()> {
                        runtime_resources.realize_invocation_history_textures(
                            device,
                            invocation.invocation_id.0.as_str(),
                            invocation_packet.surface_size,
                            invocation_packet.surface_format,
                            effective_history_signature,
                        )?;
                        self.upload_projected_uniform_buffers(
                            device,
                            queue,
                            invocation.invocation_id.0.as_str(),
                            &invocation.inputs,
                            runtime_resources,
                        )?;

                        let scheduled_passes =
                            schedule_invocation_passes(flow, &invocation.inputs)?;
                        for scheduled_pass in scheduled_passes {
                            if let Some(iteration) = scheduled_pass.fixed_step_iteration {
                                self.upload_fixed_step_iteration_uniform(
                                    device,
                                    queue,
                                    invocation.invocation_id.0.as_str(),
                                    runtime_resources,
                                    iteration.region,
                                    iteration
                                        .schedule
                                        .with_substep_index(iteration.substep_index),
                                )?;
                            }
                            let pass = scheduled_pass.pass;
                            if !self.pass_targets_active_view(
                                pass,
                                view.view_id.as_str(),
                                view.kind,
                            ) {
                                continue;
                            }
                            let pass_id = execution_pass_id(pass);
                            let pass_label = pass_id.to_string();
                            if let Some(feature_id) = execution_pass_feature_id(pass) {
                                match self.resolve_feature_pass_action(
                                    feature_id,
                                    pass_id,
                                    &invocation_packet,
                                )? {
                                    FeaturePassAction::Execute => {}
                                    FeaturePassAction::Skip => continue,
                                }
                            }
                            ensure_compiled_pass_is_supported(pass)?;
                            if capture_runtime.should_attempt_stage(CaptureStage::Before) {
                                self.queue_pass_texture_captures(
                                    device,
                                    &mut encoder,
                                    frame_texture,
                                    &invocation_packet,
                                    flow,
                                    pass,
                                    runtime_resources,
                                    CaptureStage::Before,
                                    &mut capture_runtime,
                                    &mut pending_capture_readbacks,
                                )?;
                            }
                            let pass_encode_start = Instant::now();
                            let pass_kind = execution_pass_kind_name(pass).to_string();
                            let gpu_timestamp_indices = if pass_supports_gpu_timestamp_writes(pass)
                            {
                                gpu_pass_timing_frame.as_mut().and_then(|frame| {
                                    frame.reserve_pass(
                                        frame_index,
                                        prepared_frame.surface.render_surface_id.raw(),
                                        flow.flow_id.to_string(),
                                        pass_label.clone(),
                                        pass_kind.clone(),
                                    )
                                })
                            } else {
                                None
                            };
                            let gpu_timestamp_writes = gpu_timestamp_indices.and_then(|indices| {
                                gpu_pass_timing_frame
                                    .as_ref()
                                    .map(|frame| frame.timestamp_writes(indices))
                            });
                            let evidence = self.encode_compiled_pass(
                                device,
                                &mut encoder,
                                frame_texture,
                                frame_view,
                                &invocation_packet,
                                flow,
                                &invocation.inputs,
                                pass,
                                shader_registry,
                                runtime_resources,
                                gpu_timestamp_writes,
                            )?;
                            if capture_runtime.should_attempt_stage(CaptureStage::After) {
                                self.queue_pass_texture_captures(
                                    device,
                                    &mut encoder,
                                    frame_texture,
                                    &invocation_packet,
                                    flow,
                                    pass,
                                    runtime_resources,
                                    CaptureStage::After,
                                    &mut capture_runtime,
                                    &mut pending_capture_readbacks,
                                )?;
                            }
                            self.last_pass_timings.push(PassTimingSample {
                                flow_id: flow.flow_id.to_string(),
                                pass_id: pass_label.clone(),
                                pass_kind: pass_kind.clone(),
                                millis: pass_encode_start.elapsed().as_secs_f32() * 1000.0,
                                dispatch_workgroups: evidence.dispatch_workgroups,
                            });
                            if gpu_timestamp_writes.is_none() {
                                self.last_gpu_pass_timing_evidence.push(
                                    gpu_timing_diagnostic_evidence_for_pass(
                                        gpu_timing_capability,
                                        frame_index,
                                        prepared_frame.surface.render_surface_id.raw(),
                                        flow.flow_id.to_string(),
                                        pass_label.clone(),
                                        pass_kind.clone(),
                                    ),
                                );
                            }
                            if debug_control.provenance_enabled {
                                let pass_resource_truth = collect_pass_resource_truth(
                                    flow.flow_id,
                                    pass,
                                    runtime_resources,
                                );
                                let material_binding = collect_pass_material_binding_evidence(
                                    &invocation_packet,
                                    pass,
                                );
                                self.last_pass_provenance.push(RenderPassProvenanceRecord {
                                    frame_index,
                                    flow_id: flow.flow_id.to_string(),
                                    pass_id: pass_label.clone(),
                                    pass_label: pass_label.clone(),
                                    pass_kind: execution_flow_pass_kind(pass),
                                    order_index: execution_pass_order_index(pass),
                                    feature_id: execution_pass_feature_id(pass)
                                        .map(|id| id.to_string()),
                                    shader_id: evidence.shader_id,
                                    shader_revision: evidence.shader_revision,
                                    fallback_used: evidence.fallback_used,
                                    pipeline_stats_key: evidence
                                        .pipeline_key
                                        .as_ref()
                                        .map(FlowPassPipelineKey::stats_key)
                                        .unwrap_or_default(),
                                    bind_group_layout_signature_hash: evidence
                                        .pipeline_key
                                        .as_ref()
                                        .map(|key| key.bind_group_layout_signature_hash)
                                        .unwrap_or_default(),
                                    material_specialization_fragment_hash: evidence
                                        .pipeline_key
                                        .as_ref()
                                        .map(|key| key.material_specialization_fragment_hash)
                                        .unwrap_or_default(),
                                    view_signature_hash: evidence
                                        .pipeline_key
                                        .as_ref()
                                        .map(|key| key.view_signature_hash)
                                        .unwrap_or_default(),
                                    feature_runtime_version: evidence
                                        .pipeline_key
                                        .as_ref()
                                        .map(|key| key.feature_runtime_version)
                                        .unwrap_or_default(),
                                    color_formats: evidence
                                        .pipeline_key
                                        .as_ref()
                                        .map(|key| key.color_formats.clone())
                                        .unwrap_or_default(),
                                    depth_format: evidence
                                        .pipeline_key
                                        .as_ref()
                                        .and_then(|key| key.depth_format),
                                    sample_count: evidence
                                        .pipeline_key
                                        .as_ref()
                                        .map(|key| key.sample_count)
                                        .unwrap_or(1),
                                    primitive_topology_class: evidence
                                        .pipeline_key
                                        .as_ref()
                                        .map(|key| key.primitive_topology_class)
                                        .unwrap_or(FlowPrimitiveTopologyClass::None),
                                    material_binding,
                                    render_targets: pass_resource_truth.render_targets,
                                    sampled_textures: pass_resource_truth.sampled_textures,
                                    storage_textures: pass_resource_truth.storage_textures,
                                    depth_targets: pass_resource_truth.depth_targets,
                                    capture_points_available: pass_resource_truth
                                        .capture_points_available,
                                });
                            }
                        }
                        Ok(())
                    })();
                    runtime_resources.clear_active_invocation_uniform_scope();
                    invocation_result?;
                }
                self.last_runtime_resources
                    .extend(runtime_resources.inspect_entries(flow.flow_id));
            }
            if capture_runtime.should_attempt_stage(CaptureStage::Final) {
                self.queue_final_surface_capture(
                    device,
                    &mut encoder,
                    frame_texture,
                    &packet,
                    &mut capture_runtime,
                    &mut pending_capture_readbacks,
                )?;
            }
            Ok(())
        })();
        self.flow_runtime_cache = flow_runtime_cache;
        render_result?;
        timings.flow_encode_ms = flow_encode_start.elapsed().as_secs_f32() * 1000.0;
        let pending_gpu_pass_timing_readback = gpu_pass_timing_frame
            .take()
            .and_then(|frame| frame.resolve(&mut encoder));

        let encode_submit_start = Instant::now();
        {
            let _span = tracing::info_span!("renderer.encode_submit").entered();
            queue.submit(std::iter::once(encoder.finish()));
        }
        timings.encode_submit_ms = encode_submit_start.elapsed().as_secs_f32() * 1000.0;
        if let Some(pending) = pending_gpu_pass_timing_readback {
            self.last_gpu_pass_timing_evidence = read_gpu_pass_timing_evidence(device, pending);
        }
        if !pending_capture_readbacks.is_empty() {
            for pending in pending_capture_readbacks.drain(..) {
                let (selector_index, capture) = read_capture_back(device, pending);
                capture_runtime.set_terminal(selector_index, capture.terminal.clone());
                self.last_captured_textures.push(capture);
            }
        }
        capture_runtime.finalize_unresolved();
        let (capture_plan, capture_selector_results) = capture_runtime.into_plan_and_results();
        self.last_capture_plan = capture_plan;
        self.last_capture_selector_results = capture_selector_results;
        Ok(timings)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        frame_texture: &Texture,
        frame_view: &TextureView,
        prepared_frame: &PreparedRenderFrame,
        shader_registry: &mut ShaderRegistryResource,
        compiled_flows: &[CompiledRenderFlowPlan],
        ui_rect_shader: Option<ShaderHandle>,
        ui_font_atlas: &UiFontAtlasResource,
        viewport_surface_bindings: &ViewportSurfaceBindingRegistry,
        surface_format: TextureFormat,
        preflight_config: crate::plugins::render::graph::RenderPreflightValidationConfigResource,
        debug_control: &RenderDebugControlResource,
        debug_config: &RenderDebugConfigResource,
        gpu_timing_capability: RenderGpuTimingCapability,
    ) -> Result<RendererFrameTimings> {
        let packet = self.prepare_packet(
            device,
            queue,
            prepared_frame,
            shader_registry,
            ui_rect_shader,
            ui_font_atlas,
            viewport_surface_bindings,
            surface_format,
        );
        self.render_packet(
            device,
            queue,
            frame_texture,
            frame_view,
            prepared_frame,
            packet,
            compiled_flows,
            shader_registry,
            preflight_config,
            debug_control,
            debug_config,
            gpu_timing_capability,
        )
    }

    fn upload_projected_uniform_buffers(
        &self,
        device: &Device,
        queue: &Queue,
        invocation_id: &str,
        flow_inputs: &PreparedFlowInputs,
        runtime_resources: &mut FlowRuntimeResources,
    ) -> Result<()> {
        for (buffer_id, bytes) in &flow_inputs.projected_uniform_bytes {
            let runtime_buffer = runtime_resources.realize_invocation_uniform_buffer(
                device,
                invocation_id,
                *buffer_id,
                bytes.len() as u64,
            )?;
            if bytes.len() as u64 > runtime_buffer.size {
                bail!(
                    "uniform upload for '{}' in invocation '{}' writes {} bytes but runtime buffer size is {}",
                    buffer_id,
                    invocation_id,
                    bytes.len(),
                    runtime_buffer.size
                );
            }
            queue.write_buffer(&runtime_buffer.buffer, 0, bytes);
        }

        Ok(())
    }

    fn upload_fixed_step_iteration_uniform(
        &self,
        device: &Device,
        queue: &Queue,
        invocation_id: &str,
        runtime_resources: &mut FlowRuntimeResources,
        region: &CompiledFixedStepRegion,
        uniform: RenderFixedStepIterationUniform,
    ) -> Result<()> {
        let bytes = uniform.to_uniform_bytes();
        let runtime_buffer = runtime_resources.realize_invocation_uniform_buffer(
            device,
            invocation_id,
            region.iteration_uniform,
            bytes.len() as u64,
        )?;
        if bytes.len() as u64 > runtime_buffer.size {
            bail!(
                "fixed-step iteration uniform upload for region '{}' in invocation '{}' writes {} bytes but runtime buffer size is {}",
                region.region_label,
                invocation_id,
                bytes.len(),
                runtime_buffer.size
            );
        }
        queue.write_buffer(&runtime_buffer.buffer, 0, &bytes);
        Ok(())
    }

    pub(super) fn pass_targets_active_view(
        &self,
        pass: &CompiledPassExecutionPlan,
        view_id: &str,
        view_kind: crate::plugins::render::PreparedViewKind,
    ) -> bool {
        let view_mask = match pass {
            CompiledPassExecutionPlan::Compute(value) => &value.view_mask,
            CompiledPassExecutionPlan::Fullscreen(value) => &value.view_mask,
            CompiledPassExecutionPlan::Graphics(value) => &value.view_mask,
            CompiledPassExecutionPlan::Copy(value) => &value.view_mask,
            CompiledPassExecutionPlan::Present(value) => &value.view_mask,
            CompiledPassExecutionPlan::BuiltinUiComposite(value) => &value.view_mask,
        };
        view_mask.includes(view_id, view_kind)
    }

    pub(super) fn resolve_feature_pass_action(
        &self,
        feature_id: RenderFeatureId,
        pass_id: RenderPassId,
        packet: &RendererPreparedPacket,
    ) -> Result<FeaturePassAction> {
        let gate = packet
            .feature_gates
            .get(&feature_id)
            .copied()
            .unwrap_or_default();

        match gate.status {
            FeatureContributionStatus::Ready => Ok(FeaturePassAction::Execute),
            FeatureContributionStatus::Stale => match gate.fallback_policy {
                FeatureFallbackPolicy::FailFrame => {
                    bail!(
                        "feature '{:?}' is stale for pass '{}' and fallback policy is fail-frame",
                        feature_id,
                        pass_id
                    )
                }
                FeatureFallbackPolicy::SkipFeaturePasses => Ok(FeaturePassAction::Skip),
                FeatureFallbackPolicy::ReuseLastGood | FeatureFallbackPolicy::EmptyContribution => {
                    Ok(FeaturePassAction::Execute)
                }
            },
            FeatureContributionStatus::Disabled | FeatureContributionStatus::Missing => {
                match gate.fallback_policy {
                    FeatureFallbackPolicy::FailFrame => {
                        bail!(
                            "feature '{:?}' is {:?} for pass '{}' and fallback policy is fail-frame",
                            feature_id,
                            gate.status,
                            pass_id
                        )
                    }
                    FeatureFallbackPolicy::SkipFeaturePasses => Ok(FeaturePassAction::Skip),
                    FeatureFallbackPolicy::ReuseLastGood
                    | FeatureFallbackPolicy::EmptyContribution => Ok(FeaturePassAction::Execute),
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn queue_pass_texture_captures(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        frame_texture: &Texture,
        packet: &RendererPreparedPacket,
        flow: &CompiledRenderFlowPlan,
        pass: &CompiledPassExecutionPlan,
        runtime_resources: &FlowRuntimeResources,
        stage: CaptureStage,
        capture_runtime: &mut FrameCaptureRuntime,
        pending_capture_readbacks: &mut Vec<PendingCaptureReadback>,
    ) -> Result<()> {
        let pass_id = execution_pass_id(pass);
        let pass_label = pass_id.to_string();

        for selector_index in 0..capture_runtime.selectors_len() {
            let Some((selector, terminal_is_set, existing_capture_point)) =
                capture_runtime.selector_snapshot(selector_index)
            else {
                continue;
            };
            if terminal_is_set || selector.stage != stage {
                continue;
            }
            let texture_class = runtime_resources
                .capture_texture_class(selector.resource_id.as_str(), selector.texture_class);
            let capture_point = RenderCapturePointIdentity {
                flow_id: flow.flow_id.to_string(),
                pass_id: pass_id.to_string(),
                stage,
                resource_id: selector.resource_id.clone(),
                texture_class,
            };
            if !selector.matches_point(&capture_point) {
                continue;
            }
            if let Some(existing_capture_point) = existing_capture_point
                && existing_capture_point != capture_point
            {
                capture_runtime.set_terminal_with_reason(
                    selector_index,
                    RenderCaptureTerminalCode::Unsupported,
                    "selector_multiple_matches",
                    format!(
                        "selector '{}' matched multiple capture points: '{}' and '{}'",
                        selector.describe(),
                        existing_capture_point.resource_id,
                        capture_point.resource_id,
                    ),
                );
                continue;
            }
            let identity = RenderCaptureIdentity {
                frame_index: capture_runtime.frame_index,
                pass_label: pass_label.clone(),
                capture_point: capture_point.clone(),
            };
            capture_runtime.set_matched_identity(selector_index, capture_point, identity.clone());

            let resolved_texture = match runtime_resources
                .resolve_resource_key_from_input(selector.resource_id.as_str())
            {
                Some(RuntimeResourceKey::DynamicTexture(key)) => {
                    self.dynamic_texture_targets.texture_ref(pass_id, &key)
                }
                None => {
                    if let Some(key) =
                        crate::plugins::render::RenderDynamicTextureTargetKey::from_label(
                            selector.resource_id.as_str(),
                        )
                    {
                        self.dynamic_texture_targets.texture_ref(pass_id, &key)
                    } else {
                        runtime_resources.resolve_texture_from_label(
                            pass_label.as_str(),
                            selector.resource_id.as_str(),
                            frame_texture,
                            packet.surface_size,
                            packet.surface_format,
                        )
                    }
                }
                _ => runtime_resources.resolve_texture_from_label(
                    pass_label.as_str(),
                    selector.resource_id.as_str(),
                    frame_texture,
                    packet.surface_size,
                    packet.surface_format,
                ),
            };
            let resolved_texture = match resolved_texture {
                Ok(value) => value,
                Err(err) => {
                    let terminal = RenderCaptureTerminal::with_reason(
                        RenderCaptureTerminalCode::Skipped,
                        "texture_resolution_failed",
                        err.to_string(),
                    );
                    capture_runtime.set_terminal(selector_index, terminal.clone());
                    self.last_captured_textures.push(RenderCapturedTexture {
                        identity,
                        width: 0,
                        height: 0,
                        format: "unknown".to_string(),
                        bytes_rgba8: None,
                        terminal,
                    });
                    continue;
                }
            };

            let readback_format = texture_readback_format(resolved_texture.format);
            let readback_format = match readback_format {
                Some(value) => value,
                None => {
                    let terminal = RenderCaptureTerminal::with_reason(
                        RenderCaptureTerminalCode::Unsupported,
                        "unsupported_readback_format",
                        format!(
                            "readback for format {:?} is not implemented yet",
                            resolved_texture.format
                        ),
                    );
                    capture_runtime.set_terminal(selector_index, terminal.clone());
                    self.last_captured_textures.push(RenderCapturedTexture {
                        identity,
                        width: resolved_texture.size.0,
                        height: resolved_texture.size.1,
                        format: format!("{:?}", resolved_texture.format),
                        bytes_rgba8: None,
                        terminal,
                    });
                    continue;
                }
            };

            match enqueue_texture_capture_copy(
                device,
                encoder,
                selector_index,
                identity,
                resolved_texture.texture,
                resolved_texture.size,
                resolved_texture.format,
                readback_format,
            ) {
                Ok(pending) => pending_capture_readbacks.push(pending),
                Err(err) => {
                    let terminal = RenderCaptureTerminal::with_reason(
                        RenderCaptureTerminalCode::ReadbackFailed,
                        "enqueue_capture_copy_failed",
                        err.to_string(),
                    );
                    capture_runtime.set_terminal(selector_index, terminal.clone());
                    self.last_captured_textures.push(RenderCapturedTexture {
                        identity: RenderCaptureIdentity {
                            frame_index: capture_runtime.frame_index,
                            pass_label: pass_label.clone(),
                            capture_point: RenderCapturePointIdentity {
                                flow_id: flow.flow_id.to_string(),
                                pass_id: pass_id.to_string(),
                                stage,
                                resource_id: selector.resource_id.clone(),
                                texture_class,
                            },
                        },
                        width: resolved_texture.size.0,
                        height: resolved_texture.size.1,
                        format: format!("{:?}", resolved_texture.format),
                        bytes_rgba8: None,
                        terminal,
                    });
                }
            }
        }

        Ok(())
    }

    fn queue_final_surface_capture(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        frame_texture: &Texture,
        packet: &RendererPreparedPacket,
        capture_runtime: &mut FrameCaptureRuntime,
        pending_capture_readbacks: &mut Vec<PendingCaptureReadback>,
    ) -> Result<()> {
        for selector_index in 0..capture_runtime.selectors_len() {
            let Some((selector, terminal_is_set, existing_capture_point)) =
                capture_runtime.selector_snapshot(selector_index)
            else {
                continue;
            };
            if terminal_is_set || selector.stage != CaptureStage::Final {
                continue;
            }
            let capture_point = RenderCapturePointIdentity {
                flow_id: "frame".to_string(),
                pass_id: "frame.final".to_string(),
                stage: CaptureStage::Final,
                resource_id: selector.resource_id.clone(),
                texture_class: selector.texture_class,
            };
            if !selector.matches_point(&capture_point) {
                continue;
            }
            if let Some(existing_capture_point) = existing_capture_point
                && existing_capture_point != capture_point
            {
                capture_runtime.set_terminal_with_reason(
                    selector_index,
                    RenderCaptureTerminalCode::Unsupported,
                    "selector_multiple_matches",
                    format!(
                        "selector '{}' matched multiple final-stage capture points",
                        selector.describe()
                    ),
                );
                continue;
            }
            let identity = RenderCaptureIdentity {
                frame_index: capture_runtime.frame_index,
                pass_label: "frame.final".to_string(),
                capture_point: capture_point.clone(),
            };
            capture_runtime.set_matched_identity(selector_index, capture_point, identity.clone());
            if selector.resource_id != SURFACE_COLOR_RESOURCE_LABEL {
                let terminal = RenderCaptureTerminal::with_reason(
                    RenderCaptureTerminalCode::Unsupported,
                    "final_stage_resource_unsupported",
                    "final-stage capture currently supports only surface.color".to_string(),
                );
                capture_runtime.set_terminal(selector_index, terminal.clone());
                self.last_captured_textures.push(RenderCapturedTexture {
                    identity,
                    width: packet.surface_size.0,
                    height: packet.surface_size.1,
                    format: format!("{:?}", packet.surface_format),
                    bytes_rgba8: None,
                    terminal,
                });
                continue;
            }

            let Some(readback_format) = texture_readback_format(packet.surface_format) else {
                let terminal = RenderCaptureTerminal::with_reason(
                    RenderCaptureTerminalCode::Unsupported,
                    "unsupported_final_readback_format",
                    format!(
                        "readback for format {:?} is not implemented yet",
                        packet.surface_format
                    ),
                );
                capture_runtime.set_terminal(selector_index, terminal.clone());
                self.last_captured_textures.push(RenderCapturedTexture {
                    identity,
                    width: packet.surface_size.0,
                    height: packet.surface_size.1,
                    format: format!("{:?}", packet.surface_format),
                    bytes_rgba8: None,
                    terminal,
                });
                continue;
            };

            match enqueue_texture_capture_copy(
                device,
                encoder,
                selector_index,
                identity,
                frame_texture,
                packet.surface_size,
                packet.surface_format,
                readback_format,
            ) {
                Ok(pending) => pending_capture_readbacks.push(pending),
                Err(err) => {
                    let terminal = RenderCaptureTerminal::with_reason(
                        RenderCaptureTerminalCode::ReadbackFailed,
                        "enqueue_capture_copy_failed",
                        err.to_string(),
                    );
                    capture_runtime.set_terminal(selector_index, terminal.clone());
                    self.last_captured_textures.push(RenderCapturedTexture {
                        identity: RenderCaptureIdentity {
                            frame_index: capture_runtime.frame_index,
                            pass_label: "frame.final".to_string(),
                            capture_point: RenderCapturePointIdentity {
                                flow_id: "frame".to_string(),
                                pass_id: "frame.final".to_string(),
                                stage: CaptureStage::Final,
                                resource_id: SURFACE_COLOR_RESOURCE_LABEL.to_string(),
                                texture_class: selector.texture_class,
                            },
                        },
                        width: packet.surface_size.0,
                        height: packet.surface_size.1,
                        format: format!("{:?}", packet.surface_format),
                        bytes_rgba8: None,
                        terminal,
                    });
                }
            }
        }
        Ok(())
    }
}

fn gpu_timing_pass_capacity(
    prepared_frame: &PreparedRenderFrame,
    compiled_flows: &[CompiledRenderFlowPlan],
) -> usize {
    compiled_flows
        .iter()
        .map(|flow| {
            prepared_frame
                .flow_invocations_for_flow(flow.flow_id)
                .map(|invocation| {
                    scheduled_timestamped_pass_count(flow, &invocation.inputs).unwrap_or_else(
                        || {
                            flow.execution
                                .passes
                                .iter()
                                .filter(|pass| pass_supports_gpu_timestamp_writes(pass))
                                .count()
                        },
                    )
                })
                .sum::<usize>()
        })
        .sum()
}

#[derive(Clone, Copy)]
struct ScheduledFixedStepIteration<'a> {
    region: &'a CompiledFixedStepRegion,
    schedule: RenderFixedStepIterationUniform,
    substep_index: u32,
}

#[derive(Clone, Copy)]
struct ScheduledCompiledPass<'a> {
    pass: &'a CompiledPassExecutionPlan,
    fixed_step_iteration: Option<ScheduledFixedStepIteration<'a>>,
}

fn schedule_invocation_passes<'a>(
    flow: &'a CompiledRenderFlowPlan,
    flow_inputs: &PreparedFlowInputs,
) -> Result<Vec<ScheduledCompiledPass<'a>>> {
    let mut scheduled = Vec::<ScheduledCompiledPass<'a>>::new();
    let mut consumed_region_passes = BTreeSet::<RenderPassId>::new();

    for pass in &flow.execution.passes {
        let pass_id = execution_pass_id(pass);
        if consumed_region_passes.contains(&pass_id) {
            continue;
        }

        if let Some(region) = fixed_step_region_starting_at(flow, pass_id) {
            let schedule = fixed_step_schedule_for_region(region, flow_inputs)?;
            for substep_index in 0..schedule.submitted_substeps {
                for region_pass_id in &region.pass_ids {
                    let region_pass =
                        compiled_pass_by_id(flow, *region_pass_id).ok_or_else(|| {
                            anyhow::anyhow!(
                                "fixed-step region '{}' references missing compiled pass '{}'",
                                region.region_label,
                                region_pass_id
                            )
                        })?;
                    scheduled.push(ScheduledCompiledPass {
                        pass: region_pass,
                        fixed_step_iteration: Some(ScheduledFixedStepIteration {
                            region,
                            schedule,
                            substep_index,
                        }),
                    });
                }
            }
            consumed_region_passes.extend(region.pass_ids.iter().copied());
        } else {
            scheduled.push(ScheduledCompiledPass {
                pass,
                fixed_step_iteration: None,
            });
        }
    }

    Ok(scheduled)
}

fn scheduled_timestamped_pass_count(
    flow: &CompiledRenderFlowPlan,
    flow_inputs: &PreparedFlowInputs,
) -> Option<usize> {
    schedule_invocation_passes(flow, flow_inputs)
        .ok()
        .map(|passes| {
            passes
                .iter()
                .filter(|scheduled| pass_supports_gpu_timestamp_writes(scheduled.pass))
                .count()
        })
}

fn fixed_step_region_starting_at(
    flow: &CompiledRenderFlowPlan,
    pass_id: RenderPassId,
) -> Option<&CompiledFixedStepRegion> {
    flow.execution
        .fixed_step_regions
        .iter()
        .find(|region| region.pass_ids.first().copied() == Some(pass_id))
}

fn fixed_step_schedule_for_region(
    region: &CompiledFixedStepRegion,
    flow_inputs: &PreparedFlowInputs,
) -> Result<RenderFixedStepIterationUniform> {
    let bytes = flow_inputs
        .projected_uniform_bytes
        .get(&region.iteration_uniform)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "fixed-step region '{}' is missing prepared iteration uniform '{}'",
                region.region_label,
                region.iteration_uniform
            )
        })?;
    let mut schedule =
        RenderFixedStepIterationUniform::from_uniform_bytes(bytes).ok_or_else(|| {
            anyhow::anyhow!(
                "fixed-step region '{}' prepared iteration uniform '{}' has invalid byte shape",
                region.region_label,
                region.iteration_uniform
            )
        })?;
    schedule.submitted_substeps = schedule.submitted_substeps.min(region.max_substeps);
    schedule.max_substeps = region.max_substeps;
    Ok(schedule)
}

fn compiled_pass_by_id(
    flow: &CompiledRenderFlowPlan,
    pass_id: RenderPassId,
) -> Option<&CompiledPassExecutionPlan> {
    flow.execution
        .passes
        .iter()
        .find(|pass| execution_pass_id(pass) == pass_id)
}

fn pass_supports_gpu_timestamp_writes(pass: &CompiledPassExecutionPlan) -> bool {
    matches!(
        pass,
        CompiledPassExecutionPlan::Compute(_)
            | CompiledPassExecutionPlan::Fullscreen(_)
            | CompiledPassExecutionPlan::Graphics(_)
            | CompiledPassExecutionPlan::BuiltinUiComposite(_)
    )
}

fn gpu_timing_diagnostic_evidence_for_pass(
    capability: RenderGpuTimingCapability,
    frame_index: u64,
    render_surface_id: u64,
    flow_id: String,
    pass_id: String,
    pass_kind: String,
) -> RenderPassTimingEvidence {
    let diagnostic = match capability {
        RenderGpuTimingCapability::Supported => RenderGpuTimingDiagnostic::unavailable_this_frame(
            "timestamp queries are supported, but GPU pass timestamp resolve/readback is not available for this frame",
        ),
        RenderGpuTimingCapability::Unsupported => RenderGpuTimingDiagnostic::unsupported(
            "timestamp queries are not supported by the active WGPU backend",
        ),
        RenderGpuTimingCapability::UnavailableThisFrame => {
            RenderGpuTimingDiagnostic::unavailable_this_frame(
                "GPU pass timestamp data is unavailable for this frame",
            )
        }
        RenderGpuTimingCapability::ReadbackPending => {
            RenderGpuTimingDiagnostic::readback_pending("GPU pass timestamp readback is pending")
        }
    };
    RenderPassTimingEvidence::gpu_diagnostic(
        Some(frame_index),
        Some(render_surface_id),
        flow_id,
        pass_id,
        pass_kind,
        diagnostic,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::{GpuStorage, GpuUniform, RenderFlow, compile_flow_plan};

    #[derive(Debug, Clone, Copy, GpuStorage)]
    struct TestCell {
        value: u32,
    }

    #[derive(Debug, Clone, Copy, GpuUniform)]
    struct TestParams {
        value: u32,
    }

    #[derive(Debug, Clone, ecs::Resource)]
    struct TestState;

    impl TestState {
        fn params(&self) -> TestParams {
            TestParams { value: 1 }
        }

        fn dispatch(&self) -> [u32; 3] {
            [1, 1, 1]
        }
    }

    fn fixed_step_test_plan() -> CompiledRenderFlowPlan {
        let (flow, cells) = RenderFlow::new("fixed.step.schedule")
            .with_state::<TestState>()
            .storage_array::<TestCell>("cells", 4);
        let flow = flow
            .compute_pass("step.a")
            .uniform_from_state(TestState::params)
            .bind_storage(cells.clone())
            .dispatch_from_state(TestState::dispatch)
            .finish()
            .compute_pass("step.b")
            .uniform_from_state(TestState::params)
            .bind_storage(cells)
            .dispatch_from_state(TestState::dispatch)
            .depends_on("step.a")
            .finish()
            .fixed_step_region("simulation", 4, ["step.a", "step.b"])
            .validate()
            .expect("fixed-step test flow should validate");
        compile_flow_plan(&flow).expect("fixed-step test flow should compile")
    }

    fn inputs_for_substeps(
        plan: &CompiledRenderFlowPlan,
        submitted_substeps: u32,
    ) -> PreparedFlowInputs {
        let mut inputs = PreparedFlowInputs::default();
        for pass in &plan.execution.passes {
            if let CompiledPassExecutionPlan::Compute(pass) = pass {
                inputs
                    .projected_dispatch_workgroups
                    .insert(pass.pass_id, [1, 1, 1]);
                for uniform_id in &pass.bindings.uniform_order {
                    inputs
                        .projected_uniform_bytes
                        .entry(*uniform_id)
                        .or_insert_with(|| vec![0; 16]);
                }
            }
        }
        let region = plan
            .execution
            .fixed_step_regions
            .first()
            .expect("test plan should compile fixed-step region");
        inputs.projected_uniform_bytes.insert(
            region.iteration_uniform,
            RenderFixedStepIterationUniform::new(
                0,
                submitted_substeps,
                region.max_substeps,
                0,
                1.0 / 60.0,
                0.0,
            )
            .to_uniform_bytes(),
        );
        inputs
    }

    fn scheduled_pass_ids(
        plan: &CompiledRenderFlowPlan,
        submitted_substeps: u32,
    ) -> Vec<RenderPassId> {
        let inputs = inputs_for_substeps(plan, submitted_substeps);
        schedule_invocation_passes(plan, &inputs)
            .expect("fixed-step schedule should expand")
            .into_iter()
            .map(|scheduled| execution_pass_id(scheduled.pass))
            .collect()
    }

    #[test]
    fn fixed_step_scheduler_expands_zero_one_and_many_substeps() {
        let plan = fixed_step_test_plan();
        let region = &plan.execution.fixed_step_regions[0];

        assert!(scheduled_pass_ids(&plan, 0).is_empty());
        assert_eq!(scheduled_pass_ids(&plan, 1), region.pass_ids);

        let many = scheduled_pass_ids(&plan, 3);
        assert_eq!(many.len(), region.pass_ids.len() * 3);
        assert_eq!(&many[0..2], region.pass_ids.as_slice());
        assert_eq!(&many[2..4], region.pass_ids.as_slice());
        assert_eq!(&many[4..6], region.pass_ids.as_slice());
    }

    #[test]
    fn fixed_step_scheduler_clamps_to_region_max_substeps() {
        let plan = fixed_step_test_plan();
        let region = &plan.execution.fixed_step_regions[0];
        let scheduled = scheduled_pass_ids(&plan, 99);

        assert_eq!(
            scheduled.len(),
            region.pass_ids.len() * region.max_substeps as usize
        );
    }
}
