use bytemuck::{Pod, Zeroable};

pub trait GpuParams {
    type Raw: Pod + Zeroable + Copy + 'static;

    fn to_gpu(&self) -> Self::Raw;
}

pub trait GpuUniform: GpuParams {}

pub trait GpuStorage: GpuParams {}
