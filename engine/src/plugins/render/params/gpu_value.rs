use bytemuck::{Pod, Zeroable};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
pub struct GpuBoolU32(pub u32);

pub const fn align_up_const(value: usize, alignment: usize) -> usize {
    if alignment <= 1 {
        return value;
    }
    let remainder = value % alignment;
    if remainder == 0 {
        value
    } else {
        value + (alignment - remainder)
    }
}

pub trait ToGpuValue {
    type Gpu: Pod + Zeroable + Copy + 'static;

    fn to_gpu_value(&self) -> Self::Gpu;
}

pub trait GpuUniformField: ToGpuValue {
    const ABI_ALIGN: usize;
    const ABI_SIZE: usize;
}

pub fn write_uniform_field<T>(bytes: &mut [u8], offset: usize, value: &T)
where
    T: ToGpuValue,
{
    let gpu = value.to_gpu_value();
    let raw = bytemuck::bytes_of(&gpu);
    let end = offset.saturating_add(raw.len());
    if end <= bytes.len() {
        bytes[offset..end].copy_from_slice(raw);
    }
}

impl ToGpuValue for bool {
    type Gpu = GpuBoolU32;

    fn to_gpu_value(&self) -> Self::Gpu {
        GpuBoolU32(u32::from(*self))
    }
}

impl GpuUniformField for bool {
    const ABI_ALIGN: usize = 4;
    const ABI_SIZE: usize = 4;
}

impl ToGpuValue for u32 {
    type Gpu = u32;

    fn to_gpu_value(&self) -> Self::Gpu {
        *self
    }
}

impl GpuUniformField for u32 {
    const ABI_ALIGN: usize = 4;
    const ABI_SIZE: usize = 4;
}

impl ToGpuValue for i32 {
    type Gpu = i32;

    fn to_gpu_value(&self) -> Self::Gpu {
        *self
    }
}

impl GpuUniformField for i32 {
    const ABI_ALIGN: usize = 4;
    const ABI_SIZE: usize = 4;
}

impl ToGpuValue for f32 {
    type Gpu = f32;

    fn to_gpu_value(&self) -> Self::Gpu {
        *self
    }
}

impl GpuUniformField for f32 {
    const ABI_ALIGN: usize = 4;
    const ABI_SIZE: usize = 4;
}

impl<T, const N: usize> ToGpuValue for [T; N]
where
    T: ToGpuValue,
{
    type Gpu = [T::Gpu; N];

    fn to_gpu_value(&self) -> Self::Gpu {
        core::array::from_fn(|index| self[index].to_gpu_value())
    }
}

impl GpuUniformField for [u32; 2] {
    const ABI_ALIGN: usize = 8;
    const ABI_SIZE: usize = 8;
}

impl GpuUniformField for [u32; 3] {
    const ABI_ALIGN: usize = 16;
    const ABI_SIZE: usize = 16;
}

impl GpuUniformField for [u32; 4] {
    const ABI_ALIGN: usize = 16;
    const ABI_SIZE: usize = 16;
}

impl GpuUniformField for [i32; 2] {
    const ABI_ALIGN: usize = 8;
    const ABI_SIZE: usize = 8;
}

impl GpuUniformField for [i32; 3] {
    const ABI_ALIGN: usize = 16;
    const ABI_SIZE: usize = 16;
}

impl GpuUniformField for [i32; 4] {
    const ABI_ALIGN: usize = 16;
    const ABI_SIZE: usize = 16;
}

impl GpuUniformField for [f32; 2] {
    const ABI_ALIGN: usize = 8;
    const ABI_SIZE: usize = 8;
}

impl GpuUniformField for [f32; 3] {
    const ABI_ALIGN: usize = 16;
    const ABI_SIZE: usize = 16;
}

impl GpuUniformField for [f32; 4] {
    const ABI_ALIGN: usize = 16;
    const ABI_SIZE: usize = 16;
}

impl<const N: usize> GpuUniformField for [[f32; 4]; N] {
    const ABI_ALIGN: usize = 16;
    const ABI_SIZE: usize = 16 * N;
}

impl<const N: usize> GpuUniformField for [[u32; 4]; N] {
    const ABI_ALIGN: usize = 16;
    const ABI_SIZE: usize = 16 * N;
}

impl ToGpuValue for glam::Vec2 {
    type Gpu = [f32; 2];

    fn to_gpu_value(&self) -> Self::Gpu {
        self.to_array()
    }
}

impl GpuUniformField for glam::Vec2 {
    const ABI_ALIGN: usize = 8;
    const ABI_SIZE: usize = 8;
}

impl ToGpuValue for glam::Vec3 {
    type Gpu = [f32; 4];

    fn to_gpu_value(&self) -> Self::Gpu {
        [self.x, self.y, self.z, 0.0]
    }
}

impl GpuUniformField for glam::Vec3 {
    const ABI_ALIGN: usize = 16;
    const ABI_SIZE: usize = 16;
}

impl ToGpuValue for glam::Vec4 {
    type Gpu = [f32; 4];

    fn to_gpu_value(&self) -> Self::Gpu {
        self.to_array()
    }
}

impl GpuUniformField for glam::Vec4 {
    const ABI_ALIGN: usize = 16;
    const ABI_SIZE: usize = 16;
}

impl ToGpuValue for glam::UVec2 {
    type Gpu = [u32; 2];

    fn to_gpu_value(&self) -> Self::Gpu {
        self.to_array()
    }
}

impl GpuUniformField for glam::UVec2 {
    const ABI_ALIGN: usize = 8;
    const ABI_SIZE: usize = 8;
}

impl ToGpuValue for glam::UVec3 {
    type Gpu = [u32; 4];

    fn to_gpu_value(&self) -> Self::Gpu {
        [self.x, self.y, self.z, 0]
    }
}

impl GpuUniformField for glam::UVec3 {
    const ABI_ALIGN: usize = 16;
    const ABI_SIZE: usize = 16;
}

impl ToGpuValue for glam::UVec4 {
    type Gpu = [u32; 4];

    fn to_gpu_value(&self) -> Self::Gpu {
        self.to_array()
    }
}

impl GpuUniformField for glam::UVec4 {
    const ABI_ALIGN: usize = 16;
    const ABI_SIZE: usize = 16;
}

impl ToGpuValue for glam::IVec2 {
    type Gpu = [i32; 2];

    fn to_gpu_value(&self) -> Self::Gpu {
        self.to_array()
    }
}

impl GpuUniformField for glam::IVec2 {
    const ABI_ALIGN: usize = 8;
    const ABI_SIZE: usize = 8;
}

impl ToGpuValue for glam::IVec3 {
    type Gpu = [i32; 4];

    fn to_gpu_value(&self) -> Self::Gpu {
        [self.x, self.y, self.z, 0]
    }
}

impl GpuUniformField for glam::IVec3 {
    const ABI_ALIGN: usize = 16;
    const ABI_SIZE: usize = 16;
}

impl ToGpuValue for glam::IVec4 {
    type Gpu = [i32; 4];

    fn to_gpu_value(&self) -> Self::Gpu {
        self.to_array()
    }
}

impl GpuUniformField for glam::IVec4 {
    const ABI_ALIGN: usize = 16;
    const ABI_SIZE: usize = 16;
}

impl ToGpuValue for glam::Mat4 {
    type Gpu = [[f32; 4]; 4];

    fn to_gpu_value(&self) -> Self::Gpu {
        self.to_cols_array_2d()
    }
}

impl GpuUniformField for glam::Mat4 {
    const ABI_ALIGN: usize = 16;
    const ABI_SIZE: usize = 64;
}
