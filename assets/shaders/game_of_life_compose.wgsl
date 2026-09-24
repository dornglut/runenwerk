struct Cell {
    alive: u32,
};

struct ComposeParams {
    grid_size: vec2<u32>,
    surface_size: vec2<f32>,
    alive_mix: f32,
    current_is_a: u32,
};

@group(0) @binding(0)
var<uniform> params: ComposeParams;

@group(0) @binding(1)
var<storage, read> cells_a: array<Cell>;

@group(0) @binding(2)
var<storage, read> cells_b: array<Cell>;

struct VsOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

fn linear_index(x: u32, y: u32, width: u32) -> u32 {
    return y * width + x;
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );

    let p = pos[vertex_index];
    var out: VsOut;
    out.clip_position = vec4<f32>(p, 0.0, 1.0);
    out.uv = vec2<f32>((p.x + 1.0) * 0.5, 1.0 - (p.y + 1.0) * 0.5);
    return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let width = max(params.grid_size.x, 1u);
    let height = max(params.grid_size.y, 1u);
    let grid_size_f = vec2<f32>(f32(width), f32(height));

    let grid_uv = input.uv * grid_size_f;
    let cell_x = min(u32(floor(grid_uv.x)), width - 1u);
    let cell_y = min(u32(floor(grid_uv.y)), height - 1u);
    let index = linear_index(cell_x, cell_y, width);
    let local = fract(grid_uv) - vec2<f32>(0.5, 0.5);

    let use_a = params.current_is_a != 0u;
    let alive_u32 = min(select(cells_b[index].alive, cells_a[index].alive, use_a), 1u);
    let alive = f32(alive_u32) * clamp(params.alive_mix, 0.0, 1.0);

    let radius = 0.36;
    let edge_softness = 0.028;
    let cell_sdf = length(local) - radius;
    let alive_mask = 1.0 - smoothstep(0.0, edge_softness, cell_sdf);
    let glow_mask = 1.0 - smoothstep(0.0, edge_softness * 3.5, cell_sdf);

    let alive_color = vec3<f32>(0.22, 0.90, 1.00);
    let dead_color = vec3<f32>(0.045, 0.055, 0.070);
    let background = vec3<f32>(0.010, 0.012, 0.016);
    let grid_color = vec3<f32>(0.14, 0.16, 0.20);

    let edge_distance = max(abs(local.x), abs(local.y));
    let grid_mask = smoothstep(0.47, 0.50, edge_distance);

    var color = mix(background, dead_color, 0.82);
    color = mix(color, alive_color, alive * alive_mask);
    color = color + alive_color * alive * glow_mask * 0.22;
    color = mix(color, grid_color, grid_mask * 0.48);

    return vec4<f32>(color, 1.0);
}
