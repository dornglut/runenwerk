struct Cell {
    alive: u32,
};

@group(0) @binding(0)
var<storage, read_write> cells_a: array<Cell>;

@group(0) @binding(1)
var<storage, read_write> cells_b: array<Cell>;

const WIDTH: u32 = 160u;
const HEIGHT: u32 = 90u;
const SEED: u32 = 0xC0FFEE11u;

fn linear_index(x: u32, y: u32) -> u32 {
    return y * WIDTH + x;
}

fn hash_cell(x: u32, y: u32) -> u32 {
    var h = x * 1664525u + y * 1013904223u + SEED * 747796405u + 2891336453u;
    h = (h ^ (h >> 16u)) * 2246822519u;
    h = (h ^ (h >> 13u)) * 3266489917u;
    h = h ^ (h >> 16u);
    return h;
}

fn seeded_alive(x: u32, y: u32) -> u32 {
    let threshold = 0x002AAAAAu;
    let noise = select(0u, 1u, (hash_cell(x, y) & 0x00FFFFFFu) < threshold);
    let local_x = x % 24u;
    let local_y = y % 24u;
    let blinker = local_y == 12u && local_x >= 10u && local_x <= 12u;
    return select(noise, 1u, blinker);
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= WIDTH || gid.y >= HEIGHT) {
        return;
    }

    let index = linear_index(gid.x, gid.y);
    let alive = seeded_alive(gid.x, gid.y);
    cells_a[index].alive = alive;
    cells_b[index].alive = alive;
}
