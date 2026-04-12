use crate::plugins::inspect::{
    RenderCaptureSelector, RenderCaptureTerminal, RenderCaptureTerminalCode,
    RenderCaptureTerminalReason, RenderCapturedTexture, RenderCapturedTextureState,
    RenderDebugConfigResource, RenderDebugControlResource, RenderDebugFrameReport,
    RenderDebugFrameReportState, RenderDebugTimingsState, RenderPassProvenanceState,
    RenderPixelCoordinate, RenderPixelProbeAssertionMode, RenderPixelProbeRequest,
    RenderPixelProbeResult, RenderPixelProbeStatus, RenderPixelSampleMode,
    RenderRuntimeResourceInspectorState, RenderTextureDiffMetrics, RenderTextureDiffMismatchSample,
    RenderTextureDiffRequest, RenderTextureDiffResult, RenderTextureDiffStatus,
    RenderTextureInspectorState, export_captured_textures, validate_selector_terminal_invariant,
};
use crate::plugins::pipelines::{PipelineCacheResource, PipelineCacheStats};
use crate::plugins::render::*;
use crate::plugins::scene::ui::UiRenderShaderConfig;
use crate::plugins::scene::*;
use crate::plugins::time::domain::Time;
use crate::runtime::{Res, ResMut, WorldMut};
use crate::state::{DebugMetricsState, StartupState, UiOverlayState};
use anyhow::anyhow;
use scheduler::set_slow_node_logging_enabled;
use std::any::{Any, TypeId};
use std::collections::{BTreeMap, BTreeSet};
use wgpu::SurfaceError;

const FRAME_TIMING_LOG_THRESHOLD_MS: f32 = 20.0;
const MESH_HOT_PATH_LOG_THRESHOLD_MS: f32 = 8.0;
const SCENE_OVERLAY_UI_PRODUCER_ID: &str = "scene.overlay";
const DEBUG_METRICS_UI_PRODUCER_ID: &str = "debug.metrics";

fn render_timing_logging_enabled() -> bool {
    std::env::var("GROTTO_RENDER_TIMING_LOG")
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

pub(crate) fn collect_runtime_ui_frame_submissions_system(mut world: WorldMut) {
    let Some(mut submissions) = world.remove_resource::<UiFrameSubmissionRegistryResource>() else {
        return;
    };

    let scene_submission = world
        .resource::<SceneResource>()
        .ok()
        .and_then(|scene_resource| scene_resource.manager.as_ref())
        .map(|manager| {
            let rect_shader_asset_id = manager
                .overlay_runtime
                .world
                .resource::<UiRenderShaderConfig>()
                .ok()
                .map(|config| config.rect_shader_asset_id.trim().to_string())
                .filter(|id| !id.is_empty());
            (
                manager.overlay_runtime.ui.frame.clone(),
                rect_shader_asset_id,
            )
        });

    match scene_submission {
        Some((frame, rect_shader_asset_id)) if !frame.is_empty() => {
            submissions.replace(
                UiFrameSubmission::new(SCENE_OVERLAY_UI_PRODUCER_ID)
                    .with_route(UiFrameRoute::Screen)
                    .with_order(UiFrameSubmissionOrder::new(0, 0))
                    .with_frame(frame)
                    .with_rect_shader_asset_id(rect_shader_asset_id),
            );
        }
        _ => {
            submissions.remove(&UiFrameProducerId::new(SCENE_OVERLAY_UI_PRODUCER_ID));
        }
    }

    let debug_frame = world
        .resource::<UiOverlayState>()
        .ok()
        .map(|overlay| overlay.debug_frame.clone())
        .unwrap_or_default();
    if debug_frame.is_empty() {
        submissions.remove(&UiFrameProducerId::new(DEBUG_METRICS_UI_PRODUCER_ID));
    } else {
        submissions.replace(
            UiFrameSubmission::new(DEBUG_METRICS_UI_PRODUCER_ID)
                .with_route(UiFrameRoute::Screen)
                .with_order(UiFrameSubmissionOrder::new(100, 0))
                .with_frame(debug_frame),
        );
    }

    world.insert_resource(submissions);
}

pub(crate) fn frame_render_prepare_system(
    mut world: WorldMut,
    mut scene_resource: ResMut<SceneResource>,
) -> anyhow::Result<()> {
    let Some(manager) = scene_resource.manager.as_mut() else {
        clear_prepared_frame(&mut world);
        return Ok(());
    };

    let Some(mut shader_registry) = world.remove_resource::<ShaderRegistryResource>() else {
        clear_prepared_frame(&mut world);
        return Ok(());
    };

    let _ = shader_registry.poll_updates();
    let shader_reload_messages = shader_registry.drain_message_lines();
    if !shader_reload_messages.is_empty() {
        for msg in shader_reload_messages {
            manager
                .overlay_runtime
                .ui
                .log_lines
                .push(format!("[world] {msg}"));
        }
        clamp_lines(
            &mut manager.overlay_runtime.ui.log_lines,
            manager.overlay_runtime.ui.max_lines,
        );
        manager.overlay_runtime.ui.log_scroll_lines_from_bottom = 0;
    }

    let target_size = {
        let (window_w, window_h) = manager.overlay_runtime.ui.screen_size;
        (
            (window_w.max(1.0)).round() as u32,
            (window_h.max(1.0)).round() as u32,
        )
    };

    let (flow_registry_revision, compiled_flows, execution_feature_ids, flows) = {
        let flow_registry = match world.resource::<RenderFlowRegistryResource>() {
            Ok(registry) => registry,
            Err(_) => {
                world.insert_resource(shader_registry);
                clear_prepared_frame(&mut world);
                return Ok(());
            }
        };
        let compiled_flows = flow_registry.compiled_flows();
        let execution_feature_ids = collect_execution_feature_ids(compiled_flows);
        let extracted = collect_flow_declared_state_resources(&world, compiled_flows);
        let flows = build_prepared_flow_inputs(compiled_flows, &extracted, target_size)?;
        (
            flow_registry.revision(),
            compiled_flows.len(),
            execution_feature_ids,
            flows,
        )
    };

    let (frame_index, prepare_epoch) = {
        if let Ok(prepared_resource) = world.resource_mut::<PreparedRenderFrameResource>() {
            (
                prepared_resource.allocate_frame_index(),
                prepared_resource.allocate_prepare_epoch(),
            )
        } else {
            (0, 0)
        }
    };

    let contributions = build_frame_feature_contributions(
        &world,
        manager.world.active.label().to_string(),
        manager.active_overlay().label().to_string(),
        &execution_feature_ids,
    );

    let prepared = PreparedRenderFrame {
        context: PreparedFrameContext {
            frame_index,
            flow_registry_revision,
            shader_registry_revision: shader_registry.revision(),
            prepare_epoch,
        },
        surface: PreparedSurfaceInfo {
            target_size_px: target_size,
        },
        views: vec![PreparedViewFrame::main(target_size)],
        flows,
        contributions,
        shader: PreparedShaderSnapshot {
            registry_revision: shader_registry.revision(),
        },
    };

    if let Ok(prepared_resource) = world.resource_mut::<PreparedRenderFrameResource>() {
        prepared_resource.publish(prepared);
    } else {
        let mut prepared_resource = PreparedRenderFrameResource::default();
        prepared_resource.publish(prepared);
        world.insert_resource(prepared_resource);
    }

    world.insert_resource(shader_registry);

    if compiled_flows == 0 {
        clear_prepared_frame(&mut world);
    }

    Ok(())
}

pub(crate) fn frame_render_submit_system(
    mut world: WorldMut,
    time: Res<Time>,
    scene_resource: ResMut<SceneResource>,
    mut startup: ResMut<StartupState>,
    mut debug_metrics: ResMut<DebugMetricsState>,
) -> anyhow::Result<()> {
    if scene_resource.manager.is_none() {
        return Ok(());
    }

    let _submit_span = tracing::info_span!("systems.frame_render_submit").entered();
    let startup_ready_before = startup.is_ready();
    let timing_log_enabled = render_timing_logging_enabled();
    set_slow_node_logging_enabled(startup_ready_before);

    let prepared_frame = {
        let Some(mut prepared_resource) = world.remove_resource::<PreparedRenderFrameResource>()
        else {
            return Ok(());
        };
        let prepared_frame = match prepared_resource.take() {
            Some(value) => value,
            None => {
                world.insert_resource(prepared_resource);
                return Ok(());
            }
        };
        world.insert_resource(prepared_resource);
        prepared_frame
    };

    let Some(mut shader_registry) = world.remove_resource::<ShaderRegistryResource>() else {
        return Ok(());
    };

    let Some(mut gfx) = world.remove_resource::<Gfx>() else {
        world.insert_resource(shader_registry);
        return Ok(());
    };

    let (target_w, target_h) = prepared_frame
        .main_view()
        .map(|value| value.target_size_px)
        .unwrap_or(prepared_frame.surface.target_size_px);
    if gfx.ctx.surface_config.width != target_w || gfx.ctx.surface_config.height != target_h {
        gfx.resize(target_w, target_h);
    }
    let ui_font_atlas = world
        .resource::<UiFontAtlasResource>()
        .ok()
        .map(|resource| resource.clone())
        .unwrap_or_default();
    let debug_control = world
        .resource::<RenderDebugControlResource>()
        .ok()
        .map(|resource| resource.clone())
        .unwrap_or_default();
    let debug_config = world
        .resource::<RenderDebugConfigResource>()
        .ok()
        .map(|resource| resource.clone())
        .unwrap_or_default();

    let render_result = {
        let flow_registry = match world.resource::<RenderFlowRegistryResource>() {
            Ok(registry) => registry,
            Err(_) => {
                world.insert_resource(shader_registry);
                world.insert_resource(gfx);
                return Ok(());
            }
        };
        if flow_registry.revision() != prepared_frame.context.flow_registry_revision {
            world.insert_resource(shader_registry);
            world.insert_resource(gfx);
            return Ok(());
        }
        let compiled_flows = flow_registry.compiled_flows();

        let ui_rect_shader: Option<ShaderHandle> = prepared_frame
            .ui()
            .and_then(|ui| ui.first_rect_shader_asset_id())
            .and_then(|id| shader_registry.handle(id));

        gfx.render(
            &prepared_frame,
            &mut shader_registry,
            compiled_flows,
            ui_rect_shader,
            &ui_font_atlas,
            &debug_control,
            &debug_config,
        )
    };

    let result = match render_result {
        Ok(timings) => {
            debug_metrics.last_timings = Some(timings);
            if let Ok(render_debug_timings) = world.resource_mut::<RenderDebugTimingsState>() {
                render_debug_timings.observe_frame_timings(timings);
                render_debug_timings.observe_pass_timings(gfx.renderer.last_pass_timings());
            }
            let cache_stats = gfx.renderer.flow_pipeline_cache_stats();
            if let Ok(cache_resource) = world.resource_mut::<PipelineCacheResource>() {
                cache_resource.observe_stats(PipelineCacheStats {
                    hits: cache_stats.hits,
                    misses: cache_stats.misses,
                });
            }
            if let Ok(runtime_resources) =
                world.resource_mut::<RenderRuntimeResourceInspectorState>()
            {
                runtime_resources.observe_runtime_resources(gfx.renderer.last_runtime_resources());
                runtime_resources
                    .observe_pipeline_cache_stats(cache_stats.hits, cache_stats.misses);
            }
            if let Ok(pass_provenance) = world.resource_mut::<RenderPassProvenanceState>() {
                pass_provenance.observe_frame(
                    prepared_frame.context.frame_index,
                    gfx.renderer.last_pass_provenance(),
                );
            }
            if let Ok(captured_textures) = world.resource_mut::<RenderCapturedTextureState>() {
                captured_textures.observe_frame(
                    prepared_frame.context.frame_index,
                    gfx.renderer.last_captured_textures(),
                );
            }
            if let Ok(texture_inspector) = world.resource_mut::<RenderTextureInspectorState>() {
                texture_inspector.observe_captures(gfx.renderer.last_captured_textures());
            }
            let mut selector_results = gfx.renderer.last_capture_selector_results().to_vec();
            let mut artifact_manifest_path = None;
            if debug_control.artifact_export_enabled
                && !gfx.renderer.last_captured_textures().is_empty()
            {
                match export_captured_textures(
                    debug_control.artifact_output_dir.as_path(),
                    prepared_frame.context.frame_index,
                    gfx.renderer.last_captured_textures(),
                ) {
                    Ok(export) => {
                        artifact_manifest_path = Some(export.manifest_path.clone());
                        for exported in &export.exported_capture_images {
                            let exported_point = exported.frame_identity.capture_point.clone();
                            for result in &mut selector_results {
                                if result.capture_point != exported_point {
                                    continue;
                                }
                                if result.terminal.code == RenderCaptureTerminalCode::Completed {
                                    result.artifact_path = Some(exported.image_path.clone());
                                }
                            }
                        }
                    }
                    Err(err) => {
                        let reason = RenderCaptureTerminalReason::new(
                            "artifact_export_failed",
                            err.to_string(),
                        );
                        for result in &mut selector_results {
                            if result.terminal.code == RenderCaptureTerminalCode::Completed {
                                result.terminal = RenderCaptureTerminal::new(
                                    RenderCaptureTerminalCode::ExportFailed,
                                    Some(reason.clone()),
                                );
                            }
                        }
                        tracing::warn!(error = %err, "render capture artifact export failed");
                    }
                }
            }

            let mut frame_report = RenderDebugFrameReport {
                frame_index: prepared_frame.context.frame_index,
                provenance: gfx.renderer.last_pass_provenance().to_vec(),
                capture_plan: gfx.renderer.last_capture_plan().clone(),
                capture_results: selector_results,
                artifact_manifest_path,
                pixel_probe_results: evaluate_pixel_probes(
                    &debug_config.pixel_probes,
                    gfx.renderer.last_captured_textures(),
                ),
                texture_diff_results: evaluate_texture_diffs(
                    &debug_config.texture_diffs,
                    gfx.renderer.last_captured_textures(),
                ),
                warnings: Vec::new(),
                errors: Vec::new(),
            };
            if let Err(violations) = validate_selector_terminal_invariant(
                &debug_config.capture_selectors,
                &frame_report.capture_results,
            ) {
                frame_report
                    .errors
                    .extend(violations.into_iter().map(|value| {
                        format!(
                            "selector invariant violation at index {}: {}",
                            value.selector_index, value.message
                        )
                    }));
            }
            if let Ok(report_state) = world.resource_mut::<RenderDebugFrameReportState>() {
                report_state.observe_frame(frame_report);
            }
            let mesh_hot = timings.renderer.mesh_hot_path;
            let warm_frame = mesh_hot.is_warm_frame();
            let warmup_completed =
                startup.observe_render_warm_frame(warm_frame, time.delta_seconds.max(0.0));
            if warmup_completed {
                tracing::info!(
                    elapsed_loading_seconds = startup.elapsed_loading_seconds,
                    stable_frames = startup.stable_frames,
                    required_stable_frames = startup.required_stable_frames,
                    warm_frame,
                    "startup warmup complete; scene flow can transition out of loading screen"
                );
            }
            let total_ms = timings.acquire_ms
                + timings.renderer.prepare_ui_ms
                + timings.renderer.prepare_mesh_ms
                + timings.renderer.world_prepare_ms
                + timings.renderer.encode_submit_ms
                + timings.present_ms;
            let workload_ms = timings.renderer.prepare_ui_ms
                + timings.renderer.prepare_mesh_ms
                + timings.renderer.world_prepare_ms
                + timings.renderer.encode_submit_ms;
            if startup_ready_before
                && timing_log_enabled
                && workload_ms > FRAME_TIMING_LOG_THRESHOLD_MS
            {
                tracing::info!(
                    workload_ms = workload_ms,
                    total_ms = total_ms,
                    acquire_ms = timings.acquire_ms,
                    prepare_ui_ms = timings.renderer.prepare_ui_ms,
                    prepare_mesh_ms = timings.renderer.prepare_mesh_ms,
                    world_prepare_ms = timings.renderer.world_prepare_ms,
                    encode_submit_ms = timings.renderer.encode_submit_ms,
                    present_ms = timings.present_ms,
                    mesh_model_collect_ms = mesh_hot.model_collect_ms,
                    mesh_chunk_collect_ms = mesh_hot.chunk_collect_ms,
                    mesh_merge_filter_ms = mesh_hot.merge_filter_ms,
                    mesh_camera_update_ms = mesh_hot.camera_update_ms,
                    mesh_static_upload_ms = mesh_hot.static_upload_ms,
                    mesh_agent_upload_ms = mesh_hot.agent_upload_ms,
                    mesh_model_meshes = mesh_hot.model_meshes,
                    mesh_chunk_meshes = mesh_hot.chunk_meshes,
                    mesh_merged_meshes = mesh_hot.merged_meshes,
                    mesh_skipped_meshes = mesh_hot.skipped_meshes,
                    mesh_draw_items = mesh_hot.draw_items,
                    mesh_textured_meshes = mesh_hot.textured_meshes,
                    mesh_vertex_count = mesh_hot.vertex_count,
                    mesh_index_count = mesh_hot.index_count,
                    mesh_texture_upload_bytes = mesh_hot.texture_upload_bytes,
                    mesh_vertex_upload_bytes = mesh_hot.vertex_upload_bytes,
                    mesh_index_upload_bytes = mesh_hot.index_upload_bytes,
                    mesh_instance_upload_bytes = mesh_hot.instance_upload_bytes,
                    mesh_uniform_upload_bytes = mesh_hot.uniform_upload_bytes,
                    mesh_agent_instances = mesh_hot.agent_instances,
                    mesh_static_cache_hits = mesh_hot.static_cache_hits,
                    mesh_static_cache_misses = mesh_hot.static_cache_misses,
                    "frame render timing breakdown"
                );
            }
            if startup_ready_before
                && timing_log_enabled
                && timings.renderer.prepare_mesh_ms > MESH_HOT_PATH_LOG_THRESHOLD_MS
            {
                tracing::info!(
                    prepare_mesh_ms = timings.renderer.prepare_mesh_ms,
                    model_collect_ms = mesh_hot.model_collect_ms,
                    chunk_collect_ms = mesh_hot.chunk_collect_ms,
                    merge_filter_ms = mesh_hot.merge_filter_ms,
                    static_upload_ms = mesh_hot.static_upload_ms,
                    agent_upload_ms = mesh_hot.agent_upload_ms,
                    model_meshes = mesh_hot.model_meshes,
                    chunk_meshes = mesh_hot.chunk_meshes,
                    merged_meshes = mesh_hot.merged_meshes,
                    skipped_meshes = mesh_hot.skipped_meshes,
                    draw_items = mesh_hot.draw_items,
                    textured_meshes = mesh_hot.textured_meshes,
                    vertex_count = mesh_hot.vertex_count,
                    index_count = mesh_hot.index_count,
                    texture_upload_bytes = mesh_hot.texture_upload_bytes,
                    vertex_upload_bytes = mesh_hot.vertex_upload_bytes,
                    index_upload_bytes = mesh_hot.index_upload_bytes,
                    instance_upload_bytes = mesh_hot.instance_upload_bytes,
                    uniform_upload_bytes = mesh_hot.uniform_upload_bytes,
                    agent_instances = mesh_hot.agent_instances,
                    static_cache_hits = mesh_hot.static_cache_hits,
                    static_cache_misses = mesh_hot.static_cache_misses,
                    "mesh prepare hot path breakdown"
                );
            }
            Ok(())
        }
        Err(err) => {
            if let Some(surface_error) = err.downcast_ref::<SurfaceError>() {
                match surface_error {
                    SurfaceError::Lost | SurfaceError::Outdated => {
                        gfx.resize(target_w, target_h);
                        Ok(())
                    }
                    SurfaceError::Timeout => Ok(()),
                    SurfaceError::OutOfMemory => anyhow::bail!("surface out of memory"),
                    SurfaceError::Other => Ok(()),
                }
            } else {
                Err(anyhow!("render backend execution failed: {err:#}"))
            }
        }
    };

    world.insert_resource(shader_registry);
    world.insert_resource(gfx);
    result
}

fn clear_prepared_frame(world: &mut WorldMut) {
    if let Ok(prepared_resource) = world.resource_mut::<PreparedRenderFrameResource>() {
        prepared_resource.clear();
    }
}

fn build_frame_feature_contributions(
    world: &ecs::World,
    world_scene_label: String,
    overlay_scene_label: String,
    execution_feature_ids: &[RenderFeatureId],
) -> PreparedFrameContributions {
    let mut contributions = PreparedFrameContributions::default();

    let scene_policy = feature_policy(
        world,
        RenderFeatureId::new(SCENE_ROUTE_RENDER_FEATURE_ID),
        FeatureFallbackPolicy::EmptyContribution,
    );
    contributions.insert_scene_route(
        world_scene_label,
        overlay_scene_label,
        FeatureContributionStatus::Ready,
        scene_policy,
    );

    if let Ok(resource) = world.resource::<PreparedUiFrameResource>() {
        let ui_policy = feature_policy(
            world,
            RenderFeatureId::new(UI_RENDER_FEATURE_ID),
            resource.fallback_policy,
        );
        contributions.insert_ui(resource.payload.clone(), resource.status, ui_policy);
    }

    if let Ok(resource) = world.resource::<PreparedWorldFeatureResource>() {
        let world_policy = feature_policy(
            world,
            RenderFeatureId::new(WORLD_DRAW_RENDER_FEATURE_ID),
            resource.fallback_policy,
        );
        contributions.insert_world(resource.payload.clone(), resource.status, world_policy);
    }

    if let Ok(resource) = world.resource::<PreparedDrawFeatureResource>() {
        let world_feature_id = RenderFeatureId::new(WORLD_DRAW_RENDER_FEATURE_ID);
        let should_publish_draw = !matches!(resource.status, FeatureContributionStatus::Missing)
            || contributions.feature(&world_feature_id).is_none();
        if should_publish_draw {
            let draw_policy = feature_policy(world, world_feature_id, resource.fallback_policy);
            contributions.insert_draw(resource.payload.clone(), resource.status, draw_policy);
        }
    }

    if let Ok(resource) = world.resource::<PreparedCaveFeatureResource>() {
        let cave_policy = feature_policy(
            world,
            RenderFeatureId::new(CAVE_INTERIOR_RENDER_FEATURE_ID),
            resource.fallback_policy,
        );
        contributions.insert_caves(resource.payload.clone(), resource.status, cave_policy);
    }

    if let Ok(resource) = world.resource::<PreparedDetailFeatureResource>() {
        let detail_policy = feature_policy(
            world,
            RenderFeatureId::new(DETAIL_RENDER_FEATURE_ID),
            resource.fallback_policy,
        );
        contributions.insert_detail(resource.payload.clone(), resource.status, detail_policy);
    }

    if let Ok(resource) = world.resource::<PreparedProceduralWorldFeatureResource>() {
        let procedural_policy = feature_policy(
            world,
            RenderFeatureId::new(PROCEDURAL_WORLD_RENDER_FEATURE_ID),
            resource.fallback_policy,
        );
        contributions.insert_procedural_world(
            resource.payload.clone(),
            resource.status,
            procedural_policy,
        );
    }

    if let Ok(resource) = world.resource::<PreparedMaterialFeatureResource>() {
        let material_policy = feature_policy(
            world,
            RenderFeatureId::new(MATERIAL_RENDER_FEATURE_ID),
            resource.fallback_policy,
        );
        contributions.insert_material(resource.payload.clone(), resource.status, material_policy);
    }

    if let Ok(resource) = world.resource::<PreparedDeformationFeatureResource>() {
        let deformation_policy = feature_policy(
            world,
            RenderFeatureId::new(DEFORMATION_RENDER_FEATURE_ID),
            resource.fallback_policy,
        );
        contributions.insert_deformation(
            resource.payload.clone(),
            resource.status,
            deformation_policy,
        );
    }

    if let Ok(resource) = world.resource::<PreparedWindFieldFeatureResource>() {
        let wind_policy = feature_policy(
            world,
            RenderFeatureId::new(WIND_FIELDS_RENDER_FEATURE_ID),
            resource.fallback_policy,
        );
        contributions.insert_wind_fields(resource.payload.clone(), resource.status, wind_policy);
    }

    for feature_id in execution_feature_ids {
        if contributions.feature(feature_id).is_some() {
            continue;
        }
        let fallback_policy = feature_policy(
            world,
            feature_id.clone(),
            FeatureFallbackPolicy::SkipFeaturePasses,
        );
        contributions.insert_missing(feature_id.clone(), fallback_policy);
    }

    if let Ok(feature_registry) = world.resource::<RenderFeatureRegistryResource>() {
        for feature_id in feature_registry.resolved_order() {
            if contributions.feature(feature_id).is_some() {
                continue;
            }
            let fallback_policy = feature_registry
                .descriptor(feature_id)
                .map(|descriptor| descriptor.fallback_policy)
                .unwrap_or(FeatureFallbackPolicy::SkipFeaturePasses);
            contributions.insert_missing(feature_id.clone(), fallback_policy);
        }
    }

    contributions
}

fn feature_policy(
    world: &ecs::World,
    feature_id: RenderFeatureId,
    fallback: FeatureFallbackPolicy,
) -> FeatureFallbackPolicy {
    world
        .resource::<RenderFeatureRegistryResource>()
        .ok()
        .and_then(|registry| registry.descriptor(&feature_id))
        .map(|descriptor| descriptor.fallback_policy)
        .unwrap_or(fallback)
}

fn collect_execution_feature_ids(
    compiled_flows: &[CompiledRenderFlowPlan],
) -> Vec<RenderFeatureId> {
    let mut ids = BTreeSet::<String>::new();
    for flow in compiled_flows {
        for pass in &flow.execution.passes {
            let feature_id = match pass {
                CompiledPassExecutionPlan::Compute(value) => value.feature_id.as_deref(),
                CompiledPassExecutionPlan::Fullscreen(value) => value.feature_id.as_deref(),
                CompiledPassExecutionPlan::Graphics(value) => value.feature_id.as_deref(),
                CompiledPassExecutionPlan::Copy(value) => value.feature_id.as_deref(),
                CompiledPassExecutionPlan::Present(value) => value.feature_id.as_deref(),
                CompiledPassExecutionPlan::BuiltinUiComposite(value) => {
                    Some(value.feature_id.as_str())
                }
            };
            if let Some(feature_id) = feature_id.map(str::trim).filter(|value| !value.is_empty()) {
                ids.insert(feature_id.to_string());
            }
        }
    }
    ids.into_iter().map(RenderFeatureId::new).collect()
}

type ExtractedRenderStateMap<'a> = BTreeMap<TypeId, &'a dyn Any>;

fn collect_flow_declared_state_resources<'a>(
    world: &'a ecs::World,
    compiled_flows: &[crate::plugins::render::CompiledRenderFlowPlan],
) -> ExtractedRenderStateMap<'a> {
    let mut values = ExtractedRenderStateMap::new();
    let mut type_ids = BTreeSet::<TypeId>::new();
    for flow in compiled_flows {
        for declaration in &flow.resources.state_resources {
            type_ids.insert(declaration.type_id);
        }
    }

    for type_id in type_ids {
        if let Some(resource) = world.resource_by_type_id(type_id) {
            values.insert(type_id, resource);
        }
    }

    values
}

fn build_prepared_flow_inputs(
    compiled_flows: &[CompiledRenderFlowPlan],
    extracted_state: &ExtractedRenderStateMap<'_>,
    surface_size: (u32, u32),
) -> anyhow::Result<BTreeMap<String, PreparedFlowInputs>> {
    let mut outputs = BTreeMap::<String, PreparedFlowInputs>::new();

    for flow in compiled_flows {
        let mut projected_uniform_bytes = BTreeMap::<String, Vec<u8>>::new();

        for pass in &flow.pass_order {
            for binding in &pass.node().uniform_bindings {
                if !flow.resources.has_state_resource(binding.state_type_id()) {
                    anyhow::bail!(
                        "uniform projection for flow '{}' pass '{}' requires undeclared state '{}'",
                        flow.flow_id,
                        pass.pass_id(),
                        binding.state_type_name()
                    );
                }

                if !flow.resources.has_uniform_buffer(binding.uniform_id()) {
                    anyhow::bail!(
                        "uniform projection for flow '{}' pass '{}' references unknown uniform buffer '{}'",
                        flow.flow_id,
                        pass.pass_id(),
                        binding.uniform_id().as_str()
                    );
                }

                let state = extracted_state
                    .get(&binding.state_type_id())
                    .copied()
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "uniform projection for flow '{}' pass '{}' requires missing ecs state '{}'",
                            flow.flow_id,
                            pass.pass_id(),
                            binding.state_type_name()
                        )
                    })?;

                let bytes = binding.project_bytes(state, surface_size).ok_or_else(|| {
                    anyhow::anyhow!(
                        "uniform projection for flow '{}' pass '{}' failed for state '{}'",
                        flow.flow_id,
                        pass.pass_id(),
                        binding.state_type_name()
                    )
                })?;

                let key = binding.uniform_id().as_str().to_string();
                if let Some(existing) = projected_uniform_bytes.get(&key) {
                    if existing != &bytes {
                        anyhow::bail!(
                            "uniform projection conflict for buffer '{}' in flow '{}': pass '{}' produced bytes that disagree with prior projection",
                            binding.uniform_id().as_str(),
                            flow.flow_id,
                            pass.pass_id()
                        );
                    }
                    continue;
                }
                projected_uniform_bytes.insert(key, bytes);
            }
        }

        let mut projected_dispatch_workgroups = BTreeMap::<String, [u32; 3]>::new();
        for pass in &flow.pass_order {
            if !matches!(pass.node().kind, RenderPassKind::Compute) {
                continue;
            }
            let dispatch = project_dispatch_for_pass(pass.node(), extracted_state)?;
            projected_dispatch_workgroups.insert(pass.pass_id().to_string(), dispatch);
        }

        let required_state_types = flow
            .resources
            .state_resources
            .iter()
            .map(|value| PreparedStateTypeInfo {
                type_name: value.type_name,
            })
            .collect::<Vec<_>>();

        outputs.insert(
            flow.flow_id.clone(),
            PreparedFlowInputs {
                projected_uniform_bytes,
                projected_dispatch_workgroups,
                required_state_types,
            },
        );
    }

    Ok(outputs)
}

fn project_dispatch_for_pass(
    node: &crate::plugins::render::RenderPassNode,
    extracted_state: &ExtractedRenderStateMap<'_>,
) -> anyhow::Result<[u32; 3]> {
    let dispatch = match &node.compute_dispatch {
        Some(ComputeDispatchDescriptor::Fixed(value)) => *value,
        Some(ComputeDispatchDescriptor::State(binding)) => {
            let state = extracted_state
                .get(&binding.state_type_id())
                .copied()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "compute pass '{}' dispatch_state requires missing ecs resource '{}'",
                        node.id.as_str(),
                        binding.state_type_name()
                    )
                })?;
            binding.project_dispatch(state).ok_or_else(|| {
                anyhow::anyhow!(
                    "compute pass '{}' failed to project dispatch_state for '{}'",
                    node.id.as_str(),
                    binding.state_type_name()
                )
            })?
        }
        None => {
            anyhow::bail!(
                "compute pass '{}' must declare explicit dispatch_workgroups(...) or dispatch_state(...)",
                node.id.as_str()
            );
        }
    };

    if dispatch[0] == 0 || dispatch[1] == 0 || dispatch[2] == 0 {
        anyhow::bail!(
            "compute pass '{}' resolved invalid dispatch dimensions ({}, {}, {})",
            node.id.as_str(),
            dispatch[0],
            dispatch[1],
            dispatch[2]
        );
    }

    Ok(dispatch)
}

fn evaluate_pixel_probes(
    probes: &[RenderPixelProbeRequest],
    captures: &[RenderCapturedTexture],
) -> Vec<RenderPixelProbeResult> {
    probes
        .iter()
        .map(|probe| {
            let capture = find_capture_for_selector(captures, &probe.selector);
            let capture_point_identity = capture
                .map(|value| value.identity.capture_point.clone())
                .unwrap_or_else(|| probe.selector.stable_point_fallback());
            let frame_identity = capture.map(|value| value.identity.clone());
            let comparison_mode = probe.assertion.clone();
            let mut result = RenderPixelProbeResult {
                probe_id: probe.id.clone(),
                capture_point_identity,
                frame_identity,
                sample_mode: probe.sample_mode,
                resolved_coordinate: None,
                comparison_mode,
                sampled_rgba8: None,
                compared_rgba8: None,
                status: RenderPixelProbeStatus::Skipped,
                message: None,
            };

            let Some(capture) = capture else {
                result.message = Some(RenderCaptureTerminalReason::new(
                    "capture_missing_for_probe",
                    "no completed capture matched the probe selector",
                ));
                return result;
            };
            if capture.terminal.code != RenderCaptureTerminalCode::Completed {
                result.message = Some(RenderCaptureTerminalReason::new(
                    "capture_not_completed",
                    format!(
                        "probe target capture terminal state is '{}'",
                        capture.terminal.code.as_str()
                    ),
                ));
                return result;
            }

            let Some(coordinate) = resolve_probe_coordinate(probe.sample_mode, capture) else {
                result.message = Some(RenderCaptureTerminalReason::new(
                    "invalid_probe_coordinate",
                    "probe sample coordinate is outside capture bounds",
                ));
                return result;
            };
            result.resolved_coordinate = Some(coordinate);
            let sampled = capture.sample_pixel_rgba8(coordinate);
            result.sampled_rgba8 = sampled;

            match &probe.assertion {
                RenderPixelProbeAssertionMode::None => {
                    result.status = if sampled.is_some() {
                        RenderPixelProbeStatus::Sampled
                    } else {
                        RenderPixelProbeStatus::Skipped
                    };
                    if sampled.is_none() {
                        result.message = Some(RenderCaptureTerminalReason::new(
                            "probe_sample_missing",
                            "probe target did not expose sampled rgba8 pixel",
                        ));
                    }
                }
                RenderPixelProbeAssertionMode::Exact(expected) => match sampled {
                    Some(sampled) => {
                        if sampled == *expected {
                            result.status = RenderPixelProbeStatus::Passed;
                        } else {
                            result.status = RenderPixelProbeStatus::Failed;
                            result.message = Some(RenderCaptureTerminalReason::new(
                                "probe_exact_mismatch",
                                format!(
                                    "expected {:?} but sampled {:?} at ({}, {})",
                                    expected, sampled, coordinate.x, coordinate.y
                                ),
                            ));
                        }
                    }
                    None => {
                        result.status = RenderPixelProbeStatus::Skipped;
                        result.message = Some(RenderCaptureTerminalReason::new(
                            "probe_sample_missing",
                            "probe target did not expose sampled rgba8 pixel",
                        ));
                    }
                },
                RenderPixelProbeAssertionMode::Tolerance {
                    expected,
                    tolerance,
                } => match sampled {
                    Some(sampled) => {
                        let max_distance = sampled
                            .into_iter()
                            .zip(expected.iter().copied())
                            .map(|(actual, wanted)| actual.abs_diff(wanted))
                            .max()
                            .unwrap_or(0);
                        if max_distance <= *tolerance {
                            result.status = RenderPixelProbeStatus::Passed;
                        } else {
                            result.status = RenderPixelProbeStatus::Failed;
                            result.message = Some(RenderCaptureTerminalReason::new(
                                "probe_tolerance_mismatch",
                                format!(
                                    "max channel delta {} exceeds tolerance {}",
                                    max_distance, tolerance
                                ),
                            ));
                        }
                    }
                    None => {
                        result.status = RenderPixelProbeStatus::Skipped;
                        result.message = Some(RenderCaptureTerminalReason::new(
                            "probe_sample_missing",
                            "probe target did not expose sampled rgba8 pixel",
                        ));
                    }
                },
                RenderPixelProbeAssertionMode::CompareToCapture {
                    other_selector,
                    tolerance,
                } => {
                    let Some(sampled) = sampled else {
                        result.status = RenderPixelProbeStatus::Skipped;
                        result.message = Some(RenderCaptureTerminalReason::new(
                            "probe_sample_missing",
                            "probe target did not expose sampled rgba8 pixel",
                        ));
                        return result;
                    };
                    let Some(other_capture) = find_capture_for_selector(captures, other_selector)
                    else {
                        result.status = RenderPixelProbeStatus::Skipped;
                        result.message = Some(RenderCaptureTerminalReason::new(
                            "probe_compare_capture_missing",
                            "comparison capture selector did not resolve to a completed capture",
                        ));
                        return result;
                    };
                    if other_capture.terminal.code != RenderCaptureTerminalCode::Completed {
                        result.status = RenderPixelProbeStatus::Skipped;
                        result.message = Some(RenderCaptureTerminalReason::new(
                            "probe_compare_capture_not_completed",
                            format!(
                                "comparison capture terminal state is '{}'",
                                other_capture.terminal.code.as_str()
                            ),
                        ));
                        return result;
                    }
                    let Some(other_coordinate) =
                        resolve_probe_coordinate(probe.sample_mode, other_capture)
                    else {
                        result.status = RenderPixelProbeStatus::Skipped;
                        result.message = Some(RenderCaptureTerminalReason::new(
                            "probe_compare_coordinate_invalid",
                            "comparison capture coordinate is outside capture bounds",
                        ));
                        return result;
                    };
                    let Some(other_sampled) = other_capture.sample_pixel_rgba8(other_coordinate)
                    else {
                        result.status = RenderPixelProbeStatus::Skipped;
                        result.message = Some(RenderCaptureTerminalReason::new(
                            "probe_compare_sample_missing",
                            "comparison capture did not expose sampled rgba8 pixel",
                        ));
                        return result;
                    };
                    result.compared_rgba8 = Some(other_sampled);
                    let max_distance = sampled
                        .into_iter()
                        .zip(other_sampled)
                        .map(|(left, right)| left.abs_diff(right))
                        .max()
                        .unwrap_or(0);
                    if max_distance <= *tolerance {
                        result.status = RenderPixelProbeStatus::Passed;
                    } else {
                        result.status = RenderPixelProbeStatus::Failed;
                        result.message = Some(RenderCaptureTerminalReason::new(
                            "probe_compare_mismatch",
                            format!(
                                "max channel delta {} exceeds tolerance {}",
                                max_distance, tolerance
                            ),
                        ));
                    }
                }
            }

            result
        })
        .collect()
}

fn evaluate_texture_diffs(
    diffs: &[RenderTextureDiffRequest],
    captures: &[RenderCapturedTexture],
) -> Vec<RenderTextureDiffResult> {
    diffs
        .iter()
        .map(|request| {
            let left_capture = find_capture_for_selector(captures, &request.left_selector);
            let right_capture = find_capture_for_selector(captures, &request.right_selector);
            let mut result = RenderTextureDiffResult {
                diff_id: request.id.clone(),
                request: request.clone(),
                left_capture_point: left_capture
                    .map(|value| value.identity.capture_point.clone())
                    .unwrap_or_else(|| request.left_selector.stable_point_fallback()),
                right_capture_point: right_capture
                    .map(|value| value.identity.capture_point.clone())
                    .unwrap_or_else(|| request.right_selector.stable_point_fallback()),
                left_frame_identity: left_capture.map(|value| value.identity.clone()),
                right_frame_identity: right_capture.map(|value| value.identity.clone()),
                status: RenderTextureDiffStatus::Skipped,
                metrics: None,
                mismatch_samples: Vec::new(),
                diff_image_path: None,
                message: None,
            };

            let Some(left) = left_capture else {
                result.message = Some(RenderCaptureTerminalReason::new(
                    "diff_left_capture_missing",
                    "left selector did not resolve to a completed capture",
                ));
                return result;
            };
            let Some(right) = right_capture else {
                result.message = Some(RenderCaptureTerminalReason::new(
                    "diff_right_capture_missing",
                    "right selector did not resolve to a completed capture",
                ));
                return result;
            };
            if left.terminal.code != RenderCaptureTerminalCode::Completed
                || right.terminal.code != RenderCaptureTerminalCode::Completed
            {
                result.message = Some(RenderCaptureTerminalReason::new(
                    "diff_capture_not_completed",
                    "both captures must be completed before running texture diff",
                ));
                return result;
            }
            let (Some(left_pixels), Some(right_pixels)) =
                (left.bytes_rgba8.as_ref(), right.bytes_rgba8.as_ref())
            else {
                result.message = Some(RenderCaptureTerminalReason::new(
                    "diff_pixels_missing",
                    "both captures must include rgba8 bytes before running texture diff",
                ));
                return result;
            };
            if left.width != right.width || left.height != right.height {
                result.status = RenderTextureDiffStatus::Failed;
                result.message = Some(RenderCaptureTerminalReason::new(
                    "diff_dimensions_mismatch",
                    format!(
                        "left is {}x{} but right is {}x{}",
                        left.width, left.height, right.width, right.height
                    ),
                ));
                return result;
            }

            let pixel_count = (left.width as usize).saturating_mul(left.height as usize);
            if pixel_count == 0 {
                result.status = RenderTextureDiffStatus::Compared;
                result.metrics = Some(RenderTextureDiffMetrics {
                    changed_pixel_count: 0,
                    max_delta: 0,
                    mean_delta: 0.0,
                });
                return result;
            }

            let mut changed_pixel_count = 0u64;
            let mut max_delta = 0u8;
            let mut total_delta = 0u64;
            for pixel_index in 0..pixel_count {
                let byte_index = pixel_index * 4;
                let left_rgba = [
                    left_pixels[byte_index],
                    left_pixels[byte_index + 1],
                    left_pixels[byte_index + 2],
                    left_pixels[byte_index + 3],
                ];
                let right_rgba = [
                    right_pixels[byte_index],
                    right_pixels[byte_index + 1],
                    right_pixels[byte_index + 2],
                    right_pixels[byte_index + 3],
                ];
                let pixel_delta = left_rgba
                    .into_iter()
                    .zip(right_rgba)
                    .map(|(left_value, right_value)| left_value.abs_diff(right_value))
                    .max()
                    .unwrap_or(0);
                max_delta = max_delta.max(pixel_delta);
                total_delta = total_delta.saturating_add(pixel_delta as u64);
                if pixel_delta == 0 {
                    continue;
                }
                changed_pixel_count = changed_pixel_count.saturating_add(1);
                if result.mismatch_samples.len() < request.mismatch_sample_limit {
                    let x = (pixel_index as u32) % left.width.max(1);
                    let y = (pixel_index as u32) / left.width.max(1);
                    result
                        .mismatch_samples
                        .push(RenderTextureDiffMismatchSample {
                            coordinate: RenderPixelCoordinate { x, y },
                            left_rgba8: left_rgba,
                            right_rgba8: right_rgba,
                            max_channel_delta: pixel_delta,
                        });
                }
            }
            result.status = RenderTextureDiffStatus::Compared;
            result.metrics = Some(RenderTextureDiffMetrics {
                changed_pixel_count,
                max_delta,
                mean_delta: total_delta as f32 / pixel_count as f32,
            });

            result
        })
        .collect()
}

fn find_capture_for_selector<'a>(
    captures: &'a [RenderCapturedTexture],
    selector: &RenderCaptureSelector,
) -> Option<&'a RenderCapturedTexture> {
    captures
        .iter()
        .find(|capture| selector.matches_point(&capture.identity.capture_point))
}

fn resolve_probe_coordinate(
    mode: RenderPixelSampleMode,
    capture: &RenderCapturedTexture,
) -> Option<RenderPixelCoordinate> {
    if capture.width == 0 || capture.height == 0 {
        return None;
    }
    match mode {
        RenderPixelSampleMode::Center => Some(RenderPixelCoordinate {
            x: capture.width / 2,
            y: capture.height / 2,
        }),
        RenderPixelSampleMode::Pixel(coordinate) => {
            if coordinate.x >= capture.width || coordinate.y >= capture.height {
                return None;
            }
            Some(coordinate)
        }
        RenderPixelSampleMode::Uv(uv) => {
            let clamped_u = uv.u.clamp(0.0, 1.0);
            let clamped_v = uv.v.clamp(0.0, 1.0);
            let x = ((capture.width - 1) as f32 * clamped_u).round() as u32;
            let y = ((capture.height - 1) as f32 * clamped_v).round() as u32;
            Some(RenderPixelCoordinate { x, y })
        }
    }
}

fn clamp_lines(lines: &mut Vec<String>, max_lines: usize) {
    if max_lines == 0 {
        lines.clear();
        return;
    }
    let overflow = lines.len().saturating_sub(max_lines);
    if overflow > 0 {
        lines.drain(..overflow);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::frame::{
        PreparedDeformationFeatureContribution, PreparedDeformationStream, PreparedDrawBatch,
        PreparedDrawFeatureContribution, PreparedFeaturePayload,
        PreparedMaterialFeatureContribution, PreparedMaterialInstanceInput,
    };
    use crate::plugins::render::inspect::{
        CaptureStage, CaptureTextureClass, RenderCaptureIdentity, RenderCaptureSelector,
        RenderCaptureTerminal, RenderPixelProbeAssertionMode, RenderPixelProbeRequest,
        RenderPixelSampleMode, RenderTextureDiffRequest,
    };

    fn test_world() -> ecs::World {
        let mut world = ecs::World::default();
        let mut registry = RenderFeatureRegistryResource::default();
        registry.sync_order();
        world.insert_resource(registry);
        world
    }

    #[test]
    fn frame_prepare_ingests_draw_material_deformation_feature_resources() {
        let mut world = test_world();
        world.insert_resource(PreparedDrawFeatureResource {
            status: FeatureContributionStatus::Ready,
            fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
            payload: PreparedDrawFeatureContribution {
                batches: vec![PreparedDrawBatch {
                    batch_id: "batch.0".to_string(),
                    mesh_ref: "mesh.0".to_string(),
                    material_ref: "material.0".to_string(),
                    instance_count: 2,
                }],
            },
        });
        world.insert_resource(PreparedMaterialFeatureResource {
            status: FeatureContributionStatus::Ready,
            fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
            payload: PreparedMaterialFeatureContribution {
                instances: vec![PreparedMaterialInstanceInput {
                    material_instance_id: "mat.instance".to_string(),
                    specialization_key_fragment: "opaque".to_string(),
                    parameter_blob: vec![1, 2, 3],
                }],
            },
        });
        world.insert_resource(PreparedDeformationFeatureResource {
            status: FeatureContributionStatus::Stale,
            fallback_policy: FeatureFallbackPolicy::ReuseLastGood,
            payload: PreparedDeformationFeatureContribution {
                streams: vec![PreparedDeformationStream {
                    stream_id: "skin.stream".to_string(),
                    input_pose_ref: "pose.current".to_string(),
                    output_buffer_ref: "buffer.skinning".to_string(),
                }],
            },
        });

        let contributions = build_frame_feature_contributions(
            &world,
            "world".to_string(),
            "overlay".to_string(),
            &[],
        );

        let draw = contributions
            .feature(&RenderFeatureId::new(WORLD_DRAW_RENDER_FEATURE_ID))
            .expect("draw contribution should be published");
        assert_eq!(draw.status, FeatureContributionStatus::Ready);
        assert!(matches!(draw.payload, PreparedFeaturePayload::Draw(_)));

        let material = contributions
            .feature(&RenderFeatureId::new(MATERIAL_RENDER_FEATURE_ID))
            .expect("material contribution should be published");
        assert_eq!(material.status, FeatureContributionStatus::Ready);
        assert!(matches!(
            material.payload,
            PreparedFeaturePayload::Material(_)
        ));

        let deformation = contributions
            .feature(&RenderFeatureId::new(DEFORMATION_RENDER_FEATURE_ID))
            .expect("deformation contribution should be published");
        assert_eq!(deformation.status, FeatureContributionStatus::Stale);
        assert_eq!(
            deformation.fallback_policy,
            FeatureFallbackPolicy::SkipFeaturePasses
        );
        assert!(matches!(
            deformation.payload,
            PreparedFeaturePayload::Deformation(_)
        ));
    }

    #[test]
    fn prepare_inserts_missing_gate_for_execution_referenced_feature_without_payload() {
        let world = test_world();
        let execution_feature_ids = vec![RenderFeatureId::new("custom.feature")];
        let contributions = build_frame_feature_contributions(
            &world,
            "world".to_string(),
            "overlay".to_string(),
            &execution_feature_ids,
        );

        let missing = contributions
            .feature(&RenderFeatureId::new("custom.feature"))
            .expect("execution-referenced feature should still publish gate");
        assert_eq!(missing.status, FeatureContributionStatus::Missing);
        assert_eq!(
            missing.fallback_policy,
            FeatureFallbackPolicy::SkipFeaturePasses
        );
    }

    fn test_selector(pass_id: &str) -> RenderCaptureSelector {
        RenderCaptureSelector {
            flow_id: Some("flow.main".to_string()),
            pass_id: Some(pass_id.to_string()),
            stage: CaptureStage::After,
            resource_id: "surface.color".to_string(),
            texture_class: CaptureTextureClass::ImportedTexture,
        }
    }

    fn completed_capture(
        frame_index: u64,
        selector: &RenderCaptureSelector,
        pixels: [u8; 4],
    ) -> RenderCapturedTexture {
        RenderCapturedTexture {
            identity: RenderCaptureIdentity {
                frame_index,
                pass_label: selector
                    .pass_id
                    .clone()
                    .unwrap_or_else(|| "pass".to_string()),
                capture_point: selector.stable_point_fallback(),
            },
            width: 1,
            height: 1,
            format: "Rgba8Unorm".to_string(),
            bytes_rgba8: Some(pixels.to_vec()),
            terminal: RenderCaptureTerminal::completed(),
        }
    }

    #[test]
    fn pixel_probe_results_include_identity_sampling_and_assertion_metadata() {
        let selector = test_selector("pass.viewport");
        let captures = vec![completed_capture(3, &selector, [10, 20, 30, 255])];
        let probes = vec![RenderPixelProbeRequest {
            id: "center-probe".to_string(),
            selector: selector.clone(),
            sample_mode: RenderPixelSampleMode::Center,
            assertion: RenderPixelProbeAssertionMode::Exact([10, 20, 30, 255]),
        }];

        let results = evaluate_pixel_probes(&probes, &captures);
        assert_eq!(results.len(), 1);
        let result = &results[0];
        assert_eq!(
            result.capture_point_identity,
            selector.stable_point_fallback()
        );
        assert!(result.frame_identity.is_some());
        assert_eq!(result.sample_mode, RenderPixelSampleMode::Center);
        assert_eq!(
            result.comparison_mode,
            RenderPixelProbeAssertionMode::Exact([10, 20, 30, 255])
        );
        assert_eq!(
            result.resolved_coordinate,
            Some(RenderPixelCoordinate { x: 0, y: 0 })
        );
        assert_eq!(result.status, RenderPixelProbeStatus::Passed);
    }

    #[test]
    fn texture_diffs_emit_structured_metrics_even_without_diff_image() {
        let left_selector = test_selector("pass.left");
        let right_selector = test_selector("pass.right");
        let captures = vec![
            completed_capture(9, &left_selector, [1, 2, 3, 255]),
            completed_capture(9, &right_selector, [1, 2, 9, 255]),
        ];
        let diffs = vec![RenderTextureDiffRequest::new(
            "left-vs-right",
            left_selector,
            right_selector,
        )];

        let results = evaluate_texture_diffs(&diffs, &captures);
        assert_eq!(results.len(), 1);
        let result = &results[0];
        assert_eq!(result.status, RenderTextureDiffStatus::Compared);
        assert!(result.metrics.is_some());
        let metrics = result
            .metrics
            .as_ref()
            .expect("diff metrics should be present");
        assert_eq!(metrics.changed_pixel_count, 1);
        assert_eq!(metrics.max_delta, 6);
        assert!(metrics.mean_delta > 0.0);
        assert!(result.diff_image_path.is_none());
    }
}
