@group(0) @binding(0)
var<storage, read> input_words: array<u32>;

@group(0) @binding(1)
var<storage, read> current_words: array<u32>;

@group(0) @binding(2)
var<storage, read> defined_words: array<u32>;

@group(0) @binding(3)
var<storage, read_write> history_words: array<u32>;

@compute @workgroup_size(64)
fn main(
    @builtin(workgroup_id) workgroup: vec3<u32>,
    @builtin(num_workgroups) workgroups: vec3<u32>,
    @builtin(local_invocation_index) local_index: u32,
) {
    let group_index = workgroup.y * workgroups.x + workgroup.x;
    let output_index = group_index * 64u + local_index;

    let requested_width = input_words[22u];
    let requested_height = input_words[23u];
    let requested_count = requested_width * requested_height;
    if output_index >= requested_count {
        return;
    }

    let output_x = output_index % requested_width;
    let output_y = output_index / requested_width;
    let evaluation_width = input_words[1u];
    let evaluation_height = input_words[2u];
    let evaluation_stride = input_words[3u];
    let history_stride = input_words[27u];

    let evaluation_x = min(
        u32(floor((f32(output_x) + 0.5) * f32(evaluation_width) / f32(requested_width))),
        evaluation_width - 1u,
    );
    let evaluation_y = min(
        u32(floor((f32(output_y) + 0.5) * f32(evaluation_height) / f32(requested_height))),
        evaluation_height - 1u,
    );
    let evaluation_index = evaluation_y * evaluation_width + evaluation_x;
    if defined_words[evaluation_index] == 0u {
        return;
    }

    let current_index = evaluation_y * evaluation_stride + evaluation_x;
    let history_index = output_y * history_stride + output_x;
    let current = bitcast<f32>(current_words[current_index]);
    let age = min(input_words[26u], 4u);
    if age == 0u {
        history_words[history_index] = bitcast<u32>(current);
        return;
    }

    let previous = bitcast<f32>(history_words[history_index]);
    let weight = f32(age);
    let reconstructed = (previous * weight + current) / (weight + 1.0);
    history_words[history_index] = bitcast<u32>(reconstructed);
}
