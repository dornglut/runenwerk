pub mod domain;

use self::domain::{
    Gfx, RenderFrameDataRegistry, RenderFrameResourceBindings, RenderGraphRegistryResource,
    RenderPassExecutorRegistryResource, ShaderHandle, ShaderRegistryResource,
};
use crate::app::App;
use crate::plugin::Plugin;
use crate::plugins::scene::SceneResource;
use crate::plugins::time::domain::Time;
use crate::plugins::ui::domain::UiRenderShaderConfig;
use crate::runtime::{RenderPrepare, RenderSubmit, Res, ResMut, WorldMut};
use crate::state::{DebugMetricsState, StartupState};
use scheduler::set_slow_node_logging_enabled;
use wgpu::SurfaceError;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneResource>();
        app.init_resource::<RenderFrameResourceBindings>();
        app.init_resource::<ShaderRegistryResource>();
        app.init_resource::<RenderGraphRegistryResource>();
        app.init_resource::<RenderPassExecutorRegistryResource>();
        app.init_resource::<StartupState>();
        app.init_resource::<DebugMetricsState>();
        app.add_systems(RenderPrepare, frame_render_prepare_system);
        app.add_systems(RenderSubmit, ui_render_submit_system);
    }
}

const FRAME_TIMING_LOG_THRESHOLD_MS: f32 = 20.0;
const MESH_HOT_PATH_LOG_THRESHOLD_MS: f32 = 8.0;

fn frame_render_prepare_system() -> anyhow::Result<()> {
    Ok(())
}

fn ui_render_submit_system(
    mut world: WorldMut,
    time: Res<Time>,
    mut scene_resource: ResMut<SceneResource>,
    mut startup: ResMut<StartupState>,
    mut debug_metrics: ResMut<DebugMetricsState>,
) -> anyhow::Result<()> {
    let Some(manager) = scene_resource.manager.as_mut() else {
        return Ok(());
    };

    let _submit_span = tracing::info_span!("systems.ui_render_submit").entered();
    let startup_ready_before = startup.is_ready();
    set_slow_node_logging_enabled(startup_ready_before);

    let Some(mut shader_registry) = world.remove_resource::<ShaderRegistryResource>() else {
        return Ok(());
    };
    let _ = shader_registry.poll_updates();
    let shader_reload_messages = shader_registry.drain_event_lines();
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

    let ui_rect_shader_id = manager
        .overlay_runtime
        .world
        .resource::<UiRenderShaderConfig>()
        .ok()
        .map(|config| config.rect_shader_asset_id.trim().to_string())
        .filter(|id| !id.is_empty());

    let Some(mut gfx) = world.remove_resource::<Gfx>() else {
        world.insert_resource(shader_registry);
        return Ok(());
    };

    let render_frame_bindings = match world.resource::<RenderFrameResourceBindings>() {
        Ok(bindings) => bindings,
        Err(_) => {
            world.insert_resource(shader_registry);
            world.insert_resource(gfx);
            return Ok(());
        }
    };

    let mut frame_data = RenderFrameDataRegistry::new();
    render_frame_bindings.collect_frame_data(&world, &mut frame_data);

    let render_result = {
        let render_graph_registry = match world.resource::<RenderGraphRegistryResource>() {
            Ok(registry) => registry,
            Err(_) => {
                world.insert_resource(shader_registry);
                world.insert_resource(gfx);
                return Ok(());
            }
        };
        let render_executor_registry = match world.resource::<RenderPassExecutorRegistryResource>()
        {
            Ok(registry) => registry,
            Err(_) => {
                world.insert_resource(shader_registry);
                world.insert_resource(gfx);
                return Ok(());
            }
        };
        let ui_rect_shader: Option<ShaderHandle> =
            ui_rect_shader_id.and_then(|id| shader_registry.handle(id));

        gfx.render(
            &frame_data,
            &manager.overlay_runtime.ui.draw_list,
            &mut shader_registry,
            &render_graph_registry,
            &render_executor_registry,
            ui_rect_shader,
        )
    };

    let result = match render_result {
        Ok(timings) => {
            debug_metrics.last_timings = Some(timings);
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
            let (w, h) = manager.overlay_runtime.ui.screen_size;
            gfx.resize(w as u32, h as u32);
            Ok(())
        }
        Err(SurfaceError::Timeout) => Ok(()),
        Err(SurfaceError::OutOfMemory) => anyhow::bail!("surface out of memory"),
        Err(SurfaceError::Other) => Ok(()),
    };

    world.insert_resource(shader_registry);
    world.insert_resource(gfx);
    result
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
