//! Current `GpuParams` lowering. G4 deletes this adapter when admitted shader
//! interfaces and purpose-specific encoders replace the transitional traits.

use crate::plugins::gpu::{
    GpuDataEncoder, GpuDataLayout, GpuDataPreparationError, GpuResourceProvenance, PreparedGpuData,
    StorageData, UniformData, prepare_gpu_data,
};
use crate::plugins::render::GpuParams;
use std::any::{TypeId, type_name};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderGpuParamsLayout {
    gpu: GpuDataLayout,
    params_type_id: TypeId,
    params_type_name: &'static str,
}

impl RenderGpuParamsLayout {
    pub fn uniform<Params: GpuParams + 'static>(
        label: &str,
    ) -> Result<Self, GpuDataPreparationError> {
        Self::for_elements::<Params>(label, 1)
    }

    pub fn storage<Params: GpuParams + 'static>(
        label: &str,
        element_count: u64,
    ) -> Result<Self, GpuDataPreparationError> {
        Self::for_elements::<Params>(label, element_count)
    }

    fn for_elements<Params: GpuParams + 'static>(
        label: &str,
        element_count: u64,
    ) -> Result<Self, GpuDataPreparationError> {
        let alignment = u64::try_from(core::mem::align_of::<Params::Raw>()).map_err(|_| {
            GpuDataPreparationError::Invalid {
                operation: "lower current render parameter layout",
                label: label.to_string(),
                cause: crate::plugins::gpu::GpuDataPreparationCause::ArithmeticOverflow,
                correction: "use a parameter alignment representable by the platform",
            }
        })?;
        let raw_size = u64::try_from(core::mem::size_of::<Params::Raw>()).map_err(|_| {
            GpuDataPreparationError::Invalid {
                operation: "lower current render parameter layout",
                label: label.to_string(),
                cause: crate::plugins::gpu::GpuDataPreparationCause::ArithmeticOverflow,
                correction: "use a parameter size representable by the platform",
            }
        })?;
        let stride = raw_size.max(alignment);
        let byte_len =
            stride
                .checked_mul(element_count)
                .ok_or_else(|| GpuDataPreparationError::Invalid {
                    operation: "lower current render parameter layout",
                    label: label.to_string(),
                    cause: crate::plugins::gpu::GpuDataPreparationCause::ArithmeticOverflow,
                    correction: "reduce the render parameter element count",
                })?;
        Ok(Self {
            gpu: GpuDataLayout::new(label, byte_len, alignment, stride, element_count)?,
            params_type_id: TypeId::of::<Params>(),
            params_type_name: type_name::<Params>(),
        })
    }

    pub const fn gpu(self) -> GpuDataLayout {
        self.gpu
    }

    pub const fn params_type_id(self) -> TypeId {
        self.params_type_id
    }

    pub const fn params_type_name(self) -> &'static str {
        self.params_type_name
    }
}

struct RenderUniformEncoder;

impl<Params: GpuParams + 'static> GpuDataEncoder<UniformData, Params> for RenderUniformEncoder {
    fn encode(
        &self,
        label: &str,
        source: &Params,
    ) -> Result<(Vec<u8>, GpuDataLayout), GpuDataPreparationError> {
        let layout = RenderGpuParamsLayout::uniform::<Params>(label)?.gpu();
        let raw = source.to_gpu();
        Ok((bytemuck::bytes_of(&raw).to_vec(), layout))
    }

    fn diagnostic_type_name(&self) -> Option<&'static str> {
        Some(type_name::<Params>())
    }
}

struct RenderStorageEncoder;

impl<Params: GpuParams + 'static> GpuDataEncoder<StorageData, Params> for RenderStorageEncoder {
    fn encode(
        &self,
        label: &str,
        source: &Params,
    ) -> Result<(Vec<u8>, GpuDataLayout), GpuDataPreparationError> {
        let layout = RenderGpuParamsLayout::storage::<Params>(label, 1)?.gpu();
        let raw = source.to_gpu();
        Ok((bytemuck::bytes_of(&raw).to_vec(), layout))
    }

    fn diagnostic_type_name(&self) -> Option<&'static str> {
        Some(type_name::<Params>())
    }
}

/// Transitional facade lowering into an explicit uniform-purpose encoder.
///
/// ```
/// use engine::plugins::gpu::{
///     GpuResourceLabel, GpuResourceProvenance, PreparedGpuData, StorageData, UniformData,
/// };
/// use engine::plugins::render::{
///     GpuParams, prepare_render_storage, prepare_render_uniform,
/// };
/// struct Params(u32);
/// impl GpuParams for Params {
///     type Raw = u32;
///     fn to_gpu(&self) -> Self::Raw { self.0 }
/// }
/// let label = GpuResourceLabel::new("render params")?;
/// let provenance = GpuResourceProvenance::new(label, None, None);
/// let uniform: PreparedGpuData<UniformData> =
///     prepare_render_uniform("render params", &Params(7), provenance.clone())?;
/// let storage: PreparedGpuData<StorageData> =
///     prepare_render_storage("render params", &Params(7), provenance)?;
/// assert_eq!(uniform.as_bytes(), storage.as_bytes());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn prepare_render_uniform<Params: GpuParams + 'static>(
    label: impl Into<String>,
    params: &Params,
    provenance: GpuResourceProvenance,
) -> Result<PreparedGpuData<UniformData>, GpuDataPreparationError> {
    prepare_gpu_data(label, params, provenance, &RenderUniformEncoder)
}

/// Transitional facade lowering into an explicit storage-purpose encoder.
pub fn prepare_render_storage<Params: GpuParams + 'static>(
    label: impl Into<String>,
    params: &Params,
    provenance: GpuResourceProvenance,
) -> Result<PreparedGpuData<StorageData>, GpuDataPreparationError> {
    prepare_gpu_data(label, params, provenance, &RenderStorageEncoder)
}

pub(crate) fn prepare_projected_uniform_bytes(
    label: impl Into<String>,
    bytes: Vec<u8>,
    layout: RenderGpuParamsLayout,
    provenance: GpuResourceProvenance,
) -> Result<PreparedGpuData<UniformData>, GpuDataPreparationError> {
    PreparedGpuData::<UniformData>::from_render_adapter(
        label,
        bytes,
        layout.gpu(),
        provenance,
        Some(layout.params_type_name()),
    )
}
