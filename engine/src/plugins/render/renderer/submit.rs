use crate::plugins::render::api::project_uniform_bindings_for_pass;
use crate::plugins::render::backend::{BackendPipelineCacheResource, BackendPipelineCacheStats};
use crate::plugins::render::composition::RenderFlowRegistryResource;
use crate::plugins::render::frame_packet::{
    PreparedFlowInputs, PreparedRenderFrame, PreparedRenderFrameResource, PreparedSceneInfo,
    PreparedShaderSnapshot, PreparedStateTypeInfo, PreparedSurfaceInfo, PreparedUiInput,
};
use crate::plugins::render::inspect::{
    RenderDebugTimingsState, RenderRuntimeResourceInspectorState,
};
use crate::plugins::render::pipelines::{PipelineCacheResource, PipelineCacheStats};
use crate::plugins::render::renderer::Gfx;
use crate::plugins::render::renderer::frame_bindings::RenderFrameDataRegistry;
use crate::plugins::render::shader::{ShaderHandle, ShaderRegistryResource};
use crate::plugins::render::{CompiledRenderFlowPlan, ParamProjectionError};
use crate::plugins::scene::SceneResource;
use crate::plugins::time::domain::Time;
use crate::plugins::ui::domain::UiRenderShaderConfig;
use crate::runtime::{Res, ResMut, WorldMut};
use crate::state::{DebugMetricsState, StartupState};
use anyhow::anyhow;
use scheduler::set_slow_node_logging_enabled;
use std::any::TypeId;
use std::collections::{BTreeMap, BTreeSet};
use wgpu::SurfaceError;

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

    let target_size = {
        let (window_w, window_h) = manager.overlay_runtime.ui.screen_size;
        (
            (window_w.max(1.0)).round() as u32,
            (window_h.max(1.0)).round() as u32,
        )
    };

    let scene = PreparedSceneInfo {
        world_scene_label: manager.world.active.label().to_string(),
        overlay_scene_label: manager.active_overlay().label().to_string(),
    };
    let ui = PreparedUiInput::RawDrawList(manager.overlay_runtime.ui.draw_list.clone());

    let (flow_registry_revision, compiled_flows, flows) = {
        let flow_registry = match world.resource::<RenderFlowRegistryResource>() {
            Ok(registry) => registry,
            Err(_) => {
                world.insert_resource(shader_registry);
                clear_prepared_frame(&mut world);
                return Ok(());
            }
        };
        let compiled_flows = flow_registry.compiled_flows();
        let mut frame_data = RenderFrameDataRegistry::new();
        collect_flow_declared_frame_resources(&world, compiled_flows, &mut frame_data);
        let flows = build_prepared_flow_inputs(compiled_flows, &frame_data, target_size)?;
        (flow_registry.revision(), compiled_flows.len(), flows)
    };

    let frame_index = {
        if let Ok(mut prepared_resource) = world.resource_mut::<PreparedRenderFrameResource>() {
            prepared_resource.allocate_frame_index()
        } else {
            0
        }
    };

    let prepared = PreparedRenderFrame {
        frame_index,
        flow_registry_revision,
        surface: PreparedSurfaceInfo {
            target_size_px: target_size,
        },
        scene,
        ui,
        flows,
        shader: PreparedShaderSnapshot {
            registry_revision: shader_registry.revision(),
        },
        ui_rect_shader_id,
    };

    if let Ok(mut prepared_resource) = world.resource_mut::<PreparedRenderFrameResource>() {
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

pub(crate) fn ui_render_submit_system(
    mut world: WorldMut,
    time: Res<Time>,
    mut scene_resource: ResMut<SceneResource>,
    mut startup: ResMut<StartupState>,
    mut debug_metrics: ResMut<DebugMetricsState>,
) -> anyhow::Result<()> {
    if scene_resource.manager.is_none() {
        return Ok(());
    }

    let _submit_span = tracing::info_span!("systems.ui_render_submit").entered();
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

    let (target_w, target_h) = prepared_frame.surface.target_size_px;
    if gfx.ctx.surface_config.width != target_w || gfx.ctx.surface_config.height != target_h {
        gfx.resize(target_w, target_h);
    }

    let render_result = {
        let flow_registry = match world.resource::<RenderFlowRegistryResource>() {
            Ok(registry) => registry,
            Err(_) => {
                world.insert_resource(shader_registry);
                world.insert_resource(gfx);
                return Ok(());
            }
        };
        if flow_registry.revision() != prepared_frame.flow_registry_revision {
            world.insert_resource(shader_registry);
            world.insert_resource(gfx);
            return Ok(());
        }
        let compiled_flows = flow_registry.compiled_flows();

        let ui_rect_shader: Option<ShaderHandle> = prepared_frame
            .ui_rect_shader_id
            .as_ref()
            .and_then(|id| shader_registry.handle(id));

        gfx.render(
            &prepared_frame,
            &mut shader_registry,
            compiled_flows,
            ui_rect_shader,
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
            if let Ok(mut cache_resource) = world.resource_mut::<PipelineCacheResource>() {
                cache_resource.observe_stats(PipelineCacheStats {
                    hits: cache_stats.hits,
                    misses: cache_stats.misses,
                });
            }
            if let Ok(mut backend_cache_resource) =
                world.resource_mut::<BackendPipelineCacheResource>()
            {
                backend_cache_resource.observe_stats(BackendPipelineCacheStats {
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
    if let Ok(mut prepared_resource) = world.resource_mut::<PreparedRenderFrameResource>() {
        prepared_resource.clear();
    }
}

fn build_prepared_flow_inputs(
    compiled_flows: &[CompiledRenderFlowPlan],
    frame_data: &RenderFrameDataRegistry<'_>,
    surface_size: (u32, u32),
) -> anyhow::Result<BTreeMap<String, PreparedFlowInputs>> {
    let mut outputs = BTreeMap::<String, PreparedFlowInputs>::new();

    for flow in compiled_flows {
        let mut projected_uniform_bytes = BTreeMap::<String, Vec<u8>>::new();

        for pass in &flow.pass_order {
            let pass_id = pass.pass_id().to_string();
            let projected = project_uniform_bindings_for_pass(
                pass.node(),
                &flow.resources,
                frame_data,
                surface_size,
            )
            .map_err(|errors: Vec<ParamProjectionError>| {
                anyhow::anyhow!(
                    "uniform projection failed for flow '{}': {}",
                    flow.flow_id,
                    errors
                        .into_iter()
                        .map(|err| err.to_string())
                        .collect::<Vec<_>>()
                        .join("; ")
                )
            })?;

            for buffer in projected {
                let key = buffer.buffer_id.as_str().to_string();
                if let Some(existing) = projected_uniform_bytes.get(&key) {
                    if existing != &buffer.bytes {
                        anyhow::bail!(
                            "uniform projection conflict for buffer '{}' in flow '{}': pass '{}' wrote different bytes than a prior pass",
                            buffer.buffer_id.as_str(),
                            flow.flow_id,
                            pass_id
                        );
                    }
                    continue;
                }
                projected_uniform_bytes.insert(key, buffer.bytes);
            }
        }

        let mut projected_dispatch_workgroups = BTreeMap::<String, [u32; 3]>::new();
        for pass in &flow.pass_order {
            if !matches!(
                pass.node().kind,
                crate::plugins::render::RenderPassKind::Compute
            ) {
                continue;
            }
            let dispatch = project_dispatch_for_pass(pass.node(), frame_data)?;
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
    frame_data: &RenderFrameDataRegistry<'_>,
) -> anyhow::Result<[u32; 3]> {
    let dispatch = match &node.compute_dispatch {
        Some(crate::plugins::render::ComputeDispatchDescriptor::Fixed(value)) => *value,
        Some(crate::plugins::render::ComputeDispatchDescriptor::State(binding)) => {
            let state = frame_data
                .get_by_type_id(binding.state_type_id())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "compute pass '{}' dispatch_state requires missing ECS resource '{}'",
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

fn collect_flow_declared_frame_resources<'a>(
    world: &'a ecs::World,
    compiled_flows: &[crate::plugins::render::CompiledRenderFlowPlan],
    frame_data: &mut RenderFrameDataRegistry<'a>,
) {
    let mut type_ids = BTreeSet::<TypeId>::new();
    for flow in compiled_flows {
        for declaration in &flow.resources.state_resources {
            type_ids.insert(declaration.type_id);
        }
    }

    for type_id in type_ids {
        if let Some(resource) = world.resource_by_type_id(type_id) {
            frame_data.insert_by_type_id(type_id, resource);
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
