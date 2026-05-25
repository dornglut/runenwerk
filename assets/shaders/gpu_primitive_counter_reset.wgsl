override ELEMENT_COUNT: u32 = 0u;
override RESET_VALUE: u32 = 0u;

struct U32Cell {
    value: u32,
};

@group(0) @binding(0)
var<storage, read_write> counters: array<U32Cell>;

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    let count = min(ELEMENT_COUNT, arrayLength(&counters));
    if (index < count) {
        counters[index].value = RESET_VALUE;
    }
}
