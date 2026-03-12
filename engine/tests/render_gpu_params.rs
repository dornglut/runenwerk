use engine::plugins::render::{GpuBoolU32, GpuParams, GpuStorage, GpuUniform, ToGpuValue};

#[derive(Debug, Clone, Copy, GpuUniform)]
struct ExampleParams {
    grid_size: [u32; 2],
    enabled: bool,
}

#[derive(Debug, Clone, Copy, GpuStorage)]
struct StorageExample {
    indices: [i32; 4],
}

#[test]
fn generated_raw_conversion_uses_to_gpu_value_for_each_field() {
    let raw = ExampleParams {
        grid_size: [160, 90],
        enabled: true,
    }
    .to_gpu();

    assert_eq!(raw.grid_size, [160, 90]);
    assert_eq!(raw.enabled, GpuBoolU32(1));
}

#[test]
fn bool_conversion_uses_gpu_bool_u32() {
    let value = true.to_gpu_value();
    assert_eq!(value, GpuBoolU32(1));

    let value = false.to_gpu_value();
    assert_eq!(value, GpuBoolU32(0));
}

#[test]
fn array_conversion_is_supported() {
    let raw = [1.0_f32, 2.0, 3.0, 4.0].to_gpu_value();
    assert_eq!(raw, [1.0_f32, 2.0, 3.0, 4.0]);

    let raw = [true, false, true].to_gpu_value();
    assert_eq!(raw, [GpuBoolU32(1), GpuBoolU32(0), GpuBoolU32(1)]);
}

#[test]
fn gpu_storage_derive_produces_gpu_params_impl() {
    let raw = StorageExample {
        indices: [1, 2, 3, 4],
    }
    .to_gpu();
    assert_eq!(raw.indices, [1, 2, 3, 4]);
}

#[test]
fn unsupported_field_types_fail_to_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/render_gpu_params_unsupported.rs");
}
