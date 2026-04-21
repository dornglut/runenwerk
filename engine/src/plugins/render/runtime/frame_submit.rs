use crate::plugins::inspect::{
	export_captured_textures, submit_render_frame_report_to_diagnostics,
	validate_selector_terminal_invariant, RenderCaptureTerminal, RenderCaptureTerminalCode,
	RenderCaptureTerminalReason, RenderCapturedTextureState, RenderDebugConfigResource,
	RenderDebugControlResource, RenderDebugFrameReport, RenderDebugFrameReportState,
	RenderDebugTimingsState, RenderPassProvenanceState, RenderRuntimeResourceInspectorState,
	RenderTextureInspectorState,
};
use crate::plugins::pipelines::{PipelineCacheResource, PipelineCacheStats};
use crate::plugins::render::runtime::debug_eval::{evaluate_pixel_probes, evaluate_texture_diffs};
use crate::plugins::render::*;
use crate::plugins::time::domain::Time;
use crate::runtime::{Res, ResMut, WorldMut};
use crate::state::{DebugMetricsState, StartupState};
use anyhow::anyhow;
use scheduler::set_slow_node_logging_enabled;
use wgpu::SurfaceError;
use crate::plugins::SceneResource;

const FRAME_TIMING_LOG_THRESHOLD_MS: f32 = 20.0;
const MESH_HOT_PATH_LOG_THRESHOLD_MS: f32 = 8.0;

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
		.cloned()
		.unwrap_or_default();
	let viewport_surface_bindings = world
		.resource::<ViewportSurfaceBindingRegistryResource>()
		.ok()
		.map(|resource| resource.registry().clone())
		.unwrap_or_default();
	let debug_control = world
		.resource::<RenderDebugControlResource>()
		.ok()
		.cloned()
		.unwrap_or_default();
	let debug_config = world
		.resource::<RenderDebugConfigResource>()
		.ok()
		.cloned()
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
			&viewport_surface_bindings,
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
				frame_report.errors.extend(violations.into_iter().map(|value| {
					format!(
						"selector invariant violation at index {}: {}",
						value.selector_index, value.message
					)
				}));
			}

			if let Err(err) = submit_render_frame_report_to_diagnostics(&mut world, &frame_report) {
				tracing::warn!(
                    frame = frame_report.frame_index,
                    error = %err,
                    "failed submitting render diagnostics report to canonical diagnostics core"
                );
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