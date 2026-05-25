override OUTPUT_INDEX: u32 = 0u;
override VERTEX_COUNT: u32 = 0u;
override INSTANCE_COUNT: u32 = 0u;
override FIRST_VERTEX: u32 = 0u;
override FIRST_INSTANCE: u32 = 0u;

struct DrawIndirectArgs {
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
};

@group(0) @binding(0)
var<storage, read_write> output_args: array<DrawIndirectArgs>;

@compute @workgroup_size(1)
fn cs_main() {
    if (OUTPUT_INDEX < arrayLength(&output_args)) {
        output_args[OUTPUT_INDEX].vertex_count = VERTEX_COUNT;
        output_args[OUTPUT_INDEX].instance_count = INSTANCE_COUNT;
        output_args[OUTPUT_INDEX].first_vertex = FIRST_VERTEX;
        output_args[OUTPUT_INDEX].first_instance = FIRST_INSTANCE;
    }
}
