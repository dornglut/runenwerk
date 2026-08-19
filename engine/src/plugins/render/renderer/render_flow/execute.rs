use super::*;
use super::{
    canonical_work::{
        CanonicalInvocationPreparation, CanonicalPassProjection, RealizedLogicalBufferUpload,
        allocate_aux_occurrence, prepare_canonical_invocation,
    },
    logical_operations::project_buffer_upload,
    logical_timing::LogicalGpuPassTimingPlan,
    occurrences::expand_render_pass_occurrences,
};
use crate::plugins::gpu::GpuWorkOperation;
use crate::plugins::render::{
    PreparedRenderWorkPlan, RenderGpuWorkOccurrenceId, RenderGpuWorkPayload, RenderPassId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeaturePassAction {
    Execute,
    Skip,
}

/// Renderer-local handoff between the G4C1/G4C2/G4C3 realization phase and the G5 operation
/// phase. It holds no raw device or queue reference.
struct RendererRealizationBatch<'a> {
    packet: RendererPreparedPacket,
    capture_runtime: FrameCaptureRuntime,
    invocations: Vec<RealizedFlowInvocation<'a>>,
    final_captures: Vec<PreparedCaptureReadback>,
}

struct RealizedFlowInvocation<'a> {
    flow: &'a CompiledRenderFlowPlan,
    invocation: &'a crate::plugins::render::PreparedFlowInvocation,
    packet: RendererPreparedPacket,
    projected_uploads: Vec<RealizedLogicalBufferUpload>,
    scheduled_passes: Vec<RealizedScheduledPass<'a>>,
    timing_frame: Option<GpuPassTimingFrame>,
    canonical_work: Option<Box<PreparedRenderWorkPlan>>,
}

struct RealizedScheduledPass<'a> {
    occurrence: RenderGpuWorkOccurrenceId,
    control_order_after: Vec<RenderGpuWorkOccurrenceId>,
    fixed_step_upload: Option<RealizedLogicalBufferUpload>,
    execution: RealizedPassExecution<'a>,
}

/// One actual render-domain execution occurrence after view/feature/fixed-step control has been
/// resolved. G5A deliberately resolves those decisions before late logical GPU work is formed, so
/// this value never represents a skipped pass or a ghost fixed-step mutation.
struct RealizedPassExecution<'a> {
    pass: &'a CompiledPassExecutionPlan,
    timestamp_indices: Option<GpuPassTimestampIndices>,
    pipeline: Option<PreparedPipelinePass>,
    before_captures: Vec<PreparedCaptureReadback>,
    after_captures: Vec<PreparedCaptureReadback>,
}

#[derive(Debug, Clone)]
struct ScheduledInvocationWork {
    operation: GpuWorkOperation,
    payload: RenderGpuWorkPayload,
}

impl Renderer {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_packet(
        &mut self,
        context: &GpuContext,
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
        self.last_capture_plan = ResolvedRenderCapturePlan::default();
        self.last_capture_selector_results.clear();
        self.last_captured_textures.clear();

        let preflight_start = Instant::now();
        self.last_preflight_report =
            self.preflight_prepared_frame(prepared_frame, compiled_flows, preflight_config)?;
        timings.preflight_ms = preflight_start.elapsed().as_secs_f32() * 1000.0;

        let flow_encode_start = Instant::now();
        // Phase one: all G4C1/G4C2/G4C3 realization completes without a raw device/queue loan.
        let mut batch = self.realize_render_batch(
            context,
            frame_texture,
            frame_view,
            prepared_frame,
            packet,
            compiled_flows,
            shader_registry,
            debug_control,
            debug_config,
            gpu_timing_capability,
        )?;

        let encode_submit_start = Instant::now();
        // Phase two: one non-reentrant raw loan covers only the temporary pre-G5B physical
        // realization path. Generic GPU meaning is no longer reconstructed here.
        {
            let _span = tracing::info_span!("renderer.encode_submit").entered();
            let loan = context.current_render_device_queue();
            let mut encoder = loan
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("engine_render_encoder"),
                });
            std::mem::take(&mut batch.packet.pending_operations).apply(context, loan.queue)?;
            let upload_report = self.dynamic_texture_targets.apply_uploads(
                context,
                loan.queue,
                &prepared_frame.dynamic_texture_uploads,
            );
            for diagnostic in &upload_report.diagnostics {
                tracing::warn!(
                    target = "renderer.dynamic_texture_upload",
                    target_key = %diagnostic.target_key,
                    message = %diagnostic.message,
                    "dynamic texture upload rejected"
                );
            }
            let (pending_gpu_pass_timing_readbacks, mut pending_capture_readbacks) = self
                .execute_realized_batch(
                    context,
                    loan.device,
                    loan.queue,
                    &mut encoder,
                    frame_texture,
                    frame_view,
                    prepared_frame,
                    debug_control,
                    gpu_timing_capability,
                    &mut batch,
                )?;
            loan.queue.submit(std::iter::once(encoder.finish()));
            if !pending_gpu_pass_timing_readbacks.is_empty() {
                self.last_gpu_pass_timing_evidence.clear();
                for pending in pending_gpu_pass_timing_readbacks {
                    self.last_gpu_pass_timing_evidence
                        .extend(read_gpu_pass_timing_evidence(context, loan.device, pending));
                }
            }
            for pending in pending_capture_readbacks.drain(..) {
                let (selector_index, capture) = read_capture_back(context, loan.device, pending);
                batch
                    .capture_runtime
                    .set_terminal(selector_index, capture.terminal.clone());
                self.last_captured_textures.push(capture);
            }
        }
        timings.flow_encode_ms = flow_encode_start.elapsed().as_secs_f32() * 1000.0;
        timings.encode_submit_ms = encode_submit_start.elapsed().as_secs_f32() * 1000.0;
        batch.capture_runtime.finalize_unresolved();
        let (capture_plan, capture_selector_results) =
            batch.capture_runtime.into_plan_and_results();
        self.last_capture_plan = capture_plan;
        self.last_capture_selector_results = capture_selector_results;
        Ok(timings)
    }

    #[allow(clippy::too_many_arguments)]
    fn realize_render_batch<'a>(
        &mut self,
        context: &GpuContext,
        frame_texture: &Texture,
        frame_view: &TextureView,
        prepared_frame: &'a PreparedRenderFrame,
        mut packet: RendererPreparedPacket,
        compiled_flows: &'a [CompiledRenderFlowPlan],
        shader_registry: &ShaderRegistryResource,
        debug_control: &RenderDebugControlResource,
        debug_config: &RenderDebugConfigResource,
        gpu_timing_capability: RenderGpuTimingCapability,
    ) -> Result<RendererRealizationBatch<'a>> {
        let dynamic_target_history_signatures =
            prepared_frame.dynamic_target_history_signatures()?;
        self.dynamic_texture_targets.realize_for_frame(
            context,
            &prepared_frame.dynamic_texture_targets,
            &dynamic_target_history_signatures,
        )?;
        let (viewport, product_surface) = self.realize_ui_dynamic_bind_groups(
            context,
            &packet.prepared_ui,
            &packet.viewport_surface_bindings,
        )?;
        packet.ui_dynamic_bind_groups = UiDynamicBindGroups {
            viewport,
            product_surface,
        };

        let frame_index = prepared_frame.context.frame_index;
        let mut capture_runtime =
            FrameCaptureRuntime::new(frame_index, debug_control, &debug_config.capture_selectors);
        let mut flow_runtime_cache = std::mem::take(&mut self.flow_runtime_cache);
        let realization_result = (|| -> Result<Vec<RealizedFlowInvocation<'a>>> {
            let active_flow_ids = compiled_flows
                .iter()
                .map(|flow| flow.flow_id)
                .collect::<Vec<_>>();
            flow_runtime_cache.retain(|flow_id, _| active_flow_ids.contains(flow_id));
            self.flow_pipeline_cache.retain_flows(&active_flow_ids);

            let mut invocations = Vec::new();
            for flow in compiled_flows {
                let runtime_resources = flow_runtime_cache.entry(flow.flow_id).or_default();
                runtime_resources.realize_for_frame(
                    context,
                    flow,
                    packet.surface_size,
                    packet.surface_format,
                )?;
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
                    invocation_packet.pending_operations = RendererPendingOperations::default();
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

                    let invocation_result = (|| -> Result<RealizedFlowInvocation<'a>> {
                        runtime_resources.realize_invocation_history_textures(
                            context,
                            invocation.invocation_id.0.as_str(),
                            invocation_packet.surface_size,
                            invocation_packet.surface_format,
                            effective_history_signature,
                        )?;

                        // Resolve render-domain runtime control before any canonical G3 work is
                        // constructed. A skipped pass has no GPU occurrence and therefore no
                        // hidden fixed-step mutation or timestamp slot.
                        let occurrences =
                            expand_render_pass_occurrences(flow, &invocation.inputs, |pass| {
                                if !self.pass_targets_active_view(
                                    pass,
                                    view.view_id.as_str(),
                                    view.kind,
                                ) {
                                    return Ok(false);
                                }
                                let pass_id = execution_pass_id(pass);
                                if let Some(feature_id) = execution_pass_feature_id(pass)
                                    && self.resolve_feature_pass_action(
                                        feature_id,
                                        pass_id,
                                        &invocation_packet,
                                    )? == FeaturePassAction::Skip
                                {
                                    return Ok(false);
                                }
                                ensure_compiled_pass_is_supported(pass)?;
                                Ok(true)
                            })?;
                        let mut maximum_occurrence = occurrences
                            .iter()
                            .map(|occurrence| occurrence.occurrence_id.raw())
                            .max()
                            .unwrap_or(0);
                        let projected_uploads = self.realize_projected_uniform_uploads(
                            context,
                            invocation.invocation_id.0.as_str(),
                            &invocation.inputs,
                            runtime_resources,
                            &mut maximum_occurrence,
                        )?;

                        let logical_timing_plan =
                            if gpu_timing_capability == RenderGpuTimingCapability::Supported {
                                Some(LogicalGpuPassTimingPlan::new(
                                    occurrences.iter().map(|occurrence| occurrence.pass),
                                )?)
                            } else {
                                None
                            };
                        let timing_frame = match logical_timing_plan
                            .as_ref()
                            .and_then(LogicalGpuPassTimingPlan::timing)
                        {
                            Some(timing) => Some(GpuPassTimingFrame::new(
                                context,
                                timing.query_set(),
                                timing.resolve_buffer(),
                                timing.readback_buffer(),
                                timing.query_capacity(),
                            )?),
                            None => None,
                        };
                        let mut realized_passes = Vec::new();
                        for (ordinal, occurrence) in occurrences.into_iter().enumerate() {
                            let fixed_step_upload = occurrence
                                .fixed_step_iteration
                                .map(|iteration| {
                                    self.realize_fixed_step_iteration_upload(
                                        context,
                                        invocation.invocation_id.0.as_str(),
                                        runtime_resources,
                                        iteration.region,
                                        iteration
                                            .schedule
                                            .with_substep_index(iteration.substep_index),
                                        &mut maximum_occurrence,
                                        occurrence.control_order_after.clone(),
                                    )
                                })
                                .transpose()?;
                            let pass = occurrence.pass;
                            let mut before_captures = Vec::new();
                            if capture_runtime.should_attempt_stage(CaptureStage::Before) {
                                self.prepare_pass_texture_captures(
                                    context,
                                    frame_texture,
                                    &invocation_packet,
                                    flow,
                                    pass,
                                    runtime_resources,
                                    CaptureStage::Before,
                                    &mut capture_runtime,
                                    &mut before_captures,
                                )?;
                            }
                            let pipeline = self.realize_compiled_pass(
                                context,
                                frame_texture,
                                frame_view,
                                &invocation_packet,
                                flow,
                                &invocation.inputs,
                                pass,
                                shader_registry,
                                runtime_resources,
                            )?;
                            let mut after_captures = Vec::new();
                            if capture_runtime.should_attempt_stage(CaptureStage::After) {
                                self.prepare_pass_texture_captures(
                                    context,
                                    frame_texture,
                                    &invocation_packet,
                                    flow,
                                    pass,
                                    runtime_resources,
                                    CaptureStage::After,
                                    &mut capture_runtime,
                                    &mut after_captures,
                                )?;
                            }
                            let timestamp_indices = logical_timing_plan
                                .as_ref()
                                .map(|plan| plan.range_for_occurrence(ordinal))
                                .transpose()?
                                .flatten();
                            realized_passes.push(RealizedScheduledPass {
                                occurrence: occurrence.occurrence_id,
                                control_order_after: occurrence.control_order_after,
                                fixed_step_upload,
                                execution: RealizedPassExecution {
                                    pass,
                                    timestamp_indices,
                                    pipeline,
                                    before_captures,
                                    after_captures,
                                },
                            });
                        }
                        let canonical_projections = realized_passes
                            .iter()
                            .map(|scheduled| CanonicalPassProjection {
                                occurrence: scheduled.occurrence,
                                control_order_after: &scheduled.control_order_after,
                                pass: scheduled.execution.pass,
                                pipeline: scheduled.execution.pipeline.as_ref(),
                                timestamp_indices: scheduled.execution.timestamp_indices,
                                fixed_step_upload: scheduled.fixed_step_upload.as_ref(),
                                has_capture_work: !scheduled.execution.before_captures.is_empty()
                                    || !scheduled.execution.after_captures.is_empty(),
                            })
                            .collect::<Vec<_>>();
                        let canonical_work = match prepare_canonical_invocation(
                            context,
                            flow,
                            &invocation.inputs,
                            runtime_resources,
                            &projected_uploads,
                            &canonical_projections,
                            logical_timing_plan
                                .as_ref()
                                .and_then(LogicalGpuPassTimingPlan::timing),
                        )? {
                            CanonicalInvocationPreparation::Prepared(work) => Some(work),
                            CanonicalInvocationPreparation::PreG7Residual => None,
                        };
                        Ok(RealizedFlowInvocation {
                            flow,
                            invocation,
                            packet: invocation_packet,
                            projected_uploads,
                            scheduled_passes: realized_passes,
                            timing_frame,
                            canonical_work,
                        })
                    })();
                    runtime_resources.clear_active_invocation_uniform_scope();
                    invocations.push(invocation_result?);
                }
                self.last_runtime_resources
                    .extend(runtime_resources.inspect_entries(flow.flow_id));
            }
            Ok(invocations)
        })();
        self.flow_runtime_cache = flow_runtime_cache;
        let mut invocations = realization_result?;
        let mut final_captures = Vec::new();
        if capture_runtime.should_attempt_stage(CaptureStage::Final) {
            self.prepare_final_surface_capture(
                context,
                frame_texture,
                &packet,
                &mut capture_runtime,
                &mut final_captures,
            )?;
        }
        if !final_captures.is_empty() {
            for invocation in &mut invocations {
                invocation.canonical_work = None;
            }
        }
        Ok(RendererRealizationBatch {
            packet,
            capture_runtime,
            invocations,
            final_captures,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn execute_realized_batch(
        &mut self,
        context: &GpuContext,
        _device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        frame_texture: &Texture,
        frame_view: &TextureView,
        prepared_frame: &PreparedRenderFrame,
        debug_control: &RenderDebugControlResource,
        gpu_timing_capability: RenderGpuTimingCapability,
        batch: &mut RendererRealizationBatch<'_>,
    ) -> Result<(
        Vec<PendingGpuPassTimingReadback>,
        Vec<PendingCaptureReadback>,
    )> {
        let frame_index = prepared_frame.context.frame_index;
        let mut pending_capture_readbacks = Vec::new();
        let mut pending_gpu_pass_timing_readbacks = Vec::new();
        let mut flow_runtime_cache = std::mem::take(&mut self.flow_runtime_cache);
        let execution_result = (|| -> Result<()> {
            for invocation in &mut batch.invocations {
                let runtime_resources = flow_runtime_cache
                    .get_mut(&invocation.flow.flow_id)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "flow '{}' lost its realized runtime resources before raw execution",
                            invocation.flow.flow_id
                        )
                    })?;
                runtime_resources.target_alias_bindings =
                    invocation.invocation.target_alias_bindings.clone();
                runtime_resources.set_active_invocation_uniform_scope(
                    invocation.invocation.invocation_id.0.clone(),
                );
                tracing::trace!(
                    flow_id = %invocation.flow.flow_id,
                    invocation_id = %invocation.invocation.invocation_id.0,
                    canonical_work_prepared = invocation.canonical_work.is_some(),
                    "renderer G5A invocation execution authority"
                );

                if invocation.canonical_work.is_some() {
                    let schedule = schedule_invocation_passes(invocation)?;
                    let timestamp_period_available = invocation
                        .timing_frame
                        .as_mut()
                        .map(|frame| frame.activate(queue))
                        .unwrap_or(false);

                    for scheduled in schedule {
                        tracing::trace!(
                            operation_kind = ?scheduled.operation.kind(),
                            occurrence = scheduled.payload.occurrence().raw(),
                            "encode canonical renderer GPU work in prepared G3 order"
                        );
                        match scheduled.payload {
                            RenderGpuWorkPayload::Upload { occurrence } => {
                                let GpuWorkOperation::Upload(operation) = &scheduled.operation
                                else {
                                    bail!(
                                        "prepared G3 upload occurrence '{}' carries non-upload operation kind {:?}",
                                        occurrence,
                                        scheduled.operation.kind()
                                    );
                                };
                                let projected = invocation
                                    .projected_uploads
                                    .iter()
                                    .find(|upload| upload.occurrence == occurrence)
                                    .or_else(|| {
                                        invocation.scheduled_passes.iter().find_map(|pass| {
                                            pass.fixed_step_upload
                                                .as_ref()
                                                .filter(|upload| upload.occurrence == occurrence)
                                        })
                                    })
                                    .ok_or_else(|| {
                                        anyhow::anyhow!(
                                            "prepared G3 upload occurrence '{}' has no realized physical payload",
                                            occurrence
                                        )
                                    })?;
                                self.encode_canonical_upload_operation(
                                    context,
                                    queue,
                                    operation,
                                    &projected.realized,
                                )?;
                            }
                            RenderGpuWorkPayload::Pass { occurrence, .. } => {
                                let scheduled_pass = invocation
                                    .scheduled_passes
                                    .iter_mut()
                                    .find(|pass| pass.occurrence == occurrence)
                                    .ok_or_else(|| {
                                        anyhow::anyhow!(
                                            "prepared G3 pass occurrence '{}' has no realized renderer payload",
                                            occurrence
                                        )
                                    })?;
                                let execution = &mut scheduled_pass.execution;
                                let pass = execution.pass;
                                let pass_encode_start = Instant::now();
                                let pass_label = execution_pass_id(pass).to_string();
                                let pass_kind = execution_pass_kind_name(pass).to_string();
                                let gpu_timestamp_indices =
                                    execution.timestamp_indices.and_then(|indices| {
                                        invocation.timing_frame.as_mut().and_then(|frame| {
                                            frame.register_pass(
                                                indices,
                                                frame_index,
                                                prepared_frame.surface.render_surface_id.raw(),
                                                invocation.flow.flow_id.to_string(),
                                                pass_label.clone(),
                                                pass_kind.clone(),
                                            )
                                        })
                                    });
                                let gpu_timestamp_writes =
                                    gpu_timestamp_indices.and_then(|indices| {
                                        invocation
                                            .timing_frame
                                            .as_ref()
                                            .map(|frame| frame.timestamp_writes(indices))
                                    });
                                let has_gpu_timestamp_writes = gpu_timestamp_writes.is_some();
                                let evidence = match &scheduled.operation {
                                    GpuWorkOperation::Compute(operation) => {
                                        if !matches!(pass, CompiledPassExecutionPlan::Compute(_)) {
                                            bail!(
                                                "canonical compute operation occurrence '{}' is paired with non-compute renderer identity '{}'",
                                                occurrence,
                                                pass_label
                                            );
                                        }
                                        let prepared = execution.pipeline.as_ref().ok_or_else(|| {
                                            anyhow::anyhow!(
                                                "canonical compute occurrence '{}' has no G4C3 realized pipeline",
                                                occurrence
                                            )
                                        })?;
                                        self.encode_canonical_compute_operation(
                                            context,
                                            encoder,
                                            operation,
                                            prepared,
                                            gpu_timestamp_writes,
                                        )?
                                    }
                                    GpuWorkOperation::Render(_) | GpuWorkOperation::Copy(_) => self
                                        .encode_compiled_pass(
                                            context,
                                            encoder,
                                            frame_texture,
                                            frame_view,
                                            &invocation.packet,
                                            invocation.flow,
                                            &invocation.invocation.inputs,
                                            pass,
                                            runtime_resources,
                                            execution.pipeline.as_ref(),
                                            gpu_timestamp_writes,
                                        )?,
                                    other => {
                                        bail!(
                                            "canonical render-pass occurrence '{}' carries unsupported operation kind {:?}",
                                            occurrence,
                                            other.kind()
                                        )
                                    }
                                };
                                self.record_encoded_pass(
                                    frame_index,
                                    prepared_frame.surface.render_surface_id.raw(),
                                    invocation.flow,
                                    &invocation.packet,
                                    pass,
                                    runtime_resources,
                                    debug_control,
                                    if timestamp_period_available {
                                        gpu_timing_capability
                                    } else {
                                        RenderGpuTimingCapability::UnavailableThisFrame
                                    },
                                    pass_label,
                                    pass_kind,
                                    pass_encode_start,
                                    has_gpu_timestamp_writes,
                                    &evidence,
                                );
                            }
                            RenderGpuWorkPayload::TimingResolve { occurrence } => {
                                let GpuWorkOperation::Resolve(operation) = &scheduled.operation
                                else {
                                    bail!(
                                        "prepared timing-resolve occurrence '{}' carries non-resolve operation kind {:?}",
                                        occurrence,
                                        scheduled.operation.kind()
                                    );
                                };
                                let frame = invocation.timing_frame.as_mut().ok_or_else(|| {
                                    anyhow::anyhow!(
                                        "prepared timing-resolve occurrence '{}' has no physical timing frame",
                                        occurrence
                                    )
                                })?;
                                if !frame.encode_resolve(context, encoder, operation)? {
                                    bail!(
                                        "prepared timing-resolve occurrence '{}' had no registered timestamp queries",
                                        occurrence
                                    );
                                }
                            }
                            RenderGpuWorkPayload::TimingReadbackCopy { occurrence } => {
                                let GpuWorkOperation::Copy(operation) = &scheduled.operation else {
                                    bail!(
                                        "prepared timing-readback occurrence '{}' carries non-copy operation kind {:?}",
                                        occurrence,
                                        scheduled.operation.kind()
                                    );
                                };
                                let frame = invocation.timing_frame.take().ok_or_else(|| {
                                    anyhow::anyhow!(
                                        "prepared timing-readback occurrence '{}' has no physical timing frame",
                                        occurrence
                                    )
                                })?;
                                if let Some(pending) =
                                    frame.encode_readback_copy(context, encoder, operation)?
                                {
                                    pending_gpu_pass_timing_readbacks.push(pending);
                                }
                            }
                        }
                    }
                    runtime_resources.clear_active_invocation_uniform_scope();
                    continue;
                }

                let invocation_result = (|| -> Result<()> {
                    for upload in &invocation.projected_uploads {
                        self.encode_canonical_upload_operation(
                            context,
                            queue,
                            &upload.operation,
                            &upload.realized,
                        )?;
                    }
                    let timestamp_active = invocation
                        .timing_frame
                        .as_mut()
                        .map(|frame| frame.activate(queue))
                        .unwrap_or(false);
                    for scheduled_pass in &mut invocation.scheduled_passes {
                        if let Some(upload) = scheduled_pass.fixed_step_upload.as_ref() {
                            self.encode_canonical_upload_operation(
                                context,
                                queue,
                                &upload.operation,
                                &upload.realized,
                            )?;
                        }
                        let execution = &mut scheduled_pass.execution;
                        self.encode_prepared_capture_copies(
                            context,
                            encoder,
                            frame_texture,
                            &mut batch.capture_runtime,
                            std::mem::take(&mut execution.before_captures),
                            &mut pending_capture_readbacks,
                        )?;
                        let pass = execution.pass;
                        let pass_encode_start = Instant::now();
                        let pass_label = execution_pass_id(pass).to_string();
                        let pass_kind = execution_pass_kind_name(pass).to_string();
                        let gpu_timestamp_indices = if timestamp_active {
                            execution.timestamp_indices.and_then(|indices| {
                                invocation.timing_frame.as_mut().and_then(|frame| {
                                    frame.register_pass(
                                        indices,
                                        frame_index,
                                        prepared_frame.surface.render_surface_id.raw(),
                                        invocation.flow.flow_id.to_string(),
                                        pass_label.clone(),
                                        pass_kind.clone(),
                                    )
                                })
                            })
                        } else {
                            None
                        };
                        let gpu_timestamp_writes = gpu_timestamp_indices.and_then(|indices| {
                            invocation
                                .timing_frame
                                .as_ref()
                                .map(|frame| frame.timestamp_writes(indices))
                        });
                        let has_gpu_timestamp_writes = gpu_timestamp_writes.is_some();
                        let evidence = self.encode_compiled_pass(
                            context,
                            encoder,
                            frame_texture,
                            frame_view,
                            &invocation.packet,
                            invocation.flow,
                            &invocation.invocation.inputs,
                            pass,
                            runtime_resources,
                            execution.pipeline.as_ref(),
                            gpu_timestamp_writes,
                        )?;
                        self.encode_prepared_capture_copies(
                            context,
                            encoder,
                            frame_texture,
                            &mut batch.capture_runtime,
                            std::mem::take(&mut execution.after_captures),
                            &mut pending_capture_readbacks,
                        )?;
                        self.record_encoded_pass(
                            frame_index,
                            prepared_frame.surface.render_surface_id.raw(),
                            invocation.flow,
                            &invocation.packet,
                            pass,
                            runtime_resources,
                            debug_control,
                            if timestamp_active {
                                gpu_timing_capability
                            } else {
                                RenderGpuTimingCapability::UnavailableThisFrame
                            },
                            pass_label,
                            pass_kind,
                            pass_encode_start,
                            has_gpu_timestamp_writes,
                            &evidence,
                        );
                    }
                    if let Some(frame) = invocation.timing_frame.take()
                        && timestamp_active
                        && let Some(pending) = encode_prepared_timing_tail(frame, context, encoder)?
                    {
                        pending_gpu_pass_timing_readbacks.push(pending);
                    }
                    Ok(())
                })();
                runtime_resources.clear_active_invocation_uniform_scope();
                invocation_result?;
            }
            self.encode_prepared_capture_copies(
                context,
                encoder,
                frame_texture,
                &mut batch.capture_runtime,
                std::mem::take(&mut batch.final_captures),
                &mut pending_capture_readbacks,
            )?;
            Ok(())
        })();
        self.flow_runtime_cache = flow_runtime_cache;
        execution_result?;
        Ok((pending_gpu_pass_timing_readbacks, pending_capture_readbacks))
    }

    #[allow(clippy::too_many_arguments)]
    fn record_encoded_pass(
        &mut self,
        frame_index: u64,
        render_surface_id: u64,
        flow: &CompiledRenderFlowPlan,
        packet: &RendererPreparedPacket,
        pass: &CompiledPassExecutionPlan,
        runtime_resources: &FlowRuntimeResources,
        debug_control: &RenderDebugControlResource,
        timing_capability: RenderGpuTimingCapability,
        pass_label: String,
        pass_kind: String,
        pass_encode_start: Instant,
        has_gpu_timestamp_writes: bool,
        evidence: &EncodedPassEvidence,
    ) {
        self.last_pass_timings.push(PassTimingSample {
            flow_id: flow.flow_id.to_string(),
            pass_id: pass_label.clone(),
            pass_kind: pass_kind.clone(),
            millis: pass_encode_start.elapsed().as_secs_f32() * 1000.0,
            dispatch_workgroups: evidence.dispatch_workgroups,
        });
        if !has_gpu_timestamp_writes {
            self.last_gpu_pass_timing_evidence
                .push(gpu_timing_diagnostic_evidence_for_pass(
                    timing_capability,
                    frame_index,
                    render_surface_id,
                    flow.flow_id.to_string(),
                    pass_label.clone(),
                    pass_kind.clone(),
                ));
        }
        if !debug_control.provenance_enabled {
            return;
        }

        let pass_resource_truth =
            collect_pass_resource_truth(flow.flow_id, pass, runtime_resources);
        let material_binding = collect_pass_material_binding_evidence(packet, pass);
        self.last_pass_provenance.push(RenderPassProvenanceRecord {
            frame_index,
            flow_id: flow.flow_id.to_string(),
            pass_id: pass_label.clone(),
            pass_label,
            pass_kind: execution_flow_pass_kind(pass),
            authoring_index: execution_pass_authoring_index(pass),
            feature_id: execution_pass_feature_id(pass).map(|id| id.to_string()),
            shader_id: evidence.shader_id.clone(),
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
                .map(FlowPassPipelineKey::primary_bind_group_layout_diagnostic_hash)
                .unwrap_or_default(),
            material_specialization_fragment_hash: material_specialization_fragment_hash(
                packet,
                execution_pass_feature_id(pass),
            ),
            view_signature_hash: hash_view_signature(packet.view_id.as_str(), packet.surface_size),
            feature_runtime_version: feature_runtime_version(
                packet,
                execution_pass_feature_id(pass),
            ),
            color_formats: evidence
                .pipeline_key
                .as_ref()
                .and_then(FlowPassPipelineKey::render_pipeline_state)
                .and_then(|state| state.fragment_output())
                .map(|output| {
                    output
                        .color_targets()
                        .map(|target| target.format())
                        .collect()
                })
                .unwrap_or_default(),
            depth_format: evidence
                .pipeline_key
                .as_ref()
                .and_then(FlowPassPipelineKey::render_pipeline_state)
                .and_then(|state| state.depth_stencil())
                .map(|depth| depth.format()),
            sample_count: evidence
                .pipeline_key
                .as_ref()
                .and_then(FlowPassPipelineKey::render_pipeline_state)
                .map(|state| state.multisample().sample_count())
                .unwrap_or(1),
            primitive_topology: evidence
                .pipeline_key
                .as_ref()
                .and_then(FlowPassPipelineKey::render_pipeline_state)
                .map(|state| state.primitive().topology()),
            material_binding,
            render_targets: pass_resource_truth.render_targets,
            sampled_textures: pass_resource_truth.sampled_textures,
            storage_textures: pass_resource_truth.storage_textures,
            depth_targets: pass_resource_truth.depth_targets,
            capture_points_available: pass_resource_truth.capture_points_available,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        context: &GpuContext,
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
            context,
            prepared_frame,
            shader_registry,
            ui_rect_shader,
            ui_font_atlas,
            viewport_surface_bindings,
            surface_format,
        )?;
        self.render_packet(
            context,
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

    fn realize_projected_uniform_uploads(
        &self,
        context: &GpuContext,
        invocation_id: &str,
        flow_inputs: &PreparedFlowInputs,
        runtime_resources: &mut FlowRuntimeResources,
        maximum_occurrence: &mut u64,
    ) -> Result<Vec<RealizedLogicalBufferUpload>> {
        let mut uploads = Vec::new();
        for (buffer_id, bytes) in &flow_inputs.projected_uniform_bytes {
            let prepared = runtime_resources.prepare_uniform_upload(*buffer_id, bytes)?;
            let runtime_buffer = runtime_resources.realize_invocation_uniform_buffer(
                context,
                invocation_id,
                *buffer_id,
                prepared.layout().byte_len(),
            )?;
            if prepared.layout().byte_len() > runtime_buffer.size {
                bail!(
                    "uniform upload for '{}' in invocation '{}' writes {} bytes but runtime buffer size is {}",
                    buffer_id,
                    invocation_id,
                    prepared.layout().byte_len(),
                    runtime_buffer.size
                );
            }
            let operation = project_buffer_upload(&runtime_buffer.handle, prepared.as_bytes())?;
            uploads.push(RealizedLogicalBufferUpload {
                occurrence: allocate_aux_occurrence(maximum_occurrence)?,
                operation,
                realized: runtime_buffer.realized.clone(),
                control_order_after: Vec::new(),
            });
        }

        Ok(uploads)
    }

    #[allow(clippy::too_many_arguments)]
    fn realize_fixed_step_iteration_upload(
        &self,
        context: &GpuContext,
        invocation_id: &str,
        runtime_resources: &mut FlowRuntimeResources,
        region: &CompiledFixedStepRegion,
        uniform: RenderFixedStepIterationUniform,
        maximum_occurrence: &mut u64,
        control_order_after: Vec<RenderGpuWorkOccurrenceId>,
    ) -> Result<RealizedLogicalBufferUpload> {
        let bytes = uniform.to_uniform_bytes();
        let prepared =
            runtime_resources.prepare_uniform_upload(region.iteration_uniform, &bytes)?;
        let runtime_buffer = runtime_resources.realize_invocation_uniform_buffer(
            context,
            invocation_id,
            region.iteration_uniform,
            prepared.layout().byte_len(),
        )?;
        if prepared.layout().byte_len() > runtime_buffer.size {
            bail!(
                "fixed-step iteration uniform upload for region '{}' in invocation '{}' writes {} bytes but runtime buffer size is {}",
                region.region_label,
                invocation_id,
                prepared.layout().byte_len(),
                runtime_buffer.size
            );
        }
        let operation = project_buffer_upload(&runtime_buffer.handle, prepared.as_bytes())?;
        Ok(RealizedLogicalBufferUpload {
            occurrence: allocate_aux_occurrence(maximum_occurrence)?,
            operation,
            realized: runtime_buffer.realized.clone(),
            control_order_after,
        })
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
    fn prepare_pass_texture_captures(
        &mut self,
        context: &GpuContext,
        frame_texture: &Texture,
        packet: &RendererPreparedPacket,
        flow: &CompiledRenderFlowPlan,
        pass: &CompiledPassExecutionPlan,
        runtime_resources: &FlowRuntimeResources,
        stage: CaptureStage,
        capture_runtime: &mut FrameCaptureRuntime,
        prepared_captures: &mut Vec<PreparedCaptureReadback>,
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

            match prepare_texture_capture_copy(
                context,
                selector_index,
                identity,
                match resolved_texture.texture {
                    RuntimeTextureRef::Surface(_) => CaptureTextureSource::Surface,
                    RuntimeTextureRef::Realized(texture) => CaptureTextureSource::Realized(texture),
                },
                resolved_texture.size,
                resolved_texture.format,
                readback_format,
            ) {
                Ok(prepared) => prepared_captures.push(prepared),
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

    fn prepare_final_surface_capture(
        &mut self,
        context: &GpuContext,
        _frame_texture: &Texture,
        packet: &RendererPreparedPacket,
        capture_runtime: &mut FrameCaptureRuntime,
        prepared_captures: &mut Vec<PreparedCaptureReadback>,
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

            match prepare_texture_capture_copy(
                context,
                selector_index,
                identity,
                CaptureTextureSource::Surface,
                packet.surface_size,
                packet.surface_format,
                readback_format,
            ) {
                Ok(prepared) => prepared_captures.push(prepared),
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

    fn encode_prepared_capture_copies(
        &mut self,
        context: &GpuContext,
        encoder: &mut CommandEncoder,
        frame_texture: &Texture,
        capture_runtime: &mut FrameCaptureRuntime,
        prepared_captures: Vec<PreparedCaptureReadback>,
        pending_capture_readbacks: &mut Vec<PendingCaptureReadback>,
    ) -> Result<()> {
        for prepared in prepared_captures {
            let selector_index = prepared.selector_index;
            let identity = prepared.identity.clone();
            let width = prepared.width;
            let height = prepared.height;
            let format = format!("{:?}", prepared.source_format);
            match encode_prepared_texture_capture_copy(
                context,
                encoder,
                Some(frame_texture),
                prepared,
            ) {
                Ok(pending) => pending_capture_readbacks.push(pending),
                Err(error) => {
                    let terminal = RenderCaptureTerminal::with_reason(
                        RenderCaptureTerminalCode::ReadbackFailed,
                        "enqueue_capture_copy_failed",
                        error.to_string(),
                    );
                    capture_runtime.set_terminal(selector_index, terminal.clone());
                    self.last_captured_textures.push(RenderCapturedTexture {
                        identity,
                        width,
                        height,
                        format,
                        bytes_rgba8: None,
                        terminal,
                    });
                }
            }
        }
        Ok(())
    }
}

fn schedule_invocation_passes(
    invocation: &RealizedFlowInvocation<'_>,
) -> Result<Vec<ScheduledInvocationWork>> {
    let work = invocation.canonical_work.as_ref().ok_or_else(|| {
        anyhow::anyhow!("canonical invocation scheduling requires a prepared G3 work plan")
    })?;
    work.ordered_payloads()?
        .into_iter()
        .map(|(node_id, payload)| {
            let operation = work
                .graph()
                .nodes()
                .iter()
                .find(|node| node.id() == node_id)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "prepared G3 node '{:?}' is missing from its own canonical work graph",
                        node_id
                    )
                })?
                .node()
                .operation()
                .clone();
            Ok(ScheduledInvocationWork {
                operation,
                payload: payload.clone(),
            })
        })
        .collect()
}

fn encode_prepared_timing_tail(
    mut frame: GpuPassTimingFrame,
    context: &GpuContext,
    encoder: &mut CommandEncoder,
) -> Result<Option<PendingGpuPassTimingReadback>> {
    let resolve_operation = frame.resolve_operation().clone();
    let readback_copy_operation = frame.readback_copy_operation().clone();
    if !frame.encode_resolve(context, encoder, &resolve_operation)? {
        return Ok(None);
    }
    frame.encode_readback_copy(context, encoder, &readback_copy_operation)
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
