use bytemuck::{Pod, Zeroable};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
pub struct GpuBoolU32(pub u32);

pub trait ToGpuValue {
    type Gpu: Pod + Zeroable + Copy + 'static;

    fn to_gpu_value(&self) -> Self::Gpu;
}

impl ToGpuValue for bool {
    type Gpu = GpuBoolU32;

    fn to_gpu_value(&self) -> Self::Gpu {
        GpuBoolU32(u32::from(*self))
    }
}

impl ToGpuValue for u32 {
    type Gpu = u32;

    fn to_gpu_value(&self) -> Self::Gpu {
        *self
    }
}

impl ToGpuValue for i32 {
    type Gpu = i32;

    fn to_gpu_value(&self) -> Self::Gpu {
        *self
    }
}

impl ToGpuValue for f32 {
    type Gpu = f32;

    fn to_gpu_value(&self) -> Self::Gpu {
        *self
    }
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

impl ToGpuValue for glam::Vec2 {
    type Gpu = [f32; 2];

    fn to_gpu_value(&self) -> Self::Gpu {
        self.to_array()
    }
}

impl ToGpuValue for glam::Vec3 {
    type Gpu = [f32; 4];

    fn to_gpu_value(&self) -> Self::Gpu {
        [self.x, self.y, self.z, 0.0]
    }
}

impl ToGpuValue for glam::Vec4 {
    type Gpu = [f32; 4];

    fn to_gpu_value(&self) -> Self::Gpu {
        self.to_array()
    }
}

impl ToGpuValue for glam::Mat4 {
    type Gpu = [[f32; 4]; 4];

    fn to_gpu_value(&self) -> Self::Gpu {
        self.to_cols_array_2d()
    }
}
