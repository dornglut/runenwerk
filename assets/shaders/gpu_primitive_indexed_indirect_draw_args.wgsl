override OUTPUT_INDEX: u32 = 0u;
override INDEX_COUNT: u32 = 0u;
override INSTANCE_COUNT: u32 = 0u;
override FIRST_INDEX: u32 = 0u;
override BASE_VERTEX: i32 = 0;
override FIRST_INSTANCE: u32 = 0u;

struct DrawIndexedIndirectArgs {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
};

@group(0) @binding(0)
var<storage, read_write> output_args: array<DrawIndexedIndirectArgs>;

@compute @workgroup_size(1)
fn cs_main() {
    if (OUTPUT_INDEX < arrayLength(&output_args)) {
        output_args[OUTPUT_INDEX].index_count = INDEX_COUNT;
        output_args[OUTPUT_INDEX].instance_count = INSTANCE_COUNT;
        output_args[OUTPUT_INDEX].first_index = FIRST_INDEX;
        output_args[OUTPUT_INDEX].base_vertex = BASE_VERTEX;
        output_args[OUTPUT_INDEX].first_instance = FIRST_INSTANCE;
    }
}
