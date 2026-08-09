use super::super::contract_diagnostics::{GpuProgramContractCause, GpuProgramContractError};
use super::{
    GpuBindingClass, GpuBindingKey, GpuShaderStage, GpuStorageBufferAccess,
    GpuStorageTextureAccess, GpuTextureViewDimension,
};
use crate::plugins::gpu::GpuTextureFormat;
use core::num::{NonZeroU32, NonZeroU64};

/// Stages that statically use one reflected resource binding.
///
/// Unlike declared [`super::GpuShaderStages`], an observed set may be empty:
/// canonical WGSL can declare a resource that no selected entry point uses.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuObservedShaderStages(u8);

impl GpuObservedShaderStages {
    const COMPUTE_BIT: u8 = 1 << 0;
    const VERTEX_BIT: u8 = 1 << 1;
    const FRAGMENT_BIT: u8 = 1 << 2;

    pub fn new(stages: impl IntoIterator<Item = GpuShaderStage>) -> Self {
        let mut bits = 0u8;
        for stage in stages {
            bits |= match stage {
                GpuShaderStage::Compute => Self::COMPUTE_BIT,
                GpuShaderStage::Vertex => Self::VERTEX_BIT,
                GpuShaderStage::Fragment => Self::FRAGMENT_BIT,
            };
        }
        Self(bits)
    }

    pub const fn contains(self, stage: GpuShaderStage) -> bool {
        let bit = match stage {
            GpuShaderStage::Compute => Self::COMPUTE_BIT,
            GpuShaderStage::Vertex => Self::VERTEX_BIT,
            GpuShaderStage::Fragment => Self::FRAGMENT_BIT,
        };
        self.0 & bit != 0
    }

    pub fn iter(self) -> impl Iterator<Item = GpuShaderStage> {
        [
            GpuShaderStage::Compute,
            GpuShaderStage::Vertex,
            GpuShaderStage::Fragment,
        ]
        .into_iter()
        .filter(move |stage| self.contains(*stage))
    }
}

/// Numeric sampled-texture facts that canonical WGSL can establish.
///
/// Float sampling does not encode filtering policy in WGSL, so it deliberately
/// remains one observed fact rather than duplicating the explicit layout's
/// filtering and non-filtering alternatives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuObservedTextureSampleClass {
    Float,
    Depth,
    Sint,
    Uint,
}

/// Sampler comparison semantics that canonical WGSL can establish.
///
/// Ordinary WGSL sampler declarations do not distinguish filtering from
/// non-filtering layout policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuObservedSamplerClass {
    NonComparison,
    Comparison,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum GpuObservedBindingKindInner {
    UniformBuffer {
        minimum_size: Option<NonZeroU64>,
    },
    StorageBuffer {
        access: GpuStorageBufferAccess,
        minimum_size: Option<NonZeroU64>,
    },
    SampledTexture {
        sample_class: GpuObservedTextureSampleClass,
        view_dimension: GpuTextureViewDimension,
        multisampled: bool,
    },
    StorageTexture {
        access: GpuStorageTextureAccess,
        format: GpuTextureFormat,
        view_dimension: GpuTextureViewDimension,
    },
    Sampler {
        class: GpuObservedSamplerClass,
    },
}

/// Backend-neutral facts a WGSL reflection path can establish for one resource.
///
/// Dynamic-offset policy is intentionally absent: it is explicit layout/runtime
/// policy rather than shader-reflected authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuObservedBindingKind(GpuObservedBindingKindInner);

impl GpuObservedBindingKind {
    pub const fn uniform_buffer(minimum_size: Option<NonZeroU64>) -> Self {
        Self(GpuObservedBindingKindInner::UniformBuffer { minimum_size })
    }

    pub const fn storage_buffer(
        access: GpuStorageBufferAccess,
        minimum_size: Option<NonZeroU64>,
    ) -> Self {
        Self(GpuObservedBindingKindInner::StorageBuffer {
            access,
            minimum_size,
        })
    }

    pub fn sampled_texture(
        sample_class: GpuObservedTextureSampleClass,
        view_dimension: GpuTextureViewDimension,
        multisampled: bool,
    ) -> Result<Self, GpuProgramContractError> {
        if multisampled && view_dimension != GpuTextureViewDimension::D2 {
            return Err(GpuProgramContractError::invalid(
                "construct observed GPU sampled-texture binding",
                format!("view_dimension={view_dimension:?}"),
                GpuProgramContractCause::ProgramInterfaceMismatch,
                "normalize multisampled observed textures with D2 view dimension",
            ));
        }
        Ok(Self(GpuObservedBindingKindInner::SampledTexture {
            sample_class,
            view_dimension,
            multisampled,
        }))
    }

    pub fn storage_texture(
        access: GpuStorageTextureAccess,
        format: GpuTextureFormat,
        view_dimension: GpuTextureViewDimension,
    ) -> Result<Self, GpuProgramContractError> {
        if format.is_depth()
            || matches!(
                view_dimension,
                GpuTextureViewDimension::Cube | GpuTextureViewDimension::CubeArray
            )
        {
            return Err(GpuProgramContractError::invalid(
                "construct observed GPU storage-texture binding",
                format!("format={format:?}, view_dimension={view_dimension:?}"),
                GpuProgramContractCause::ProgramInterfaceMismatch,
                "normalize only supported non-depth observed storage-texture facts",
            ));
        }
        Ok(Self(GpuObservedBindingKindInner::StorageTexture {
            access,
            format,
            view_dimension,
        }))
    }

    pub const fn sampler(class: GpuObservedSamplerClass) -> Self {
        Self(GpuObservedBindingKindInner::Sampler { class })
    }

    pub const fn class(self) -> GpuBindingClass {
        match self.0 {
            GpuObservedBindingKindInner::UniformBuffer { .. } => GpuBindingClass::UniformBuffer,
            GpuObservedBindingKindInner::StorageBuffer { .. } => GpuBindingClass::StorageBuffer,
            GpuObservedBindingKindInner::SampledTexture { .. } => GpuBindingClass::SampledTexture,
            GpuObservedBindingKindInner::StorageTexture { .. } => GpuBindingClass::StorageTexture,
            GpuObservedBindingKindInner::Sampler { .. } => GpuBindingClass::Sampler,
        }
    }

    pub const fn minimum_buffer_size(self) -> Option<NonZeroU64> {
        match self.0 {
            GpuObservedBindingKindInner::UniformBuffer { minimum_size }
            | GpuObservedBindingKindInner::StorageBuffer { minimum_size, .. } => minimum_size,
            _ => None,
        }
    }

    pub const fn storage_buffer_access(self) -> Option<GpuStorageBufferAccess> {
        match self.0 {
            GpuObservedBindingKindInner::StorageBuffer { access, .. } => Some(access),
            _ => None,
        }
    }

    pub const fn texture_view_dimension(self) -> Option<GpuTextureViewDimension> {
        match self.0 {
            GpuObservedBindingKindInner::SampledTexture { view_dimension, .. }
            | GpuObservedBindingKindInner::StorageTexture { view_dimension, .. } => {
                Some(view_dimension)
            }
            _ => None,
        }
    }

    pub const fn texture_sample_class(self) -> Option<GpuObservedTextureSampleClass> {
        match self.0 {
            GpuObservedBindingKindInner::SampledTexture { sample_class, .. } => Some(sample_class),
            _ => None,
        }
    }

    pub const fn is_multisampled_texture(self) -> bool {
        matches!(
            self.0,
            GpuObservedBindingKindInner::SampledTexture {
                multisampled: true,
                ..
            }
        )
    }

    pub const fn storage_texture_access(self) -> Option<GpuStorageTextureAccess> {
        match self.0 {
            GpuObservedBindingKindInner::StorageTexture { access, .. } => Some(access),
            _ => None,
        }
    }

    pub const fn storage_texture_format(self) -> Option<GpuTextureFormat> {
        match self.0 {
            GpuObservedBindingKindInner::StorageTexture { format, .. } => Some(format),
            _ => None,
        }
    }

    pub const fn sampler_class(self) -> Option<GpuObservedSamplerClass> {
        match self.0 {
            GpuObservedBindingKindInner::Sampler { class } => Some(class),
            _ => None,
        }
    }
}

/// One normalized, shader-reflected resource binding.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuObservedBindingDeclaration {
    key: GpuBindingKey,
    kind: GpuObservedBindingKind,
    array_count: Option<NonZeroU32>,
    statically_used_stages: GpuObservedShaderStages,
}

impl GpuObservedBindingDeclaration {
    pub const fn new(
        key: GpuBindingKey,
        kind: GpuObservedBindingKind,
        array_count: Option<NonZeroU32>,
        statically_used_stages: GpuObservedShaderStages,
    ) -> Self {
        Self {
            key,
            kind,
            array_count,
            statically_used_stages,
        }
    }

    pub const fn key(&self) -> GpuBindingKey {
        self.key
    }

    pub const fn kind(&self) -> GpuObservedBindingKind {
        self.kind
    }

    pub const fn array_count(&self) -> Option<NonZeroU32> {
        self.array_count
    }

    pub const fn statically_used_stages(&self) -> GpuObservedShaderStages {
        self.statically_used_stages
    }
}

/// Ordered, reflection-only resource-interface evidence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuObservedProgramInterface {
    bindings: Vec<GpuObservedBindingDeclaration>,
}

impl GpuObservedProgramInterface {
    pub fn new(
        bindings: impl IntoIterator<Item = GpuObservedBindingDeclaration>,
    ) -> Result<Self, GpuProgramContractError> {
        let mut bindings = bindings.into_iter().collect::<Vec<_>>();
        bindings.sort_by_key(GpuObservedBindingDeclaration::key);
        if let Some(duplicate) = bindings
            .windows(2)
            .find(|pair| pair[0].key() == pair[1].key())
            .map(|pair| pair[0].key())
        {
            return Err(GpuProgramContractError::invalid(
                "construct observed GPU program resource interface",
                duplicate.to_string(),
                GpuProgramContractCause::DuplicateBindingKey,
                "normalize each observed group/binding key exactly once",
            ));
        }
        Ok(Self { bindings })
    }

    pub fn bindings(&self) -> impl ExactSizeIterator<Item = &GpuObservedBindingDeclaration> {
        self.bindings.iter()
    }

    pub fn binding(&self, key: GpuBindingKey) -> Option<&GpuObservedBindingDeclaration> {
        self.bindings
            .binary_search_by_key(&key, GpuObservedBindingDeclaration::key)
            .ok()
            .map(|index| &self.bindings[index])
    }
}
