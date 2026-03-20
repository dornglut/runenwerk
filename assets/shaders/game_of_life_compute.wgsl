struct Cell {
    alive: u32,
};

struct ComputeParams {
    tick: u32,
    seed: u32,
    step: u32,
    read_from_a: u32,
    grid_size: vec2<u32>,
};

@group(0) @binding(0)
var<uniform> params: ComputeParams;

@group(0) @binding(1)
var<storage, read_write> cells_a: array<Cell>;

@group(0) @binding(2)
var<storage, read_write> cells_b: array<Cell>;

fn linear_index(x: u32, y: u32, width: u32) -> u32 {
    return y * width + x;
}

fn wrap_signed(value: i32, limit: u32) -> u32 {
    let limit_i = i32(limit);
    let m = value % limit_i;
    return u32(select(m + limit_i, m, m >= 0));
}

fn hash_cell(x: u32, y: u32, seed: u32) -> u32 {
    var h = x * 1664525u + y * 1013904223u + seed * 747796405u + 2891336453u;
    h = (h ^ (h >> 16u)) * 2246822519u;
    h = (h ^ (h >> 13u)) * 3266489917u;
    h = h ^ (h >> 16u);
    return h;
}

fn seeded_alive(x: u32, y: u32, seed: u32) -> u32 {
    let threshold = 0x002aaaaau;
    let noise = select(0u, 1u, (hash_cell(x, y, seed) & 0x00ffffffu) < threshold);
    let local_x = x % 24u;
    let local_y = y % 24u;
    let blinker = local_y == 12u && local_x >= 10u && local_x <= 12u;
    return select(noise, 1u, blinker);
}

fn read_alive(index: u32, read_from_a: bool) -> u32 {
    return select(cells_b[index].alive, cells_a[index].alive, read_from_a);
}

fn write_alive(index: u32, read_from_a: bool, value: u32) {
    if (read_from_a) {
        cells_b[index].alive = value;
    } else {
        cells_a[index].alive = value;
    }
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let width = max(params.grid_size.x, 1u);
    let height = max(params.grid_size.y, 1u);

    if (gid.x >= width || gid.y >= height) {
        return;
    }

    let x = gid.x;
    let y = gid.y;
    let index = linear_index(x, y, width);
    let read_from_a = params.read_from_a != 0u;

    if (params.tick == 0u) {
        let seeded = seeded_alive(x, y, params.seed);
        cells_a[index].alive = seeded;
        cells_b[index].alive = seeded;
        return;
    }

    let current = read_alive(index, read_from_a);

    if (params.step == 0u) {
        write_alive(index, read_from_a, current);
        return;
    }

    var neighbors: u32 = 0u;
    for (var dy: i32 = -1; dy <= 1; dy = dy + 1) {
        for (var dx: i32 = -1; dx <= 1; dx = dx + 1) {
            if (dx == 0 && dy == 0) {
                continue;
            }

            let nx = wrap_signed(i32(x) + dx, width);
            let ny = wrap_signed(i32(y) + dy, height);
            let neighbor_index = linear_index(nx, ny, width);
            neighbors = neighbors + read_alive(neighbor_index, read_from_a);
        }
    }

    var next_alive = 0u;
    if (current == 1u && (neighbors == 2u || neighbors == 3u)) {
        next_alive = 1u;
    } else if (current == 0u && neighbors == 3u) {
        next_alive = 1u;
    }

    write_alive(index, read_from_a, next_alive);
}
