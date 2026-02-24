pub mod domain;

use crate::runtime::{EngineData, EnginePlugin, EngineScheduleBuilder};
use anyhow::Result;
use scheduler::set_slow_node_logging_enabled;
use wgpu::SurfaceError;

pub struct RenderPlugin;

impl EnginePlugin for RenderPlugin {
    fn name(&self) -> &'static str {
        "render"
    }

    fn configure(&self, builder: &mut EngineScheduleBuilder) -> Result<()> {
        builder.add_node_with_edges(
            "frame_render_submit",
            ui_render_submit_system,
            &["overlay_ui_render_extract"],
        );
        Ok(())
    }
}

const FRAME_TIMING_LOG_THRESHOLD_MS: f32 = 20.0;
const MESH_HOT_PATH_LOG_THRESHOLD_MS: f32 = 8.0;

pub fn ui_render_submit_system(data: &mut EngineData) -> anyhow::Result<()> {
    let _submit_span = tracing::info_span!("systems.ui_render_submit").entered();
    let startup_ready_before = data.startup.is_ready();
    // Keep scheduler slow-node logs muted while startup warmup is still in loading.
    set_slow_node_logging_enabled(startup_ready_before);

    let shader_reload_messages = data.gfx.poll_shader_hot_reload();
    if !shader_reload_messages.is_empty() {
        for msg in shader_reload_messages {
            data.scene
                .overlay_runtime
                .ui
                .log_lines
                .push(format!("[world] {msg}"));
        }
        clamp_lines(
            &mut data.scene.overlay_runtime.ui.log_lines,
            data.scene.overlay_runtime.ui.max_lines,
        );
        data.scene.overlay_runtime.ui.log_scroll_lines_from_bottom = 0;
    }
    let model_reload_messages = data.gfx.poll_model_hot_reload();
    if !model_reload_messages.is_empty() {
        for msg in model_reload_messages {
            data.scene
                .overlay_runtime
                .ui
                .log_lines
                .push(format!("[world] {msg}"));
        }
        clamp_lines(
            &mut data.scene.overlay_runtime.ui.log_lines,
            data.scene.overlay_runtime.ui.max_lines,
        );
        data.scene.overlay_runtime.ui.log_scroll_lines_from_bottom = 0;
    }

    match data.gfx.render(
        &data.world_render,
        &data.scene.overlay_runtime.ui.draw_list,
        &data.render_graph_registry,
    ) {
        Ok(timings) => {
            data.debug_metrics.last_timings = Some(timings);
            let mesh_hot = timings.renderer.mesh_hot_path;
            let warm_frame = mesh_hot.is_warm_frame();
            let warmup_completed = data
                .startup
                .observe_render_warm_frame(warm_frame, data.time.delta_seconds);
            if warmup_completed {
                tracing::info!(
                    elapsed_loading_seconds = data.startup.elapsed_loading_seconds,
                    stable_frames = data.startup.stable_frames,
                    required_stable_frames = data.startup.required_stable_frames,
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
            // "Workload" excludes swapchain acquire/present waiting (vsync/compositor pacing).
            let workload_ms = timings.renderer.prepare_ui_ms
                + timings.renderer.prepare_mesh_ms
                + timings.renderer.world_prepare_ms
                + timings.renderer.encode_submit_ms;
            if startup_ready_before && workload_ms > FRAME_TIMING_LOG_THRESHOLD_MS {
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
        Err(SurfaceError::Lost | SurfaceError::Outdated) => {
            let (w, h) = data.scene.overlay_runtime.ui.screen_size;
            data.gfx.resize(w as u32, h as u32);
            Ok(())
        }
        Err(SurfaceError::Timeout) => Ok(()),
        Err(SurfaceError::OutOfMemory) => anyhow::bail!("surface out of memory"),
        Err(SurfaceError::Other) => Ok(()),
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
