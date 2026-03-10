use super::*;

// Owner: Cavern Hunt SDF Renderer - Render Executor Runtime
fn ensure_gpu_pass(
    shared: &mut CavernGpuSharedState,
    device: &Device,
    surface_format: TextureFormat,
    surface_size: (u32, u32),
) {
    let size = (surface_size.0.max(1), surface_size.1.max(1));
    let needs_rebuild = shared
        .pass
        .as_ref()
        .is_none_or(|pass| pass.surface_format != surface_format || pass.size != size);
    if needs_rebuild {
        shared.pass = Some(build_gpu_pass(device, surface_format, size));
    }
}

pub(crate) struct CavernComputeExecutor {
    shared: Arc<Mutex<CavernGpuSharedState>>,
}

impl CavernComputeExecutor {
    pub(crate) fn new(shared: Arc<Mutex<CavernGpuSharedState>>) -> Self {
        Self { shared }
    }
}

impl RenderPassExecutor for CavernComputeExecutor {
    fn prepare(&self, ctx: &mut RenderPassPrepareContext<'_>) -> Result<()> {
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| anyhow!("cavern gpu shared state lock poisoned"))?;
        ensure_gpu_pass(
            &mut shared,
            ctx.device(),
            ctx.surface_format(),
            ctx.surface_size(),
        );
        let pass = shared
            .pass
            .as_ref()
            .ok_or_else(|| anyhow!("cavern gpu pass unavailable after setup"))?;

        let frame = ctx
            .frame_data::<CavernSdfWorldFrame>()
            .ok_or_else(|| anyhow!("missing CavernSdfWorldFrame in render prepare context"))?;
        let params = CavernWorldParamsRaw {
            screen_size: [pass.size.0 as f32, pass.size.1 as f32],
            _pad0: [0.0; 2],
            world_min: [frame.world_bounds[0], frame.world_bounds[1]],
            _pad1: [0.0; 2],
            world_max: [frame.world_bounds[2], frame.world_bounds[3]],
            _pad2: [0.0; 2],
            primitive_count: frame.geometry_primitives.len().min(MAX_GEOMETRY_PRIMITIVES) as u32,
            agent_count: frame.agents.len().min(MAX_AGENTS) as u32,
            material_program_count: frame
                .material_program_headers
                .len()
                .min(MAX_MATERIAL_PROGRAMS) as u32,
            material_op_count: frame.material_ops.len().min(MAX_MATERIAL_OPS) as u32,
            material_constant_count: frame.material_constants.len().min(MAX_MATERIAL_CONSTANTS)
                as u32,
            render_mode: frame.render_mode,
            gi_mode: frame.gi_mode,
            gi_quality: frame.gi_quality,
            gi_sample_budget: frame.gi_sample_budget.max(1),
            _pad3: [0; 3],
            floor_rock_height: [frame.floor_height, frame.rock_height, 0.0, 0.0],
            camera_target_time: [
                frame.camera.target[0],
                frame.camera.target[1],
                frame.camera.target[2],
                0.0,
            ],
            camera_orbit: [
                frame.camera.yaw,
                frame.camera.pitch,
                frame.camera.distance,
                frame.camera.fov_y_radians,
            ],
        };
        let roof_clip_y = frame.camera.target[1] + 1.6;
        let params = CavernWorldParamsRaw {
            floor_rock_height: [frame.floor_height, frame.rock_height, roof_clip_y, 0.0],
            ..params
        };
        ctx.queue()
            .write_buffer(&pass.params_buffer, 0, bytemuck::bytes_of(&params));

        let primitives = frame
            .geometry_primitives
            .iter()
            .take(MAX_GEOMETRY_PRIMITIVES)
            .map(|primitive| CavernGeometryPrimitiveRaw {
                shape_kind: primitive.shape_kind,
                op_kind: primitive.op_kind,
                material_class: primitive.material_class,
                material_instance: primitive.material_instance,
                p0: primitive.p0,
                p1: primitive.p1,
                p2: primitive.p2,
            })
            .collect::<Vec<_>>();
        if !primitives.is_empty() {
            ctx.queue().write_buffer(
                &pass.primitives_buffer,
                0,
                bytemuck::cast_slice(&primitives),
            );
        }

        let agents = frame
            .agents
            .iter()
            .take(MAX_AGENTS)
            .map(|agent| CavernAgentRaw {
                pos: agent.pos,
                radius: agent.radius,
                health: agent.health_ratio,
                team: agent.team,
                kind: agent.kind,
                _pad0: [0; 2],
            })
            .collect::<Vec<_>>();
        if !agents.is_empty() {
            ctx.queue()
                .write_buffer(&pass.agents_buffer, 0, bytemuck::cast_slice(&agents));
        }

        let program_headers = frame
            .material_program_headers
            .iter()
            .take(MAX_MATERIAL_PROGRAMS)
            .map(|header| CavernMaterialProgramHeaderRaw {
                class_id: header.class_id,
                op_offset: header.op_offset,
                op_count: header.op_count,
                const_offset: header.const_offset,
                const_count: header.const_count,
                base_color_slot: header.base_color_slot,
                roughness_slot: header.roughness_slot,
                metallic_slot: header.metallic_slot,
                normal_perturb_slot: header.normal_perturb_slot,
                ao_slot: header.ao_slot,
                emissive_slot: header.emissive_slot,
                _pad0: [0; 3],
            })
            .collect::<Vec<_>>();
        if !program_headers.is_empty() {
            ctx.queue().write_buffer(
                &pass.material_program_headers_buffer,
                0,
                bytemuck::cast_slice(&program_headers),
            );
        }

        let material_ops = frame
            .material_ops
            .iter()
            .take(MAX_MATERIAL_OPS)
            .map(|op| CavernMaterialOpRaw {
                op: op.op,
                dst: op.dst,
                src_a: op.src_a,
                src_b: op.src_b,
                src_c: op.src_c,
                const_idx: op.const_idx,
                flags: op.flags,
                _pad0: 0,
            })
            .collect::<Vec<_>>();
        if !material_ops.is_empty() {
            ctx.queue().write_buffer(
                &pass.material_ops_buffer,
                0,
                bytemuck::cast_slice(&material_ops),
            );
        }

        let material_constants = frame
            .material_constants
            .iter()
            .take(MAX_MATERIAL_CONSTANTS)
            .copied()
            .collect::<Vec<_>>();
        if !material_constants.is_empty() {
            ctx.queue().write_buffer(
                &pass.material_constants_buffer,
                0,
                bytemuck::cast_slice(&material_constants),
            );
        }

        Ok(())
    }

    fn encode(&self, ctx: &mut RenderPassEncodeContext<'_>) -> Result<()> {
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| anyhow!("cavern gpu shared state lock poisoned"))?;
        ensure_gpu_pass(
            &mut shared,
            ctx.device(),
            ctx.surface_format(),
            ctx.surface_size(),
        );
        let pass = shared
            .pass
            .as_ref()
            .ok_or_else(|| anyhow!("cavern gpu pass unavailable during encode"))?;
        let mut compute = ctx.encoder().begin_compute_pass(&ComputePassDescriptor {
            label: Some("cavern_hunt.compute"),
            timestamp_writes: None,
        });
        compute.set_pipeline(&pass.compute_pipeline);
        compute.set_bind_group(0, &pass.compute_bind_group, &[]);
        compute.dispatch_workgroups(pass.size.0.div_ceil(8), pass.size.1.div_ceil(8), 1);
        Ok(())
    }
}

pub(crate) struct CavernComposeExecutor {
    shared: Arc<Mutex<CavernGpuSharedState>>,
}

impl CavernComposeExecutor {
    pub(crate) fn new(shared: Arc<Mutex<CavernGpuSharedState>>) -> Self {
        Self { shared }
    }
}

impl RenderPassExecutor for CavernComposeExecutor {
    fn encode(&self, ctx: &mut RenderPassEncodeContext<'_>) -> Result<()> {
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| anyhow!("cavern gpu shared state lock poisoned"))?;
        ensure_gpu_pass(
            &mut shared,
            ctx.device(),
            ctx.surface_format(),
            ctx.surface_size(),
        );
        let pass = shared
            .pass
            .as_ref()
            .ok_or_else(|| anyhow!("cavern gpu pass unavailable during encode"))?;
        let frame_view = ctx.frame_view();
        let mut render = ctx.encoder().begin_render_pass(&RenderPassDescriptor {
            label: Some("cavern_hunt.compose"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: frame_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(CLEAR_COLOR),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render.set_pipeline(&pass.compose_pipeline);
        render.set_bind_group(0, &pass.compose_bind_group, &[]);
        render.draw(0..3, 0..1);
        Ok(())
    }
}

