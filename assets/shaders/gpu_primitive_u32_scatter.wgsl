override ELEMENT_COUNT: u32 = 0u;
override OUTPUT_CAPACITY: u32 = 0u;

struct U32Cell {
    value: u32,
};

@group(0) @binding(0)
var<storage, read> source_indices: array<U32Cell>;

@group(0) @binding(1)
var<storage, read> prefix_offsets: array<U32Cell>;

@group(0) @binding(2)
var<storage, read_write> output_indices: array<U32Cell>;

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    let element_count = min(ELEMENT_COUNT, min(arrayLength(&source_indices), arrayLength(&prefix_offsets)));
    let output_capacity = min(OUTPUT_CAPACITY, arrayLength(&output_indices));
    if (index < element_count) {
        let output_index = prefix_offsets[index].value;
        if (output_index < output_capacity) {
            output_indices[output_index].value = source_indices[index].value;
        }
    }
}
