use super::super::dynamic_targets::RendererPreparedDynamicTextureUploadBatch;
use super::super::retained_sessions::PreparedRetainedOccurrence;
use super::*;
use super::{
    canonical_work::{
        CanonicalFrameResolution, CanonicalInvocationProjection, CanonicalInvocationResolution,
        CanonicalPassProjection, RealizedLogicalBufferUpload, allocate_aux_occurrence,
        resolve_canonical_frame, resolve_canonical_invocation,
    },
    logical_operations::project_buffer_upload,
    logical_timing::LogicalGpuPassTimingPlan,
    occurrences::expand_render_pass_occurrences_in_frame,
};
use crate::plugins::render::{
    RenderGpuWorkOccurrenceId, RenderPassId, ResolvedRenderGpuWorkNode,
    RunenUiMixedWork, prepare_render_gpu_frame_work,
    prepare_render_gpu_frame_work_with_mixed_ui,
};
use runen_render::execution_2d::{Render2dPreparedContribution, Render2dTarget};
use runen_gpu::{
    GpuPresentOperation, GpuRenderDraw, GpuResourceLabel, GpuResourceProvenance, GpuTextureHandle,
    GpuTextureViewHandle, GpuWorkFragment, GpuWorkImport,
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
    dynamic_texture_uploads: RendererPreparedDynamicTextureUploadBatch,
    capture_runtime: FrameCaptureRuntime,
    invocations: Vec<RealizedFlowInvocation<'a>>,
    final_captures: Vec<PreparedCaptureReadback>,
    maximum_occurrence: u64,
    /// Exact GPU target-admitted F2 preparations; never presentation evidence.
    prepared_runenui: Vec<(RunenUiPublicationId, u32, Render2dPreparedContribution)>,
    /// Source-ordered legacy GPU draws, before any mixed compositor flattening.
    ordered_legacy_draws: Vec<(u32, GpuRenderDraw)>,
}

/// Exact retained occurrences and their public composition projection cross submission together.
struct PreparedDeterministicCompositions {
    fragments: Vec<GpuWorkFragment>,
    imports: Vec<GpuWorkImport>,
    occurrences: Vec<PreparedRetainedOccurrence>,
}

struct RealizedFlowInvocation<'a> {
    flow: &'a CompiledRenderFlowPlan,
    invocation: &'a crate::plugins::render::PreparedFlowInvocation,
    packet: RendererPreparedPacket,
    scheduled_passes: Vec<RealizedScheduledPass<'a>>,
    timing_frame: Option<GpuPassTimingFrame>,
    /// Owned semantic authority retained through realization and consumed by the frame graph.
    canonical_resolution: Option<CanonicalInvocationResolution>,
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

impl Renderer {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_packet(
        &mut self,
        context: &GpuContext,
        surface_texture: &GpuTextureHandle,
        surface_view: &GpuTextureViewHandle,
        acquired_surface_extent: (u32, u32),
        prepared_frame: &PreparedRenderFrame,
        deterministic_contributions: &[crate::plugins::render::RenderDeterministicFrameContribution],
        target_requests: &crate::plugins::render::RenderDynamicTextureTargetRequestRegistryResource,
        packet: RendererPreparedPacket,
        compiled_flows: &[CompiledRenderFlowPlan],
        shader_registry: &ShaderRegistryResource,
        preflight_config: crate::plugins::render::graph::RenderPreflightValidationConfigResource,
        debug_control: &RenderDebugControlResource,
        debug_config: &RenderDebugConfigResource,
        gpu_timing_capability: RenderGpuTimingCapability,
        composed_gpu_timing_capability: Option<RenderGpuTimingCapability>,
    ) -> Result<RendererFrameTimings> {
        let mut timings = packet.prepare_timings;
        self.last_pass_timings.clear();
        self.last_runtime_resources.clear();
        self.last_pass_provenance.clear();

        let preflight_start = Instant::now();
        self.last_preflight_report =
            self.preflight_prepared_frame(prepared_frame, compiled_flows, preflight_config)?;
        timings.preflight_ms = preflight_start.elapsed().as_secs_f32() * 1000.0;

        let flow_encode_start = Instant::now();
        // Phase one: all G4C1/G4C2/G4C3 realization plus renderer-owned logical G5 operation
        // formation completes without a raw device/queue loan.
        let mut batch = self.realize_render_batch(
            context,
            surface_texture,
            surface_view,
            acquired_surface_extent,
            prepared_frame,
            packet,
            compiled_flows,
            shader_registry,
            debug_control,
            debug_config,
            gpu_timing_capability,
        )?;

        let canonical_resolutions = batch
            .invocations
            .iter_mut()
            .map(|invocation| {
                let resolution = invocation.canonical_resolution.take().ok_or_else(|| {
                    anyhow::anyhow!(
                        "flow '{}' invocation '{}' lost canonical resolution before frame preparation",
                        invocation.flow.flow_id,
                        invocation.invocation.invocation_id.0
                    )
                })?;
                Ok(resolution)
            })
            .collect::<Result<Vec<_>>>()?;
        let CanonicalFrameResolution::Resolved(mut frame) =
            resolve_canonical_frame(canonical_resolutions)
        else {
            bail!("normal renderer frame retained a residual non-canonical GPU operation");
        };

        let mut nodes = Vec::new();
        for operation in std::mem::take(&mut batch.packet.pending_operations).into_operations() {
            let occurrence = allocate_aux_occurrence(&mut batch.maximum_occurrence)?;
            nodes.push(ResolvedRenderGpuWorkNode::upload(
                occurrence,
                GpuResourceLabel::new(format!("render.frame.pending-upload.{}", occurrence.raw()))?,
                operation,
                [],
            ));
        }
        let (accepted_dynamic_uploads, dynamic_upload_report) = self
            .dynamic_texture_targets
            .validate_prepared_uploads(std::mem::take(&mut batch.dynamic_texture_uploads));
        for diagnostic in &dynamic_upload_report.diagnostics {
            tracing::warn!(
                target = "renderer.dynamic_texture_upload",
                target_key = %diagnostic.target_key,
                message = %diagnostic.message,
                "dynamic texture upload rejected before frame submission"
            );
        }
        for upload in &accepted_dynamic_uploads {
            let occurrence = allocate_aux_occurrence(&mut batch.maximum_occurrence)?;
            nodes.push(ResolvedRenderGpuWorkNode::upload(
                occurrence,
                GpuResourceLabel::new(format!("render.frame.dynamic-upload.{}", occurrence.raw()))?,
                upload.operation().clone(),
                [],
            ));
        }
        nodes.append(&mut frame.nodes);

        let deterministic = self.prepare_deterministic_compositions(
            context,
            prepared_frame,
            deterministic_contributions,
            target_requests,
        )?;

        let mut terminal_controls =
            resolve_terminal_present_controls(frame.terminal_present_controls)?;
        if !batch.final_captures.is_empty() {
            let mut final_capture_occurrences = Vec::with_capacity(batch.final_captures.len());
            for capture in &batch.final_captures {
                let occurrence = allocate_aux_occurrence(&mut batch.maximum_occurrence)?;
                nodes.push(ResolvedRenderGpuWorkNode::capture_readback(
                    occurrence,
                    GpuResourceLabel::new(format!(
                        "render.frame.final-capture.{}",
                        occurrence.raw()
                    ))?,
                    capture.canonical_operation().clone(),
                    terminal_controls.iter().copied(),
                ));
                final_capture_occurrences.push(occurrence);
            }
            terminal_controls = final_capture_occurrences;
        }
        let present_occurrence = allocate_aux_occurrence(&mut batch.maximum_occurrence)?;
        let present = GpuPresentOperation::new(
            surface_view.clone().into(),
            surface_view.descriptor().subresources(),
        )?;
        nodes.push(ResolvedRenderGpuWorkNode::present(
            present_occurrence,
            GpuResourceLabel::new(format!("render.frame.present.{}", present_occurrence.raw()))?,
            present,
            terminal_controls,
        ));

        let has_capture_observation_tails = !batch.final_captures.is_empty()
            || batch.invocations.iter().any(|invocation| {
                invocation.scheduled_passes.iter().any(|pass| {
                    !pass.execution.before_captures.is_empty()
                        || !pass.execution.after_captures.is_empty()
                })
            });
        let composed_gpu_timing = match composed_gpu_timing_capability {
            Some(RenderGpuTimingCapability::Supported) if !has_capture_observation_tails => {
                Some(prepare_composed_gpu_timing(
                    context,
                    prepared_frame.context.frame_index,
                    prepared_frame.surface.render_surface_id.raw(),
                )?)
            }
            _ => None,
        };
        let composed_terminal_evidence = match composed_gpu_timing_capability {
            Some(RenderGpuTimingCapability::Unsupported) => {
                Some(RenderComposedFrameGpuTimingEvidence::gpu_diagnostic(
                    prepared_frame.context.frame_index,
                    prepared_frame.surface.render_surface_id.raw(),
                    RenderGpuTimingDiagnostic::unsupported(
                        "GPU timestamp queries are unavailable for composed renderer timing",
                    ),
                ))
            }
            Some(RenderGpuTimingCapability::Supported) if has_capture_observation_tails => {
                Some(RenderComposedFrameGpuTimingEvidence::gpu_diagnostic(
                    prepared_frame.context.frame_index,
                    prepared_frame.surface.render_surface_id.raw(),
                    RenderGpuTimingDiagnostic::unavailable_this_frame(
                        "composed renderer timing is unavailable while capture/readback instrumentation is active",
                    ),
                ))
            }
            _ => None,
        };

        let encode_submit_start = Instant::now();
        let _span = tracing::info_span!("renderer.prepare_submit").entered();
        let graph_label = GpuResourceLabel::new(format!(
            "render.frame.{}.surface.{}",
            prepared_frame.context.frame_index,
            prepared_frame.surface.render_surface_id.raw()
        ))?;
        let timing_bracket = composed_gpu_timing.as_ref()
            .map(PreparedComposedGpuTiming::bracket);
        let (graph, runenui_witnesses) = if batch.prepared_runenui.is_empty() {
            (
                prepare_render_gpu_frame_work(
                    context,
                    graph_label,
                    nodes,
                    &deterministic.fragments,
                    &deterministic.imports,
                    timing_bracket,
                )?,
                Vec::new(),
            )
        } else {
            let mut ui_occurrences = nodes.iter().filter_map(
                ResolvedRenderGpuWorkNode::builtin_ui_occurrence
            );
            let ui_occurrence = ui_occurrences.next().ok_or_else(|| anyhow::anyhow!(
                "RunenUI paint has no admitted canonical UI occurrence"
            ))?;
            if ui_occurrences.next().is_some() {
                bail!("RunenUI paint targets more than one canonical UI occurrence");
            }
            let mixed = RunenUiMixedWork {
                ui_occurrence,
                legacy_draws: std::mem::take(&mut batch.ordered_legacy_draws),
                contributions: std::mem::take(&mut batch.prepared_runenui),
            };
            let authored = prepare_render_gpu_frame_work_with_mixed_ui(
                context,
                graph_label,
                nodes,
                &deterministic.fragments,
                &deterministic.imports,
                timing_bracket,
                mixed,
                present_occurrence,
            )?;
            let witnesses = authored.f2_tokens.into_iter().map(|(publication_id, token)| {
                RunenUiGpuPresentationWitness {
                    publication_id,
                    contribution: token,
                    present_node: authored.present_node.clone(),
                }
            }).collect::<Vec<_>>();
            (authored.graph, witnesses)
        };
        let prepared = pollster::block_on(context.prepare_submission(graph))?;
        let submission = context.submit_prepared(prepared).map_err(|rejection| {
            anyhow::anyhow!(
                "GPU frame submission rejected ({:?}): {}",
                rejection.reason().kind(),
                rejection.reason().detail()
            )
        })?;
        let association_result = self
            .render_sessions
            .associate_composed_submission(deterministic.occurrences, &submission);
        // Once G5 accepts the submission, retain every renderer-observed readback before any
        // fallible product evidence work. An accepted lifecycle handle must never be dropped merely
        // because later provenance publication fails for this frame.
        let timing_frames = batch
            .invocations
            .iter_mut()
            .filter_map(|invocation| invocation.timing_frame.take())
            .collect::<Vec<_>>();
        let mut capture_readbacks = batch
            .invocations
            .iter_mut()
            .flat_map(|invocation| {
                invocation.scheduled_passes.iter_mut().flat_map(|pass| {
                    let execution = &mut pass.execution;
                    execution
                        .before_captures
                        .drain(..)
                        .chain(execution.after_captures.drain(..))
                })
            })
            .collect::<Vec<_>>();
        capture_readbacks.append(&mut batch.final_captures);
        let mut observation_output = self.gpu_observations.accept(
            context,
            submission,
            timing_frames,
            composed_gpu_timing.map(PreparedComposedGpuTiming::into_frame),
            capture_readbacks,
            runenui_witnesses,
            &mut batch.capture_runtime,
        );
        if let Some(evidence) = composed_terminal_evidence {
            observation_output.composed_timing_evidence.push(evidence);
        }
        let RendererGpuObservationOutput {
            timing_evidence,
            composed_timing_evidence,
            captured_textures,
            capture_results,
            runenui_accepted,
            runenui_presented,
            runenui_rejected,
        } = observation_output;
        self.pending_gpu_observation_output
            .timing_evidence
            .extend(timing_evidence);
        self.pending_gpu_observation_output
            .composed_timing_evidence
            .extend(composed_timing_evidence);
        self.pending_gpu_observation_output
            .captured_textures
            .extend(captured_textures);
        self.pending_gpu_observation_output
            .capture_results
            .extend(capture_results);
        self.pending_gpu_observation_output
            .runenui_accepted
            .extend(runenui_accepted);
        self.pending_gpu_observation_output
            .runenui_presented
            .extend(runenui_presented);
        self.pending_gpu_observation_output
            .runenui_rejected
            .extend(runenui_rejected);

        // Association failure cannot discard accepted timing/capture lifecycle handles. The
        // affected continuity is quarantined by the session owner and this frame fails closed.
        association_result?;

        let accepted_upload_report = self
            .dynamic_texture_targets
            .record_accepted_uploads(&accepted_dynamic_uploads);
        for diagnostic in &accepted_upload_report.diagnostics {
            tracing::warn!(
                target = "renderer.dynamic_texture_upload",
                target_key = %diagnostic.target_key,
                message = %diagnostic.message,
                "accepted dynamic texture upload bookkeeping rejected"
            );
        }

        if debug_control.provenance_enabled {
            let mut runtime_cache = std::mem::take(&mut self.flow_runtime_cache);
            for invocation in &batch.invocations {
                let runtime_resources = runtime_cache
                    .get_mut(&invocation.flow.flow_id)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "flow '{}' lost runtime resources before accepted provenance publication",
                            invocation.flow.flow_id
                        )
                    })?;
                runtime_resources.target_alias_bindings =
                    invocation.invocation.target_alias_bindings.clone();
                runtime_resources.set_active_invocation_uniform_scope(
                    invocation.invocation.invocation_id.0.clone(),
                );
                for scheduled in &invocation.scheduled_passes {
                    let pass = scheduled.execution.pass;
                    let pipeline = scheduled.execution.pipeline.as_ref();
                    let evidence = EncodedPassEvidence {
                        shader_id: pipeline
                            .map(|prepared| prepared.shader_id.clone())
                            .unwrap_or_else(|| {
                                format!("builtin:{}", execution_pass_kind_name(pass))
                            }),
                        shader_revision: pipeline
                            .map(|prepared| prepared.shader_revision)
                            .unwrap_or(0),
                        fallback_used: pipeline
                            .map(|prepared| prepared.fallback_used)
                            .unwrap_or(false),
                        pipeline_key: pipeline
                            .map(|prepared| prepared.bindings.pipeline_key.clone()),
                    };
                    self.last_pass_provenance.push(accepted_pass_provenance(
                        prepared_frame.context.frame_index,
                        invocation.flow,
                        &invocation.packet,
                        pass,
                        runtime_resources,
                        &evidence,
                    ));
                }
                runtime_resources.clear_active_invocation_uniform_scope();
            }
            self.flow_runtime_cache = runtime_cache;
        }

        timings.flow_encode_ms = flow_encode_start.elapsed().as_secs_f32() * 1000.0;
        timings.encode_submit_ms = encode_submit_start.elapsed().as_secs_f32() * 1000.0;
        batch.capture_runtime.finalize_unresolved();
        let (capture_plan, capture_selector_results) =
            batch.capture_runtime.into_plan_and_results();
        self.last_capture_plan = capture_plan;
        self.last_capture_selector_results
            .extend(capture_selector_results);
        Ok(timings)
    }

    #[allow(clippy::too_many_arguments)]
    fn realize_render_batch<'a>(
        &mut self,
        context: &GpuContext,
        surface_texture: &GpuTextureHandle,
        surface_view: &GpuTextureViewHandle,
        acquired_surface_extent: (u32, u32),
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
        let dynamic_texture_uploads = self
            .dynamic_texture_targets
            .prepare_uploads(&prepared_frame.dynamic_texture_uploads);
        let (viewport, product_surface) = self.realize_ui_dynamic_bind_groups(
            &packet.prepared_ui,
            &packet.viewport_surface_bindings,
        )?;
        packet.ui_dynamic_bind_groups = UiDynamicBindGroups {
            viewport,
            product_surface,
        };
        // Keep each draw's exact outer producer position until this point. The
        // existing generic BuiltinUiComposite path remains unchanged for legacy
        // frames; the F2 cutover will consume these stable boundaries to author
        // interleaved GPU nodes rather than re-sorting an already flat list.
        let ordered_legacy_draws = self.lower_ui_draws_with_submission_positions(
            &packet.prepared_ui,
            &packet.viewport_surface_bindings,
            &packet.ui_dynamic_bind_groups.viewport,
            &packet.ui_dynamic_bind_groups.product_surface,
            acquired_surface_extent,
        )?;
        if ordered_legacy_draws
            .windows(2)
            .any(|pair| pair[0].0 > pair[1].0)
        {
            bail!("UI draw preparation lost its producer ordering before GPU work authoring");
        }
        let builtin_ui_draws = ordered_legacy_draws
            .iter()
            .map(|(_, draw)| draw.clone())
            .collect::<Vec<_>>();

        let frame_index = prepared_frame.context.frame_index;
        let mut capture_runtime =
            FrameCaptureRuntime::new(frame_index, debug_control, &debug_config.capture_selectors);
        let mut flow_runtime_cache = std::mem::take(&mut self.flow_runtime_cache);
        let realization_result = (|| -> Result<(
            Vec<RealizedFlowInvocation<'a>>,
            u64,
            Vec<(RunenUiPublicationId, u32, Render2dPreparedContribution)>,
        )> {
            let active_flow_ids = compiled_flows
                .iter()
                .map(|flow| flow.flow_id)
                .collect::<Vec<_>>();
            flow_runtime_cache.retain(|flow_id, _| active_flow_ids.contains(flow_id));
            self.flow_pipeline_cache.retain_flows(&active_flow_ids);

            // Reserve every ordinary pass occurrence in one frame-owned identity space before any
            // projected-uniform, fixed-step, or timing-tail auxiliary occurrence is allocated.
            // The resulting schedules are then consumed in the same flow/invocation order while
            // each invocation's mutable runtime-resource scope is active.
            let mut maximum_occurrence = 0_u64;
            let mut scheduled_invocations = std::collections::VecDeque::new();
            for flow in compiled_flows {
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
                    let occurrences = expand_render_pass_occurrences_in_frame(
                        flow,
                        &invocation.inputs,
                        &mut maximum_occurrence,
                        |pass| {
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
                            Ok(true)
                        },
                    )?;
                    scheduled_invocations.push_back((
                        flow.flow_id.to_string(),
                        invocation.invocation_id.0.clone(),
                        invocation_packet,
                        occurrences,
                    ));
                }
            }

            let admitted_ui_passes = scheduled_invocations
                .iter()
                .flat_map(|(_, _, _, occurrences)| occurrences.iter())
                .filter(|occurrence| {
                    matches!(occurrence.pass, CompiledPassExecutionPlan::BuiltinUiComposite(_))
                })
                .count();
            if !packet.runenui_2d.is_empty() && admitted_ui_passes != 1 {
                bail!(
                    "RunenUI F2 requires exactly one admitted UI composite pass per acquired surface (got {admitted_ui_passes})"
                );
            }
            let mut prepared_runenui = Vec::new();
            if admitted_ui_passes == 1 {
                for publication in &packet.runenui_2d {
                    let target = Render2dTarget::new(
                        surface_view.clone(),
                        publication.logical_extent.0,
                        publication.logical_extent.1,
                        publication.raster_scale,
                    )?;
                    let contribution = self.runenui_2d_executor.prepare(
                        context,
                        publication.semantic.composition(),
                        publication.semantic.bindings(),
                        &target,
                    )?;
                    prepared_runenui.push((
                        publication.publication_id,
                        publication.compositor_position,
                        contribution,
                    ));
                }
            }

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
                    let Some((
                        scheduled_flow_id,
                        scheduled_invocation_id,
                        invocation_packet,
                        occurrences,
                    )) = scheduled_invocations.pop_front()
                    else {
                        bail!(
                            "frame occurrence reservation is missing flow '{}' invocation '{}'",
                            flow.flow_id,
                            invocation.invocation_id.0
                        );
                    };
                    if scheduled_flow_id != flow.flow_id.to_string()
                        || scheduled_invocation_id.as_str() != invocation.invocation_id.0.as_str()
                    {
                        bail!(
                            "frame occurrence reservation order mismatch: expected flow '{}' invocation '{}', found flow '{}' invocation '{}'",
                            flow.flow_id,
                            invocation.invocation_id.0,
                            scheduled_flow_id,
                            scheduled_invocation_id
                        );
                    }
                    let Some(view) = prepared_frame.view(invocation.view_id.as_str()) else {
                        bail!(
                            "prepared flow invocation '{}' references missing view '{}'",
                            invocation.invocation_id.0,
                            invocation.view_id
                        );
                    };
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
                            invocation.invocation_id.0.as_str(),
                            invocation_packet.surface_size,
                            invocation_packet.surface_format,
                            effective_history_signature,
                        )?;

                        let projected_uploads = self.realize_projected_uniform_uploads(
                            context,
                            flow,
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
                        let mut timing_frame = match logical_timing_plan
                            .as_ref()
                            .and_then(LogicalGpuPassTimingPlan::timing)
                        {
                            Some(timing) => Some(GpuPassTimingFrame::new(
                                context,
                                timing.query_set(),
                                timing.resolve_buffer(),
                                timing.readback_id(),
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
                                    surface_texture,
                                    acquired_surface_extent,
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
                                    surface_texture,
                                    acquired_surface_extent,
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
                            if let Some(indices) = timestamp_indices {
                                let frame = timing_frame.as_mut().ok_or_else(|| {
                                    anyhow::anyhow!(
                                        "timestampable pass '{}' has no realized timing resources",
                                        execution_pass_id(pass)
                                    )
                                })?;
                                if !frame.register_pass_metadata(
                                    indices,
                                    frame_index,
                                    prepared_frame.surface.render_surface_id.raw(),
                                    flow.flow_id.to_string(),
                                    execution_pass_id(pass).to_string(),
                                    execution_pass_kind_name(pass).to_string(),
                                ) {
                                    bail!(
                                        "renderer timing metadata for flow '{}' pass '{}' disagrees with its admitted query range",
                                        flow.flow_id,
                                        execution_pass_id(pass)
                                    );
                                }
                            }
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
                                before_captures: &scheduled.execution.before_captures,
                                after_captures: &scheduled.execution.after_captures,
                            })
                            .collect::<Vec<_>>();
                        let canonical_resolution = resolve_canonical_invocation(
                            flow,
                            &invocation.inputs,
                            runtime_resources,
                            Some(&self.dynamic_texture_targets),
                            CanonicalInvocationProjection {
                                projected_uploads: &projected_uploads,
                                passes: &canonical_projections,
                                surface_color_view: Some(surface_view),
                                builtin_ui_draws: Some(&builtin_ui_draws),
                                runenui_paint_present: !invocation_packet.runenui_2d.is_empty(),
                                timing: logical_timing_plan
                                    .as_ref()
                                    .and_then(LogicalGpuPassTimingPlan::timing),
                            },
                            &mut maximum_occurrence,
                        )?;
                        Ok(RealizedFlowInvocation {
                            flow,
                            invocation,
                            packet: invocation_packet,
                            scheduled_passes: realized_passes,
                            timing_frame,
                            canonical_resolution: Some(canonical_resolution),
                        })
                    })();
                    runtime_resources.clear_active_invocation_uniform_scope();
                    invocations.push(invocation_result?);
                }
                self.last_runtime_resources
                    .extend(runtime_resources.inspect_entries(flow.flow_id));
            }
            if !scheduled_invocations.is_empty() {
                bail!(
                    "frame occurrence reservation retained {} unconsumed invocation schedules",
                    scheduled_invocations.len()
                );
            }
            Ok((invocations, maximum_occurrence, prepared_runenui))
        })();
        self.flow_runtime_cache = flow_runtime_cache;
        let (invocations, maximum_occurrence, prepared_runenui) = realization_result?;
        let mut final_captures = Vec::new();
        if capture_runtime.should_attempt_stage(CaptureStage::Final) {
            self.prepare_final_surface_capture(
                context,
                surface_texture,
                acquired_surface_extent,
                &packet,
                &mut capture_runtime,
                &mut final_captures,
            )?;
        }
        Ok(RendererRealizationBatch {
            packet,
            dynamic_texture_uploads,
            capture_runtime,
            invocations,
            final_captures,
            maximum_occurrence,
            prepared_runenui,
            ordered_legacy_draws,
        })
    }

    fn prepare_deterministic_compositions(
        &mut self,
        context: &GpuContext,
        prepared_frame: &PreparedRenderFrame,
        contributions: &[crate::plugins::render::RenderDeterministicFrameContribution],
        target_requests: &crate::plugins::render::RenderDynamicTextureTargetRequestRegistryResource,
    ) -> Result<PreparedDeterministicCompositions> {
        const TEMPORAL_EVIDENCE_FRAME_CAPACITY: usize = 32;
        self.temporal_execution_evidence
            .remove(&prepared_frame.context.frame_index);
        let mut fragments = Vec::new();
        let mut imports = Vec::new();
        let mut retained_occurrences = Vec::new();
        for contribution in contributions.iter().filter(|contribution| {
            contribution.render_surface_id == prepared_frame.surface.render_surface_id
        }) {
            let target = self
                .dynamic_texture_targets
                .texture_handle(&contribution.target_key)?;
            // The contribution owns renderer-semantic intent before its GPU target exists.
            // Physical invocation is valid only after this product-owned late target resolution.
            let binding = runen_render::admission::RenderOutputBinding::new(
                contribution.output.clone(),
                runen_render::admission::RenderOutputDestination::SampleLatticeTexture(target),
            );
            let invocation = runen_render::RenderInvocation::new(
                contribution.scene.clone(),
                contribution.request.clone(),
                contribution.semantic_inputs.clone(),
                contribution.field_semantic_inputs.clone(),
                contribution.availability.clone(),
                vec![binding],
            )
            .map_err(|error| anyhow::Error::new(error).context("render invocation failed"))?;
            let admitted = runen_render::admit_render(&invocation, context)
                .map_err(|error| anyhow::Error::new(error).context("render admission failed"))?;
            let finite_evaluation = contribution.finite_evaluation_extent.map(|extent| {
                let (width, height) = extent.dimensions();
                runen_render::RenderEvaluationSelection::new(
                    contribution.output.clone(),
                    width,
                    height,
                )
                .expect("frame finite evaluation extent is already non-zero")
            });
            let prepared = self.render_sessions.prepare(
                contribution,
                target_requests,
                admitted,
                context,
                finite_evaluation,
            )?;
            let output = prepared
                .occurrence()
                .radiance_output(&contribution.output)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "deterministic render did not produce radiance output {}",
                        contribution.output.position()
                    )
                })?;
            if let Some(evidence) = output.temporal_execution_evidence() {
                self.temporal_execution_evidence
                    .entry(prepared_frame.context.frame_index)
                    .or_default()
                    .push(evidence.clone());
            }
            fragments.extend(prepared.occurrence().work_set().fragments().iter().cloned());
            imports.push(output.import(GpuResourceProvenance::new(
                GpuResourceLabel::new(format!(
                    "render.frame.radiance.composition.{}",
                    contribution.target_key
                ))?,
                None,
                None,
            )));
            retained_occurrences.push(prepared);
        }
        while self.temporal_execution_evidence.len() > TEMPORAL_EVIDENCE_FRAME_CAPACITY {
            let Some(oldest) = self.temporal_execution_evidence.keys().next().copied() else {
                break;
            };
            self.temporal_execution_evidence.remove(&oldest);
        }
        Ok(PreparedDeterministicCompositions {
            fragments,
            imports,
            occurrences: retained_occurrences,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        context: &GpuContext,
        surface_texture: &GpuTextureHandle,
        surface_view: &GpuTextureViewHandle,
        acquired_surface_extent: (u32, u32),
        prepared_frame: &PreparedRenderFrame,
        deterministic_contributions: &[crate::plugins::render::RenderDeterministicFrameContribution],
        target_requests: &crate::plugins::render::RenderDynamicTextureTargetRequestRegistryResource,
        shader_registry: &mut ShaderRegistryResource,
        compiled_flows: &[CompiledRenderFlowPlan],
        ui_rect_shader: Option<ShaderHandle>,
        ui_font_atlas: &UiFontAtlasResource,
        viewport_surface_bindings: &ViewportSurfaceBindingRegistry,
        surface_format: GpuTextureFormat,
        preflight_config: crate::plugins::render::graph::RenderPreflightValidationConfigResource,
        debug_control: &RenderDebugControlResource,
        debug_config: &RenderDebugConfigResource,
        gpu_timing_capability: RenderGpuTimingCapability,
        composed_gpu_timing_capability: Option<RenderGpuTimingCapability>,
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
            surface_texture,
            surface_view,
            acquired_surface_extent,
            prepared_frame,
            deterministic_contributions,
            target_requests,
            packet,
            compiled_flows,
            shader_registry,
            preflight_config,
            debug_control,
            debug_config,
            gpu_timing_capability,
            composed_gpu_timing_capability,
        )
    }

    fn realize_projected_uniform_uploads(
        &self,
        _context: &GpuContext,
        flow: &CompiledRenderFlowPlan,
        invocation_id: &str,
        flow_inputs: &PreparedFlowInputs,
        runtime_resources: &mut FlowRuntimeResources,
        maximum_occurrence: &mut u64,
    ) -> Result<Vec<RealizedLogicalBufferUpload>> {
        let mut uploads = Vec::new();
        for (buffer_id, bytes) in &flow_inputs.projected_uniform_bytes {
            if flow
                .execution
                .fixed_step_regions
                .iter()
                .any(|region| region.iteration_uniform == *buffer_id)
            {
                continue;
            }
            let prepared = runtime_resources.prepare_uniform_upload(*buffer_id, bytes)?;
            let runtime_buffer = runtime_resources.realize_invocation_uniform_buffer(
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
                control_order_after: Vec::new(),
            });
        }

        Ok(uploads)
    }

    #[allow(clippy::too_many_arguments)]
    fn realize_fixed_step_iteration_upload(
        &self,
        _context: &GpuContext,
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
        surface_texture: &GpuTextureHandle,
        acquired_surface_extent: (u32, u32),
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

            let resolved_key =
                runtime_resources.resolve_resource_key_from_input(selector.resource_id.as_str());
            let resolved_texture = match resolved_key {
                Some(RuntimeResourceKey::SurfaceColor) => Ok(None),
                Some(RuntimeResourceKey::DynamicTexture(key)) => self
                    .dynamic_texture_targets
                    .texture_ref(pass_id, &key)
                    .map(Some),
                None => {
                    if let Some(key) =
                        crate::plugins::render::RenderDynamicTextureTargetKey::from_label(
                            selector.resource_id.as_str(),
                        )
                    {
                        self.dynamic_texture_targets
                            .texture_ref(pass_id, &key)
                            .map(Some)
                    } else if selector.resource_id == SURFACE_COLOR_RESOURCE_LABEL {
                        Ok(None)
                    } else {
                        runtime_resources
                            .resolve_texture_from_label_without_surface(
                                pass_label.as_str(),
                                selector.resource_id.as_str(),
                            )
                            .map(Some)
                    }
                }
                _ => runtime_resources
                    .resolve_texture_from_label_without_surface(
                        pass_label.as_str(),
                        selector.resource_id.as_str(),
                    )
                    .map(Some),
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

            let (capture_source, capture_size, capture_format) = match resolved_texture {
                None => (
                    CaptureTextureSource::Surface {
                        handle: surface_texture,
                    },
                    acquired_surface_extent,
                    packet.surface_format,
                ),
                Some(resolved) => {
                    let logical_view = resolved.view_handle.ok_or_else(|| {
                        anyhow::anyhow!(
                            "capture source '{}' has no logical texture view",
                            selector.resource_id
                        )
                    })?;
                    (
                        CaptureTextureSource::Logical {
                            handle: logical_view.descriptor().texture(),
                        },
                        resolved.size,
                        resolved.format,
                    )
                }
            };
            let readback_format = texture_readback_format(capture_format);
            let readback_format = match readback_format {
                Some(value) => value,
                None => {
                    let terminal = RenderCaptureTerminal::with_reason(
                        RenderCaptureTerminalCode::Unsupported,
                        "unsupported_readback_format",
                        format!(
                            "readback for format {:?} is not implemented yet",
                            capture_format
                        ),
                    );
                    capture_runtime.set_terminal(selector_index, terminal.clone());
                    self.last_captured_textures.push(RenderCapturedTexture {
                        identity,
                        width: capture_size.0,
                        height: capture_size.1,
                        format: format!("{:?}", capture_format),
                        bytes_rgba8: None,
                        terminal,
                    });
                    continue;
                }
            };

            match prepare_texture_capture_readback(
                context,
                selector_index,
                selector.clone(),
                identity,
                capture_source,
                capture_size,
                capture_format,
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
                        width: capture_size.0,
                        height: capture_size.1,
                        format: format!("{:?}", capture_format),
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
        surface_texture: &GpuTextureHandle,
        acquired_surface_extent: (u32, u32),
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
                    width: acquired_surface_extent.0,
                    height: acquired_surface_extent.1,
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
                    width: acquired_surface_extent.0,
                    height: acquired_surface_extent.1,
                    format: format!("{:?}", packet.surface_format),
                    bytes_rgba8: None,
                    terminal,
                });
                continue;
            };

            match prepare_texture_capture_readback(
                context,
                selector_index,
                selector.clone(),
                identity,
                CaptureTextureSource::Surface {
                    handle: surface_texture,
                },
                acquired_surface_extent,
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
                        width: acquired_surface_extent.0,
                        height: acquired_surface_extent.1,
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

fn resolve_terminal_present_controls(
    terminal_present_controls: Vec<Vec<RenderGpuWorkOccurrenceId>>,
) -> Result<Vec<RenderGpuWorkOccurrenceId>> {
    if terminal_present_controls.is_empty() {
        bail!("presenting normal frame resolved no compiled Present");
    }

    Ok(terminal_present_controls.into_iter().flatten().fold(
        Vec::new(),
        |mut controls, occurrence| {
            if !controls.contains(&occurrence) {
                controls.push(occurrence);
            }
            controls
        },
    ))
}

fn accepted_pass_provenance(
    frame_index: u64,
    flow: &CompiledRenderFlowPlan,
    packet: &RendererPreparedPacket,
    pass: &CompiledPassExecutionPlan,
    runtime_resources: &FlowRuntimeResources,
    evidence: &EncodedPassEvidence,
) -> RenderPassProvenanceRecord {
    let pass_label = execution_pass_id(pass).to_string();
    let pass_resource_truth = collect_pass_resource_truth(flow.flow_id, pass, runtime_resources);
    let material_binding = collect_pass_material_binding_evidence(packet, pass);
    RenderPassProvenanceRecord {
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
        feature_runtime_version: feature_runtime_version(packet, execution_pass_feature_id(pass)),
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn late_physical_invocation_keeps_published_output_and_typed_foreign_handle_failure() {
        use crate::plugins::render::{
            RenderDeterministicFiniteEvaluationExtent, RenderDeterministicFrameContribution,
            RenderDynamicTextureRetention, RenderDynamicTextureTargetDescriptor,
            RenderDynamicTextureTargetKey, RenderDynamicTextureTargetRequestRegistryResource,
            RenderFrameProducerId, RenderTextureSampleMode, RenderTextureTargetFormat,
            RenderTextureTargetUsage, host::RenderSurfaceId,
        };
        use runen_render::request::*;
        use runen_render::space_time::*;
        let descriptor = runen_gpu::GpuContextDescriptor::new(
            runen_gpu::GpuCapabilityProfile::ComputeBaseline.requirements(),
        )
        .require_format_role(
            runen_gpu::GpuTextureFormat::R32Float,
            runen_gpu::GpuFormatRole::CopyDestination,
        );
        let context = match pollster::block_on(GpuContext::request(descriptor)) {
            Ok(context) => context,
            Err(error)
                if error.category()
                    == runen_gpu::GpuContextRequestErrorCategory::NoAdapterAvailable =>
            {
                assert_ne!(
                    std::env::var("RUNENRENDER_R8_REQUIRE_GPU").ok().as_deref(),
                    Some("1")
                );
                return;
            }
            Err(error) => panic!("late physical binding context: {error}"),
        };
        let build_request = || {
            let shutter = RenderTimeInterval::instant(RenderTimePoint::from_seconds(0.0).unwrap());
            let mut builder = RenderRequestBuilder::new(shutter);
            let observation = builder.add_observation(RenderObservationSpec::Perspective(
                RenderPerspectiveObservation::new(
                    RenderAffineTransform3::identity(),
                    std::f64::consts::FRAC_PI_3,
                    1.0,
                    shutter,
                    RenderSamplingSupport::perspective_lattice_cell(),
                )
                .unwrap(),
            ));
            let output = builder
                .add_output(
                    &observation,
                    RenderOutputSpec::new(
                        RenderOutputValue::Radiance {
                            representation:
                                RenderRadiometricRepresentation::spectral_at_wavelength_meters(
                                    550e-9,
                                )
                                .unwrap(),
                        },
                        RenderResultTopology::sample_lattice_2d(2, 2).unwrap(),
                        RenderSemanticTolerance::exact(),
                    )
                    .unwrap(),
                )
                .unwrap();
            (builder.finish().unwrap(), output)
        };
        let (request, output) = build_request();
        let (foreign_request, foreign) = build_request();
        assert_eq!(request, foreign_request);
        let producer = RenderFrameProducerId::try_from_raw(1230).unwrap();
        let key = RenderDynamicTextureTargetKey::new("late-binding", "radiance");
        let contribution = RenderDeterministicFrameContribution {
            producer_id: producer,
            render_surface_id: RenderSurfaceId::primary(),
            scene: runen_render::scene::RenderSceneStore::new().snapshot(),
            request,
            semantic_inputs: Vec::new(),
            field_semantic_inputs: Vec::new(),
            availability: Vec::new(),
            output: output.clone(),
            target_key: key.clone(),
            finite_evaluation_extent: RenderDeterministicFiniteEvaluationExtent::new(2, 2),
        };
        let frame = PreparedRenderFrame {
            context: crate::plugins::render::PreparedFrameContext {
                frame_index: 0,
                flow_registry_revision: 0,
                shader_registry_revision: 0,
                prepare_epoch: 0,
            },
            surface: crate::plugins::render::PreparedSurfaceInfo::unbound_primary((2, 2)),
            views: Vec::new(),
            flows: BTreeMap::new(),
            flow_invocations: Vec::new(),
            dynamic_texture_targets: Vec::new(),
            dynamic_texture_uploads: Vec::new(),
            product_selections: Vec::new(),
            viewport_surface_bindings: Default::default(),
            contributions: Default::default(),
            shader: Default::default(),
        };
        let target = RenderDynamicTextureTargetDescriptor::new(
            key,
            2,
            2,
            RenderTextureTargetFormat::R32Float,
            RenderTextureTargetUsage {
                color_attachment: false,
                depth_attachment: false,
                sampled: true,
                storage: false,
                copy_src: false,
                copy_dst: true,
            },
            RenderTextureSampleMode::NonFilterableFloat,
            RenderDynamicTextureRetention::RetainWhileRequested,
        );
        let mut targets = RenderDynamicTextureTargetRequestRegistryResource::default();
        targets
            .replace_surface_contribution(producer, RenderSurfaceId::primary(), [target.clone()])
            .unwrap();
        let mut renderer = Renderer::new();
        assert!(
            renderer
                .prepare_deterministic_compositions(
                    &context,
                    &frame,
                    std::slice::from_ref(&contribution),
                    &targets,
                )
                .is_err(),
            "published semantic work cannot prepare before physical target realization"
        );
        renderer
            .dynamic_texture_targets
            .realize_for_frame(&context, &[target], &BTreeMap::new())
            .unwrap();
        let mut invalid = contribution.clone();
        invalid.output = foreign.clone();
        let error = match renderer.prepare_deterministic_compositions(
            &context,
            &frame,
            &[invalid],
            &targets,
        ) {
            Err(error) => error,
            Ok(_) => panic!("late binding must reject an equal-position foreign handle"),
        };
        assert!(
            matches!(error.downcast_ref::<runen_render::RenderInvocationError>(),
            Some(runen_render::RenderInvocationError::ForeignOutput { output }) if output == &foreign)
        );
        let prepared = renderer
            .prepare_deterministic_compositions(
                &context,
                &frame,
                std::slice::from_ref(&contribution),
                &targets,
            )
            .unwrap();
        assert_eq!(prepared.occurrences.len(), 1);
        let occurrence = prepared.occurrences[0].occurrence();
        assert_eq!(
            occurrence.radiance_output(&output).unwrap().output(),
            &output
        );
        assert!(occurrence.radiance_output(&foreign).is_none());
        assert_eq!(
            renderer.temporal_execution_evidence[&0][0].evaluation_extent,
            (2, 2)
        );
    }

    #[test]
    fn presenting_frame_rejects_when_no_compiled_present_was_resolved() {
        let error = resolve_terminal_present_controls(Vec::new())
            .expect_err("a presenting frame without a compiled Present must be rejected");

        assert_eq!(
            error.to_string(),
            "presenting normal frame resolved no compiled Present"
        );
    }

    #[test]
    fn compiled_present_without_non_data_predecessors_is_valid() {
        let controls = resolve_terminal_present_controls(vec![Vec::new()])
            .expect("an empty inner control set still records a real compiled Present");

        assert!(controls.is_empty());
    }
}
