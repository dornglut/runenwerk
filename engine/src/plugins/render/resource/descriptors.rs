use crate::plugins::render::GpuParams;
use crate::plugins::render::api::RenderResourceId;
use crate::plugins::render::resource::ResourceLifetime;
use std::any::{TypeId, type_name};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniformBufferDescriptor {
    pub id: RenderResourceId,
    pub params_type_id: TypeId,
    pub params_type_name: &'static str,
    pub size_bytes: u64,
    pub lifetime: ResourceLifetime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageBufferDescriptor {
    pub id: RenderResourceId,
    pub params_type_id: TypeId,
    pub params_type_name: &'static str,
    pub size_bytes: u64,
    pub lifetime: ResourceLifetime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampledTextureDescriptor {
    pub id: RenderResourceId,
    pub lifetime: ResourceLifetime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageTextureDescriptor {
    pub id: RenderResourceId,
    pub lifetime: ResourceLifetime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorTargetDescriptor {
    pub id: RenderResourceId,
    pub lifetime: ResourceLifetime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepthTargetDescriptor {
    pub id: RenderResourceId,
    pub lifetime: ResourceLifetime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryTextureDescriptor {
    pub id: RenderResourceId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedTextureDescriptor {
    pub id: RenderResourceId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedBufferDescriptor {
    pub id: RenderResourceId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderResourceDescriptor {
    UniformBuffer(UniformBufferDescriptor),
    StorageBuffer(StorageBufferDescriptor),
    SampledTexture(SampledTextureDescriptor),
    StorageTexture(StorageTextureDescriptor),
    ColorTarget(ColorTargetDescriptor),
    DepthTarget(DepthTargetDescriptor),
    HistoryTexture(HistoryTextureDescriptor),
    ImportedTexture(ImportedTextureDescriptor),
    ImportedBuffer(ImportedBufferDescriptor),
}

impl RenderResourceDescriptor {
    pub fn uniform_buffer<T>(id: impl Into<RenderResourceId>) -> Self
    where
        T: GpuParams + 'static,
    {
        Self::uniform_buffer_with_lifetime::<T>(id, ResourceLifetime::Persistent)
    }

    pub fn uniform_buffer_with_lifetime<T>(
        id: impl Into<RenderResourceId>,
        lifetime: ResourceLifetime,
    ) -> Self
    where
        T: GpuParams + 'static,
    {
        Self::UniformBuffer(UniformBufferDescriptor {
            id: id.into(),
            params_type_id: TypeId::of::<T>(),
            params_type_name: type_name::<T>(),
            size_bytes: std::mem::size_of::<T::Raw>().max(1) as u64,
            lifetime,
        })
    }

    pub fn storage_buffer<T>(id: impl Into<RenderResourceId>) -> Self
    where
        T: GpuParams + 'static,
    {
        Self::storage_buffer_with_lifetime::<T>(id, ResourceLifetime::Persistent)
    }

    pub fn storage_buffer_with_lifetime<T>(
        id: impl Into<RenderResourceId>,
        lifetime: ResourceLifetime,
    ) -> Self
    where
        T: GpuParams + 'static,
    {
        Self::StorageBuffer(StorageBufferDescriptor {
            id: id.into(),
            params_type_id: TypeId::of::<T>(),
            params_type_name: type_name::<T>(),
            size_bytes: std::mem::size_of::<T::Raw>().max(1) as u64,
            lifetime,
        })
    }

    pub fn sampled_texture(id: impl Into<RenderResourceId>) -> Self {
        Self::sampled_texture_with_lifetime(id, ResourceLifetime::Persistent)
    }

    pub fn sampled_texture_with_lifetime(
        id: impl Into<RenderResourceId>,
        lifetime: ResourceLifetime,
    ) -> Self {
        Self::SampledTexture(SampledTextureDescriptor {
            id: id.into(),
            lifetime,
        })
    }

    pub fn storage_texture(id: impl Into<RenderResourceId>) -> Self {
        Self::storage_texture_with_lifetime(id, ResourceLifetime::Persistent)
    }

    pub fn storage_texture_with_lifetime(
        id: impl Into<RenderResourceId>,
        lifetime: ResourceLifetime,
    ) -> Self {
        Self::StorageTexture(StorageTextureDescriptor {
            id: id.into(),
            lifetime,
        })
    }

    pub fn color_target(id: impl Into<RenderResourceId>) -> Self {
        Self::color_target_with_lifetime(id, ResourceLifetime::Persistent)
    }

    pub fn color_target_with_lifetime(
        id: impl Into<RenderResourceId>,
        lifetime: ResourceLifetime,
    ) -> Self {
        Self::ColorTarget(ColorTargetDescriptor {
            id: id.into(),
            lifetime,
        })
    }

    pub fn depth_target(id: impl Into<RenderResourceId>) -> Self {
        Self::depth_target_with_lifetime(id, ResourceLifetime::Persistent)
    }

    pub fn depth_target_with_lifetime(
        id: impl Into<RenderResourceId>,
        lifetime: ResourceLifetime,
    ) -> Self {
        Self::DepthTarget(DepthTargetDescriptor {
            id: id.into(),
            lifetime,
        })
    }

    pub fn imported_texture(id: impl Into<RenderResourceId>) -> Self {
        Self::ImportedTexture(ImportedTextureDescriptor { id: id.into() })
    }

    pub fn history_texture(id: impl Into<RenderResourceId>) -> Self {
        Self::HistoryTexture(HistoryTextureDescriptor { id: id.into() })
    }

    pub fn imported_buffer(id: impl Into<RenderResourceId>) -> Self {
        Self::ImportedBuffer(ImportedBufferDescriptor { id: id.into() })
    }

    pub fn id(&self) -> &RenderResourceId {
        match self {
            Self::UniformBuffer(value) => &value.id,
            Self::StorageBuffer(value) => &value.id,
            Self::SampledTexture(value) => &value.id,
            Self::StorageTexture(value) => &value.id,
            Self::ColorTarget(value) => &value.id,
            Self::DepthTarget(value) => &value.id,
            Self::HistoryTexture(value) => &value.id,
            Self::ImportedTexture(value) => &value.id,
            Self::ImportedBuffer(value) => &value.id,
        }
    }

    pub fn lifetime(&self) -> ResourceLifetime {
        match self {
            Self::UniformBuffer(value) => value.lifetime,
            Self::StorageBuffer(value) => value.lifetime,
            Self::SampledTexture(value) => value.lifetime,
            Self::StorageTexture(value) => value.lifetime,
            Self::ColorTarget(value) => value.lifetime,
            Self::DepthTarget(value) => value.lifetime,
            Self::HistoryTexture(_) => ResourceLifetime::Persistent,
            Self::ImportedTexture(_) | Self::ImportedBuffer(_) => ResourceLifetime::Imported,
        }
    }
}

pub fn detect_duplicate_resource_ids(descriptors: &[RenderResourceDescriptor]) -> Vec<String> {
    let mut seen = BTreeSet::<String>::new();
    let mut duplicates = BTreeSet::<String>::new();

    for descriptor in descriptors {
        let id = descriptor.id().as_str().to_string();
        if !seen.insert(id.clone()) {
            duplicates.insert(id);
        }
    }

    duplicates.into_iter().collect()
}
