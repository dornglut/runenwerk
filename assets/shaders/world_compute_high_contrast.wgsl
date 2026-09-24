struct WorldParams {
    screen_size : vec2<f32>,
    _pad0 : vec2<f32>,
    world_min : vec2<f32>,
    _pad1 : vec2<f32>,
    world_max : vec2<f32>,
    _pad2 : vec2<f32>,
    agent_count : u32,
    model_count : u32,
    paused : u32,
    _pad3 : u32,
};

struct Agent {
    pos : vec2<f32>,
    radius : f32,
    health : f32,
    team : u32,
    _pad0 : vec3<u32>,
};

struct ModelProxy {
    pos : vec2<f32>,
    radius : f32,
    _pad0 : f32,
    color : vec4<f32>,
};

@group(0) @binding(0)
var output_tex : texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<uniform> params : WorldParams;

@group(0) @binding(2)
var<storage, read> agents : array<Agent>;
@group(0) @binding(3)
var<storage, read> models : array<ModelProxy>;

fn team_color(team: u32) -> vec3<f32> {
    if (team == 0u) {
        return vec3<f32>(0.15, 0.88, 1.00);
    }
    return vec3<f32>(1.00, 0.30, 0.16);
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) gid : vec3<u32>) {
    let x = gid.x;
    let y = gid.y;
    if (x >= u32(params.screen_size.x) || y >= u32(params.screen_size.y)) {
        return;
    }

    let uv = vec2<f32>(
        f32(x) / max(params.screen_size.x - 1.0, 1.0),
        f32(y) / max(params.screen_size.y - 1.0, 1.0)
    );
    let world_span = max(params.world_max - params.world_min, vec2<f32>(0.001, 0.001));
    let world_pos = params.world_min + vec2<f32>(uv.x * world_span.x, (1.0 - uv.y) * world_span.y);

    var color = vec3<f32>(0.01, 0.01, 0.01);
    let grid_x = abs(fract((world_pos.x - params.world_min.x) / 3.0) - 0.5);
    let grid_y = abs(fract((world_pos.y - params.world_min.y) / 3.0) - 0.5);
    if (grid_x < 0.018 || grid_y < 0.018) {
        color += vec3<f32>(0.10, 0.10, 0.10);
    }

    var idx: u32 = 0u;
    loop {
        if (idx >= params.agent_count) {
            break;
        }
        let agent = agents[idx];
        let delta = world_pos - agent.pos;
        let dist = length(delta);
        if (dist <= agent.radius) {
            let edge = smoothstep(agent.radius, agent.radius * 0.55, dist);
            let hp = clamp(agent.health, 0.0, 1.0);
            let tint = team_color(agent.team) * (0.35 + hp * 0.65);
            color = mix(color, tint, edge);
        }
        idx = idx + 1u;
    }

    var model_idx: u32 = 0u;
    loop {
        if (model_idx >= params.model_count) {
            break;
        }
        let model = models[model_idx];
        let delta = world_pos - model.pos;
        let dist = length(delta);
        let sdf = dist - model.radius;
        let band = smoothstep(0.42, -0.12, sdf);
        color = mix(color, model.color.xyz, band * model.color.w);
        model_idx = model_idx + 1u;
    }

    if (params.paused == 1u) {
        color = mix(color, vec3<f32>(0.96, 0.80, 0.32), 0.22);
    }

    textureStore(output_tex, vec2<i32>(i32(x), i32(y)), vec4<f32>(color, 1.0));
}
