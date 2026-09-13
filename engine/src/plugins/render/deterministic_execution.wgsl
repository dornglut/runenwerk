struct Hit {
    valid: bool,
    found: bool,
    t: f32,
    code: u32,
    normal_scene: vec3<f32>,
    reflectance: f32,
};

struct NormalizedDirection {
    valid: bool,
    value: vec3<f32>,
};

struct ScalarEvaluation {
    valid: bool,
    value: f32,
};

@group(0) @binding(0)
var<storage, read> input_words: array<u32>;

@group(0) @binding(1)
var<storage, read_write> output_words: array<u32>;

@group(0) @binding(2)
var<storage, read_write> defined_words: array<u32>;

@group(0) @binding(3)
var<storage, read_write> status_words: array<u32>;

const F32_EXPONENT_MASK: u32 = 2139095040u;

fn finite_f32(value: f32) -> bool {
    return (bitcast<u32>(value) & F32_EXPONENT_MASK) != F32_EXPONENT_MASK;
}

fn finite_vec3(value: vec3<f32>) -> bool {
    return finite_f32(value.x) && finite_f32(value.y) && finite_f32(value.z);
}

fn load_f32(index: u32) -> f32 {
    return bitcast<f32>(input_words[index]);
}

fn miss() -> Hit {
    return Hit(true, false, 0.0, 0u, vec3<f32>(0.0), 0.0);
}

fn invalid_hit() -> Hit {
    return Hit(false, false, 0.0, 0u, vec3<f32>(0.0), 0.0);
}

fn normalize_checked(value: vec3<f32>) -> NormalizedDirection {
    let magnitude_squared = dot(value, value);
    if !finite_f32(magnitude_squared) || magnitude_squared <= 0.0 {
        return NormalizedDirection(false, vec3<f32>(0.0));
    }
    let normalized = value / sqrt(magnitude_squared);
    return NormalizedDirection(finite_vec3(normalized), normalized);
}

fn mul3(base: u32, value: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        dot(vec3<f32>(load_f32(base), load_f32(base + 1u), load_f32(base + 2u)), value),
        dot(vec3<f32>(load_f32(base + 3u), load_f32(base + 4u), load_f32(base + 5u)), value),
        dot(vec3<f32>(load_f32(base + 6u), load_f32(base + 7u), load_f32(base + 8u)), value),
    );
}

fn geometry_base(index: u32) -> u32 {
    return 24u + index * 32u;
}

fn emitter_base(index: u32) -> u32 {
    return input_words[23u] + index * 4u;
}

fn to_local_point(base: u32, point_scene: vec3<f32>) -> vec3<f32> {
    let translation = vec3<f32>(
        load_f32(base + 13u),
        load_f32(base + 14u),
        load_f32(base + 15u),
    );
    return mul3(base + 4u, point_scene - translation);
}

fn to_local_direction(base: u32, direction_scene: vec3<f32>) -> vec3<f32> {
    return mul3(base + 4u, direction_scene);
}

fn normal_to_scene(base: u32, normal_local: vec3<f32>) -> NormalizedDirection {
    return normalize_checked(mul3(base + 16u, normal_local));
}

fn intersect_sphere(base: u32, origin_scene: vec3<f32>, direction_scene: vec3<f32>) -> Hit {
    let origin = to_local_point(base, origin_scene);
    let direction = to_local_direction(base, direction_scene);
    if !finite_vec3(origin) || !finite_vec3(direction) {
        return invalid_hit();
    }

    let center = vec3<f32>(
        load_f32(base + 25u),
        load_f32(base + 26u),
        load_f32(base + 27u),
    );
    let radius = load_f32(base + 28u);
    let relative = origin - center;
    let a = dot(direction, direction);
    let b = 2.0 * dot(relative, direction);
    let c = dot(relative, relative) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    if !finite_f32(a) || a <= 0.0 || !finite_f32(discriminant) {
        return invalid_hit();
    }
    if discriminant < 0.0 {
        return miss();
    }

    let root = sqrt(discriminant);
    let denominator = 2.0 * a;
    let first = (-b - root) / denominator;
    let second = (-b + root) / denominator;
    if !finite_f32(first) || !finite_f32(second) {
        return invalid_hit();
    }

    var found = false;
    var t = 0.0;
    if first >= 0.0 {
        found = true;
        t = first;
    }
    if second >= 0.0 && (!found || second < t) {
        found = true;
        t = second;
    }
    if !found {
        return miss();
    }

    let hit_local = origin + direction * t;
    if !finite_vec3(hit_local) {
        return invalid_hit();
    }
    let normal = normalize_checked(hit_local - center);
    if !normal.valid {
        return invalid_hit();
    }
    let normal_scene = normal_to_scene(base, normal.value);
    if !normal_scene.valid {
        return invalid_hit();
    }
    return Hit(
        true,
        true,
        t,
        input_words[base + 1u],
        normal_scene.value,
        load_f32(base + 2u),
    );
}

fn intersect_plane(base: u32, origin_scene: vec3<f32>, direction_scene: vec3<f32>) -> Hit {
    let origin = to_local_point(base, origin_scene);
    let direction = to_local_direction(base, direction_scene);
    if !finite_vec3(origin) || !finite_vec3(direction) {
        return invalid_hit();
    }

    let point = vec3<f32>(
        load_f32(base + 25u),
        load_f32(base + 26u),
        load_f32(base + 27u),
    );
    let normal = normalize_checked(vec3<f32>(
        load_f32(base + 28u),
        load_f32(base + 29u),
        load_f32(base + 30u),
    ));
    if !normal.valid {
        return invalid_hit();
    }

    let denominator = dot(normal.value, direction);
    if !finite_f32(denominator) {
        return invalid_hit();
    }
    if denominator == 0.0 {
        return miss();
    }
    let t = dot(point - origin, normal.value) / denominator;
    if !finite_f32(t) {
        return invalid_hit();
    }
    if t < 0.0 {
        return miss();
    }

    let normal_scene = normal_to_scene(base, normal.value);
    if !normal_scene.valid {
        return invalid_hit();
    }
    return Hit(
        true,
        true,
        t,
        input_words[base + 1u],
        normal_scene.value,
        load_f32(base + 2u),
    );
}

fn intersect_geometry(base: u32, origin_scene: vec3<f32>, direction_scene: vec3<f32>) -> Hit {
    if input_words[base] == 2u {
        return intersect_plane(base, origin_scene, direction_scene);
    }
    return intersect_sphere(base, origin_scene, direction_scene);
}

fn nearest_hit(origin_scene: vec3<f32>, direction_scene: vec3<f32>, ignored_code: u32) -> Hit {
    let geometry_count = input_words[4u];
    var nearest = miss();
    var index = 0u;
    loop {
        if index >= geometry_count {
            break;
        }
        let base = geometry_base(index);
        let code = input_words[base + 1u];
        if code != ignored_code {
            let candidate = intersect_geometry(base, origin_scene, direction_scene);
            if !candidate.valid {
                return invalid_hit();
            }
            if candidate.found && (!nearest.found || candidate.t < nearest.t) {
                nearest = candidate;
            }
        }
        index = index + 1u;
    }
    return nearest;
}

fn direct_radiance(hit_position: vec3<f32>, hit: Hit) -> ScalarEvaluation {
    let emitter_count = input_words[5u];
    var total = 0.0;
    var index = 0u;
    loop {
        if index >= emitter_count {
            break;
        }
        let base = emitter_base(index);
        let direction = normalize_checked(vec3<f32>(
            load_f32(base),
            load_f32(base + 1u),
            load_f32(base + 2u),
        ));
        if !direction.valid {
            return ScalarEvaluation(false, 0.0);
        }

        let cosine = max(dot(hit.normal_scene, direction.value), 0.0);
        if !finite_f32(cosine) {
            return ScalarEvaluation(false, 0.0);
        }
        if cosine > 0.0 {
            let blocker = nearest_hit(hit_position, direction.value, hit.code);
            if !blocker.valid {
                return ScalarEvaluation(false, 0.0);
            }
            if !blocker.found {
                let contribution =
                    hit.reflectance * load_f32(base + 3u) * cosine / 3.14159265358979323846;
                total = total + contribution;
                if !finite_f32(total) {
                    return ScalarEvaluation(false, 0.0);
                }
            }
        }
        index = index + 1u;
    }
    return ScalarEvaluation(true, total);
}

fn observation_origin() -> vec3<f32> {
    return vec3<f32>(load_f32(8u), load_f32(9u), load_f32(10u));
}

fn sample_direction(index: u32) -> NormalizedDirection {
    if input_words[7u] == 2u {
        return normalize_checked(mul3(11u, vec3<f32>(0.0, 0.0, -1.0)));
    }

    let width = f32(input_words[1u]);
    let height = f32(input_words[2u]);
    let x = index % input_words[1u];
    let y = index / input_words[1u];
    let u = (f32(x) + 0.5) / width;
    let v = (f32(y) + 0.5) / height;
    let local = vec3<f32>(
        (2.0 * u - 1.0) * load_f32(20u) * load_f32(21u),
        (1.0 - 2.0 * v) * load_f32(20u),
        -1.0,
    );
    return normalize_checked(mul3(11u, local));
}

fn observation_forward() -> NormalizedDirection {
    return normalize_checked(mul3(11u, vec3<f32>(0.0, 0.0, -1.0)));
}

fn physical_output_index(index: u32) -> u32 {
    if input_words[7u] == 2u {
        return 0u;
    }
    let x = index % input_words[1u];
    let y = index / input_words[1u];
    return y * input_words[3u] + x;
}

fn invalidate(output_index: u32, sample_index: u32) {
    output_words[output_index] = 0u;
    defined_words[sample_index] = 0u;
    status_words[sample_index] = 1u;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let sample_index = invocation.x;
    if sample_index >= input_words[0u] {
        return;
    }

    let output_kind = input_words[6u];
    let output_index = physical_output_index(sample_index);
    let origin = observation_origin();
    let direction = sample_direction(sample_index);
    if !finite_vec3(origin) || !direction.valid {
        invalidate(output_index, sample_index);
        return;
    }

    let hit = nearest_hit(origin, direction.value, 0u);
    if !hit.valid {
        invalidate(output_index, sample_index);
        return;
    }

    if output_kind == 1u {
        if !hit.found {
            output_words[output_index] = bitcast<u32>(0.0);
            defined_words[sample_index] = 1u;
            return;
        }
        let position = origin + direction.value * hit.t;
        if !finite_vec3(position) {
            invalidate(output_index, sample_index);
            return;
        }
        let radiance = direct_radiance(position, hit);
        if !radiance.valid {
            invalidate(output_index, sample_index);
            return;
        }
        output_words[output_index] = bitcast<u32>(radiance.value);
        defined_words[sample_index] = 1u;
        return;
    }

    if output_kind == 2u {
        if !hit.found {
            return;
        }
        let forward = observation_forward();
        if !forward.valid {
            invalidate(output_index, sample_index);
            return;
        }
        let position = origin + direction.value * hit.t;
        let depth = dot(position - origin, forward.value);
        if !finite_vec3(position) || !finite_f32(depth) {
            invalidate(output_index, sample_index);
            return;
        }
        output_words[output_index] = bitcast<u32>(depth);
        defined_words[sample_index] = 1u;
        return;
    }

    if output_kind == 3u {
        if hit.found {
            output_words[output_index] = hit.code;
            defined_words[sample_index] = 1u;
        }
        return;
    }

    invalidate(output_index, sample_index);
}
