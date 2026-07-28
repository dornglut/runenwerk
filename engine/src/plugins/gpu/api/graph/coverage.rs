use super::super::{
    GpuAccessError, GpuBufferHandle, GpuBufferRange, GpuQueryRange, GpuQuerySetHandle,
    GpuResourceProvenance, GpuResourceRef, GpuTextureAccessResource, GpuTextureAspect,
    GpuTextureHandle, GpuTextureSubresourceRange, GpuWorkAuthoringCause, GpuWorkAuthoringError,
    GpuWorkAuthoringErrorContext, GpuWorkAuthoringErrorSource, GpuWorkResourceId,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuInitialCoverageKind {
    DescriptorInitialization,
    BufferRanges,
    TextureSubresources,
    QueryRanges,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum GpuInitialCoverageData {
    DescriptorInitialization,
    BufferRanges(Vec<GpuBufferRange>),
    TextureSubresources(Vec<GpuTextureSubresourceRange>),
    QueryRanges(Vec<GpuQueryRange>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuInitialCoverage {
    pub(super) resource: GpuResourceRef,
    pub(super) storage_resource: GpuWorkResourceId,
    pub(super) data: GpuInitialCoverageData,
}

impl GpuInitialCoverage {
    pub fn descriptor_initialization(
        resource: GpuResourceRef,
    ) -> Result<Self, GpuWorkAuthoringError> {
        if matches!(resource, GpuResourceRef::QuerySet(_)) {
            return Err(coverage_error(
                "construct descriptor initialization coverage",
                resource.diagnostic_identity(),
                "use explicit checked query ranges because query descriptors contain no initialized indices",
            ));
        }
        let storage_resource = storage_identity(&resource);
        Ok(Self {
            resource,
            storage_resource,
            data: GpuInitialCoverageData::DescriptorInitialization,
        })
    }

    pub fn buffer_ranges(
        buffer: &GpuBufferHandle,
        ranges: impl IntoIterator<Item = GpuBufferRange>,
    ) -> Result<Self, GpuWorkAuthoringError> {
        let mut intervals = Vec::new();
        for range in ranges {
            let checked =
                GpuBufferRange::new(buffer, range.offset(), range.size()).map_err(|source| {
                    coverage_source_error(
                        "construct initial buffer coverage",
                        buffer.diagnostic_identity(),
                        source,
                    )
                })?;
            intervals.push((checked.offset(), checked.end()));
        }
        if intervals.is_empty() {
            return Err(coverage_error(
                "construct initial buffer coverage",
                buffer.diagnostic_identity(),
                "provide at least one checked initialized byte range",
            ));
        }
        let ranges = normalize_u64_intervals(intervals)
            .into_iter()
            .map(|(start, end)| {
                GpuBufferRange::new(buffer, start, end - start).map_err(|source| {
                    coverage_source_error(
                        "normalize initial buffer coverage",
                        buffer.diagnostic_identity(),
                        source,
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            resource: GpuResourceRef::Buffer(buffer.clone()),
            storage_resource: buffer.diagnostic_identity(),
            data: GpuInitialCoverageData::BufferRanges(ranges),
        })
    }

    pub fn texture_subresources(
        resource: &GpuTextureAccessResource,
        ranges: impl IntoIterator<Item = GpuTextureSubresourceRange>,
    ) -> Result<Self, GpuWorkAuthoringError> {
        let parent = resource.parent_texture();
        let parent_aspect = texture_aspect(parent);
        let view_range = match resource {
            GpuTextureAccessResource::Texture(_) => None,
            GpuTextureAccessResource::TextureView(view) => Some(view.descriptor().subresources()),
        };
        let mut by_mip = BTreeMap::<(u32, GpuTextureAspect), Vec<(u32, u32)>>::new();
        for range in ranges {
            let checked =
                GpuTextureSubresourceRange::checked_for(parent, range).map_err(|source| {
                    coverage_source_error(
                        "construct initial texture coverage",
                        parent.diagnostic_identity(),
                        source,
                    )
                })?;
            if view_range.is_some_and(|view| !view.contains(checked, parent_aspect)) {
                return Err(coverage_error(
                    "construct initial texture-view coverage",
                    resource.diagnostic_identity(),
                    "keep initialized mip, layer, and aspect coverage inside the texture view",
                ));
            }
            let aspect = canonical_texture_aspect(checked.aspect(), parent_aspect);
            for mip in checked.base_mip_level()..checked.mip_end() {
                by_mip
                    .entry((mip, aspect))
                    .or_default()
                    .push((checked.base_array_layer(), checked.layer_end()));
            }
        }
        if by_mip.is_empty() {
            return Err(coverage_error(
                "construct initial texture coverage",
                resource.diagnostic_identity(),
                "provide at least one checked initialized texture subresource",
            ));
        }
        let mut normalized = Vec::new();
        for ((mip, aspect), intervals) in by_mip {
            for (layer_start, layer_end) in normalize_u32_intervals(intervals) {
                normalized.push(
                    GpuTextureSubresourceRange::new(
                        parent.descriptor().common().label(),
                        mip,
                        1,
                        layer_start,
                        layer_end - layer_start,
                        aspect,
                    )
                    .map_err(|_| {
                        coverage_error(
                            "normalize initial texture coverage",
                            parent.diagnostic_identity(),
                            "use checked texture subresource coverage",
                        )
                    })?,
                );
            }
        }
        let resource_ref = match resource {
            GpuTextureAccessResource::Texture(texture) => GpuResourceRef::Texture(texture.clone()),
            GpuTextureAccessResource::TextureView(view) => {
                GpuResourceRef::TextureView(view.clone())
            }
        };
        Ok(Self {
            resource: resource_ref,
            storage_resource: parent.diagnostic_identity(),
            data: GpuInitialCoverageData::TextureSubresources(normalized),
        })
    }

    pub fn query_ranges(
        query_set: &GpuQuerySetHandle,
        ranges: impl IntoIterator<Item = GpuQueryRange>,
    ) -> Result<Self, GpuWorkAuthoringError> {
        let mut intervals = Vec::new();
        for range in ranges {
            let checked =
                GpuQueryRange::new(query_set, range.first(), range.count()).map_err(|source| {
                    coverage_source_error(
                        "construct initial query coverage",
                        query_set.diagnostic_identity(),
                        source,
                    )
                })?;
            intervals.push((checked.first(), checked.end()));
        }
        if intervals.is_empty() {
            return Err(coverage_error(
                "construct initial query coverage",
                query_set.diagnostic_identity(),
                "provide at least one checked initialized query range",
            ));
        }
        let ranges = normalize_u32_intervals(intervals)
            .into_iter()
            .map(|(start, end)| {
                GpuQueryRange::new(query_set, start, end - start).map_err(|source| {
                    coverage_source_error(
                        "normalize initial query coverage",
                        query_set.diagnostic_identity(),
                        source,
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            resource: GpuResourceRef::QuerySet(query_set.clone()),
            storage_resource: query_set.diagnostic_identity(),
            data: GpuInitialCoverageData::QueryRanges(ranges),
        })
    }

    pub const fn kind(&self) -> GpuInitialCoverageKind {
        match self.data {
            GpuInitialCoverageData::DescriptorInitialization => {
                GpuInitialCoverageKind::DescriptorInitialization
            }
            GpuInitialCoverageData::BufferRanges(_) => GpuInitialCoverageKind::BufferRanges,
            GpuInitialCoverageData::TextureSubresources(_) => {
                GpuInitialCoverageKind::TextureSubresources
            }
            GpuInitialCoverageData::QueryRanges(_) => GpuInitialCoverageKind::QueryRanges,
        }
    }

    pub fn resource(&self) -> &GpuResourceRef {
        &self.resource
    }

    pub fn buffer_range_values(&self) -> Option<&[GpuBufferRange]> {
        match &self.data {
            GpuInitialCoverageData::BufferRanges(ranges) => Some(ranges),
            _ => None,
        }
    }

    pub fn texture_subresource_values(&self) -> Option<&[GpuTextureSubresourceRange]> {
        match &self.data {
            GpuInitialCoverageData::TextureSubresources(ranges) => Some(ranges),
            _ => None,
        }
    }

    pub fn query_range_values(&self) -> Option<&[GpuQueryRange]> {
        match &self.data {
            GpuInitialCoverageData::QueryRanges(ranges) => Some(ranges),
            _ => None,
        }
    }
}

fn coverage_error(
    operation: &'static str,
    resource: GpuWorkResourceId,
    correction: &'static str,
) -> GpuWorkAuthoringError {
    GpuWorkAuthoringError::invalid(
        operation,
        GpuWorkAuthoringErrorContext::new(None, None, None, Some(resource), None),
        GpuWorkAuthoringCause::InvalidCoverage,
        correction,
    )
}

pub(super) fn coverage_source_error(
    operation: &'static str,
    resource: GpuWorkResourceId,
    source: GpuAccessError,
) -> GpuWorkAuthoringError {
    GpuWorkAuthoringError::with_source(
        operation,
        GpuWorkAuthoringErrorContext::new(None, None, None, Some(resource), None),
        GpuWorkAuthoringCause::InvalidCoverage,
        "provide coverage checked against the same typed resource",
        GpuWorkAuthoringErrorSource::Access(source),
    )
}

pub(super) fn normalize_u64_intervals(mut intervals: Vec<(u64, u64)>) -> Vec<(u64, u64)> {
    intervals.sort_unstable();
    let mut normalized: Vec<(u64, u64)> = Vec::new();
    for (start, end) in intervals {
        if let Some(last) = normalized.last_mut()
            && start <= last.1
        {
            last.1 = last.1.max(end);
            continue;
        }
        normalized.push((start, end));
    }
    normalized
}

pub(super) fn normalize_u32_intervals(mut intervals: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
    intervals.sort_unstable();
    let mut normalized: Vec<(u32, u32)> = Vec::new();
    for (start, end) in intervals {
        if let Some(last) = normalized.last_mut()
            && start <= last.1
        {
            last.1 = last.1.max(end);
            continue;
        }
        normalized.push((start, end));
    }
    normalized
}

pub(super) fn texture_aspect(texture: &GpuTextureHandle) -> GpuTextureAspect {
    if texture.descriptor().format().is_depth() {
        GpuTextureAspect::DepthOnly
    } else {
        GpuTextureAspect::Color
    }
}

pub(super) fn canonical_texture_aspect(
    aspect: GpuTextureAspect,
    parent: GpuTextureAspect,
) -> GpuTextureAspect {
    if aspect == GpuTextureAspect::All {
        parent
    } else {
        aspect
    }
}

pub(super) fn storage_identity(resource: &GpuResourceRef) -> GpuWorkResourceId {
    match resource {
        GpuResourceRef::TextureView(view) => view.descriptor().texture().diagnostic_identity(),
        _ => resource.diagnostic_identity(),
    }
}

pub(super) fn canonical_storage_resource(resource: &GpuResourceRef) -> GpuResourceRef {
    match resource {
        GpuResourceRef::TextureView(view) => {
            GpuResourceRef::Texture(view.descriptor().texture().clone())
        }
        _ => resource.clone(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuWorkResourceInput {
    resource: GpuResourceRef,
    initialized_coverage: GpuInitialCoverage,
    provenance: GpuResourceProvenance,
}

impl GpuWorkResourceInput {
    pub fn new(
        resource: GpuResourceRef,
        initialized_coverage: GpuInitialCoverage,
        provenance: GpuResourceProvenance,
    ) -> Result<Self, GpuWorkAuthoringError> {
        if resource != *initialized_coverage.resource() {
            return Err(GpuWorkAuthoringError::invalid(
                "construct GPU work-resource input",
                GpuWorkAuthoringErrorContext::new(
                    None,
                    None,
                    None,
                    Some(resource.diagnostic_identity()),
                    Some(provenance),
                ),
                GpuWorkAuthoringCause::InvalidResourceKind,
                "bind initialized coverage checked against the same kind-preserving resource",
            ));
        }
        Ok(Self {
            resource,
            initialized_coverage,
            provenance,
        })
    }

    pub fn resource(&self) -> &GpuResourceRef {
        &self.resource
    }

    pub fn initialized_coverage(&self) -> &GpuInitialCoverage {
        &self.initialized_coverage
    }

    pub fn provenance(&self) -> &GpuResourceProvenance {
        &self.provenance
    }
}
