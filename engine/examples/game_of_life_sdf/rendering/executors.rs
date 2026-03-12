// Owner: Game of Life SDF Example - Custom Render Pass Executors
use crate::rendering::{
    DEFAULT_CLEAR_COLOR, GameOfLifeGpuSharedState, WORKGROUP_SIZE, ensure_game_of_life_gpu_pass,
};
use crate::runtime::GameOfLifeSdfState;
use anyhow::{Result, anyhow};
use engine::plugins::render::domain::{
    RenderPassEncodeContext, RenderPassExecutor, RenderPassPrepareContext,
};
use engine::plugins::render::GpuParams;
use std::sync::{Arc, Mutex};
use wgpu::{
    Color, ComputePassDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp,
};

pub(crate) struct GameOfLifeComputeExecutor {
    shared: Arc<Mutex<GameOfLifeGpuSharedState>>,
}

impl GameOfLifeComputeExecutor {
    pub(crate) fn new(shared: Arc<Mutex<GameOfLifeGpuSharedState>>) -> Self {
        Self { shared }
    }
}

impl RenderPassExecutor for GameOfLifeComputeExecutor {
    fn prepare(&self, ctx: &mut RenderPassPrepareContext<'_>) -> Result<()> {
        let state = ctx
            .frame_data::<GameOfLifeSdfState>()
            .ok_or_else(|| anyhow!("missing GameOfLifeSdfState during compute prepare"))?;

        let mut shared = self
            .shared
            .lock()
            .map_err(|_| anyhow!("game of life gpu shared state lock poisoned"))?;
        ensure_game_of_life_gpu_pass(
            &mut shared,
            ctx.device(),
            ctx.surface_format(),
            ctx.surface_size(),
            state,
        );
        let pass = shared
            .pass
            .as_ref()
            .ok_or_else(|| anyhow!("game of life gpu pass unavailable during compute prepare"))?;

        let params = state.compute_params().to_gpu();
        ctx.queue()
            .write_buffer(&pass.params_buffer, 0, bytemuck::bytes_of(&params));
        Ok(())
    }

    fn encode(&self, ctx: &mut RenderPassEncodeContext<'_>) -> Result<()> {
        let state = ctx
            .frame_data::<GameOfLifeSdfState>()
            .ok_or_else(|| anyhow!("missing GameOfLifeSdfState during compute encode"))?;
        let should_advance = state.step_simulation;

        let mut shared = self
            .shared
            .lock()
            .map_err(|_| anyhow!("game of life gpu shared state lock poisoned"))?;
        ensure_game_of_life_gpu_pass(
            &mut shared,
            ctx.device(),
            ctx.surface_format(),
            ctx.surface_size(),
            state,
        );
        let pass = shared
            .pass
            .as_mut()
            .ok_or_else(|| anyhow!("game of life gpu pass unavailable during compute encode"))?;

        let bind_group_index = pass.phase & 1;
        {
            let mut compute = ctx.encoder().begin_compute_pass(&ComputePassDescriptor {
                label: Some("game_of_life_sdf.compute"),
                timestamp_writes: None,
            });
            compute.set_pipeline(&pass.compute_pipeline);
            compute.set_bind_group(0, &pass.compute_bind_groups[bind_group_index], &[]);
            compute.dispatch_workgroups(
                pass.grid_size.0.div_ceil(WORKGROUP_SIZE),
                pass.grid_size.1.div_ceil(WORKGROUP_SIZE),
                1,
            );
        }

        if should_advance {
            pass.phase = (pass.phase + 1) & 1;
        }
        Ok(())
    }
}

pub(crate) struct GameOfLifeComposeExecutor {
    shared: Arc<Mutex<GameOfLifeGpuSharedState>>,
}

impl GameOfLifeComposeExecutor {
    pub(crate) fn new(shared: Arc<Mutex<GameOfLifeGpuSharedState>>) -> Self {
        Self { shared }
    }
}

impl RenderPassExecutor for GameOfLifeComposeExecutor {
    fn prepare(&self, ctx: &mut RenderPassPrepareContext<'_>) -> Result<()> {
        let state = ctx
            .frame_data::<GameOfLifeSdfState>()
            .ok_or_else(|| anyhow!("missing GameOfLifeSdfState during compose prepare"))?;

        let mut shared = self
            .shared
            .lock()
            .map_err(|_| anyhow!("game of life gpu shared state lock poisoned"))?;
        ensure_game_of_life_gpu_pass(
            &mut shared,
            ctx.device(),
            ctx.surface_format(),
            ctx.surface_size(),
            state,
        );
        let pass = shared
            .pass
            .as_ref()
            .ok_or_else(|| anyhow!("game of life gpu pass unavailable during compose prepare"))?;

        let compose_params = state.compose_params(pass.surface_size).to_gpu();
        ctx.queue().write_buffer(
            &pass.compose_params_buffer,
            0,
            bytemuck::bytes_of(&compose_params),
        );
        Ok(())
    }

    fn encode(&self, ctx: &mut RenderPassEncodeContext<'_>) -> Result<()> {
        let state = ctx
            .frame_data::<GameOfLifeSdfState>()
            .ok_or_else(|| anyhow!("missing GameOfLifeSdfState during compose encode"))?;

        let mut shared = self
            .shared
            .lock()
            .map_err(|_| anyhow!("game of life gpu shared state lock poisoned"))?;
        ensure_game_of_life_gpu_pass(
            &mut shared,
            ctx.device(),
            ctx.surface_format(),
            ctx.surface_size(),
            state,
        );
        let pass = shared
            .pass
            .as_ref()
            .ok_or_else(|| anyhow!("game of life gpu pass unavailable during compose encode"))?;

        let clear = background_clear_color(Some(state));
        let frame_view = ctx.frame_view();
        let mut compose = ctx.encoder().begin_render_pass(&RenderPassDescriptor {
            label: Some("game_of_life_sdf.compose"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: frame_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(clear),
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

fn background_clear_color(state: Option<&GameOfLifeSdfState>) -> Color {
    let color = state.map(|state| state.background_color).unwrap_or([
        DEFAULT_CLEAR_COLOR.r as f32,
        DEFAULT_CLEAR_COLOR.g as f32,
        DEFAULT_CLEAR_COLOR.b as f32,
        DEFAULT_CLEAR_COLOR.a as f32,
    ]);
    Color {
        r: color[0] as f64,
        g: color[1] as f64,
        b: color[2] as f64,
        a: color[3] as f64,
    }
}
