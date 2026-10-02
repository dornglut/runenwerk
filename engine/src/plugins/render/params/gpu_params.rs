use bytemuck::{Pod, Zeroable};

pub trait GpuParams {
    type Raw: Pod + Zeroable + Copy + 'static;

    fn to_gpu(&self) -> Self::Raw;
}

/// Uniform parameter layouts require every field to implement `GpuUniformField`.
///
/// ```compile_fail
/// use engine::plugins::render::{GpuParams, GpuUniform};
///
/// #[derive(Clone, Copy)]
/// struct Unsupported;
///
/// #[derive(GpuUniform)]
/// struct InvalidParams {
///     value: Unsupported,
/// }
///
/// fn main() {
///     let _ = InvalidParams { value: Unsupported }.to_gpu();
/// }
/// ```
pub trait GpuUniform: GpuParams {}

pub trait GpuStorage: GpuParams {}
