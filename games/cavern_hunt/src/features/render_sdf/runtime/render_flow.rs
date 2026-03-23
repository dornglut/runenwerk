use super::*;
use engine::plugins::render::{GpuParams, GpuUniform, RenderFlow};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CavernSdfFullscreenUniform {
    params: CavernWorldParamsRaw,
    primitives: [CavernGeometryPrimitiveRaw; MAX_GEOMETRY_PRIMITIVES],
    agents: [CavernAgentRaw; MAX_AGENTS],
}

impl GpuParams for CavernSdfFullscreenUniform {
    type Raw = Self;

    fn to_gpu(&self) -> Self::Raw {
        *self
    }
}

impl GpuUniform for CavernSdfFullscreenUniform {}

pub fn build_cavern_render_flow() -> RenderFlow {
    RenderFlow::new("cavern_hunt.sdf.fullscreen")
        .with_state::<CavernSdfWorldFrame>()
        .with_surface_color()
        .fullscreen_pass("cavern_hunt.sdf.compose")
        .shader_asset("assets/shaders/cavern_hunt_sdf_fullscreen.wgsl")
        .uniform_from_state_with_surface(cavern_world_uniform_from_frame)
        .write_surface_color()
        .finish()
        .builtin_ui_composite_pass("cavern_hunt.sdf.ui")
        .depends_on("cavern_hunt.sdf.compose")
        .finish()
        .validate()
        .expect("cavern_hunt.sdf.fullscreen flow should validate")
}

fn cavern_world_uniform_from_frame(
    frame: &CavernSdfWorldFrame,
    surface: (u32, u32),
) -> CavernSdfFullscreenUniform {
    let mut primitives = [CavernGeometryPrimitiveRaw::zeroed(); MAX_GEOMETRY_PRIMITIVES];
    let mut agents = [CavernAgentRaw::zeroed(); MAX_AGENTS];

    let primitive_count = frame.geometry_primitives.len().min(MAX_GEOMETRY_PRIMITIVES);
    for (dst, src) in primitives
        .iter_mut()
        .zip(frame.geometry_primitives.iter().take(primitive_count))
    {
        *dst = CavernGeometryPrimitiveRaw {
            shape_kind: src.shape_kind,
            op_kind: src.op_kind,
            material_class: src.material_class,
            material_instance: src.material_instance,
            p0: src.p0,
            p1: src.p1,
            p2: src.p2,
        };
    }

    let agent_count = frame.agents.len().min(MAX_AGENTS);
    for (dst, src) in agents.iter_mut().zip(frame.agents.iter().take(agent_count)) {
        *dst = CavernAgentRaw {
            pos: src.pos,
            radius: src.radius,
            health: src.health_ratio,
            team: src.team,
            kind: src.kind,
            _pad0: [0; 2],
        };
    }

    let roof_clip_y = frame.camera.target[1] + 1.6;
    let params = CavernWorldParamsRaw {
        screen_size: [surface.0.max(1) as f32, surface.1.max(1) as f32],
        _pad0: [0.0; 2],
        world_min: [frame.world_bounds[0], frame.world_bounds[1]],
        _pad1: [0.0; 2],
        world_max: [frame.world_bounds[2], frame.world_bounds[3]],
        _pad2: [0.0; 2],
        primitive_count: primitive_count as u32,
        agent_count: agent_count as u32,
        material_program_count: 0,
        material_op_count: 0,
        material_constant_count: 0,
        render_mode: frame.render_mode,
        gi_mode: frame.gi_mode,
        gi_quality: frame.gi_quality,
        gi_sample_budget: frame.gi_sample_budget.max(1),
        _pad3: [0; 3],
        floor_rock_height: [frame.floor_height, frame.rock_height, roof_clip_y, 0.0],
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

    CavernSdfFullscreenUniform {
        params,
        primitives,
        agents,
    }
}
