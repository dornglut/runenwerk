@group(0) @binding(0)
var<storage, read> input_words: array<u32>;

@group(0) @binding(1)
var<storage, read_write> current_radiance_words: array<u32>;

@group(0) @binding(2)
var<storage, read> defined_words: array<u32>;

@group(0) @binding(3)
var<storage, read> current_depth_words: array<u32>;

@group(0) @binding(4)
var<storage, read> current_hit_words: array<u32>;

@group(0) @binding(5)
var<storage, read> previous_history_words: array<u32>;

@group(0) @binding(6)
var<storage, read_write> current_history_words: array<u32>;

@group(0) @binding(7)
var<storage, read> camera_words: array<u32>;

const CAMERA_DEPTH_ABSOLUTE_EPSILON: f32 = 0.001;
const CAMERA_DEPTH_RELATIVE_EPSILON: f32 = 0.001;
const INVALID_HISTORY_SAMPLE_COUNT: u32 = 4294967295u;

fn load_input_f32(index: u32) -> f32 {
    return bitcast<f32>(input_words[index]);
}

fn load_camera_f32(index: u32) -> f32 {
    return bitcast<f32>(camera_words[index]);
}

fn mul_input3(base: u32, value: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        dot(vec3<f32>(load_input_f32(base), load_input_f32(base + 1u), load_input_f32(base + 2u)), value),
        dot(vec3<f32>(load_input_f32(base + 3u), load_input_f32(base + 4u), load_input_f32(base + 5u)), value),
        dot(vec3<f32>(load_input_f32(base + 6u), load_input_f32(base + 7u), load_input_f32(base + 8u)), value),
    );
}

fn mul_camera3(base: u32, value: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        dot(vec3<f32>(load_camera_f32(base), load_camera_f32(base + 1u), load_camera_f32(base + 2u)), value),
        dot(vec3<f32>(load_camera_f32(base + 3u), load_camera_f32(base + 4u), load_camera_f32(base + 5u)), value),
        dot(vec3<f32>(load_camera_f32(base + 6u), load_camera_f32(base + 7u), load_camera_f32(base + 8u)), value),
    );
}

fn normalize_camera_checked(value: vec3<f32>) -> vec4<f32> {
    let magnitude_squared = dot(value, value);
    if !finite_f32(magnitude_squared) || magnitude_squared <= 0.0 {
        return vec4<f32>(0.0);
    }
    let normalized = value / sqrt(magnitude_squared);
    if !finite_vec3(normalized) {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(normalized, 1.0);
}

fn current_origin() -> vec3<f32> {
    return vec3<f32>(load_input_f32(8u), load_input_f32(9u), load_input_f32(10u));
}

fn current_direction(index: u32) -> vec4<f32> {
    let width = input_words[1u];
    let height = input_words[2u];
    let x = index % width;
    let y = index / width;
    let phase = input_words[24u] % 4u;
    let phase_x = select(0.25, 0.75, phase == 1u || phase == 3u);
    let phase_y = select(0.25, 0.75, phase >= 2u);
    let u = (f32(x) + phase_x) / f32(width);
    let v = (f32(y) + phase_y) / f32(height);
    let local = vec3<f32>(
        (2.0 * u - 1.0) * load_input_f32(20u) * load_input_f32(21u),
        (1.0 - 2.0 * v) * load_input_f32(20u),
        -1.0,
    );
    return normalize_camera_checked(mul_input3(11u, local));
}

fn current_forward() -> vec4<f32> {
    return normalize_camera_checked(mul_input3(11u, vec3<f32>(0.0, 0.0, -1.0)));
}

fn history_base(index: u32) -> u32 {
    return index * 8u;
}

fn current_output_index(index: u32) -> u32 {
    let x = index % input_words[1u];
    let y = index / input_words[1u];
    return y * input_words[3u] + x;
}

fn write_current(index: u32, radiance: f32, depth: f32, count: u32, hit: u32) {
    let base = history_base(index);
    current_history_words[base] = bitcast<u32>(radiance);
    current_history_words[base + 1u] = bitcast<u32>(depth);
    current_history_words[base + 2u] = count;
    current_history_words[base + 3u] = hit;
    // Slots 4..7 retain one coherent motion sample independently of the stationary estimate.
    current_history_words[base + 4u] = 2143289344u; // Invalid until a finite coherent anchor is written.
    current_history_words[base + 5u] = 0u;
    current_history_words[base + 6u] = 0u;
    current_history_words[base + 7u] = 0u;
}

fn write_current_motion_sample(index: u32, radiance: f32, point: vec3<f32>, valid: bool) {
    if !valid || !finite_vec3(point) {
        return;
    }
    let base = history_base(index);
    current_history_words[base + 4u] = bitcast<u32>(radiance);
    current_history_words[base + 5u] = bitcast<u32>(point.x);
    current_history_words[base + 6u] = bitcast<u32>(point.y);
    current_history_words[base + 7u] = bitcast<u32>(point.z);
}

fn matching_world_point(current_point: vec3<f32>, previous_point: vec3<f32>) -> bool {
    if !finite_vec3(current_point) || !finite_vec3(previous_point) {
        return false;
    }
    // Represented coordinates must agree exactly; no spatial tolerance certifies visibility.
    return all(current_point == previous_point);
}

// A retained ray sample contributes only to the exact current requested footprint. The
// primary visibility and lighting query below independently validates it in the current view;
// geometric proximity, hit identity and depth tolerance cannot certify shadow visibility.
fn point_in_current_footprint(index: u32, point: vec3<f32>) -> bool {
    let local = mul_camera3(24u, point - current_origin());
    if !finite_vec3(local) || local.z >= 0.0 {
        return false;
    }
    let u = 0.5 + 0.5 * local.x / (-local.z * load_input_f32(20u) * load_input_f32(21u));
    let v = 0.5 - 0.5 * local.y / (-local.z * load_input_f32(20u));
    let cell = vec2<f32>(u * f32(input_words[22u]), v * f32(input_words[23u]));
    let x = f32(index % input_words[1u]);
    let y = f32(index / input_words[1u]);
    if !finite_f32(cell.x) || !finite_f32(cell.y)
        || cell.x < x || cell.x >= x + 1.0 || cell.y < y || cell.y >= y + 1.0 {
        return false;
    }
    return true;
}

fn retained_sample_valid(index: u32, point: vec3<f32>, radiance: f32) -> bool {
    if !point_in_current_footprint(index, point) {
        return false;
    }
    let direction = normalize_checked(point - current_origin());
    if !direction.valid {
        return false;
    }
    let hit = nearest_hit(current_origin(), direction.value, 0u);
    if !hit.valid || !hit.found {
        return false;
    }
    let visible_point = current_origin() + direction.value * hit.t;
    if !point_in_current_footprint(index, visible_point) || !matching_world_point(point, visible_point) {
        return false;
    }
    let validated = direct_radiance(visible_point, hit);
    // Exact scalar equality is conservative. It introduces no guessed lighting tolerance:
    // the retained radiance must be the value independently evaluated at this visible sample.
    return validated.valid && finite_f32(radiance)
        && bitcast<u32>(validated.value) == bitcast<u32>(radiance);
}

@compute @workgroup_size(64)
fn main(
    @builtin(workgroup_id) workgroup: vec3<u32>,
    @builtin(num_workgroups) workgroups: vec3<u32>,
    @builtin(local_invocation_index) local_index: u32,
) {
    let group_index = workgroup.y * workgroups.x + workgroup.x;
    let sample_index = group_index * 64u + local_index;
    if sample_index >= input_words[0u] {
        return;
    }

    let output_index = current_output_index(sample_index);
    if defined_words[sample_index] == 0u {
        current_radiance_words[output_index] = 0u;
        write_current(sample_index, 0.0, 0.0, INVALID_HISTORY_SAMPLE_COUNT, 0u);
        return;
    }

    let current_radiance = bitcast<f32>(current_radiance_words[output_index]);
    let current_hit_base = output_index * 4u;
    let current_hit_word = min(current_hit_words[current_hit_base], 1u);
    let current_hit = current_hit_word != 0u;
    var current_depth = 0.0;
    if current_hit {
        current_depth = bitcast<f32>(current_depth_words[output_index]);
        if !finite_f32(current_depth) {
            current_radiance_words[output_index] = 0u;
            write_current(sample_index, 0.0, 0.0, INVALID_HISTORY_SAMPLE_COUNT, 0u);
            return;
        }
    }
    let current_sample = select(0.0, current_radiance, current_hit);
    var current_point = vec4<f32>(0.0);
    if current_hit {
        let point = vec3<f32>(
            bitcast<f32>(current_hit_words[current_hit_base + 1u]),
            bitcast<f32>(current_hit_words[current_hit_base + 2u]),
            bitcast<f32>(current_hit_words[current_hit_base + 3u]),
        );
        if !finite_vec3(point) || !finite_f32(current_sample) {
            current_radiance_words[output_index] = 0u;
            write_current(sample_index, 0.0, 0.0, INVALID_HISTORY_SAMPLE_COUNT, 0u);
            return;
        }
        current_point = vec4<f32>(point, 1.0);
    }

    let previous_available = camera_words[0u] != 0u;
    if !previous_available {
        current_radiance_words[output_index] = bitcast<u32>(current_sample);
        write_current(
            sample_index,
            current_sample,
            current_depth,
            1u,
            current_hit_word,
        );
        write_current_motion_sample(sample_index, current_sample, current_point.xyz, current_point.w != 0.0);
        return;
    }

    let pose_changed = camera_words[1u] != 0u;
    if pose_changed {
        // Preserve the accepted T2 branch for actual camera motion. A current miss has no
        // geometric point to reproject and therefore remains current-only background.
        if !current_hit {
            current_radiance_words[output_index] = bitcast<u32>(0.0);
            write_current(sample_index, 0.0, 0.0, 1u, 0u);
            return;
        }

        let direction = current_direction(sample_index);
        let forward = current_forward();
        if direction.w == 0.0 || forward.w == 0.0 {
            write_current(sample_index, current_radiance, current_depth, 1u, 1u);
            write_current_motion_sample(sample_index, current_sample, current_point.xyz, current_point.w != 0.0);
            return;
        }
        let denominator = dot(direction.xyz, forward.xyz);
        if !finite_f32(denominator) || denominator <= 0.0 {
            write_current(sample_index, current_radiance, current_depth, 1u, 1u);
            write_current_motion_sample(sample_index, current_sample, current_point.xyz, current_point.w != 0.0);
            return;
        }
        let scene_point = current_point.xyz;
        let previous_origin = vec3<f32>(
            load_camera_f32(6u),
            load_camera_f32(7u),
            load_camera_f32(8u),
        );
        let local = mul_camera3(9u, scene_point - previous_origin);
        if !finite_vec3(local) || local.z >= 0.0 {
            write_current(sample_index, current_radiance, current_depth, 1u, 1u);
            write_current_motion_sample(sample_index, current_sample, current_point.xyz, current_point.w != 0.0);
            return;
        }
        let previous_tan_half_fov = load_camera_f32(21u);
        let previous_aspect = load_camera_f32(22u);
        let projected_x = local.x / (-local.z * previous_tan_half_fov * previous_aspect);
        let projected_y = local.y / (-local.z * previous_tan_half_fov);
        let u = projected_x * 0.5 + 0.5;
        let v = 0.5 - projected_y * 0.5;
        if !finite_f32(u) || !finite_f32(v) || u < 0.0 || u >= 1.0 || v < 0.0 || v >= 1.0 {
            write_current(sample_index, current_radiance, current_depth, 1u, 1u);
            write_current_motion_sample(sample_index, current_sample, current_point.xyz, current_point.w != 0.0);
            return;
        }
        let width = input_words[22u];
        let height = input_words[23u];
        let previous_x = min(u32(floor(u * f32(width))), width - 1u);
        let previous_y = min(u32(floor(v * f32(height))), height - 1u);
        let previous_index = previous_y * width + previous_x;
        let previous_base = history_base(previous_index);
        if previous_history_words[previous_base + 3u] == 0u {
            write_current(sample_index, current_radiance, current_depth, 1u, 1u);
            write_current_motion_sample(sample_index, current_sample, current_point.xyz, current_point.w != 0.0);
            return;
        }

        let previous_depth = bitcast<f32>(previous_history_words[previous_base + 1u]);
        let previous_point = vec3<f32>(
            bitcast<f32>(previous_history_words[previous_base + 5u]),
            bitcast<f32>(previous_history_words[previous_base + 6u]),
            bitcast<f32>(previous_history_words[previous_base + 7u]),
        );
        let previous_forward = vec3<f32>(
            load_camera_f32(18u),
            load_camera_f32(19u),
            load_camera_f32(20u),
        );
        let projected_depth = dot(scene_point - previous_origin, previous_forward);
        let tolerance = CAMERA_DEPTH_ABSOLUTE_EPSILON
            + CAMERA_DEPTH_RELATIVE_EPSILON * max(abs(projected_depth), abs(previous_depth));
        if !finite_f32(previous_depth)
            || !finite_f32(projected_depth)
            || abs(projected_depth - previous_depth) > tolerance {
            write_current(sample_index, current_radiance, current_depth, 1u, 1u);
            write_current_motion_sample(sample_index, current_sample, current_point.xyz, current_point.w != 0.0);
            return;
        }

        let coherent_radiance = bitcast<f32>(previous_history_words[previous_base + 4u]);
        if current_point.w == 0.0 || !retained_sample_valid(sample_index, previous_point, coherent_radiance) {
            write_current(sample_index, current_radiance, current_depth, 1u, 1u);
            write_current_motion_sample(sample_index, current_sample, current_point.xyz, current_point.w != 0.0);
            return;
        }

        let previous_raw_count = previous_history_words[previous_base + 2u];
        if previous_raw_count == 0u || previous_raw_count == INVALID_HISTORY_SAMPLE_COUNT {
            write_current(sample_index, current_radiance, current_depth, 1u, 1u);
            write_current_motion_sample(sample_index, current_sample, current_point.xyz, current_point.w != 0.0);
            return;
        }
        let previous_radiance = bitcast<f32>(previous_history_words[previous_base + 4u]);
        if !finite_f32(previous_radiance) {
            write_current(sample_index, current_radiance, current_depth, 1u, 1u);
            write_current_motion_sample(sample_index, current_sample, current_point.xyz, current_point.w != 0.0);
            return;
        }
        let reconstructed = (previous_radiance + current_radiance) * 0.5;
        current_radiance_words[output_index] = bitcast<u32>(reconstructed);
        write_current(
            sample_index,
            reconstructed,
            current_depth,
            2u,
            1u,
        );
        write_current_motion_sample(sample_index, current_sample, current_point.xyz, current_point.w != 0.0);
        return;
    }

    // Same-pose reconstruction is a bounded four-phase estimator, separate from moving-camera
    // reprojection. A defined miss contributes the real radiance sample zero instead of erasing
    // foreground history from another phase.
    let same_pose_completed_frames = min(camera_words[23u], 4u);
    if same_pose_completed_frames == 0u {
        current_radiance_words[output_index] = bitcast<u32>(current_sample);
        write_current(
            sample_index,
            current_sample,
            current_depth,
            1u,
            current_hit_word,
        );
        write_current_motion_sample(sample_index, current_sample, current_point.xyz, current_point.w != 0.0);
        return;
    }

    let previous_base = history_base(sample_index);
    let previous_raw_count = previous_history_words[previous_base + 2u];
    if previous_raw_count == INVALID_HISTORY_SAMPLE_COUNT {
        current_radiance_words[output_index] = 0u;
        write_current(
            sample_index,
            0.0,
            current_depth,
            INVALID_HISTORY_SAMPLE_COUNT,
            current_hit_word,
        );
        write_current_motion_sample(sample_index, current_sample, current_point.xyz, current_point.w != 0.0);
        return;
    }
    let previous_radiance = bitcast<f32>(previous_history_words[previous_base]);
    let previous_count = min(previous_raw_count, 4u);
    if previous_count == 0u || !finite_f32(previous_radiance) {
        current_radiance_words[output_index] = bitcast<u32>(current_sample);
        write_current(
            sample_index,
            current_sample,
            current_depth,
            1u,
            current_hit_word,
        );
        write_current_motion_sample(sample_index, current_sample, current_point.xyz, current_point.w != 0.0);
        return;
    }

    if same_pose_completed_frames >= 4u && previous_count >= 4u {
        // Radiance is formed and must remain invariant. Refresh the separate raw coherent
        // sample and its geometric anchor for later moving-camera validation.
        current_radiance_words[output_index] = bitcast<u32>(previous_radiance);
        write_current(
            sample_index,
            previous_radiance,
            current_depth,
            previous_count,
            current_hit_word,
        );
        write_current_motion_sample(sample_index, current_sample, current_point.xyz, current_point.w != 0.0);
        return;
    }

    let weight = f32(previous_count);
    let reconstructed = (previous_radiance * weight + current_sample) / (weight + 1.0);
    current_radiance_words[output_index] = bitcast<u32>(reconstructed);
    write_current(
        sample_index,
        reconstructed,
        current_depth,
        min(previous_count + 1u, 4u),
        current_hit_word,
    );
    write_current_motion_sample(sample_index, current_sample, current_point.xyz, current_point.w != 0.0);
}
