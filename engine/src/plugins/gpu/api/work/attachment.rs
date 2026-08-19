use super::mip_extent;
use super::super::{
    GpuAttachmentLoadKind, GpuAttachmentStore, GpuDepthStencilAccess, GpuTextureAccess,
    GpuTextureAccessKind, GpuTextureAccessResource, GpuTextureAspect, GpuTextureFormat,
    GpuTextureSubresourceRange, GpuWorkOperationCause, GpuWorkOperationError,
};
use core::fmt;
use core::hash::{Hash, Hasher};

#[derive(Clone, Copy)]
pub struct GpuColorClearValue {
    bits: [u64; 4],
}

impl GpuColorClearValue {
    pub fn new(red: f64, green: f64, blue: f64, alpha: f64) -> Result<Self, GpuWorkOperationError> {
        Self::from_array([red, green, blue, alpha])
    }

    pub fn from_array(components: [f64; 4]) -> Result<Self, GpuWorkOperationError> {
        let mut bits = [0; 4];
        for (index, component) in components.into_iter().enumerate() {
            if !component.is_finite() {
                return Err(GpuWorkOperationError::invalid(
                    "construct GPU color clear value",
                    format!("component {index}"),
                    None,
                    GpuWorkOperationCause::NonFiniteClearValue,
                    "provide four finite components",
                ));
            }
            bits[index] = canonical_f64_bits(component);
        }
        Ok(Self { bits })
    }

    pub fn components(self) -> [f64; 4] {
        self.bits.map(f64::from_bits)
    }
}

impl fmt::Debug for GpuColorClearValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("GpuColorClearValue")
            .field(&self.components())
            .finish()
    }
}

impl PartialEq for GpuColorClearValue {
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits
    }
}

impl Eq for GpuColorClearValue {}

impl PartialOrd for GpuColorClearValue {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GpuColorClearValue {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.bits.cmp(&other.bits)
    }
}

impl Hash for GpuColorClearValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bits.hash(state);
    }
}

#[derive(Clone, Copy)]
pub struct GpuDepthClearValue {
    bits: u32,
}

impl GpuDepthClearValue {
    pub fn new(value: f32) -> Result<Self, GpuWorkOperationError> {
        if !value.is_finite() {
            return Err(GpuWorkOperationError::invalid(
                "construct GPU depth clear value",
                "depth",
                None,
                GpuWorkOperationCause::NonFiniteClearValue,
                "provide a finite normalized depth value",
            ));
        }
        if !(0.0..=1.0).contains(&value) {
            return Err(GpuWorkOperationError::invalid(
                "construct GPU depth clear value",
                "depth",
                None,
                GpuWorkOperationCause::OutOfRangeClearValue,
                "keep depth inside 0.0 through 1.0",
            ));
        }
        Ok(Self {
            bits: canonical_f32_bits(value),
        })
    }

    pub fn value(self) -> f32 {
        f32::from_bits(self.bits)
    }
}

impl fmt::Debug for GpuDepthClearValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("GpuDepthClearValue")
            .field(&self.value())
            .finish()
    }
}

impl PartialEq for GpuDepthClearValue {
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits
    }
}

impl Eq for GpuDepthClearValue {}

impl PartialOrd for GpuDepthClearValue {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GpuDepthClearValue {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.bits.cmp(&other.bits)
    }
}

impl Hash for GpuDepthClearValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bits.hash(state);
    }
}

const fn canonical_f64_bits(value: f64) -> u64 {
    if value == 0.0 {
        0.0_f64.to_bits()
    } else {
        value.to_bits()
    }
}

const fn canonical_f32_bits(value: f32) -> u32 {
    if value == 0.0 {
        0.0_f32.to_bits()
    } else {
        value.to_bits()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuColorAttachmentLoad {
    Load,
    Clear(GpuColorClearValue),
}

impl GpuColorAttachmentLoad {
    pub const fn kind(self) -> GpuAttachmentLoadKind {
        match self {
            Self::Load => GpuAttachmentLoadKind::Load,
            Self::Clear(_) => GpuAttachmentLoadKind::Clear,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuDepthAttachmentLoad {
    Load,
    Clear(GpuDepthClearValue),
}

impl GpuDepthAttachmentLoad {
    pub const fn kind(self) -> GpuAttachmentLoadKind {
        match self {
            Self::Load => GpuAttachmentLoadKind::Load,
            Self::Clear(_) => GpuAttachmentLoadKind::Clear,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuMultisampleResolveTarget {
    destination: GpuTextureAccessResource,
    subresources: GpuTextureSubresourceRange,
    access: GpuTextureAccess,
}

impl GpuMultisampleResolveTarget {
    pub fn new(
        destination: GpuTextureAccessResource,
        subresources: GpuTextureSubresourceRange,
    ) -> Result<Self, GpuWorkOperationError> {
        let label = destination
            .parent_texture()
            .descriptor()
            .common()
            .label()
            .as_str()
            .to_string();
        let access = GpuTextureAccess::new(
            destination.clone(),
            subresources,
            GpuTextureAccessKind::MultisampleResolveDestination,
        )
        .map_err(|source| {
            GpuWorkOperationError::from_access(
                "construct GPU multisample resolve target",
                label,
                GpuWorkOperationCause::InvalidMultisampleResolve,
                "provide a checked single-sampled color-attachment destination",
                source,
            )
        })?;
        Ok(Self {
            destination,
            subresources,
            access,
        })
    }

    pub fn destination(&self) -> &GpuTextureAccessResource {
        &self.destination
    }

    pub const fn subresources(&self) -> GpuTextureSubresourceRange {
        self.subresources
    }

    pub fn access(&self) -> &GpuTextureAccess {
        &self.access
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuRenderColorAttachment {
    source: GpuTextureAccessResource,
    subresources: GpuTextureSubresourceRange,
    load: GpuColorAttachmentLoad,
    store: GpuAttachmentStore,
    resolve_target: Option<GpuMultisampleResolveTarget>,
    source_access: GpuTextureAccess,
}

impl GpuRenderColorAttachment {
    pub fn new(
        source: GpuTextureAccessResource,
        subresources: GpuTextureSubresourceRange,
        load: GpuColorAttachmentLoad,
        store: GpuAttachmentStore,
        resolve_target: Option<GpuMultisampleResolveTarget>,
    ) -> Result<Self, GpuWorkOperationError> {
        let label = source
            .parent_texture()
            .descriptor()
            .common()
            .label()
            .as_str()
            .to_string();
        let source_access = GpuTextureAccess::new(
            source.clone(),
            subresources,
            GpuTextureAccessKind::ColorAttachment {
                load_kind: load.kind(),
                store,
            },
        )
        .map_err(|source| {
            GpuWorkOperationError::from_access(
                "construct GPU render color attachment",
                label.clone(),
                GpuWorkOperationCause::InvalidAttachment,
                "provide a checked color attachment with compatible descriptor usage",
                source,
            )
        })?;
        if let Some(resolve) = &resolve_target {
            validate_multisample_resolve(&source_access, resolve)?;
        }
        Ok(Self {
            source,
            subresources,
            load,
            store,
            resolve_target,
            source_access,
        })
    }

    pub fn source(&self) -> &GpuTextureAccessResource {
        &self.source
    }

    pub const fn subresources(&self) -> GpuTextureSubresourceRange {
        self.subresources
    }

    pub const fn load(&self) -> GpuColorAttachmentLoad {
        self.load
    }

    pub const fn store(&self) -> GpuAttachmentStore {
        self.store
    }

    pub fn resolve_target(&self) -> Option<&GpuMultisampleResolveTarget> {
        self.resolve_target.as_ref()
    }

    pub fn source_access(&self) -> &GpuTextureAccess {
        &self.source_access
    }
}

fn validate_multisample_resolve(
    source: &GpuTextureAccess,
    destination: &GpuMultisampleResolveTarget,
) -> Result<(), GpuWorkOperationError> {
    let source_texture = source.normalized_texture();
    let destination_texture = destination.access().normalized_texture();
    let label = source_texture
        .descriptor()
        .common()
        .label()
        .as_str()
        .to_string();
    let source_range = source.normalized_subresources();
    let destination_range = destination.access().normalized_subresources();
    let same_shape = source_range.mip_level_count() == destination_range.mip_level_count()
        && source_range.array_layer_count() == destination_range.array_layer_count()
        && source_range.aspect() == GpuTextureAspect::Color
        && destination_range.aspect() == GpuTextureAspect::Color
        && mip_extent(source_texture, source_range.base_mip_level())
            == mip_extent(destination_texture, destination_range.base_mip_level());
    let valid = source_texture.descriptor().sample_count() > 1
        && destination_texture.descriptor().sample_count() == 1
        && effective_texture_format(source.resource())
            == effective_texture_format(destination.destination())
        && source_texture.descriptor().dimension() == destination_texture.descriptor().dimension()
        && same_shape
        && source_texture != destination_texture;
    if !valid {
        return Err(GpuWorkOperationError::invalid(
            "validate GPU multisample resolve",
            label,
            Some(source_texture.diagnostic_identity()),
            GpuWorkOperationCause::InvalidMultisampleResolve,
            "use non-aliasing multisampled source and single-sampled destination attachments with matching color format, extent, and subresources",
        ));
    }
    Ok(())
}

fn effective_texture_format(resource: &GpuTextureAccessResource) -> GpuTextureFormat {
    match resource {
        GpuTextureAccessResource::Texture(texture) => texture.descriptor().format(),
        GpuTextureAccessResource::TextureView(view) => view
            .descriptor()
            .format()
            .unwrap_or_else(|| view.descriptor().texture().descriptor().format()),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuRenderDepthStencilAttachment {
    source: GpuTextureAccessResource,
    subresources: GpuTextureSubresourceRange,
    access: GpuDepthStencilAccess,
    load: GpuDepthAttachmentLoad,
    store: GpuAttachmentStore,
    source_access: GpuTextureAccess,
}

impl GpuRenderDepthStencilAttachment {
    pub fn new(
        source: GpuTextureAccessResource,
        subresources: GpuTextureSubresourceRange,
        access: GpuDepthStencilAccess,
        load: GpuDepthAttachmentLoad,
        store: GpuAttachmentStore,
    ) -> Result<Self, GpuWorkOperationError> {
        let label = source
            .parent_texture()
            .descriptor()
            .common()
            .label()
            .as_str()
            .to_string();
        if access == GpuDepthStencilAccess::ReadOnly
            && matches!(load, GpuDepthAttachmentLoad::Clear(_))
        {
            return Err(GpuWorkOperationError::invalid(
                "construct GPU render depth attachment",
                label,
                Some(source.parent_texture().diagnostic_identity()),
                GpuWorkOperationCause::InvalidAttachment,
                "use Load for read-only depth or select read-write depth access before clearing",
            ));
        }
        let source_access = GpuTextureAccess::new(
            source.clone(),
            subresources,
            GpuTextureAccessKind::DepthStencilAttachment {
                access,
                load_kind: load.kind(),
                store,
            },
        )
        .map_err(|source| {
            GpuWorkOperationError::from_access(
                "construct GPU render depth attachment",
                label,
                GpuWorkOperationCause::InvalidAttachment,
                "provide a checked depth attachment with compatible descriptor usage",
                source,
            )
        })?;
        Ok(Self {
            source,
            subresources,
            access,
            load,
            store,
            source_access,
        })
    }

    pub fn source(&self) -> &GpuTextureAccessResource {
        &self.source
    }

    pub const fn subresources(&self) -> GpuTextureSubresourceRange {
        self.subresources
    }

    pub const fn access(&self) -> GpuDepthStencilAccess {
        self.access
    }

    pub const fn load(&self) -> GpuDepthAttachmentLoad {
        self.load
    }

    pub const fn store(&self) -> GpuAttachmentStore {
        self.store
    }

    pub fn source_access(&self) -> &GpuTextureAccess {
        &self.source_access
    }
}
