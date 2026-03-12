// Owner: SDF Renderer Example - Custom Render Pass Executors
use crate::*;

pub(crate) struct SdfComputeExecutor {
    shared: Arc<Mutex<SdfGpuSharedState>>,
}

impl SdfComputeExecutor {
    pub(crate) fn new(shared: Arc<Mutex<SdfGpuSharedState>>) -> Self {
        Self { shared }
    }
}

impl RenderPassExecutor for SdfComputeExecutor {
    fn prepare(&self, ctx: &mut RenderPassPrepareContext<'_>) -> Result<()> {
        let world_frame = ctx
            .frame_data::<SdfWorldState>()
            .ok_or_else(|| anyhow!("missing SdfWorldState in render pass prepare context"))?;
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| anyhow!("sdf compute shared gpu state lock poisoned"))?;
        ensure_sdf_gpu_pass(
            &mut shared,
            ctx.device(),
            ctx.surface_format(),
            ctx.surface_size(),
            world_frame.display_render_scale,
        );
        let pass = shared
            .pass
            .as_ref()
            .ok_or_else(|| anyhow!("sdf compute pass unavailable after setup"))?;
        let agent_count = world_frame.agents.len().min(SDF_MAX_AGENTS);
        let model_count = 0usize;
        let params = SdfWorldParamsRaw {
            screen_size: [pass.size.0 as f32, pass.size.1 as f32],
            _pad0: [0.0; 2],
            world_min: [world_frame.world_bounds[0], world_frame.world_bounds[1]],
            _pad1: [0.0; 2],
            world_max: [world_frame.world_bounds[2], world_frame.world_bounds[3]],
            _pad2: [0.0; 2],
            agent_count: agent_count as u32,
            model_count: model_count as u32,
            paused: u32::from(world_frame.world_paused),
            _pad3: 0,
            camera_target_time: [
                world_frame.camera_target[0],
                world_frame.camera_target[1],
                world_frame.camera_target[2],
                world_frame.elapsed_time_seconds.max(0.0),
            ],
            camera_orbit: [
                world_frame.camera_yaw,
                world_frame.camera_pitch,
                world_frame.camera_distance.max(0.1),
                world_frame
                    .camera_fov_y
                    .clamp(0.1, std::f32::consts::PI - 0.1),
            ],
            debug_view_mode: world_frame.debug_view_mode,
            display_fit_mode: world_frame.display_fit_mode,
            display_target_aspect: world_frame.display_target_aspect,
            _pad4: 0,
        };
        ctx.queue()
            .write_buffer(&pass.params_buffer, 0, bytemuck::bytes_of(&params));

        let mut agents = Vec::with_capacity(agent_count);
        for agent in world_frame.agents.iter().take(agent_count) {
            agents.push(SdfWorldAgentRaw {
                pos: [agent.x, agent.y],
                radius: agent.radius.max(0.2),
                health: agent.health_ratio.clamp(0.0, 1.0),
                team: agent.team,
                _pad0: [0; 3],
            });
        }
        if !agents.is_empty() {
            ctx.queue()
                .write_buffer(&pass.agents_buffer, 0, bytemuck::cast_slice(&agents));
        }

        Ok(())
    }

    fn encode(&self, ctx: &mut RenderPassEncodeContext<'_>) -> Result<()> {
        let render_scale = ctx
            .frame_data::<SdfWorldState>()
            .map(|world| world.display_render_scale)
            .unwrap_or(1.0);
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| anyhow!("sdf compute shared gpu state lock poisoned"))?;
        ensure_sdf_gpu_pass(
            &mut shared,
            ctx.device(),
            ctx.surface_format(),
            ctx.surface_size(),
            render_scale,
        );
        let pass = shared
            .pass
            .as_ref()
            .ok_or_else(|| anyhow!("sdf compute pass unavailable during encode"))?;
        let mut compute = ctx.encoder().begin_compute_pass(&ComputePassDescriptor {
            label: Some("sdf_example.compute"),
            timestamp_writes: None,
        });
        compute.set_pipeline(&pass.compute_pipeline);
        compute.set_bind_group(0, &pass.compute_bind_group, &[]);
        compute.dispatch_workgroups(pass.size.0.div_ceil(8), pass.size.1.div_ceil(8), 1);
        Ok(())
    }
}

pub(crate) struct SdfComposeExecutor {
    shared: Arc<Mutex<SdfGpuSharedState>>,
}

impl SdfComposeExecutor {
    pub(crate) fn new(shared: Arc<Mutex<SdfGpuSharedState>>) -> Self {
        Self { shared }
    }
}

impl RenderPassExecutor for SdfComposeExecutor {
    fn prepare(&self, ctx: &mut RenderPassPrepareContext<'_>) -> Result<()> {
        let (fit_mode, target_aspect, render_scale, bar_color) = ctx
            .frame_data::<SdfWorldState>()
            .map(|world| {
                (
                    world.display_fit_mode,
                    world.display_target_aspect,
                    world.display_render_scale,
                    world.display_bar_color,
                )
            })
            .unwrap_or((
                0,
                0.0,
                1.0,
                [
                    SDF_CLEAR_COLOR.r as f32,
                    SDF_CLEAR_COLOR.g as f32,
                    SDF_CLEAR_COLOR.b as f32,
                    1.0,
                ],
            ));
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| anyhow!("sdf compose shared gpu state lock poisoned"))?;
        ensure_sdf_gpu_pass(
            &mut shared,
            ctx.device(),
            ctx.surface_format(),
            ctx.surface_size(),
            render_scale,
        );
        let pass = shared
            .pass
            .as_ref()
            .ok_or_else(|| anyhow!("sdf compose pass unavailable during prepare"))?;

        let compose_params = SdfComposeParamsRaw {
            output_size: [
                ctx.surface_size().0.max(1) as f32,
                ctx.surface_size().1.max(1) as f32,
            ],
            target_aspect,
            fit_mode,
            bar_color,
        };
        ctx.queue().write_buffer(
            &pass.compose_params_buffer,
            0,
            bytemuck::bytes_of(&compose_params),
        );
        Ok(())
    }

    fn encode(&self, ctx: &mut RenderPassEncodeContext<'_>) -> Result<()> {
        let render_scale = ctx
            .frame_data::<SdfWorldState>()
            .map(|world| world.display_render_scale)
            .unwrap_or(1.0);
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| anyhow!("sdf compose shared gpu state lock poisoned"))?;
        ensure_sdf_gpu_pass(
            &mut shared,
            ctx.device(),
            ctx.surface_format(),
            ctx.surface_size(),
            render_scale,
        );
        let pass = shared
            .pass
            .as_ref()
            .ok_or_else(|| anyhow!("sdf compose pass unavailable during encode"))?;
        let frame_view = ctx.frame_view();
        let mut compose = ctx.encoder().begin_render_pass(&RenderPassDescriptor {
            label: Some("sdf_example.compose"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: frame_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(SDF_CLEAR_COLOR),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        compose.set_pipeline(&pass.compose_pipeline);
        compose.set_bind_group(0, &pass.compose_bind_group, &[]);
        compose.draw(0..3, 0..1);
        Ok(())
    }
}
