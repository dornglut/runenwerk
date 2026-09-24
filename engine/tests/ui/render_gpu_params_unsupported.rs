use engine::plugins::render::GpuUniform;

#[derive(Clone, Copy)]
struct Unsupported;

#[derive(GpuUniform)]
struct InvalidParams {
    value: Unsupported,
}

fn main() {}
