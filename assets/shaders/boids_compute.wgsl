struct BoidAgent {
    position: vec2<f32>,
    velocity: vec2<f32>,
};

struct ComputeParams {
    frame_meta: vec4<u32>, // tick, step, read_from_a, boid_count
    sim_a: vec4<f32>, // delta_seconds, max_speed, max_force, neighbor_radius
    sim_b: vec4<f32>, // separation_radius, alignment_weight, cohesion_weight, separation_weight
    sim_c: vec4<f32>, // center_weight, jitter_strength, simulation_fps, padding
};

@group(0) @binding(0)
var<uniform> params: ComputeParams;

@group(0) @binding(1)
var<storage, read_write> boids_a: array<BoidAgent>;

@group(0) @binding(2)
var<storage, read_write> boids_b: array<BoidAgent>;

fn hash_u32(value: u32) -> u32 {
    var x = value * 747796405u + 2891336453u;
    x = ((x >> ((x >> 28u) + 4u)) ^ x) * 277803737u;
    x = (x >> 22u) ^ x;
    return x;
}

fn rand01(seed: u32) -> f32 {
    return f32(hash_u32(seed) & 0x00ffffffu) / 16777215.0;
}

fn wrap_delta(delta: vec2<f32>) -> vec2<f32> {
    return delta - round(delta);
}

fn limit_magnitude(value: vec2<f32>, max_value: f32) -> vec2<f32> {
    let len = length(value);
    if (len <= max_value || len == 0.0) {
        return value;
    }
    return value * (max_value / len);
}

fn normalize_or_zero(value: vec2<f32>) -> vec2<f32> {
    let len = length(value);
    if (len <= 0.000001) {
        return vec2<f32>(0.0, 0.0);
    }
    return value / len;
}

fn read_boid(index: u32, read_from_a: bool) -> BoidAgent {
    if (read_from_a) {
        return boids_a[index];
    }
    return boids_b[index];
}

fn write_boid(index: u32, read_from_a: bool, boid: BoidAgent) {
    if (read_from_a) {
        boids_b[index] = boid;
    } else {
        boids_a[index] = boid;
    }
}

fn seed_boid(index: u32, boid_count: u32) -> BoidAgent {
    let i = f32(index);
    let count = max(f32(boid_count), 1.0);
    let golden = 2.3999632;
    let angle = i * golden;
    let radius = 0.15 + 0.34 * sqrt((i + 0.5) / count);
    let center = vec2<f32>(0.5, 0.5);
    let jitter = vec2<f32>(
        (rand01(index * 17u + 3u) - 0.5) * 0.03,
        (rand01(index * 29u + 7u) - 0.5) * 0.03
    );
    let position = fract(center + vec2<f32>(cos(angle), sin(angle)) * radius + jitter);
    let speed = 0.17 + 0.19 * rand01(index * 41u + 11u);
    let heading = angle + 1.5707964;
    let velocity = vec2<f32>(cos(heading), sin(heading)) * speed;
    return BoidAgent(position, velocity);
}

@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let index = gid.x;
    let boid_count = max(params.frame_meta.w, 1u);
    if (index >= boid_count) {
        return;
    }

    let tick = params.frame_meta.x;
    let step_enabled = params.frame_meta.y != 0u;
    let read_from_a = params.frame_meta.z != 0u;

    if (tick == 0u) {
        let seeded = seed_boid(index, boid_count);
        boids_a[index] = seeded;
        boids_b[index] = seeded;
        return;
    }

    let delta_seconds = clamp(params.sim_a.x, 0.0001, 0.05);
    let max_speed = max(params.sim_a.y, 0.02);
    let max_force = max(params.sim_a.z, 0.05);
    let neighbor_radius = max(params.sim_a.w, 0.02);
    let separation_radius = max(params.sim_b.x, 0.005);
    let alignment_weight = params.sim_b.y;
    let cohesion_weight = params.sim_b.z;
    let separation_weight = params.sim_b.w;
    let center_weight = params.sim_c.x;
    let jitter_strength = params.sim_c.y;

    var boid = read_boid(index, read_from_a);
    if (!step_enabled) {
        write_boid(index, read_from_a, boid);
        return;
    }

    var alignment = vec2<f32>(0.0, 0.0);
    var cohesion = vec2<f32>(0.0, 0.0);
    var separation = vec2<f32>(0.0, 0.0);
    var neighbor_count: u32 = 0u;
    var separation_count: u32 = 0u;

    for (var i: u32 = 0u; i < boid_count; i = i + 1u) {
        if (i == index) {
            continue;
        }
        let other = read_boid(i, read_from_a);
        let delta = wrap_delta(other.position - boid.position);
        let dist = length(delta);
        if (dist < neighbor_radius) {
            alignment = alignment + other.velocity;
            cohesion = cohesion + other.position;
            neighbor_count = neighbor_count + 1u;
        }
        if (dist < separation_radius) {
            let safe_dist = max(dist, 0.0005);
            separation = separation - delta / safe_dist;
            separation_count = separation_count + 1u;
        }
    }

    var acceleration = vec2<f32>(0.0, 0.0);
    if (neighbor_count > 0u) {
        let inv_neighbor_count = 1.0 / f32(neighbor_count);

        let mean_velocity = alignment * inv_neighbor_count;
        let desired_alignment = normalize_or_zero(mean_velocity) * max_speed;
        acceleration = acceleration + (desired_alignment - boid.velocity) * alignment_weight;

        let mean_position = cohesion * inv_neighbor_count;
        let cohesion_delta = wrap_delta(mean_position - boid.position);
        let desired_cohesion = normalize_or_zero(cohesion_delta) * max_speed;
        acceleration = acceleration + (desired_cohesion - boid.velocity) * cohesion_weight;
    }

    if (separation_count > 0u) {
        let inv_separation_count = 1.0 / f32(separation_count);
        let desired_separation = normalize_or_zero(separation * inv_separation_count) * max_speed;
        acceleration = acceleration + (desired_separation - boid.velocity) * separation_weight;
    }

    let center_delta = wrap_delta(vec2<f32>(0.5, 0.5) - boid.position);
    acceleration = acceleration + center_delta * center_weight;

    let jitter_seed = tick * 4099u + index * 23u;
    let jitter = vec2<f32>(
        rand01(jitter_seed) - 0.5,
        rand01(jitter_seed + 1u) - 0.5
    ) * jitter_strength;
    acceleration = acceleration + jitter;

    acceleration = limit_magnitude(acceleration, max_force);
    boid.velocity = boid.velocity + acceleration * delta_seconds;
    boid.velocity = limit_magnitude(boid.velocity, max_speed);

    let min_speed = max_speed * 0.25;
    if (length(boid.velocity) < min_speed) {
        let stable_dir = normalize_or_zero(boid.velocity + vec2<f32>(0.0001, 0.0001));
        boid.velocity = stable_dir * min_speed;
    }

    boid.position = fract(boid.position + boid.velocity * delta_seconds);

    write_boid(index, read_from_a, boid);
}
