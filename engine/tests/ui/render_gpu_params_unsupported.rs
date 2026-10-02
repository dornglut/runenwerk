use engine::plugins::render::{GpuParams, GpuUniform};

#[derive(Clone, Copy)]
struct Unsupported;

#[derive(GpuUniform)]
struct InvalidParams {
    value: Unsupported,
}

fn main() {
    let _ = InvalidParams { value: Unsupported }.to_gpu();
}
