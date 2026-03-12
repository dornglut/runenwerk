// Owner: SDF Renderer Example - Custom Render Pass Executors
use crate::*;
use engine::plugins::render::GpuParams;

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
        let params = world_frame.compute_params_with_surface(pass.size).to_gpu();
        ctx.queue()
            .write_buffer(&pass.params_buffer, 0, bytemuck::bytes_of(&params));

        let agents: Vec<<crate::rendering::SdfWorldAgent as GpuParams>::Raw> = world_frame
            .agent_params()
            .into_iter()
            .map(|agent| agent.to_gpu())
            .collect();
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
        let (compose_params, render_scale) = ctx
            .frame_data::<SdfWorldState>()
            .map(|world| {
                (
                    world.compose_params(ctx.surface_size()),
                    world.display_render_scale,
                )
            })
            .unwrap_or((
                crate::rendering::SdfComposeParams {
                    output_size: [
                        ctx.surface_size().0.max(1) as f32,
                        ctx.surface_size().1.max(1) as f32,
                    ],
                    target_aspect: 0.0,
                    fit_mode: 0,
                    bar_color: [
                        SDF_CLEAR_COLOR.r as f32,
                        SDF_CLEAR_COLOR.g as f32,
                        SDF_CLEAR_COLOR.b as f32,
                        1.0,
                    ],
                },
                1.0,
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
        let compose_params = compose_params.to_gpu();
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
