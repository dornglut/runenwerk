const GRID_SIZE : vec2<i32> = vec2<i32>(160, 90);

fn wrap_coord(value: i32, limit: i32) -> i32 {
    let m = value % limit;
    return select(m + limit, m, m >= 0);
}

fn hash_cell(cell: vec2<i32>) -> f32 {
    var x = u32(cell.x) * 1664525u + u32(cell.y) * 1013904223u + 747796405u;
    x = (x ^ (x >> 16u)) * 2246822519u;
    x = (x ^ (x >> 13u)) * 3266489917u;
    x = x ^ (x >> 16u);
    return f32(x & 0x00ffffffu) / 16777215.0;
}

fn seeded_alive(cell: vec2<i32>) -> u32 {
    return select(0u, 1u, hash_cell(cell) > 0.62);
}

fn next_alive(cell: vec2<i32>) -> u32 {
    let center = seeded_alive(cell);
    var neighbors : u32 = 0u;

    for (var dy : i32 = -1; dy <= 1; dy = dy + 1) {
        for (var dx : i32 = -1; dx <= 1; dx = dx + 1) {
            if (dx == 0 && dy == 0) {
                continue;
            }

            let nx = wrap_coord(cell.x + dx, GRID_SIZE.x);
            let ny = wrap_coord(cell.y + dy, GRID_SIZE.y);
            neighbors = neighbors + seeded_alive(vec2<i32>(nx, ny));
        }
    }

    if (center == 1u && (neighbors == 2u || neighbors == 3u)) {
        return 1u;
    }
    if (center == 0u && neighbors == 3u) {
        return 1u;
    }
    return 0u;
}

@compute @workgroup_size(8, 8, 1)
fn cs_main() {}

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
    let grid_size_f = vec2<f32>(f32(GRID_SIZE.x), f32(GRID_SIZE.y));
    let grid_uv = input.uv * grid_size_f;
    let cell = vec2<i32>(i32(floor(grid_uv.x)), i32(floor(grid_uv.y)));
    let local = fract(grid_uv) - vec2<f32>(0.5, 0.5);

    let alive = f32(next_alive(cell));

    let radius = 0.34;
    let edge_softness = 0.03;
    let cell_sdf = length(local) - radius;
    let alive_mask = 1.0 - smoothstep(0.0, edge_softness, cell_sdf);
    let glow_mask = 1.0 - smoothstep(0.0, edge_softness * 3.5, cell_sdf);

    let alive_color = vec3<f32>(0.24, 0.92, 0.64);
    let dead_color = vec3<f32>(0.10, 0.15, 0.13);
    let background = vec3<f32>(0.03, 0.045, 0.042);
    let grid_color = vec3<f32>(0.17, 0.25, 0.23);

    let edge_distance = max(abs(local.x), abs(local.y));
    let grid_mask = smoothstep(0.47, 0.50, edge_distance);

    var color = mix(background, dead_color, 0.55);
    color = mix(color, alive_color, alive * alive_mask);
    color = color + alive_color * alive * glow_mask * 0.28;
    color = mix(color, grid_color, grid_mask * 0.38);

    return vec4<f32>(color, 1.0);
}
