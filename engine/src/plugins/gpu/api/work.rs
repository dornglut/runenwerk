use super::{
    GpuAttachmentLoadKind, GpuAttachmentStore, GpuBufferAccess, GpuBufferAccessKind,
    GpuBufferHandle, GpuBufferRange, GpuCapabilityFeature, GpuCapabilityRequirement,
    GpuCapabilityRequirementError, GpuCapabilityRequirements, GpuComputePipelineDescriptor,
    GpuDepthStencilAccess, GpuDispatchIntent, GpuQueryAccess, GpuQueryAccessKind, GpuQueryKind,
    GpuQueryRange, GpuQuerySetHandle, GpuResourceAccess, GpuRuntimeBindingSet, GpuTextureAccess,
    GpuTextureAccessKind, GpuTextureAccessResource, GpuTextureAspect, GpuTextureDimension,
    GpuTextureFormat, GpuTextureHandle, GpuTextureSubresourceRange, GpuWorkOperationCause,
    GpuWorkOperationError, GpuWorkResourceId,
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

fn mip_extent(texture: &GpuTextureHandle, mip_level: u32) -> (u32, u32, u32) {
    let descriptor = texture.descriptor();
    let extent = descriptor.extent();
    let width = (extent.width() >> mip_level).max(1);
    let height = (extent.height() >> mip_level).max(1);
    let depth = match descriptor.dimension() {
        GpuTextureDimension::D3 => (extent.depth_or_layers() >> mip_level).max(1),
        GpuTextureDimension::D2 => extent.depth_or_layers(),
        GpuTextureDimension::D1 => 1,
    };
    (width, height, depth)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuDispatchSize {
    x: u32,
    y: u32,
    z: u32,
}

impl GpuDispatchSize {
    pub const fn new(x: u32, y: u32, z: u32) -> Result<Self, GpuWorkOperationError> {
        Ok(Self { x, y, z })
    }

    pub const fn x(self) -> u32 {
        self.x
    }
    pub const fn y(self) -> u32 {
        self.y
    }
    pub const fn z(self) -> u32 {
        self.z
    }
    pub const fn as_array(self) -> [u32; 3] {
        [self.x, self.y, self.z]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuDrawRange {
    first: u32,
    count: u32,
}

impl GpuDrawRange {
    pub fn new(first: u32, count: u32) -> Result<Self, GpuWorkOperationError> {
        if count == 0 {
            return Err(GpuWorkOperationError::invalid(
                "construct GPU draw range",
                "draw",
                None,
                GpuWorkOperationCause::ZeroDrawCount,
                "provide a nonzero draw count",
            ));
        }
        first.checked_add(count).ok_or_else(|| {
            GpuWorkOperationError::invalid(
                "construct GPU draw range",
                "draw",
                None,
                GpuWorkOperationCause::InvalidDraw,
                "reduce the first element or count",
            )
        })?;
        Ok(Self { first, count })
    }

    pub const fn first(self) -> u32 {
        self.first
    }

    pub const fn count(self) -> u32 {
        self.count
    }

    pub const fn end(self) -> u32 {
        self.first + self.count
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuDrawIntent {
    Direct {
        vertices: GpuDrawRange,
        instances: GpuDrawRange,
    },
    Indexed {
        indices: GpuDrawRange,
        base_vertex: i32,
        instances: GpuDrawRange,
    },
    Indirect {
        arguments: GpuBufferHandle,
        range: GpuBufferRange,
        indexed: bool,
    },
}

impl GpuDrawIntent {
    pub fn direct(vertices: GpuDrawRange, instances: GpuDrawRange) -> Self {
        Self::Direct {
            vertices,
            instances,
        }
    }

    pub fn indexed(indices: GpuDrawRange, base_vertex: i32, instances: GpuDrawRange) -> Self {
        Self::Indexed {
            indices,
            base_vertex,
            instances,
        }
    }

    pub fn indirect(
        arguments: &GpuBufferHandle,
        range: GpuBufferRange,
        indexed: bool,
    ) -> Result<Self, GpuWorkOperationError> {
        let expected_size = if indexed { 20 } else { 16 };
        if !range.offset().is_multiple_of(4) || range.size() != expected_size {
            return Err(GpuWorkOperationError::invalid(
                "construct GPU indirect draw intent",
                arguments.descriptor().common().label().as_str(),
                Some(arguments.diagnostic_identity()),
                GpuWorkOperationCause::InvalidDraw,
                "use one four-byte-aligned direct (16-byte) or indexed (20-byte) argument record",
            ));
        }
        GpuBufferAccess::new(arguments, range, GpuBufferAccessKind::IndirectRead).map_err(
            |source| {
                GpuWorkOperationError::from_access(
                    "construct GPU indirect draw intent",
                    arguments.descriptor().common().label().as_str(),
                    GpuWorkOperationCause::InvalidDraw,
                    "declare Indirect usage and a checked argument record",
                    source,
                )
            },
        )?;
        Ok(Self::Indirect {
            arguments: arguments.clone(),
            range,
            indexed,
        })
    }

    pub const fn is_indexed(&self) -> bool {
        matches!(
            self,
            Self::Indexed { .. } | Self::Indirect { indexed: true, .. }
        )
    }

    pub fn derived_access(&self) -> Result<Option<GpuBufferAccess>, GpuWorkOperationError> {
        match self {
            Self::Indirect {
                arguments, range, ..
            } => GpuBufferAccess::new(arguments, *range, GpuBufferAccessKind::IndirectRead)
                .map(Some)
                .map_err(|source| {
                    GpuWorkOperationError::from_access(
                        "derive GPU indirect draw access",
                        arguments.descriptor().common().label().as_str(),
                        GpuWorkOperationCause::OperationAccessContradiction,
                        "construct indirect draws through the checked constructor",
                        source,
                    )
                }),
            Self::Direct { .. } | Self::Indexed { .. } => Ok(None),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GpuComputeOperation {
    pipeline: GpuComputePipelineDescriptor,
    bindings: GpuRuntimeBindingSet,
    dispatch: GpuDispatchIntent,
    timestamp_writes: Vec<GpuQueryAccess>,
}

impl GpuComputeOperation {
    pub fn new(
        pipeline: GpuComputePipelineDescriptor,
        bindings: GpuRuntimeBindingSet,
        dispatch: GpuDispatchIntent,
    ) -> Result<Self, GpuWorkOperationError> {
        if pipeline.layout() != bindings.layout() {
            return Err(GpuWorkOperationError::invalid(
                "construct GPU compute operation",
                "pipeline bindings",
                None,
                GpuWorkOperationCause::OperationAccessContradiction,
                "use a runtime binding set constructed for the exact compute pipeline layout",
            ));
        }
        Ok(Self {
            pipeline,
            bindings,
            dispatch,
            timestamp_writes: Vec::new(),
        })
    }

    pub fn with_timestamp_writes(
        mut self,
        timestamp_writes: impl IntoIterator<Item = GpuQueryAccess>,
    ) -> Result<Self, GpuWorkOperationError> {
        let timestamp_writes = timestamp_writes.into_iter().collect::<Vec<_>>();
        if timestamp_writes
            .iter()
            .any(|access| access.kind() != GpuQueryAccessKind::WriteTimestamp)
        {
            return Err(GpuWorkOperationError::invalid(
                "construct GPU compute operation",
                "timestamp writes",
                timestamp_writes
                    .first()
                    .map(GpuQueryAccess::resource_identity),
                GpuWorkOperationCause::OperationAccessContradiction,
                "provide only WriteTimestamp query accesses as compute-side timestamp writes",
            ));
        }
        self.timestamp_writes = timestamp_writes;
        Ok(self)
    }

    pub fn pipeline(&self) -> &GpuComputePipelineDescriptor {
        &self.pipeline
    }

    pub fn bindings(&self) -> &GpuRuntimeBindingSet {
        &self.bindings
    }

    pub fn dispatch(&self) -> &GpuDispatchIntent {
        &self.dispatch
    }

    pub fn timestamp_writes(&self) -> &[GpuQueryAccess] {
        &self.timestamp_writes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuTextureOrigin {
    x: u32,
    y: u32,
    z: u32,
}

impl GpuTextureOrigin {
    pub const fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }
    pub const fn x(self) -> u32 {
        self.x
    }
    pub const fn y(self) -> u32 {
        self.y
    }
    pub const fn z(self) -> u32 {
        self.z
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuCopyExtent {
    width: u32,
    height: u32,
    depth_or_layers: u32,
}

impl GpuCopyExtent {
    pub fn new(
        width: u32,
        height: u32,
        depth_or_layers: u32,
    ) -> Result<Self, GpuWorkOperationError> {
        if width == 0 || height == 0 || depth_or_layers == 0 {
            return Err(GpuWorkOperationError::invalid(
                "construct GPU copy extent",
                "copy",
                None,
                GpuWorkOperationCause::InvalidCopyRegion,
                "provide nonzero width, height, and depth-or-layer coverage",
            ));
        }
        Ok(Self {
            width,
            height,
            depth_or_layers,
        })
    }
    pub const fn width(self) -> u32 {
        self.width
    }
    pub const fn height(self) -> u32 {
        self.height
    }
    pub const fn depth_or_layers(self) -> u32 {
        self.depth_or_layers
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuBufferRegion {
    buffer: GpuBufferHandle,
    range: GpuBufferRange,
}

impl GpuBufferRegion {
    pub fn new(
        buffer: &GpuBufferHandle,
        range: GpuBufferRange,
    ) -> Result<Self, GpuWorkOperationError> {
        GpuBufferRange::new(buffer, range.offset(), range.size()).map_err(|source| {
            GpuWorkOperationError::from_access(
                "construct GPU buffer region",
                buffer.descriptor().common().label().as_str(),
                GpuWorkOperationCause::InvalidCopyRegion,
                "provide a checked nonempty buffer region",
                source,
            )
        })?;
        Ok(Self {
            buffer: buffer.clone(),
            range,
        })
    }

    pub fn buffer(&self) -> &GpuBufferHandle {
        &self.buffer
    }

    pub const fn range(&self) -> GpuBufferRange {
        self.range
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuTextureCopyRegion {
    texture: GpuTextureHandle,
    mip_level: u32,
    origin: GpuTextureOrigin,
    aspect: GpuTextureAspect,
    extent: GpuCopyExtent,
    subresources: GpuTextureSubresourceRange,
}

impl GpuTextureCopyRegion {
    pub fn new(
        texture: &GpuTextureHandle,
        mip_level: u32,
        origin: GpuTextureOrigin,
        aspect: GpuTextureAspect,
        extent: GpuCopyExtent,
    ) -> Result<Self, GpuWorkOperationError> {
        let descriptor = texture.descriptor();
        let label = descriptor.common().label().as_str();
        if descriptor.sample_count() != 1 || mip_level >= descriptor.mip_level_count() {
            return Err(GpuWorkOperationError::invalid(
                "construct GPU texture copy region",
                label,
                Some(texture.diagnostic_identity()),
                GpuWorkOperationCause::InvalidCopyRegion,
                "use a valid mip of a single-sampled texture",
            ));
        }
        let (mip_width, mip_height, mip_depth_or_layers) = mip_extent(texture, mip_level);
        let x_end = origin.x().checked_add(extent.width());
        let y_end = origin.y().checked_add(extent.height());
        let z_end = origin.z().checked_add(extent.depth_or_layers());
        let dimension_valid = match descriptor.dimension() {
            GpuTextureDimension::D1 => {
                origin.y() == 0
                    && origin.z() == 0
                    && extent.height() == 1
                    && extent.depth_or_layers() == 1
            }
            GpuTextureDimension::D2 => true,
            GpuTextureDimension::D3 => true,
        };
        let aspect_valid = if descriptor.format().is_depth() {
            matches!(aspect, GpuTextureAspect::All | GpuTextureAspect::DepthOnly)
        } else {
            matches!(aspect, GpuTextureAspect::All | GpuTextureAspect::Color)
        };
        if !dimension_valid
            || !aspect_valid
            || x_end.is_none_or(|end| end > mip_width)
            || y_end.is_none_or(|end| end > mip_height)
            || z_end.is_none_or(|end| end > mip_depth_or_layers)
        {
            return Err(GpuWorkOperationError::invalid(
                "construct GPU texture copy region",
                label,
                Some(texture.diagnostic_identity()),
                GpuWorkOperationCause::InvalidCopyRegion,
                "keep origin, extent, and aspect inside the selected mip",
            ));
        }
        let (base_array_layer, array_layer_count) = match descriptor.dimension() {
            GpuTextureDimension::D2 => (origin.z(), extent.depth_or_layers()),
            GpuTextureDimension::D1 | GpuTextureDimension::D3 => (0, 1),
        };
        let canonical_aspect = if descriptor.format().is_depth() {
            GpuTextureAspect::DepthOnly
        } else {
            GpuTextureAspect::Color
        };
        let subresources = GpuTextureSubresourceRange::new(
            descriptor.common().label(),
            mip_level,
            1,
            base_array_layer,
            array_layer_count,
            canonical_aspect,
        )
        .map_err(|_| {
            GpuWorkOperationError::invalid(
                "construct GPU texture copy region",
                label,
                Some(texture.diagnostic_identity()),
                GpuWorkOperationCause::InvalidCopyRegion,
                "provide a checked texture subresource region",
            )
        })?;
        Ok(Self {
            texture: texture.clone(),
            mip_level,
            origin,
            aspect: canonical_aspect,
            extent,
            subresources,
        })
    }

    pub fn texture(&self) -> &GpuTextureHandle {
        &self.texture
    }
    pub const fn mip_level(&self) -> u32 {
        self.mip_level
    }
    pub const fn origin(&self) -> GpuTextureOrigin {
        self.origin
    }
    pub const fn aspect(&self) -> GpuTextureAspect {
        self.aspect
    }
    pub const fn extent(&self) -> GpuCopyExtent {
        self.extent
    }
    pub const fn subresources(&self) -> GpuTextureSubresourceRange {
        self.subresources
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuBufferTextureLayout {
    buffer: GpuBufferHandle,
    byte_offset: u64,
    bytes_per_row: u32,
    rows_per_image: u32,
}

impl GpuBufferTextureLayout {
    pub fn new(
        buffer: &GpuBufferHandle,
        byte_offset: u64,
        bytes_per_row: u32,
        rows_per_image: u32,
    ) -> Result<Self, GpuWorkOperationError> {
        if bytes_per_row == 0 {
            return Err(GpuWorkOperationError::invalid(
                "construct GPU buffer-texture layout",
                buffer.descriptor().common().label().as_str(),
                Some(buffer.diagnostic_identity()),
                GpuWorkOperationCause::InvalidCopyLayout,
                "provide a nonzero logical bytes-per-row value",
            ));
        }
        if byte_offset >= buffer.descriptor().size_bytes() {
            return Err(GpuWorkOperationError::invalid(
                "construct GPU buffer-texture layout",
                buffer.descriptor().common().label().as_str(),
                Some(buffer.diagnostic_identity()),
                GpuWorkOperationCause::InvalidCopyLayout,
                "keep the byte offset inside the buffer descriptor",
            ));
        }
        Ok(Self {
            buffer: buffer.clone(),
            byte_offset,
            bytes_per_row,
            rows_per_image,
        })
    }

    pub fn buffer(&self) -> &GpuBufferHandle {
        &self.buffer
    }
    pub const fn byte_offset(&self) -> u64 {
        self.byte_offset
    }
    pub const fn bytes_per_row(&self) -> u32 {
        self.bytes_per_row
    }
    pub const fn rows_per_image(&self) -> u32 {
        self.rows_per_image
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuCopyOperation {
    BufferToBuffer {
        source: GpuBufferRegion,
        destination: GpuBufferRegion,
    },
    BufferToTexture {
        source: GpuBufferTextureLayout,
        destination: GpuTextureCopyRegion,
    },
    TextureToBuffer {
        source: GpuTextureCopyRegion,
        destination: GpuBufferTextureLayout,
    },
    TextureToTexture {
        source: GpuTextureCopyRegion,
        destination: GpuTextureCopyRegion,
    },
}

impl GpuCopyOperation {
    pub fn buffer_to_buffer(
        source: GpuBufferRegion,
        destination: GpuBufferRegion,
    ) -> Result<Self, GpuWorkOperationError> {
        if source.range().size() != destination.range().size()
            || (source.buffer() == destination.buffer()
                && source.range().overlaps(destination.range()))
        {
            return Err(copy_error(
                "construct GPU buffer-to-buffer copy",
                source.buffer().diagnostic_identity(),
                "use equal-sized non-overlapping source and destination regions",
            ));
        }
        buffer_access(
            source.buffer(),
            source.range(),
            GpuBufferAccessKind::CopySource,
            "construct GPU buffer-to-buffer copy",
        )?;
        buffer_access(
            destination.buffer(),
            destination.range(),
            GpuBufferAccessKind::CopyDestination,
            "construct GPU buffer-to-buffer copy",
        )?;
        Ok(Self::BufferToBuffer {
            source,
            destination,
        })
    }

    pub fn buffer_to_texture(
        source: GpuBufferTextureLayout,
        destination: GpuTextureCopyRegion,
    ) -> Result<Self, GpuWorkOperationError> {
        validate_buffer_texture_layout(&source, &destination, GpuBufferAccessKind::CopySource)?;
        texture_copy_access(&destination, GpuTextureAccessKind::CopyDestination)?;
        Ok(Self::BufferToTexture {
            source,
            destination,
        })
    }

    pub fn texture_to_buffer(
        source: GpuTextureCopyRegion,
        destination: GpuBufferTextureLayout,
    ) -> Result<Self, GpuWorkOperationError> {
        texture_copy_access(&source, GpuTextureAccessKind::CopySource)?;
        validate_buffer_texture_layout(
            &destination,
            &source,
            GpuBufferAccessKind::CopyDestination,
        )?;
        Ok(Self::TextureToBuffer {
            source,
            destination,
        })
    }

    pub fn texture_to_texture(
        source: GpuTextureCopyRegion,
        destination: GpuTextureCopyRegion,
    ) -> Result<Self, GpuWorkOperationError> {
        let copy_compatible = super::gpu_texture_formats_copy_compatible(
            source.texture().descriptor().format(),
            destination.texture().descriptor().format(),
        );
        let same_extent = source.extent() == destination.extent();
        let aliases = source.texture() == destination.texture()
            && source
                .subresources()
                .overlaps(destination.subresources(), source.aspect());
        if !copy_compatible || !same_extent || source.aspect() != destination.aspect() || aliases {
            return Err(copy_error(
                "construct GPU texture-to-texture copy",
                source.texture().diagnostic_identity(),
                "use copy-compatible formats, matching aspects/extents, and non-overlapping source/destination storage",
            ));
        }
        texture_copy_access(&source, GpuTextureAccessKind::CopySource)?;
        texture_copy_access(&destination, GpuTextureAccessKind::CopyDestination)?;
        Ok(Self::TextureToTexture {
            source,
            destination,
        })
    }

    fn derived_accesses(&self) -> Result<Vec<GpuResourceAccess>, GpuWorkOperationError> {
        match self {
            Self::BufferToBuffer {
                source,
                destination,
            } => Ok(vec![
                GpuResourceAccess::Buffer(buffer_access(
                    source.buffer(),
                    source.range(),
                    GpuBufferAccessKind::CopySource,
                    "derive GPU buffer copy source access",
                )?),
                GpuResourceAccess::Buffer(buffer_access(
                    destination.buffer(),
                    destination.range(),
                    GpuBufferAccessKind::CopyDestination,
                    "derive GPU buffer copy destination access",
                )?),
            ]),
            Self::BufferToTexture {
                source,
                destination,
            } => Ok(vec![
                GpuResourceAccess::Buffer(buffer_layout_access(
                    source,
                    destination,
                    GpuBufferAccessKind::CopySource,
                )?),
                GpuResourceAccess::Texture(texture_copy_access(
                    destination,
                    GpuTextureAccessKind::CopyDestination,
                )?),
            ]),
            Self::TextureToBuffer {
                source,
                destination,
            } => Ok(vec![
                GpuResourceAccess::Texture(texture_copy_access(
                    source,
                    GpuTextureAccessKind::CopySource,
                )?),
                GpuResourceAccess::Buffer(buffer_layout_access(
                    destination,
                    source,
                    GpuBufferAccessKind::CopyDestination,
                )?),
            ]),
            Self::TextureToTexture {
                source,
                destination,
            } => Ok(vec![
                GpuResourceAccess::Texture(texture_copy_access(
                    source,
                    GpuTextureAccessKind::CopySource,
                )?),
                GpuResourceAccess::Texture(texture_copy_access(
                    destination,
                    GpuTextureAccessKind::CopyDestination,
                )?),
            ]),
        }
    }
}

fn buffer_access(
    buffer: &GpuBufferHandle,
    range: GpuBufferRange,
    kind: GpuBufferAccessKind,
    operation: &'static str,
) -> Result<GpuBufferAccess, GpuWorkOperationError> {
    GpuBufferAccess::new(buffer, range, kind).map_err(|source| {
        GpuWorkOperationError::from_access(
            operation,
            buffer.descriptor().common().label().as_str(),
            GpuWorkOperationCause::InvalidCopyRegion,
            "declare matching copy usage and checked coverage",
            source,
        )
    })
}

fn texture_copy_access(
    region: &GpuTextureCopyRegion,
    kind: GpuTextureAccessKind,
) -> Result<GpuTextureAccess, GpuWorkOperationError> {
    GpuTextureAccess::new(
        GpuTextureAccessResource::Texture(region.texture().clone()),
        region.subresources(),
        kind,
    )
    .map_err(|source| {
        GpuWorkOperationError::from_access(
            "construct GPU texture copy",
            region.texture().descriptor().common().label().as_str(),
            GpuWorkOperationCause::InvalidCopyRegion,
            "declare matching texture copy usage and checked coverage",
            source,
        )
    })
}

fn validate_buffer_texture_layout(
    layout: &GpuBufferTextureLayout,
    texture: &GpuTextureCopyRegion,
    kind: GpuBufferAccessKind,
) -> Result<GpuBufferAccess, GpuWorkOperationError> {
    buffer_layout_access(layout, texture, kind)
}

fn buffer_layout_access(
    layout: &GpuBufferTextureLayout,
    texture: &GpuTextureCopyRegion,
    kind: GpuBufferAccessKind,
) -> Result<GpuBufferAccess, GpuWorkOperationError> {
    let extent = texture.extent();
    let logical_row = extent
        .width()
        .checked_mul(texture.texture().descriptor().format().bytes_per_texel())
        .ok_or_else(|| {
            copy_layout_error(
                layout,
                "reduce the copy width so logical row size does not overflow",
            )
        })?;
    if layout.bytes_per_row() < logical_row
        || (extent.depth_or_layers() > 1 && layout.rows_per_image() < extent.height())
        || (extent.depth_or_layers() == 1
            && layout.rows_per_image() != 0
            && layout.rows_per_image() < extent.height())
    {
        return Err(copy_layout_error(
            layout,
            "provide bytes-per-row and rows-per-image covering the complete logical copy",
        ));
    }
    let image_rows = if extent.depth_or_layers() > 1 {
        layout.rows_per_image()
    } else {
        0
    };
    let image_stride = u64::from(layout.bytes_per_row())
        .checked_mul(u64::from(image_rows))
        .ok_or_else(|| copy_layout_error(layout, "reduce the logical image stride"))?;
    let preceding_images = u64::from(extent.depth_or_layers() - 1)
        .checked_mul(image_stride)
        .ok_or_else(|| copy_layout_error(layout, "reduce the copy depth or layer count"))?;
    let preceding_rows = u64::from(extent.height() - 1)
        .checked_mul(u64::from(layout.bytes_per_row()))
        .ok_or_else(|| copy_layout_error(layout, "reduce the copy height"))?;
    let size = preceding_images
        .checked_add(preceding_rows)
        .and_then(|value| value.checked_add(u64::from(logical_row)))
        .ok_or_else(|| copy_layout_error(layout, "reduce the logical copy byte coverage"))?;
    let range =
        GpuBufferRange::new(layout.buffer(), layout.byte_offset(), size).map_err(|source| {
            GpuWorkOperationError::from_access(
                "validate GPU buffer-texture layout",
                layout.buffer().descriptor().common().label().as_str(),
                GpuWorkOperationCause::InvalidCopyLayout,
                "keep the complete logical row and image coverage inside the buffer",
                source,
            )
        })?;
    buffer_access(
        layout.buffer(),
        range,
        kind,
        "construct GPU buffer-texture copy",
    )
}

fn copy_layout_error(
    layout: &GpuBufferTextureLayout,
    correction: &'static str,
) -> GpuWorkOperationError {
    GpuWorkOperationError::invalid(
        "validate GPU buffer-texture layout",
        layout.buffer().descriptor().common().label().as_str(),
        Some(layout.buffer().diagnostic_identity()),
        GpuWorkOperationCause::InvalidCopyLayout,
        correction,
    )
}

fn copy_error(
    operation: &'static str,
    resource: GpuWorkResourceId,
    correction: &'static str,
) -> GpuWorkOperationError {
    GpuWorkOperationError::invalid(
        operation,
        "copy",
        Some(resource),
        GpuWorkOperationCause::InvalidCopyRegion,
        correction,
    )
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuClearOperation {
    BufferZero(GpuBufferRegion),
}

impl GpuClearOperation {
    pub fn buffer_zero(region: GpuBufferRegion) -> Result<Self, GpuWorkOperationError> {
        GpuBufferAccess::new(
            region.buffer(),
            region.range(),
            GpuBufferAccessKind::CopyDestination,
        )
        .map_err(|source| {
            GpuWorkOperationError::from_access(
                "construct GPU buffer-zero operation",
                region.buffer().descriptor().common().label().as_str(),
                GpuWorkOperationCause::InvalidBufferZero,
                "declare CopyDestination usage and a checked nonempty buffer range",
                source,
            )
        })?;
        Ok(Self::BufferZero(region))
    }

    fn derived_accesses(&self) -> Result<Vec<GpuResourceAccess>, GpuWorkOperationError> {
        match self {
            Self::BufferZero(region) => Ok(vec![GpuResourceAccess::Buffer(
                GpuBufferAccess::new(
                    region.buffer(),
                    region.range(),
                    GpuBufferAccessKind::CopyDestination,
                )
                .map_err(|source| {
                    GpuWorkOperationError::from_access(
                        "derive GPU buffer-zero access",
                        region.buffer().descriptor().common().label().as_str(),
                        GpuWorkOperationCause::OperationAccessContradiction,
                        "construct buffer-zero work through the checked constructor",
                        source,
                    )
                })?,
            )]),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuQueryResolveOperation {
    source: GpuQuerySetHandle,
    source_range: GpuQueryRange,
    destination: GpuBufferHandle,
    destination_offset: u64,
    destination_range: GpuBufferRange,
    source_access: GpuQueryAccess,
    destination_access: GpuBufferAccess,
}

impl GpuQueryResolveOperation {
    pub fn new(
        source: &GpuQuerySetHandle,
        source_range: GpuQueryRange,
        destination: &GpuBufferHandle,
        destination_offset: u64,
    ) -> Result<Self, GpuWorkOperationError> {
        if source.descriptor().kind() != GpuQueryKind::Timestamp {
            return Err(GpuWorkOperationError::invalid(
                "construct GPU query resolve operation",
                source.descriptor().common().label().as_str(),
                Some(source.diagnostic_identity()),
                GpuWorkOperationCause::InvalidQueryResolution,
                "use a timestamp query set for the current G3 resolve operation",
            ));
        }
        let byte_len = u64::from(source_range.count())
            .checked_mul(8)
            .ok_or_else(|| {
                GpuWorkOperationError::invalid(
                    "construct GPU query resolve operation",
                    source.descriptor().common().label().as_str(),
                    Some(source.diagnostic_identity()),
                    GpuWorkOperationCause::QueryDestinationOverflow,
                    "reduce the query count",
                )
            })?;
        let destination_range = GpuBufferRange::new(destination, destination_offset, byte_len)
            .map_err(|source| {
                let cause = match source.cause() {
                    super::GpuAccessCause::ArithmeticOverflow => {
                        GpuWorkOperationCause::QueryDestinationOverflow
                    }
                    _ => GpuWorkOperationCause::QueryDestinationOutOfBounds,
                };
                GpuWorkOperationError::from_access(
                    "construct GPU query resolve destination",
                    destination.descriptor().common().label().as_str(),
                    cause,
                    "keep count-times-eight bytes at the destination offset inside the buffer",
                    source,
                )
            })?;
        let source_access =
            GpuQueryAccess::new(source, source_range, GpuQueryAccessKind::ResolveSource).map_err(
                |source| {
                    GpuWorkOperationError::from_access(
                        "construct GPU query resolve source",
                        "query resolve",
                        GpuWorkOperationCause::InvalidQueryRange,
                        "provide a checked query range",
                        source,
                    )
                },
            )?;
        let destination_access = GpuBufferAccess::new(
            destination,
            destination_range,
            GpuBufferAccessKind::QueryResolveDestination,
        )
        .map_err(|source| {
            GpuWorkOperationError::from_access(
                "construct GPU query resolve destination",
                destination.descriptor().common().label().as_str(),
                GpuWorkOperationCause::InvalidQueryResolution,
                "declare QueryResolve usage on the destination buffer",
                source,
            )
        })?;
        Ok(Self {
            source: source.clone(),
            source_range,
            destination: destination.clone(),
            destination_offset,
            destination_range,
            source_access,
            destination_access,
        })
    }

    pub fn source(&self) -> &GpuQuerySetHandle {
        &self.source
    }
    pub const fn source_range(&self) -> GpuQueryRange {
        self.source_range
    }
    pub fn destination(&self) -> &GpuBufferHandle {
        &self.destination
    }
    pub const fn destination_offset(&self) -> u64 {
        self.destination_offset
    }
    pub const fn destination_range(&self) -> GpuBufferRange {
        self.destination_range
    }
    pub fn source_access(&self) -> &GpuQueryAccess {
        &self.source_access
    }
    pub fn destination_access(&self) -> &GpuBufferAccess {
        &self.destination_access
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuPresentOperation {
    source: GpuTextureAccessResource,
    subresource: GpuTextureSubresourceRange,
    source_access: GpuTextureAccess,
}

impl GpuPresentOperation {
    pub fn new(
        source: GpuTextureAccessResource,
        subresource: GpuTextureSubresourceRange,
    ) -> Result<Self, GpuWorkOperationError> {
        if subresource.mip_level_count() != 1
            || subresource.array_layer_count() != 1
            || !matches!(
                subresource.aspect(),
                GpuTextureAspect::All | GpuTextureAspect::Color
            )
            || source.parent_texture().descriptor().format().is_depth()
            || source.parent_texture().descriptor().sample_count() != 1
        {
            return Err(GpuWorkOperationError::invalid(
                "construct GPU present operation",
                source
                    .parent_texture()
                    .descriptor()
                    .common()
                    .label()
                    .as_str(),
                Some(source.parent_texture().diagnostic_identity()),
                GpuWorkOperationCause::InvalidAttachment,
                "select exactly one single-sampled color mip and array layer",
            ));
        }
        let source_access =
            GpuTextureAccess::new(source.clone(), subresource, GpuTextureAccessKind::Present)
                .map_err(|source| {
                    GpuWorkOperationError::from_access(
                        "construct GPU present operation",
                        "present",
                        GpuWorkOperationCause::InvalidAttachment,
                        "provide one checked color source subresource",
                        source,
                    )
                })?;
        Ok(Self {
            source,
            subresource,
            source_access,
        })
    }

    pub fn source(&self) -> &GpuTextureAccessResource {
        &self.source
    }
    pub const fn subresource(&self) -> GpuTextureSubresourceRange {
        self.subresource
    }
    pub fn source_access(&self) -> &GpuTextureAccess {
        &self.source_access
    }
}

pub(crate) fn add_access_requirements(
    requirements: &mut GpuCapabilityRequirements,
    access: &GpuResourceAccess,
) -> Result<(), GpuCapabilityRequirementError> {
    match access {
        GpuResourceAccess::Texture(access)
            if matches!(
                access.kind(),
                GpuTextureAccessKind::StorageRead
                    | GpuTextureAccessKind::StorageWrite
                    | GpuTextureAccessKind::StorageReadWrite
            ) =>
        {
            requirements.insert(GpuCapabilityRequirement::Required(
                GpuCapabilityFeature::StorageTexture,
            ))?;
        }
        GpuResourceAccess::Query(access) if access.kind() == GpuQueryAccessKind::WriteTimestamp => {
            requirements.insert(GpuCapabilityRequirement::Required(
                GpuCapabilityFeature::TimestampQuery,
            ))?;
        }
        _ => {}
    }
    Ok(())
}

impl GpuResourceAccess {
    pub fn derived_requirements(
        &self,
    ) -> Result<GpuCapabilityRequirements, GpuCapabilityRequirementError> {
        let mut requirements = GpuCapabilityRequirements::new();
        add_access_requirements(&mut requirements, self)?;
        Ok(requirements)
    }
}

#[cfg(test)]
mod tests {
    use super::super::operation::{GpuRenderOperation, GpuWorkOperation};
    use super::*;
    use crate::plugins::gpu::{
        GpuBufferDescriptor, GpuBufferInitialization, GpuBufferUsage, GpuBufferUsages,
        GpuMemoryIntent, GpuQuerySetDescriptor, GpuReconstruction, GpuResourceCommon,
        GpuResourceLabel, GpuResourceLifetime, GpuResourceProvenance, GpuTextureDescriptor,
        GpuTextureExtent, GpuTextureInitialization, GpuTextureUsage, GpuTextureUsages,
        GpuWorkResourceIdAllocator,
    };
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
        num::NonZeroU64,
    };

    fn label(value: &str) -> GpuResourceLabel {
        GpuResourceLabel::new(value).unwrap()
    }

    fn common(value: &str) -> GpuResourceCommon {
        let label = label(value);
        GpuResourceCommon::owned(
            label.clone(),
            GpuResourceLifetime::Transient,
            GpuMemoryIntent::Device,
            GpuReconstruction::SourceBacked,
            GpuResourceProvenance::new(label, None, None),
        )
        .unwrap()
    }

    fn allocator() -> GpuWorkResourceIdAllocator {
        GpuWorkResourceIdAllocator::for_owner_scope(NonZeroU64::new(73).unwrap())
    }

    fn semantic_hash(value: impl Hash) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    fn buffer(
        allocator: &mut GpuWorkResourceIdAllocator,
        name: &str,
        size: u64,
        usages: impl IntoIterator<Item = GpuBufferUsage>,
    ) -> GpuBufferHandle {
        let label = label(name);
        allocator
            .allocate_buffer_handle(
                GpuBufferDescriptor::new(
                    common(name),
                    size,
                    GpuBufferUsages::new(&label, usages).unwrap(),
                    GpuBufferInitialization::Uninitialized,
                )
                .unwrap(),
            )
            .unwrap()
    }

    #[derive(Clone, Copy)]
    struct TestTextureShape {
        width: u32,
        height: u32,
        layers: u32,
        mip_levels: u32,
        sample_count: u32,
        format: GpuTextureFormat,
    }

    fn texture_with_shape(
        allocator: &mut GpuWorkResourceIdAllocator,
        name: &str,
        shape: TestTextureShape,
        usages: impl IntoIterator<Item = GpuTextureUsage>,
    ) -> GpuTextureHandle {
        let label = label(name);
        allocator
            .allocate_texture_handle(
                GpuTextureDescriptor::new(
                    common(name),
                    GpuTextureDimension::D2,
                    GpuTextureExtent::new(
                        &label,
                        GpuTextureDimension::D2,
                        shape.width,
                        shape.height,
                        shape.layers,
                    )
                    .unwrap(),
                    shape.mip_levels,
                    shape.sample_count,
                    shape.format,
                    GpuTextureUsages::new(&label, usages).unwrap(),
                    GpuTextureInitialization::Uninitialized,
                )
                .unwrap(),
            )
            .unwrap()
    }

    fn texture(
        allocator: &mut GpuWorkResourceIdAllocator,
        name: &str,
        sample_count: u32,
        format: GpuTextureFormat,
        usages: impl IntoIterator<Item = GpuTextureUsage>,
    ) -> GpuTextureHandle {
        texture_with_shape(
            allocator,
            name,
            TestTextureShape {
                width: 16,
                height: 16,
                layers: 1,
                mip_levels: 1,
                sample_count,
                format,
            },
            usages,
        )
    }

    #[test]
    fn clear_values_keep_color_generic_depth_normalized_and_signed_zero_canonical() {
        let negative_zero = GpuColorClearValue::new(-0.0, 0.0, 1.0, 1.0).unwrap();
        let positive_zero = GpuColorClearValue::new(0.0, -0.0, 1.0, 1.0).unwrap();
        assert_eq!(negative_zero, positive_zero);
        assert_eq!(semantic_hash(negative_zero), semantic_hash(positive_zero));
        assert_eq!(negative_zero.components()[0].to_bits(), 0.0_f64.to_bits());
        assert_eq!(
            GpuColorClearValue::new(-2.0, 3.5, 1.1, 7.0)
                .unwrap()
                .components(),
            [-2.0, 3.5, 1.1, 7.0]
        );
        let negative_zero = GpuDepthClearValue::new(-0.0).unwrap();
        let positive_zero = GpuDepthClearValue::new(0.0).unwrap();
        assert_eq!(negative_zero, positive_zero);
        assert_eq!(semantic_hash(negative_zero), semantic_hash(positive_zero));
        assert_eq!(negative_zero.value().to_bits(), 0.0_f32.to_bits());
        assert!(GpuColorClearValue::new(f64::NAN, 0.0, 0.0, 1.0).is_err());
        assert!(GpuColorClearValue::new(f64::INFINITY, 0.0, 0.0, 1.0).is_err());
        assert!(GpuDepthClearValue::new(f32::INFINITY).is_err());
        assert!(GpuDepthClearValue::new(-0.1).is_err());
    }

    #[test]
    fn dispatch_draw_and_indirect_access_are_checked() {
        assert_eq!(GpuDispatchSize::new(0, 1, 1).unwrap().as_array(), [0, 1, 1]);
        assert_eq!(GpuDispatchSize::new(2, 3, 4).unwrap().as_array(), [2, 3, 4]);
        assert!(GpuDrawRange::new(0, 0).is_err());
        let mut allocator = allocator();
        let arguments = buffer(&mut allocator, "arguments", 64, [GpuBufferUsage::Indirect]);
        let range = GpuBufferRange::new(&arguments, 0, 16).unwrap();
        let draw = GpuDrawIntent::indirect(&arguments, range, false).unwrap();
        assert!(draw.derived_access().unwrap().is_some());
        assert!(GpuDrawIntent::indirect(&arguments, range, true).is_err());
        let elements = GpuDrawRange::new(3, 9).unwrap();
        let instances = GpuDrawRange::new(0, 2).unwrap();
        assert!(!GpuDrawIntent::direct(elements, instances).is_indexed());
        assert!(GpuDrawIntent::indexed(elements, -2, instances).is_indexed());
    }

    #[test]
    fn multisample_resolve_is_an_attachment_relation() {
        let mut allocator = allocator();
        let source = texture(
            &mut allocator,
            "msaa",
            4,
            GpuTextureFormat::Rgba8Unorm,
            [GpuTextureUsage::ColorAttachment],
        );
        let destination = texture(
            &mut allocator,
            "resolved",
            1,
            GpuTextureFormat::Rgba8Unorm,
            [GpuTextureUsage::ColorAttachment],
        );
        let source_range = GpuTextureSubresourceRange::whole(&source).unwrap();
        let destination_range = GpuTextureSubresourceRange::whole(&destination).unwrap();
        let resolve = GpuMultisampleResolveTarget::new(
            GpuTextureAccessResource::Texture(destination),
            destination_range,
        )
        .unwrap();
        let attachment = GpuRenderColorAttachment::new(
            GpuTextureAccessResource::Texture(source),
            source_range,
            GpuColorAttachmentLoad::Clear(GpuColorClearValue::new(0.0, 0.0, 0.0, 1.0).unwrap()),
            GpuAttachmentStore::Discard,
            Some(resolve),
        )
        .unwrap();
        let operation = GpuRenderOperation::new([attachment], None, [], []).unwrap();
        assert_eq!(operation.accesses().len(), 2);
    }

    #[test]
    fn multisample_resolve_rejects_sample_format_and_alias_mismatches() {
        let mut allocator = allocator();
        let single_source = texture(
            &mut allocator,
            "single source",
            1,
            GpuTextureFormat::Rgba8Unorm,
            [GpuTextureUsage::ColorAttachment],
        );
        let single_destination = texture(
            &mut allocator,
            "single destination",
            1,
            GpuTextureFormat::Rgba8Unorm,
            [GpuTextureUsage::ColorAttachment],
        );
        let source_range = GpuTextureSubresourceRange::whole(&single_source).unwrap();
        let destination_range = GpuTextureSubresourceRange::whole(&single_destination).unwrap();
        let resolve = GpuMultisampleResolveTarget::new(
            GpuTextureAccessResource::Texture(single_destination),
            destination_range,
        )
        .unwrap();
        assert!(GpuRenderColorAttachment::new(
            GpuTextureAccessResource::Texture(single_source),
            source_range,
            GpuColorAttachmentLoad::Clear(
                GpuColorClearValue::new(0.0, 0.0, 0.0, 1.0).unwrap(),
            ),
            GpuAttachmentStore::Store,
            Some(resolve),
        )
        .is_err());

        let multisampled = texture(
            &mut allocator,
            "multisampled",
            4,
            GpuTextureFormat::Rgba8Unorm,
            [GpuTextureUsage::ColorAttachment],
        );
        let wrong_samples = texture(
            &mut allocator,
            "wrong samples",
            4,
            GpuTextureFormat::Rgba8Unorm,
            [GpuTextureUsage::ColorAttachment],
        );
        let source_range = GpuTextureSubresourceRange::whole(&multisampled).unwrap();
        let resolve = GpuMultisampleResolveTarget::new(
            GpuTextureAccessResource::Texture(wrong_samples.clone()),
            GpuTextureSubresourceRange::whole(&wrong_samples).unwrap(),
        )
        .unwrap();
        assert!(GpuRenderColorAttachment::new(
            GpuTextureAccessResource::Texture(multisampled.clone()),
            source_range,
            GpuColorAttachmentLoad::Clear(
                GpuColorClearValue::new(0.0, 0.0, 0.0, 1.0).unwrap(),
            ),
            GpuAttachmentStore::Store,
            Some(resolve),
        )
        .is_err());

        let wrong_format = texture(
            &mut allocator,
            "wrong format",
            1,
            GpuTextureFormat::Bgra8Unorm,
            [GpuTextureUsage::ColorAttachment],
        );
        let resolve = GpuMultisampleResolveTarget::new(
            GpuTextureAccessResource::Texture(wrong_format.clone()),
            GpuTextureSubresourceRange::whole(&wrong_format).unwrap(),
        )
        .unwrap();
        assert!(GpuRenderColorAttachment::new(
            GpuTextureAccessResource::Texture(multisampled.clone()),
            source_range,
            GpuColorAttachmentLoad::Clear(
                GpuColorClearValue::new(0.0, 0.0, 0.0, 1.0).unwrap(),
            ),
            GpuAttachmentStore::Store,
            Some(resolve),
        )
        .is_err());

        let alias = GpuMultisampleResolveTarget::new(
            GpuTextureAccessResource::Texture(multisampled.clone()),
            source_range,
        )
        .unwrap();
        assert!(GpuRenderColorAttachment::new(
            GpuTextureAccessResource::Texture(multisampled),
            source_range,
            GpuColorAttachmentLoad::Clear(
                GpuColorClearValue::new(0.0, 0.0, 0.0, 1.0).unwrap(),
            ),
            GpuAttachmentStore::Store,
            Some(alias),
        )
        .is_err());
    }

    #[test]
    fn multisample_resolve_rejects_extent_and_subresource_mismatches() {
        let mut allocator = allocator();
        let source = texture(
            &mut allocator,
            "multisample source",
            4,
            GpuTextureFormat::Rgba8Unorm,
            [GpuTextureUsage::ColorAttachment],
        );
        let wrong_extent = texture_with_shape(
            &mut allocator,
            "wrong extent",
            TestTextureShape {
                width: 8,
                height: 8,
                layers: 1,
                mip_levels: 1,
                sample_count: 1,
                format: GpuTextureFormat::Rgba8Unorm,
            },
            [GpuTextureUsage::ColorAttachment],
        );
        let source_range = GpuTextureSubresourceRange::whole(&source).unwrap();
        let wrong_extent_resolve = GpuMultisampleResolveTarget::new(
            GpuTextureAccessResource::Texture(wrong_extent.clone()),
            GpuTextureSubresourceRange::whole(&wrong_extent).unwrap(),
        )
        .unwrap();
        assert!(GpuRenderColorAttachment::new(
            GpuTextureAccessResource::Texture(source.clone()),
            source_range,
            GpuColorAttachmentLoad::Clear(
                GpuColorClearValue::new(0.0, 0.0, 0.0, 1.0).unwrap(),
            ),
            GpuAttachmentStore::Discard,
            Some(wrong_extent_resolve),
        )
        .is_err());

        let extra_mip = texture_with_shape(
            &mut allocator,
            "extra destination mip",
            TestTextureShape {
                width: 16,
                height: 16,
                layers: 1,
                mip_levels: 2,
                sample_count: 1,
                format: GpuTextureFormat::Rgba8Unorm,
            },
            [GpuTextureUsage::ColorAttachment],
        );
        let mismatched_subresources = GpuMultisampleResolveTarget::new(
            GpuTextureAccessResource::Texture(extra_mip.clone()),
            GpuTextureSubresourceRange::whole(&extra_mip).unwrap(),
        )
        .unwrap();
        assert!(GpuRenderColorAttachment::new(
            GpuTextureAccessResource::Texture(source),
            source_range,
            GpuColorAttachmentLoad::Clear(
                GpuColorClearValue::new(0.0, 0.0, 0.0, 1.0).unwrap(),
            ),
            GpuAttachmentStore::Discard,
            Some(mismatched_subresources),
        )
        .is_err());
    }

    #[test]
    fn load_store_only_render_is_rejected_but_clear_and_timestamp_are_work() {
        let mut allocator = allocator();
        let target = texture(
            &mut allocator,
            "target",
            1,
            GpuTextureFormat::Rgba8Unorm,
            [GpuTextureUsage::ColorAttachment],
        );
        let range = GpuTextureSubresourceRange::whole(&target).unwrap();
        let load = GpuRenderColorAttachment::new(
            GpuTextureAccessResource::Texture(target.clone()),
            range,
            GpuColorAttachmentLoad::Load,
            GpuAttachmentStore::Store,
            None,
        )
        .unwrap();
        assert!(load.source_access().kind().reads());
        assert!(load.source_access().kind().writes());
        assert!(GpuRenderOperation::new([load], None, [], []).is_err());
        let clear = GpuRenderColorAttachment::new(
            GpuTextureAccessResource::Texture(target),
            range,
            GpuColorAttachmentLoad::Clear(GpuColorClearValue::new(0.0, 0.0, 0.0, 1.0).unwrap()),
            GpuAttachmentStore::Store,
            None,
        )
        .unwrap();
        assert!(GpuRenderOperation::new([clear], None, [], []).is_ok());

        let queries = allocator
            .allocate_query_set_handle(
                GpuQuerySetDescriptor::new(common("queries"), GpuQueryKind::Timestamp, 2).unwrap(),
            )
            .unwrap();
        let query = GpuQueryAccess::new(
            &queries,
            GpuQueryRange::new(&queries, 0, 1).unwrap(),
            GpuQueryAccessKind::WriteTimestamp,
        )
        .unwrap();
        assert!(
            GpuResourceAccess::Query(query.clone())
                .derived_requirements()
                .unwrap()
                .get(GpuCapabilityFeature::TimestampQuery)
                .is_some()
        );
        assert!(GpuRenderOperation::new([], None, [], [query]).is_ok());
    }

    #[test]
    fn depth_attachment_load_clear_store_and_requirements_are_typed() {
        let mut allocator = allocator();
        let depth = texture(
            &mut allocator,
            "depth",
            1,
            GpuTextureFormat::Depth32Float,
            [GpuTextureUsage::DepthStencilAttachment],
        );
        let range = GpuTextureSubresourceRange::whole(&depth).unwrap();
        let read_only = GpuRenderDepthStencilAttachment::new(
            GpuTextureAccessResource::Texture(depth.clone()),
            range,
            GpuDepthStencilAccess::ReadOnly,
            GpuDepthAttachmentLoad::Load,
            GpuAttachmentStore::Store,
        )
        .unwrap();
        assert!(read_only.source_access().kind().reads());
        assert!(!read_only.source_access().kind().writes());
        assert!(
            GpuRenderDepthStencilAttachment::new(
                GpuTextureAccessResource::Texture(depth.clone()),
                range,
                GpuDepthStencilAccess::ReadOnly,
                GpuDepthAttachmentLoad::Clear(GpuDepthClearValue::new(1.0).unwrap()),
                GpuAttachmentStore::Store,
            )
            .is_err()
        );

        let clear = GpuRenderDepthStencilAttachment::new(
            GpuTextureAccessResource::Texture(depth),
            range,
            GpuDepthStencilAccess::ReadWrite,
            GpuDepthAttachmentLoad::Clear(GpuDepthClearValue::new(0.5).unwrap()),
            GpuAttachmentStore::Discard,
        )
        .unwrap();
        assert!(!clear.source_access().kind().reads());
        assert!(clear.source_access().kind().writes());
        let operation =
            GpuWorkOperation::Render(GpuRenderOperation::new([], Some(clear), [], []).unwrap());
        let requirements = operation.derived_requirements().unwrap();
        assert!(
            requirements
                .get(GpuCapabilityFeature::RenderPipeline)
                .is_some()
        );
        assert!(
            requirements
                .get(GpuCapabilityFeature::DepthAttachment)
                .is_some()
        );
        assert_eq!(operation.derived_accesses().unwrap().len(), 1);
    }

    #[test]
    fn all_copy_directions_validate_logical_coverage() {
        let mut allocator = allocator();
        let source = buffer(&mut allocator, "source", 2048, [GpuBufferUsage::CopySource]);
        let destination = buffer(
            &mut allocator,
            "destination",
            2048,
            [GpuBufferUsage::CopyDestination],
        );
        let buffer_copy = GpuCopyOperation::buffer_to_buffer(
            GpuBufferRegion::new(&source, GpuBufferRange::new(&source, 0, 64).unwrap()).unwrap(),
            GpuBufferRegion::new(
                &destination,
                GpuBufferRange::new(&destination, 0, 64).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(buffer_copy.derived_accesses().unwrap().len(), 2);
        let texture_source = texture(
            &mut allocator,
            "texture source",
            1,
            GpuTextureFormat::Rgba8Unorm,
            [GpuTextureUsage::CopySource],
        );
        let texture_destination = texture(
            &mut allocator,
            "texture destination",
            1,
            GpuTextureFormat::Rgba8Unorm,
            [GpuTextureUsage::CopyDestination],
        );
        let extent = GpuCopyExtent::new(16, 16, 1).unwrap();
        let source_region = GpuTextureCopyRegion::new(
            &texture_source,
            0,
            GpuTextureOrigin::new(0, 0, 0),
            GpuTextureAspect::Color,
            extent,
        )
        .unwrap();
        let destination_region = GpuTextureCopyRegion::new(
            &texture_destination,
            0,
            GpuTextureOrigin::new(0, 0, 0),
            GpuTextureAspect::Color,
            extent,
        )
        .unwrap();
        let source_layout = GpuBufferTextureLayout::new(&source, 0, 64, 0).unwrap();
        let destination_layout = GpuBufferTextureLayout::new(&destination, 0, 64, 0).unwrap();
        let buffer_texture =
            GpuCopyOperation::buffer_to_texture(source_layout, destination_region.clone()).unwrap();
        let accesses = buffer_texture.derived_accesses().unwrap();
        assert_eq!(accesses.len(), 2);
        assert!(matches!(
            &accesses[0],
            GpuResourceAccess::Buffer(access) if access.range().size() == 1_024
        ));
        let texture_buffer =
            GpuCopyOperation::texture_to_buffer(source_region.clone(), destination_layout).unwrap();
        assert_eq!(texture_buffer.derived_accesses().unwrap().len(), 2);
        let texture_texture =
            GpuCopyOperation::texture_to_texture(source_region, destination_region).unwrap();
        assert_eq!(texture_texture.derived_accesses().unwrap().len(), 2);
        let invalid_layout = GpuBufferTextureLayout::new(&source, 0, 63, 0).unwrap();
        assert!(
            GpuCopyOperation::buffer_to_texture(
                invalid_layout,
                GpuTextureCopyRegion::new(
                    &texture_destination,
                    0,
                    GpuTextureOrigin::new(0, 0, 0),
                    GpuTextureAspect::Color,
                    extent,
                )
                .unwrap(),
            )
            .is_err()
        );
    }

    #[test]
    fn buffer_zero_and_query_resolve_derive_exact_accesses() {
        let mut allocator = allocator();
        let zero = buffer(
            &mut allocator,
            "zero",
            64,
            [GpuBufferUsage::CopyDestination],
        );
        let zero_region =
            GpuBufferRegion::new(&zero, GpuBufferRange::new(&zero, 8, 16).unwrap()).unwrap();
        let clear = GpuClearOperation::buffer_zero(zero_region).unwrap();
        assert_eq!(clear.derived_accesses().unwrap().len(), 1);
        assert!(
            GpuWorkOperation::Clear(clear.clone())
                .derived_requirements()
                .unwrap()
                .get(GpuCapabilityFeature::Copy)
                .is_some()
        );
        let wrong_zero = buffer(&mut allocator, "wrong zero", 64, [GpuBufferUsage::Storage]);
        assert_eq!(
            GpuClearOperation::buffer_zero(
                GpuBufferRegion::new(
                    &wrong_zero,
                    GpuBufferRange::new(&wrong_zero, 0, 16).unwrap(),
                )
                .unwrap(),
            )
            .unwrap_err()
            .cause(),
            GpuWorkOperationCause::InvalidBufferZero
        );

        let queries = allocator
            .allocate_query_set_handle(
                GpuQuerySetDescriptor::new(common("queries"), GpuQueryKind::Timestamp, 4).unwrap(),
            )
            .unwrap();
        let resolve = buffer(
            &mut allocator,
            "resolve",
            64,
            [GpuBufferUsage::QueryResolve, GpuBufferUsage::CopySource],
        );
        let operation = GpuQueryResolveOperation::new(
            &queries,
            GpuQueryRange::new(&queries, 1, 2).unwrap(),
            &resolve,
            8,
        )
        .unwrap();
        assert_eq!(operation.destination_range().offset(), 8);
        assert_eq!(operation.destination_range().size(), 16);
        assert_eq!(
            operation.source_access().kind(),
            GpuQueryAccessKind::ResolveSource
        );
        assert_eq!(
            operation.destination_access().kind(),
            GpuBufferAccessKind::QueryResolveDestination
        );
        assert_eq!(
            GpuWorkOperation::Resolve(operation.clone())
                .derived_accesses()
                .unwrap()
                .len(),
            2
        );
        assert!(
            GpuWorkOperation::Resolve(operation.clone())
                .derived_requirements()
                .unwrap()
                .get(GpuCapabilityFeature::TimestampQuery)
                .is_some()
        );

        let wrong_usage = buffer(
            &mut allocator,
            "wrong resolve usage",
            64,
            [GpuBufferUsage::CopyDestination],
        );
        assert_eq!(
            GpuQueryResolveOperation::new(
                &queries,
                GpuQueryRange::new(&queries, 0, 1).unwrap(),
                &wrong_usage,
                0,
            )
            .unwrap_err()
            .cause(),
            GpuWorkOperationCause::InvalidQueryResolution
        );
        let too_small = buffer(
            &mut allocator,
            "small resolve",
            8,
            [GpuBufferUsage::QueryResolve],
        );
        assert_eq!(
            GpuQueryResolveOperation::new(
                &queries,
                GpuQueryRange::new(&queries, 0, 2).unwrap(),
                &too_small,
                0,
            )
            .unwrap_err()
            .cause(),
            GpuWorkOperationCause::QueryDestinationOutOfBounds
        );
        let huge = buffer(
            &mut allocator,
            "huge resolve",
            u64::MAX,
            [GpuBufferUsage::QueryResolve],
        );
        assert_eq!(
            GpuQueryResolveOperation::new(
                &queries,
                GpuQueryRange::new(&queries, 0, 2).unwrap(),
                &huge,
                u64::MAX - 7,
            )
            .unwrap_err()
            .cause(),
            GpuWorkOperationCause::QueryDestinationOverflow
        );
    }
}
