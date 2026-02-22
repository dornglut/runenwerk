use crate::render::{MAX_WORLD_RENDER_AGENTS, WorldRenderAgent};
use crate::runtime::EngineData;
use crate::runtime::{AgentHealth, AgentPosition, AgentPrevPosition, AgentTeam};
use wgpu::SurfaceError;

const FRAME_TIMING_LOG_THRESHOLD_MS: f32 = 20.0;
const MESH_HOT_PATH_LOG_THRESHOLD_MS: f32 = 8.0;

pub fn world_render_extract_system(data: &mut EngineData) -> anyhow::Result<()> {
    let frame = &mut data.world_render;
    frame.agents.clear();
    frame.model_proxies.clear();
    frame.world_scene_label = data.scene.world.active.label().to_string();
    frame.overlay_scene_label = data.scene.active_overlay().label().to_string();
    frame.world_paused = data.scene.world.paused;
    frame.camera_yaw = data.scene.world_runtime.ctx.camera_yaw;
    frame.camera_pitch = data.scene.world_runtime.ctx.camera_pitch;
    frame.camera_distance = data.scene.world_runtime.ctx.camera_distance;
    frame.camera_pitch_min = data
        .scene
        .world_runtime
        .ctx
        .gameplay_config
        .camera
        .pitch_min;
    frame.camera_pitch_max = data
        .scene
        .world_runtime
        .ctx
        .gameplay_config
        .camera
        .pitch_max;
    frame.camera_distance_min = data
        .scene
        .world_runtime
        .ctx
        .gameplay_config
        .camera
        .distance_min;
    frame.camera_distance_max = data
        .scene
        .world_runtime
        .ctx
        .gameplay_config
        .camera
        .distance_max;
    frame.camera_follow_dampening = data
        .scene
        .world_runtime
        .ctx
        .gameplay_config
        .camera
        .follow_dampening;
    frame.chunk_size = data.scene.world_runtime.ctx.gameplay_config.chunk_size;
    frame.chunk_load_radius = data
        .scene
        .world_runtime
        .ctx
        .gameplay_config
        .chunk_load_radius;
    frame.infinite_world = data.scene.world_runtime.ctx.gameplay_config.infinite_world;

    let bounds = &data.scene.world_runtime.ctx.gameplay_config.bounds;
    frame.world_bounds = [bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y];

    if !data.scene.world.visible {
        return Ok(());
    }

    let world = &data.scene.world_runtime.ctx.world;
    let fixed_dt = data
        .scene
        .world_runtime
        .ctx
        .fixed_step_seconds
        .max(1.0 / 240.0);
    let interp_alpha =
        (data.scene.world_runtime.ctx.fixed_step_accumulator / fixed_dt).clamp(0.0, 1.0);
    let entities: Vec<_> = world.entities_with::<AgentPosition>().collect();
    for entity in entities.into_iter().take(MAX_WORLD_RENDER_AGENTS) {
        let Some(position) = world.get_component::<AgentPosition>(entity).copied() else {
            continue;
        };
        let prev = world
            .get_component::<AgentPrevPosition>(entity)
            .copied()
            .unwrap_or(AgentPrevPosition {
                x: position.x,
                y: position.y,
            });
        let render_x = prev.x + (position.x - prev.x) * interp_alpha;
        let render_y = prev.y + (position.y - prev.y) * interp_alpha;
        let health = world
            .get_component::<AgentHealth>(entity)
            .copied()
            .unwrap_or(AgentHealth { current: 1, max: 1 });
        let team = world
            .get_component::<AgentTeam>(entity)
            .copied()
            .unwrap_or(AgentTeam::Enemy);
        let health_ratio = if health.max > 0 {
            (health.current as f32 / health.max as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let team_id = match team {
            AgentTeam::Player => 0,
            AgentTeam::Enemy => 1,
        };
        frame.agents.push(WorldRenderAgent {
            x: render_x,
            y: render_y,
            radius: 0.95,
            health_ratio,
            team: team_id,
        });
    }

    if data.scene.world_runtime.ctx.gameplay_config.infinite_world {
        let center = frame
            .agents
            .iter()
            .find(|agent| agent.team == 0)
            .or_else(|| frame.agents.first())
            .map(|a| (a.x, a.y))
            .unwrap_or((0.0, 0.0));
        let chunk = data
            .scene
            .world_runtime
            .ctx
            .gameplay_config
            .chunk_size
            .max(4.0);
        let radius_chunks = data
            .scene
            .world_runtime
            .ctx
            .gameplay_config
            .chunk_load_radius
            .max(2) as f32;
        let extent = chunk * (radius_chunks + 2.0);
        frame.world_bounds = [
            center.0 - extent,
            center.1 - extent,
            center.0 + extent,
            center.1 + extent,
        ];
    }

    Ok(())
}

pub fn ui_render_submit_system(data: &mut EngineData) -> anyhow::Result<()> {
    let _submit_span = tracing::info_span!("systems.ui_render_submit").entered();
    let shader_reload_messages = data.gfx.poll_shader_hot_reload();
    if !shader_reload_messages.is_empty() {
        for msg in shader_reload_messages {
            data.scene
                .overlay_runtime
                .ui
                .log_lines
                .push(format!("[world] shader {msg}"));
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
                .push(format!("[world] model {msg}"));
        }
        clamp_lines(
            &mut data.scene.overlay_runtime.ui.log_lines,
            data.scene.overlay_runtime.ui.max_lines,
        );
        data.scene.overlay_runtime.ui.log_scroll_lines_from_bottom = 0;
    }

    match data
        .gfx
        .render(&data.world_render, &data.scene.overlay_runtime.ui.draw_list)
    {
        Ok(timings) => {
            let mesh_hot = timings.renderer.mesh_hot_path;
            let total_ms = timings.acquire_ms
                + timings.renderer.prepare_ui_ms
                + timings.renderer.prepare_mesh_ms
                + timings.renderer.world_prepare_ms
                + timings.renderer.encode_submit_ms
                + timings.present_ms;
            if total_ms > FRAME_TIMING_LOG_THRESHOLD_MS {
                tracing::info!(
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
            if timings.renderer.prepare_mesh_ms > MESH_HOT_PATH_LOG_THRESHOLD_MS {
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
