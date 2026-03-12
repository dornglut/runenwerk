struct SimParams {
    grid_size : vec2<u32>,
    step : u32,
    _pad0 : u32,
};

struct ComposeParams {
    output_size : vec2<f32>,
    grid_size : vec2<f32>,
    cell_radius : f32,
    edge_softness : f32,
    grid_line_width : f32,
    glow_strength : f32,
    alive_color : vec4<f32>,
    dead_color : vec4<f32>,
    grid_color : vec4<f32>,
    background_color : vec4<f32>,
};

@group(0) @binding(0)
var<uniform> sim : SimParams;
@group(0) @binding(1)
var<storage, read> cells_in : array<u32>;
@group(0) @binding(2)
var<storage, read_write> cells_out : array<u32>;
@group(0) @binding(3)
var cells_texture : texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(4)
var cells_sampled_texture : texture_2d<f32>;
@group(0) @binding(5)
var cells_sampler : sampler;
@group(0) @binding(6)
var<uniform> compose : ComposeParams;

fn wrap_coord(value: i32, limit: i32) -> u32 {
    return u32((value + limit) % limit);
}

fn cell_index(x: u32, y: u32, width: u32) -> u32 {
    return y * width + x;
}

fn cell_value(x: i32, y: i32) -> u32 {
    let width = max(sim.grid_size.x, 1u);
    let height = max(sim.grid_size.y, 1u);
    let wrapped_x = wrap_coord(x, i32(width));
    let wrapped_y = wrap_coord(y, i32(height));
    return cells_in[cell_index(wrapped_x, wrapped_y, width)] & 1u;
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) gid : vec3<u32>) {
    let width = sim.grid_size.x;
    let height = sim.grid_size.y;
    if (gid.x >= width || gid.y >= height) {
        return;
    }

    let x = i32(gid.x);
    let y = i32(gid.y);
    let alive = cell_value(x, y);

    var neighbors : u32 = 0u;
    for (var dy : i32 = -1; dy <= 1; dy = dy + 1) {
        for (var dx : i32 = -1; dx <= 1; dx = dx + 1) {
            if (dx == 0 && dy == 0) {
                continue;
            }
            neighbors = neighbors + cell_value(x + dx, y + dy);
        }
    }

    var next = alive;
    if (sim.step == 1u) {
        if (alive == 1u && (neighbors == 2u || neighbors == 3u)) {
            next = 1u;
        } else if (alive == 0u && neighbors == 3u) {
            next = 1u;
        } else {
            next = 0u;
        }
    }

    let idx = cell_index(gid.x, gid.y, width);
    cells_out[idx] = next;

    let value = f32(next);
    textureStore(
        cells_texture,
        vec2<i32>(i32(gid.x), i32(gid.y)),
        vec4<f32>(value, value, value, 1.0)
    );
}

struct VsOut {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) uv : vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index : u32) -> VsOut {
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );
    let p = pos[vertex_index];
    var out : VsOut;
    out.clip_position = vec4<f32>(p, 0.0, 1.0);
    out.uv = vec2<f32>((p.x + 1.0) * 0.5, 1.0 - (p.y + 1.0) * 0.5);
    return out;
}

@fragment
fn fs_main(input : VsOut) -> @location(0) vec4<f32> {
    let grid = max(compose.grid_size, vec2<f32>(1.0, 1.0));
    let cell_space = input.uv * grid;
    let cell_id = floor(cell_space);
    let local = fract(cell_space) - vec2<f32>(0.5, 0.5);

    let sample_uv = (cell_id + vec2<f32>(0.5, 0.5)) / grid;
    let alive = textureSampleLevel(cells_sampled_texture, cells_sampler, sample_uv, 0.0).r;

    let radius = clamp(compose.cell_radius, 0.05, 0.49);
    let edge = max(compose.edge_softness, 0.0001);
    let cell_sdf = length(local) - radius;
    let alive_mask = 1.0 - smoothstep(0.0, edge, cell_sdf);
    let glow_mask = 1.0 - smoothstep(0.0, edge * 3.0, cell_sdf);

    let line_distance = min(abs(local.x), abs(local.y));
    let line_width = clamp(compose.grid_line_width, 0.0, 0.2);
    let grid_mask = 1.0 - smoothstep(line_width, line_width + edge, line_distance);

    var color = compose.background_color.rgb;
    color = mix(color, compose.dead_color.rgb, (1.0 - alive) * 0.5);
    color = mix(color, compose.alive_color.rgb, alive * alive_mask);
    color = color + compose.alive_color.rgb * alive * glow_mask * compose.glow_strength;
    color = mix(color, compose.grid_color.rgb, grid_mask * compose.grid_color.a);

    return vec4<f32>(color, 1.0);
}
