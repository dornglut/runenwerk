use super::{
    GpuAttachmentStore, GpuBufferAccess, GpuBufferAccessKind, GpuBufferHandle,
    GpuBufferInitialization, GpuBufferRange, GpuCapabilityRequirementError,
    GpuCapabilityRequirements, GpuExportKey, GpuExportRelationship, GpuQueryRange,
    GpuQuerySetHandle, GpuResourceAccess, GpuResourceAccessIntent, GpuResourceLabel,
    GpuResourceProvenance, GpuResourceRef, GpuTextureAccess, GpuTextureAccessKind,
    GpuTextureAccessResource, GpuTextureAspect, GpuTextureDimension, GpuTextureHandle,
    GpuTextureInitialization, GpuTextureSubresourceRange, GpuWorkAuthoringCause,
    GpuWorkAuthoringError, GpuWorkAuthoringErrorContext, GpuWorkAuthoringErrorSource,
    GpuWorkGraphCause, GpuWorkGraphError, GpuWorkGraphErrorContext, GpuWorkGraphErrorSource,
    GpuWorkNodeKind, GpuWorkOperation, GpuWorkResourceId,
};
use core::fmt;
use core::hash::{Hash, Hasher};
use core::num::NonZeroU64;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Fragment-local opaque work identity.
///
/// ```compile_fail
/// use engine::plugins::gpu::GpuWorkNodeId;
///
/// let _ = GpuWorkNodeId::from_raw(1);
/// ```
#[derive(Clone)]
pub struct GpuWorkNodeId {
    fragment_identity: Arc<()>,
    local: NonZeroU64,
}

impl GpuWorkNodeId {
    fn new(fragment_identity: &Arc<()>, local: NonZeroU64) -> Self {
        Self {
            fragment_identity: Arc::clone(fragment_identity),
            local,
        }
    }

    pub const fn diagnostic_local(&self) -> u64 {
        self.local.get()
    }

    fn belongs_to(&self, fragment_identity: &Arc<()>) -> bool {
        Arc::ptr_eq(&self.fragment_identity, fragment_identity)
    }
}

impl fmt::Debug for GpuWorkNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GpuWorkNodeId")
            .field("local", &self.local)
            .finish_non_exhaustive()
    }
}

impl fmt::Display for GpuWorkNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "local-node:{}", self.local)
    }
}

impl PartialEq for GpuWorkNodeId {
    fn eq(&self, other: &Self) -> bool {
        self.local == other.local && Arc::ptr_eq(&self.fragment_identity, &other.fragment_identity)
    }
}

impl Eq for GpuWorkNodeId {}

impl Hash for GpuWorkNodeId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.local.hash(state);
        Arc::as_ptr(&self.fragment_identity).hash(state);
    }
}

/// Deterministic process-local prepared identity. No raw reconstruction API is
/// exposed because this is not a persistence, replay, cache, or wire key.
///
/// ```compile_fail
/// use engine::plugins::gpu::GpuPreparedWorkNodeId;
///
/// let _ = GpuPreparedWorkNodeId::from_raw(0, 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuPreparedWorkNodeId {
    fragment_ordinal: u32,
    local_node: NonZeroU64,
}

impl GpuPreparedWorkNodeId {
    fn new(fragment_ordinal: u32, local_node: NonZeroU64) -> Self {
        Self {
            fragment_ordinal,
            local_node,
        }
    }

    pub const fn fragment_ordinal(self) -> u32 {
        self.fragment_ordinal
    }

    pub const fn local_node(self) -> u64 {
        self.local_node.get()
    }
}

impl fmt::Display for GpuPreparedWorkNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.fragment_ordinal, self.local_node)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuInitialCoverageKind {
    DescriptorInitialization,
    BufferRanges,
    TextureSubresources,
    QueryRanges,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GpuInitialCoverageData {
    DescriptorInitialization,
    BufferRanges(Vec<GpuBufferRange>),
    TextureSubresources(Vec<GpuTextureSubresourceRange>),
    QueryRanges(Vec<GpuQueryRange>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuInitialCoverage {
    resource: GpuResourceRef,
    storage_resource: GpuWorkResourceId,
    data: GpuInitialCoverageData,
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

fn coverage_source_error(
    operation: &'static str,
    resource: GpuWorkResourceId,
    source: super::GpuAccessError,
) -> GpuWorkAuthoringError {
    GpuWorkAuthoringError::with_source(
        operation,
        GpuWorkAuthoringErrorContext::new(None, None, None, Some(resource), None),
        GpuWorkAuthoringCause::InvalidCoverage,
        "provide coverage checked against the same typed resource",
        GpuWorkAuthoringErrorSource::Access(source),
    )
}

fn normalize_u64_intervals(mut intervals: Vec<(u64, u64)>) -> Vec<(u64, u64)> {
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

fn normalize_u32_intervals(mut intervals: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
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

fn texture_aspect(texture: &GpuTextureHandle) -> GpuTextureAspect {
    if texture.descriptor().format().is_depth() {
        GpuTextureAspect::DepthOnly
    } else {
        GpuTextureAspect::Color
    }
}

fn canonical_texture_aspect(
    aspect: GpuTextureAspect,
    parent: GpuTextureAspect,
) -> GpuTextureAspect {
    if aspect == GpuTextureAspect::All {
        parent
    } else {
        aspect
    }
}

fn storage_identity(resource: &GpuResourceRef) -> GpuWorkResourceId {
    match resource {
        GpuResourceRef::TextureView(view) => view.descriptor().texture().diagnostic_identity(),
        _ => resource.diagnostic_identity(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum GpuExecutionPreference {
    #[default]
    Automatic,
    ComputePreferred,
    GraphicsRequired,
    TransferPreferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuWorkImport {
    resource: GpuResourceRef,
    producer: GpuExportKey,
    required_initial_access: GpuResourceAccessIntent,
    provenance: GpuResourceProvenance,
}

impl GpuWorkImport {
    pub fn new(
        resource: GpuResourceRef,
        producer: GpuExportKey,
        required_initial_access: GpuResourceAccessIntent,
        provenance: GpuResourceProvenance,
    ) -> Self {
        Self {
            resource,
            producer,
            required_initial_access,
            provenance,
        }
    }

    pub fn resource(&self) -> &GpuResourceRef {
        &self.resource
    }

    pub fn producer(&self) -> &GpuExportKey {
        &self.producer
    }

    pub const fn required_initial_access(&self) -> GpuResourceAccessIntent {
        self.required_initial_access
    }

    pub fn provenance(&self) -> &GpuResourceProvenance {
        &self.provenance
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuWorkOutput {
    relationship: GpuExportRelationship,
    final_initialized_coverage: GpuInitialCoverage,
}

impl GpuWorkOutput {
    pub fn new(
        relationship: GpuExportRelationship,
        final_initialized_coverage: GpuInitialCoverage,
    ) -> Result<Self, GpuWorkAuthoringError> {
        if relationship.resource() != final_initialized_coverage.resource() {
            return Err(GpuWorkAuthoringError::invalid(
                "construct GPU work output",
                GpuWorkAuthoringErrorContext::new(
                    None,
                    None,
                    None,
                    Some(relationship.resource().diagnostic_identity()),
                    Some(relationship.provenance().clone()),
                ),
                GpuWorkAuthoringCause::InvalidCoverage,
                "bind final coverage checked against the exported kind-preserving resource",
            ));
        }
        Ok(Self {
            relationship,
            final_initialized_coverage,
        })
    }

    pub fn relationship(&self) -> &GpuExportRelationship {
        &self.relationship
    }

    pub fn final_initialized_coverage(&self) -> &GpuInitialCoverage {
        &self.final_initialized_coverage
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuExplicitOrder {
    before: GpuWorkNodeId,
    after: GpuWorkNodeId,
    reason: String,
}

impl GpuExplicitOrder {
    pub fn new(
        before: &GpuWorkNodeId,
        after: &GpuWorkNodeId,
        reason: impl Into<String>,
    ) -> Result<Self, GpuWorkAuthoringError> {
        let reason = reason.into();
        let reason = reason.trim();
        if reason.is_empty() || before == after {
            return Err(GpuWorkAuthoringError::invalid(
                "construct explicit GPU work order",
                GpuWorkAuthoringErrorContext::new(None, None, Some(before.clone()), None, None),
                GpuWorkAuthoringCause::InvalidExplicitOrder,
                "provide distinct fragment-local nodes and a nonempty non-data reason",
            ));
        }
        if !Arc::ptr_eq(&before.fragment_identity, &after.fragment_identity) {
            return Err(GpuWorkAuthoringError::invalid(
                "construct explicit GPU work order",
                GpuWorkAuthoringErrorContext::new(None, None, Some(before.clone()), None, None),
                GpuWorkAuthoringCause::ForeignIdentity,
                "order only nodes allocated by the same fragment builder",
            ));
        }
        Ok(Self {
            before: before.clone(),
            after: after.clone(),
            reason: reason.to_string(),
        })
    }

    pub fn before(&self) -> &GpuWorkNodeId {
        &self.before
    }

    pub fn after(&self) -> &GpuWorkNodeId {
        &self.after
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// Immutable checked work node. Operation kind is derived from `operation()`;
/// there is no duplicate mutable kind field.
///
/// ```compile_fail
/// use engine::plugins::gpu::{GpuWorkNode, GpuWorkNodeKind};
///
/// fn bypass(node: &mut GpuWorkNode) {
///     node.kind = GpuWorkNodeKind::Copy;
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuWorkNode {
    id: GpuWorkNodeId,
    operation: GpuWorkOperation,
    accesses: Vec<GpuResourceAccess>,
    requirements: GpuCapabilityRequirements,
    execution_preference: GpuExecutionPreference,
    label: GpuResourceLabel,
    provenance: GpuResourceProvenance,
}

impl GpuWorkNode {
    pub fn id(&self) -> &GpuWorkNodeId {
        &self.id
    }

    pub fn operation(&self) -> &GpuWorkOperation {
        &self.operation
    }

    pub fn kind(&self) -> GpuWorkNodeKind {
        self.operation.kind()
    }

    pub fn accesses(&self) -> &[GpuResourceAccess] {
        &self.accesses
    }

    pub fn requirements(&self) -> &GpuCapabilityRequirements {
        &self.requirements
    }

    pub const fn execution_preference(&self) -> GpuExecutionPreference {
        self.execution_preference
    }

    pub fn label(&self) -> &GpuResourceLabel {
        &self.label
    }

    pub fn provenance(&self) -> &GpuResourceProvenance {
        &self.provenance
    }
}

/// Immutable authored fragment. Mutation is available only through the
/// checked builder transaction.
///
/// ```compile_fail
/// use engine::plugins::gpu::GpuWorkFragment;
///
/// fn mutate(fragment: &mut GpuWorkFragment) {
///     fragment.nodes.clear();
/// }
/// ```
#[derive(Debug, Clone)]
pub struct GpuWorkFragment {
    identity: Arc<()>,
    label: GpuResourceLabel,
    resources: Vec<GpuResourceRef>,
    inputs: Vec<GpuWorkResourceInput>,
    imports: Vec<GpuWorkImport>,
    outputs: Vec<GpuWorkOutput>,
    nodes: Vec<GpuWorkNode>,
    explicit_orders: Vec<GpuExplicitOrder>,
    provenance: GpuResourceProvenance,
}

impl GpuWorkFragment {
    pub fn build<F>(label: GpuResourceLabel, author: F) -> Result<Self, GpuWorkAuthoringError>
    where
        F: FnOnce(&mut GpuWorkFragmentBuilder) -> Result<(), GpuWorkAuthoringError>,
    {
        let provenance = GpuResourceProvenance::new(label.clone(), None, None);
        Self::build_with_provenance(label, provenance, author)
    }

    pub fn build_with_provenance<F>(
        label: GpuResourceLabel,
        provenance: GpuResourceProvenance,
        author: F,
    ) -> Result<Self, GpuWorkAuthoringError>
    where
        F: FnOnce(&mut GpuWorkFragmentBuilder) -> Result<(), GpuWorkAuthoringError>,
    {
        let mut builder = GpuWorkFragmentBuilder::new(label, provenance);
        author(&mut builder)?;
        builder.finish()
    }

    pub fn label(&self) -> &GpuResourceLabel {
        &self.label
    }

    pub fn resources(&self) -> &[GpuResourceRef] {
        &self.resources
    }

    pub fn inputs(&self) -> &[GpuWorkResourceInput] {
        &self.inputs
    }

    pub fn imports(&self) -> &[GpuWorkImport] {
        &self.imports
    }

    pub fn outputs(&self) -> &[GpuWorkOutput] {
        &self.outputs
    }

    pub fn nodes(&self) -> &[GpuWorkNode] {
        &self.nodes
    }

    pub fn explicit_orders(&self) -> &[GpuExplicitOrder] {
        &self.explicit_orders
    }

    pub fn provenance(&self) -> &GpuResourceProvenance {
        &self.provenance
    }
}

#[derive(Debug)]
pub struct GpuWorkFragmentBuilder {
    identity: Arc<()>,
    label: GpuResourceLabel,
    resources: BTreeMap<GpuWorkResourceId, GpuResourceRef>,
    inputs: Vec<GpuWorkResourceInput>,
    imports: Vec<GpuWorkImport>,
    outputs: Vec<GpuWorkOutput>,
    nodes: Vec<GpuWorkNode>,
    explicit_orders: Vec<GpuExplicitOrder>,
    next_node: Option<NonZeroU64>,
    provenance: GpuResourceProvenance,
}

impl GpuWorkFragmentBuilder {
    pub fn new(label: GpuResourceLabel, provenance: GpuResourceProvenance) -> Self {
        Self {
            identity: Arc::new(()),
            label,
            resources: BTreeMap::new(),
            inputs: Vec::new(),
            imports: Vec::new(),
            outputs: Vec::new(),
            nodes: Vec::new(),
            explicit_orders: Vec::new(),
            next_node: NonZeroU64::new(1),
            provenance,
        }
    }

    pub fn declare_resource(
        &mut self,
        resource: GpuResourceRef,
    ) -> Result<(), GpuWorkAuthoringError> {
        let identity = resource.diagnostic_identity();
        if self.resources.insert(identity, resource).is_some() {
            return Err(self.error(
                "declare GPU work resource",
                None,
                Some(identity),
                GpuWorkAuthoringCause::DuplicateResource,
                "declare each kind-preserving resource once per fragment",
                None,
            ));
        }
        Ok(())
    }

    pub fn add_input(&mut self, input: GpuWorkResourceInput) -> Result<(), GpuWorkAuthoringError> {
        self.require_resource(input.resource(), "add GPU work-resource input")?;
        let identity = input.resource().diagnostic_identity();
        if self
            .inputs
            .iter()
            .any(|existing| existing.resource().diagnostic_identity() == identity)
        {
            return Err(self.error(
                "add GPU work-resource input",
                None,
                Some(identity),
                GpuWorkAuthoringCause::DuplicateInput,
                "combine exact entry coverage into one input per resource",
                Some(input.provenance().clone()),
            ));
        }
        self.inputs.push(input);
        Ok(())
    }

    pub fn add_import(&mut self, import: GpuWorkImport) -> Result<(), GpuWorkAuthoringError> {
        self.require_resource(import.resource(), "add GPU work import")?;
        if self
            .imports
            .iter()
            .any(|existing| existing.producer() == import.producer())
        {
            return Err(self.error(
                "add GPU work import",
                None,
                Some(import.resource().diagnostic_identity()),
                GpuWorkAuthoringCause::DuplicateImport,
                "bind each producer export key once per fragment",
                Some(import.provenance().clone()),
            ));
        }
        self.imports.push(import);
        Ok(())
    }

    pub fn add_output(&mut self, output: GpuWorkOutput) -> Result<(), GpuWorkAuthoringError> {
        self.require_resource(output.relationship().resource(), "add GPU work output")?;
        let key = output.relationship().export_key();
        if self
            .outputs
            .iter()
            .any(|existing| existing.relationship().export_key() == key)
        {
            return Err(self.error(
                "add GPU work output",
                None,
                Some(output.relationship().resource().diagnostic_identity()),
                GpuWorkAuthoringCause::DuplicateExportKey,
                "use one semantic export key for one fragment output",
                Some(output.relationship().provenance().clone()),
            ));
        }
        self.outputs.push(output);
        Ok(())
    }

    pub fn add_node(
        &mut self,
        label: GpuResourceLabel,
        operation: GpuWorkOperation,
        accesses: impl IntoIterator<Item = GpuResourceAccess>,
        caller_requirements: GpuCapabilityRequirements,
        execution_preference: GpuExecutionPreference,
        provenance: GpuResourceProvenance,
    ) -> Result<GpuWorkNodeId, GpuWorkAuthoringError> {
        operation.validate_shape().map_err(|source| {
            GpuWorkAuthoringError::with_source(
                "add GPU work node",
                GpuWorkAuthoringErrorContext::new(
                    Some(self.label.as_str().to_string()),
                    Some(label.as_str().to_string()),
                    None,
                    source.resource(),
                    Some(provenance.clone()),
                ),
                GpuWorkAuthoringCause::OperationAccessContradiction,
                "construct the operation through its checked operation constructor",
                GpuWorkAuthoringErrorSource::Operation(source),
            )
        })?;
        let derived = operation.derived_accesses().map_err(|source| {
            GpuWorkAuthoringError::with_source(
                "derive GPU work-node access",
                GpuWorkAuthoringErrorContext::new(
                    Some(self.label.as_str().to_string()),
                    Some(label.as_str().to_string()),
                    None,
                    source.resource(),
                    Some(provenance.clone()),
                ),
                GpuWorkAuthoringCause::OperationAccessContradiction,
                "use one internally consistent checked operation",
                GpuWorkAuthoringErrorSource::Operation(source),
            )
        })?;
        let caller = accesses.into_iter().collect::<Vec<_>>();
        for access in derived.iter().chain(&caller) {
            let identity = access.declared_resource_identity();
            if !self.resources.contains_key(&identity) {
                return Err(self.error(
                    "add GPU work-node access",
                    Some(label.as_str().to_string()),
                    Some(identity),
                    GpuWorkAuthoringCause::UnknownIdentity,
                    "declare every operation-derived and caller-declared resource in the fragment",
                    Some(provenance.clone()),
                ));
            }
        }
        let normalized =
            normalize_node_accesses(&self.label, &label, &provenance, derived, caller)?;
        validate_indexed_draw_access(&self.label, &label, &provenance, &operation, &normalized)?;
        let requirements = merge_node_requirements(
            &self.label,
            &label,
            &provenance,
            &operation,
            &normalized,
            &caller_requirements,
        )?;
        let local = self.next_node.ok_or_else(|| {
            self.error(
                "allocate GPU work-node identity",
                Some(label.as_str().to_string()),
                None,
                GpuWorkAuthoringCause::IdentityExhausted,
                "split work into another fragment before exhausting local node identity",
                Some(provenance.clone()),
            )
        })?;
        self.next_node = local.get().checked_add(1).and_then(NonZeroU64::new);
        let id = GpuWorkNodeId::new(&self.identity, local);
        self.nodes.push(GpuWorkNode {
            id: id.clone(),
            operation,
            accesses: normalized,
            requirements,
            execution_preference,
            label,
            provenance,
        });
        Ok(id)
    }

    pub fn add_explicit_order(
        &mut self,
        order: GpuExplicitOrder,
    ) -> Result<(), GpuWorkAuthoringError> {
        for endpoint in [&order.before, &order.after] {
            if !endpoint.belongs_to(&self.identity) {
                return Err(self.error(
                    "add explicit GPU work order",
                    None,
                    None,
                    GpuWorkAuthoringCause::ForeignIdentity,
                    "use endpoints allocated by this fragment builder",
                    None,
                ));
            }
            if !self.nodes.iter().any(|node| node.id() == endpoint) {
                return Err(self.error(
                    "add explicit GPU work order",
                    None,
                    None,
                    GpuWorkAuthoringCause::UnknownIdentity,
                    "add both endpoint nodes before adding explicit order",
                    None,
                ));
            }
        }
        if self.explicit_orders.iter().any(|existing| {
            existing.before() == order.before() && existing.after() == order.after()
        }) {
            return Err(self.error(
                "add explicit GPU work order",
                None,
                None,
                GpuWorkAuthoringCause::DuplicateExplicitOrder,
                "retain one non-data reason for each explicit directed edge",
                None,
            ));
        }
        self.explicit_orders.push(order);
        Ok(())
    }

    pub fn finish(self) -> Result<GpuWorkFragment, GpuWorkAuthoringError> {
        Ok(GpuWorkFragment {
            identity: self.identity,
            label: self.label,
            resources: self.resources.into_values().collect(),
            inputs: self.inputs,
            imports: self.imports,
            outputs: self.outputs,
            nodes: self.nodes,
            explicit_orders: self.explicit_orders,
            provenance: self.provenance,
        })
    }

    fn require_resource(
        &self,
        resource: &GpuResourceRef,
        operation: &'static str,
    ) -> Result<(), GpuWorkAuthoringError> {
        match self.resources.get(&resource.diagnostic_identity()) {
            Some(declared) if declared == resource => Ok(()),
            Some(_) => Err(self.error(
                operation,
                None,
                Some(resource.diagnostic_identity()),
                GpuWorkAuthoringCause::InvalidResourceKind,
                "use the exact kind-preserving resource declared by the fragment",
                Some(resource.common().provenance().clone()),
            )),
            None => Err(self.error(
                operation,
                None,
                Some(resource.diagnostic_identity()),
                GpuWorkAuthoringCause::UnknownIdentity,
                "declare the resource before binding fragment evidence",
                Some(resource.common().provenance().clone()),
            )),
        }
    }

    fn error(
        &self,
        operation: &'static str,
        node_label: Option<String>,
        resource: Option<GpuWorkResourceId>,
        cause: GpuWorkAuthoringCause,
        correction: &'static str,
        provenance: Option<GpuResourceProvenance>,
    ) -> GpuWorkAuthoringError {
        GpuWorkAuthoringError::invalid(
            operation,
            GpuWorkAuthoringErrorContext::new(
                Some(self.label.as_str().to_string()),
                node_label,
                None,
                resource,
                provenance,
            ),
            cause,
            correction,
        )
    }
}

fn normalize_node_accesses(
    fragment_label: &GpuResourceLabel,
    node_label: &GpuResourceLabel,
    provenance: &GpuResourceProvenance,
    mut derived: Vec<GpuResourceAccess>,
    mut caller: Vec<GpuResourceAccess>,
) -> Result<Vec<GpuResourceAccess>, GpuWorkAuthoringError> {
    derived.sort();
    derived.dedup();
    caller.sort();
    caller.dedup();
    if let Some(duplicate) = caller.iter().find(|access| derived.contains(access)) {
        return Err(node_authoring_error(
            "normalize GPU work-node access",
            fragment_label,
            node_label,
            Some(duplicate.resource_identity()),
            GpuWorkAuthoringCause::OperationAccessContradiction,
            "remove caller access already derived by the typed operation",
            provenance,
        ));
    }
    let mut normalized = Vec::<GpuResourceAccess>::new();
    for access in derived.into_iter().chain(caller) {
        insert_normalized_access(
            fragment_label,
            node_label,
            provenance,
            &mut normalized,
            access,
        )?;
    }
    normalized.sort();
    Ok(normalized)
}

fn insert_normalized_access(
    fragment_label: &GpuResourceLabel,
    node_label: &GpuResourceLabel,
    provenance: &GpuResourceProvenance,
    normalized: &mut Vec<GpuResourceAccess>,
    mut access: GpuResourceAccess,
) -> Result<(), GpuWorkAuthoringError> {
    let mut index = 0;
    while index < normalized.len() {
        let existing = &normalized[index];
        if existing == &access {
            return Ok(());
        }
        if existing.resource_identity() != access.resource_identity()
            || !accesses_overlap(existing, &access)
        {
            index += 1;
            continue;
        }
        if let Some(merged) = merge_storage_access(existing, &access).map_err(|source| {
            GpuWorkAuthoringError::with_source(
                "normalize GPU storage access",
                GpuWorkAuthoringErrorContext::new(
                    Some(fragment_label.as_str().to_string()),
                    Some(node_label.as_str().to_string()),
                    None,
                    Some(access.resource_identity()),
                    Some(provenance.clone()),
                ),
                GpuWorkAuthoringCause::OperationAccessContradiction,
                "declare compatible exact storage read/write coverage",
                GpuWorkAuthoringErrorSource::Access(source),
            )
        })? {
            normalized.remove(index);
            access = merged;
            index = 0;
            continue;
        }
        if !existing.writes() && !access.writes() {
            index += 1;
            continue;
        }
        return Err(node_authoring_error(
            "normalize GPU work-node access",
            fragment_label,
            node_label,
            Some(access.resource_identity()),
            GpuWorkAuthoringCause::IncompatibleSameNodeAccess,
            "split incompatible overlapping roles into ordered nodes or use exact storage read-write access",
            provenance,
        ));
    }
    normalized.push(access);
    Ok(())
}

fn merge_storage_access(
    left: &GpuResourceAccess,
    right: &GpuResourceAccess,
) -> Result<Option<GpuResourceAccess>, super::GpuAccessError> {
    match (left, right) {
        (GpuResourceAccess::Buffer(left), GpuResourceAccess::Buffer(right))
            if left.buffer() == right.buffer()
                && left.range() == right.range()
                && storage_buffer_kinds_merge(left.kind(), right.kind()) =>
        {
            GpuBufferAccess::new(
                left.buffer(),
                left.range(),
                GpuBufferAccessKind::StorageReadWrite,
            )
            .map(GpuResourceAccess::Buffer)
            .map(Some)
        }
        (GpuResourceAccess::Texture(left), GpuResourceAccess::Texture(right))
            if left.normalized_texture() == right.normalized_texture()
                && left.normalized_subresources() == right.normalized_subresources()
                && storage_texture_kinds_merge(left.kind(), right.kind()) =>
        {
            GpuTextureAccess::new(
                GpuTextureAccessResource::Texture(left.normalized_texture().clone()),
                left.normalized_subresources(),
                GpuTextureAccessKind::StorageReadWrite,
            )
            .map(GpuResourceAccess::Texture)
            .map(Some)
        }
        _ => Ok(None),
    }
}

fn storage_buffer_kinds_merge(left: GpuBufferAccessKind, right: GpuBufferAccessKind) -> bool {
    use GpuBufferAccessKind::{StorageRead, StorageReadWrite, StorageWrite};
    matches!(
        (left, right),
        (StorageRead, StorageWrite)
            | (StorageWrite, StorageRead)
            | (StorageReadWrite, StorageRead)
            | (StorageReadWrite, StorageWrite)
            | (StorageRead, StorageReadWrite)
            | (StorageWrite, StorageReadWrite)
    )
}

fn storage_texture_kinds_merge(left: GpuTextureAccessKind, right: GpuTextureAccessKind) -> bool {
    use GpuTextureAccessKind::{StorageRead, StorageReadWrite, StorageWrite};
    matches!(
        (left, right),
        (StorageRead, StorageWrite)
            | (StorageWrite, StorageRead)
            | (StorageReadWrite, StorageRead)
            | (StorageReadWrite, StorageWrite)
            | (StorageRead, StorageReadWrite)
            | (StorageWrite, StorageReadWrite)
    )
}

fn accesses_overlap(left: &GpuResourceAccess, right: &GpuResourceAccess) -> bool {
    match (left, right) {
        (GpuResourceAccess::Buffer(left), GpuResourceAccess::Buffer(right)) => {
            left.range().overlaps(right.range())
        }
        (GpuResourceAccess::Texture(left), GpuResourceAccess::Texture(right)) => {
            left.normalized_subresources().overlaps(
                right.normalized_subresources(),
                texture_aspect(left.normalized_texture()),
            )
        }
        (GpuResourceAccess::Query(left), GpuResourceAccess::Query(right)) => {
            left.range().overlaps(right.range())
        }
        (GpuResourceAccess::Sampler(_), GpuResourceAccess::Sampler(_)) => true,
        _ => false,
    }
}

fn validate_indexed_draw_access(
    fragment_label: &GpuResourceLabel,
    node_label: &GpuResourceLabel,
    provenance: &GpuResourceProvenance,
    operation: &GpuWorkOperation,
    accesses: &[GpuResourceAccess],
) -> Result<(), GpuWorkAuthoringError> {
    let GpuWorkOperation::Render(render) = operation else {
        return Ok(());
    };
    if render.draws().iter().any(super::GpuDrawIntent::is_indexed)
        && !accesses.iter().any(|access| {
            matches!(
                access,
                GpuResourceAccess::Buffer(access)
                    if access.kind() == GpuBufferAccessKind::IndexRead
            )
        })
    {
        return Err(node_authoring_error(
            "validate indexed GPU draw",
            fragment_label,
            node_label,
            None,
            GpuWorkAuthoringCause::OperationAccessContradiction,
            "declare a checked index-buffer access for indexed draw intent",
            provenance,
        ));
    }
    Ok(())
}

fn merge_node_requirements(
    fragment_label: &GpuResourceLabel,
    node_label: &GpuResourceLabel,
    provenance: &GpuResourceProvenance,
    operation: &GpuWorkOperation,
    accesses: &[GpuResourceAccess],
    caller: &GpuCapabilityRequirements,
) -> Result<GpuCapabilityRequirements, GpuWorkAuthoringError> {
    let mut requirements = operation.derived_requirements().map_err(|source| {
        requirement_authoring_error(fragment_label, node_label, provenance, source)
    })?;
    for access in accesses {
        let access_requirements = access.derived_requirements().map_err(|source| {
            requirement_authoring_error(fragment_label, node_label, provenance, source)
        })?;
        requirements = requirements.merge(&access_requirements).map_err(|source| {
            requirement_authoring_error(fragment_label, node_label, provenance, source)
        })?;
    }
    requirements = requirements.merge(caller).map_err(|source| {
        requirement_authoring_error(fragment_label, node_label, provenance, source)
    })?;
    Ok(requirements)
}

fn requirement_authoring_error(
    fragment_label: &GpuResourceLabel,
    node_label: &GpuResourceLabel,
    provenance: &GpuResourceProvenance,
    source: GpuCapabilityRequirementError,
) -> GpuWorkAuthoringError {
    GpuWorkAuthoringError::with_source(
        "merge GPU work-node capability requirements",
        GpuWorkAuthoringErrorContext::new(
            Some(fragment_label.as_str().to_string()),
            Some(node_label.as_str().to_string()),
            None,
            None,
            Some(provenance.clone()),
        ),
        GpuWorkAuthoringCause::MechanicalCapabilityContradiction,
        "remove a caller constraint that disables or ambiguously redefines a mechanically required capability",
        GpuWorkAuthoringErrorSource::Capability(source),
    )
}

fn node_authoring_error(
    operation: &'static str,
    fragment_label: &GpuResourceLabel,
    node_label: &GpuResourceLabel,
    resource: Option<GpuWorkResourceId>,
    cause: GpuWorkAuthoringCause,
    correction: &'static str,
    provenance: &GpuResourceProvenance,
) -> GpuWorkAuthoringError {
    GpuWorkAuthoringError::invalid(
        operation,
        GpuWorkAuthoringErrorContext::new(
            Some(fragment_label.as_str().to_string()),
            Some(node_label.as_str().to_string()),
            None,
            resource,
            Some(provenance.clone()),
        ),
        cause,
        correction,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum InitializedCoverage {
    Buffer(Vec<(u64, u64)>),
    Texture(BTreeMap<(u32, GpuTextureAspect), Vec<(u32, u32)>>),
    Query(Vec<(u32, u32)>),
    Immutable,
}

impl InitializedCoverage {
    fn empty_for(resource: &GpuResourceRef) -> Self {
        match resource {
            GpuResourceRef::Buffer(_) => Self::Buffer(Vec::new()),
            GpuResourceRef::Texture(_) | GpuResourceRef::TextureView(_) => {
                Self::Texture(BTreeMap::new())
            }
            GpuResourceRef::QuerySet(_) => Self::Query(Vec::new()),
            GpuResourceRef::Sampler(_) => Self::Immutable,
        }
    }

    fn union(&mut self, other: &Self) -> bool {
        match (self, other) {
            (Self::Buffer(left), Self::Buffer(right)) => {
                left.extend(right.iter().copied());
                *left = normalize_u64_intervals(core::mem::take(left));
                true
            }
            (Self::Texture(left), Self::Texture(right)) => {
                for (key, intervals) in right {
                    let entry = left.entry(*key).or_default();
                    entry.extend(intervals.iter().copied());
                    *entry = normalize_u32_intervals(core::mem::take(entry));
                }
                true
            }
            (Self::Query(left), Self::Query(right)) => {
                left.extend(right.iter().copied());
                *left = normalize_u32_intervals(core::mem::take(left));
                true
            }
            (Self::Immutable, Self::Immutable) => true,
            _ => false,
        }
    }

    fn contains(&self, required: &Self) -> bool {
        match (self, required) {
            (Self::Buffer(have), Self::Buffer(required)) => required
                .iter()
                .all(|range| interval_set_contains_u64(have, *range)),
            (Self::Texture(have), Self::Texture(required)) => {
                required.iter().all(|(key, intervals)| {
                    have.get(key).is_some_and(|have_intervals| {
                        intervals
                            .iter()
                            .all(|range| interval_set_contains_u32(have_intervals, *range))
                    })
                })
            }
            (Self::Query(have), Self::Query(required)) => required
                .iter()
                .all(|range| interval_set_contains_u32(have, *range)),
            (Self::Immutable, Self::Immutable) => true,
            _ => false,
        }
    }

    fn remove(&mut self, removed: &Self) -> bool {
        match (self, removed) {
            (Self::Buffer(have), Self::Buffer(removed)) => {
                for interval in removed {
                    *have = subtract_u64_intervals(core::mem::take(have), *interval);
                }
                true
            }
            (Self::Texture(have), Self::Texture(removed)) => {
                for (key, intervals) in removed {
                    if let Some(have_intervals) = have.get_mut(key) {
                        for interval in intervals {
                            *have_intervals =
                                subtract_u32_intervals(core::mem::take(have_intervals), *interval);
                        }
                    }
                }
                have.retain(|_, intervals| !intervals.is_empty());
                true
            }
            (Self::Query(have), Self::Query(removed)) => {
                for interval in removed {
                    *have = subtract_u32_intervals(core::mem::take(have), *interval);
                }
                true
            }
            (Self::Immutable, Self::Immutable) => true,
            _ => false,
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Self::Buffer(ranges) => ranges.is_empty(),
            Self::Texture(ranges) => ranges.is_empty(),
            Self::Query(ranges) => ranges.is_empty(),
            Self::Immutable => false,
        }
    }
}

fn interval_set_contains_u64(intervals: &[(u64, u64)], required: (u64, u64)) -> bool {
    intervals
        .iter()
        .any(|&(start, end)| start <= required.0 && end >= required.1)
}

fn interval_set_contains_u32(intervals: &[(u32, u32)], required: (u32, u32)) -> bool {
    intervals
        .iter()
        .any(|&(start, end)| start <= required.0 && end >= required.1)
}

fn subtract_u64_intervals(intervals: Vec<(u64, u64)>, removed: (u64, u64)) -> Vec<(u64, u64)> {
    let mut result = Vec::new();
    for (start, end) in intervals {
        if end <= removed.0 || start >= removed.1 {
            result.push((start, end));
            continue;
        }
        if start < removed.0 {
            result.push((start, removed.0));
        }
        if end > removed.1 {
            result.push((removed.1, end));
        }
    }
    result
}

fn subtract_u32_intervals(intervals: Vec<(u32, u32)>, removed: (u32, u32)) -> Vec<(u32, u32)> {
    let mut result = Vec::new();
    for (start, end) in intervals {
        if end <= removed.0 || start >= removed.1 {
            result.push((start, end));
            continue;
        }
        if start < removed.0 {
            result.push((start, removed.0));
        }
        if end > removed.1 {
            result.push((removed.1, end));
        }
    }
    result
}

fn coverage_for_access(access: &GpuResourceAccess) -> InitializedCoverage {
    match access {
        GpuResourceAccess::Buffer(access) => {
            InitializedCoverage::Buffer(vec![(access.range().offset(), access.range().end())])
        }
        GpuResourceAccess::Texture(access) => {
            let range = access.normalized_subresources();
            let aspect = canonical_texture_aspect(
                range.aspect(),
                texture_aspect(access.normalized_texture()),
            );
            let mut subresources = BTreeMap::new();
            for mip in range.base_mip_level()..range.mip_end() {
                subresources.insert(
                    (mip, aspect),
                    vec![(range.base_array_layer(), range.layer_end())],
                );
            }
            InitializedCoverage::Texture(subresources)
        }
        GpuResourceAccess::Query(access) => {
            InitializedCoverage::Query(vec![(access.range().first(), access.range().end())])
        }
        GpuResourceAccess::Sampler(_) => InitializedCoverage::Immutable,
    }
}

fn descriptor_coverage(resource: &GpuResourceRef) -> InitializedCoverage {
    match resource {
        GpuResourceRef::Buffer(buffer) => match buffer.descriptor().initialization() {
            GpuBufferInitialization::Uninitialized => InitializedCoverage::Buffer(Vec::new()),
            GpuBufferInitialization::Zeroed | GpuBufferInitialization::Prepared(_) => {
                InitializedCoverage::Buffer(vec![(0, buffer.descriptor().size_bytes())])
            }
        },
        GpuResourceRef::Texture(texture) => descriptor_texture_coverage(texture),
        GpuResourceRef::TextureView(view) => {
            let parent = view.descriptor().texture();
            let mut coverage = descriptor_texture_coverage(parent);
            let view_coverage = texture_range_coverage(parent, view.descriptor().subresources());
            intersect_coverage(&mut coverage, &view_coverage);
            coverage
        }
        GpuResourceRef::Sampler(_) => InitializedCoverage::Immutable,
        GpuResourceRef::QuerySet(_) => InitializedCoverage::Query(Vec::new()),
    }
}

fn descriptor_texture_coverage(texture: &GpuTextureHandle) -> InitializedCoverage {
    let descriptor = texture.descriptor();
    let mip_count = match descriptor.initialization() {
        GpuTextureInitialization::Uninitialized => 0,
        GpuTextureInitialization::Prepared(_) => 1,
        GpuTextureInitialization::Zeroed => descriptor.mip_level_count(),
    };
    let layers = match descriptor.dimension() {
        GpuTextureDimension::D2 => descriptor.extent().depth_or_layers(),
        GpuTextureDimension::D1 | GpuTextureDimension::D3 => 1,
    };
    let mut ranges = BTreeMap::new();
    for mip in 0..mip_count {
        ranges.insert((mip, texture_aspect(texture)), vec![(0, layers)]);
    }
    InitializedCoverage::Texture(ranges)
}

fn texture_range_coverage(
    texture: &GpuTextureHandle,
    range: GpuTextureSubresourceRange,
) -> InitializedCoverage {
    let aspect = canonical_texture_aspect(range.aspect(), texture_aspect(texture));
    let mut ranges = BTreeMap::new();
    for mip in range.base_mip_level()..range.mip_end() {
        ranges.insert(
            (mip, aspect),
            vec![(range.base_array_layer(), range.layer_end())],
        );
    }
    InitializedCoverage::Texture(ranges)
}

fn intersect_coverage(left: &mut InitializedCoverage, right: &InitializedCoverage) -> bool {
    match (left, right) {
        (InitializedCoverage::Buffer(left), InitializedCoverage::Buffer(right)) => {
            *left = intersect_u64_intervals(left, right);
            true
        }
        (InitializedCoverage::Texture(left), InitializedCoverage::Texture(right)) => {
            left.retain(|key, left_intervals| {
                let Some(right_intervals) = right.get(key) else {
                    return false;
                };
                *left_intervals = intersect_u32_intervals(left_intervals, right_intervals);
                !left_intervals.is_empty()
            });
            true
        }
        (InitializedCoverage::Query(left), InitializedCoverage::Query(right)) => {
            *left = intersect_u32_intervals(left, right);
            true
        }
        (InitializedCoverage::Immutable, InitializedCoverage::Immutable) => true,
        _ => false,
    }
}

fn intersect_u64_intervals(left: &[(u64, u64)], right: &[(u64, u64)]) -> Vec<(u64, u64)> {
    let mut result = Vec::new();
    for &(left_start, left_end) in left {
        for &(right_start, right_end) in right {
            let start = left_start.max(right_start);
            let end = left_end.min(right_end);
            if start < end {
                result.push((start, end));
            }
        }
    }
    normalize_u64_intervals(result)
}

fn intersect_u32_intervals(left: &[(u32, u32)], right: &[(u32, u32)]) -> Vec<(u32, u32)> {
    let mut result = Vec::new();
    for &(left_start, left_end) in left {
        for &(right_start, right_end) in right {
            let start = left_start.max(right_start);
            let end = left_end.min(right_end);
            if start < end {
                result.push((start, end));
            }
        }
    }
    normalize_u32_intervals(result)
}

fn initial_coverage_value(coverage: &GpuInitialCoverage) -> InitializedCoverage {
    match &coverage.data {
        GpuInitialCoverageData::DescriptorInitialization => {
            descriptor_coverage(coverage.resource())
        }
        GpuInitialCoverageData::BufferRanges(ranges) => InitializedCoverage::Buffer(
            ranges
                .iter()
                .map(|range| (range.offset(), range.end()))
                .collect(),
        ),
        GpuInitialCoverageData::TextureSubresources(ranges) => {
            let texture = match coverage.resource() {
                GpuResourceRef::Texture(texture) => texture,
                GpuResourceRef::TextureView(view) => view.descriptor().texture(),
                _ => return InitializedCoverage::Texture(BTreeMap::new()),
            };
            let mut value = InitializedCoverage::Texture(BTreeMap::new());
            for range in ranges {
                let _ = value.union(&texture_range_coverage(texture, *range));
            }
            value
        }
        GpuInitialCoverageData::QueryRanges(ranges) => InitializedCoverage::Query(
            ranges
                .iter()
                .map(|range| (range.first(), range.end()))
                .collect(),
        ),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuDependencyReason {
    ReadAfterWrite { resource: GpuWorkResourceId },
    WriteAfterRead { resource: GpuWorkResourceId },
    WriteAfterWrite { resource: GpuWorkResourceId },
    ExplicitNonData { reason: String },
}

impl GpuDependencyReason {
    pub const fn resource(&self) -> Option<GpuWorkResourceId> {
        match self {
            Self::ReadAfterWrite { resource }
            | Self::WriteAfterRead { resource }
            | Self::WriteAfterWrite { resource } => Some(*resource),
            Self::ExplicitNonData { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuWorkDependency {
    before: GpuPreparedWorkNodeId,
    after: GpuPreparedWorkNodeId,
    reasons: Vec<GpuDependencyReason>,
}

impl GpuWorkDependency {
    pub const fn before(&self) -> GpuPreparedWorkNodeId {
        self.before
    }

    pub const fn after(&self) -> GpuPreparedWorkNodeId {
        self.after
    }

    pub fn reasons(&self) -> &[GpuDependencyReason] {
        &self.reasons
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuPreparedResourceInitialization {
    resource: GpuResourceRef,
    initial: Option<GpuInitialCoverage>,
    final_coverage: Option<GpuInitialCoverage>,
}

impl GpuPreparedResourceInitialization {
    pub fn resource(&self) -> &GpuResourceRef {
        &self.resource
    }

    pub fn initial(&self) -> Option<&GpuInitialCoverage> {
        self.initial.as_ref()
    }

    pub fn final_coverage(&self) -> Option<&GpuInitialCoverage> {
        self.final_coverage.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuPreparedWorkDiagnostic {
    Dependency(GpuWorkDependency),
    ResourceInitialization(GpuPreparedResourceInitialization),
    Output {
        export_key: GpuExportKey,
        resource: GpuWorkResourceId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuPreparedWorkNode {
    id: GpuPreparedWorkNodeId,
    fragment_label: GpuResourceLabel,
    node: GpuWorkNode,
}

impl GpuPreparedWorkNode {
    pub const fn id(&self) -> GpuPreparedWorkNodeId {
        self.id
    }

    pub fn fragment_label(&self) -> &GpuResourceLabel {
        &self.fragment_label
    }

    pub fn node(&self) -> &GpuWorkNode {
        &self.node
    }
}

/// Immutable deterministic result of graph preparation.
///
/// ```compile_fail
/// use engine::plugins::gpu::GpuPreparedWorkGraph;
///
/// fn mutate(graph: &mut GpuPreparedWorkGraph) {
///     graph.topological_order.clear();
/// }
/// ```
#[derive(Debug, Clone)]
pub struct GpuPreparedWorkGraph {
    label: GpuResourceLabel,
    nodes: Vec<GpuPreparedWorkNode>,
    topological_order: Vec<GpuPreparedWorkNodeId>,
    dependencies: Vec<GpuWorkDependency>,
    initialization: Vec<GpuPreparedResourceInitialization>,
    requirements: GpuCapabilityRequirements,
    outputs: Vec<GpuWorkOutput>,
    diagnostics: Vec<GpuPreparedWorkDiagnostic>,
}

impl GpuPreparedWorkGraph {
    pub fn prepare(
        label: GpuResourceLabel,
        fragments: impl IntoIterator<Item = GpuWorkFragment>,
    ) -> Result<Self, GpuWorkGraphError> {
        let fragments = fragments.into_iter().collect::<Vec<_>>();
        let graph_label = label.as_str();
        let mut declared_resources = BTreeMap::<GpuWorkResourceId, GpuResourceRef>::new();
        let mut storage_resources = BTreeMap::<GpuWorkResourceId, GpuResourceRef>::new();
        let mut prepared_nodes = Vec::new();
        let mut node_locations = BTreeMap::<GpuPreparedWorkNodeId, (usize, usize)>::new();

        for (fragment_index, fragment) in fragments.iter().enumerate() {
            let fragment_ordinal = u32::try_from(fragment_index).map_err(|_| {
                graph_error(
                    "assign prepared fragment identity",
                    graph_label,
                    GraphErrorOrigin::new(Some(fragment), None),
                    None,
                    None,
                    GpuWorkGraphCause::UnknownIdentity,
                    "prepare fewer fragments in one bounded graph",
                )
            })?;
            register_fragment_resources(
                graph_label,
                fragment,
                &mut declared_resources,
                &mut storage_resources,
            )?;
            for (node_index, node) in fragment.nodes().iter().enumerate() {
                let expected_local = u64::try_from(node_index)
                    .ok()
                    .and_then(|index| index.checked_add(1));
                if !node.id().belongs_to(&fragment.identity)
                    || expected_local != Some(node.id().diagnostic_local())
                {
                    return Err(graph_error(
                        "validate fragment-local GPU work-node identity",
                        graph_label,
                        GraphErrorOrigin::new(Some(fragment), Some(node)),
                        None,
                        None,
                        GpuWorkGraphCause::ForeignIdentity,
                        "retain nodes allocated monotonically by their originating fragment builder",
                    ));
                }
                validate_node_resources(graph_label, fragment, node)?;
                let id = GpuPreparedWorkNodeId::new(fragment_ordinal, node.id().local);
                if node_locations
                    .insert(id, (fragment_index, node_index))
                    .is_some()
                {
                    return Err(graph_error(
                        "validate prepared GPU work-node identity",
                        graph_label,
                        GraphErrorOrigin::new(Some(fragment), Some(node)),
                        Some(id),
                        None,
                        GpuWorkGraphCause::UnknownIdentity,
                        "use distinct monotonically allocated local node identities",
                    ));
                }
                prepared_nodes.push(GpuPreparedWorkNode {
                    id,
                    fragment_label: fragment.label().clone(),
                    node: node.clone(),
                });
            }
        }

        let output_bindings = collect_output_bindings(graph_label, &fragments)?;
        let (import_bindings, relations) = bind_imports(graph_label, &fragments, &output_bindings)?;
        validate_boundary_access_intents(graph_label, &fragments)?;
        let mut inferred_edges = BTreeMap::<
            (GpuPreparedWorkNodeId, GpuPreparedWorkNodeId),
            BTreeSet<GpuDependencyReason>,
        >::new();
        infer_fragment_hazards(graph_label, &fragments, &mut inferred_edges)?;
        infer_cross_fragment_hazards(graph_label, &fragments, &relations, &mut inferred_edges)?;
        add_explicit_orders(graph_label, &fragments, &mut inferred_edges)?;
        let topological_order =
            topological_node_order(graph_label, &fragments, &node_locations, &inferred_edges)?;
        let fragment_order = topological_fragment_order(graph_label, &fragments, &import_bindings)?;
        validate_fragment_initialization(
            graph_label,
            &fragments,
            &fragment_order,
            &import_bindings,
        )?;

        let mut requirements = GpuCapabilityRequirements::new();
        for prepared in &prepared_nodes {
            requirements = requirements
                .merge(prepared.node().requirements())
                .map_err(|source| {
                    GpuWorkGraphError::with_source(
                        "merge GPU work-graph capability requirements",
                        GpuWorkGraphErrorContext::new(
                            graph_label,
                            Some(prepared.fragment_label().as_str().to_string()),
                            Some(prepared.node().label().as_str().to_string()),
                            Some(prepared.id()),
                            None,
                            None,
                            Some(prepared.node().provenance().clone()),
                        ),
                        GpuWorkGraphCause::MechanicalCapabilityContradiction,
                        "remove graph-wide caller constraints that disable mechanically required capabilities",
                        GpuWorkGraphErrorSource::Capability(source),
                    )
                })?;
        }

        let (initialization, initialization_diagnostics) = simulate_prepared_initialization(
            graph_label,
            &fragments,
            &storage_resources,
            &node_locations,
            &topological_order,
        )?;
        let dependencies = inferred_edges
            .into_iter()
            .map(|((before, after), reasons)| GpuWorkDependency {
                before,
                after,
                reasons: reasons.into_iter().collect(),
            })
            .collect::<Vec<_>>();
        let outputs = fragments
            .iter()
            .flat_map(|fragment| fragment.outputs().iter().cloned())
            .collect::<Vec<_>>();
        let mut diagnostics = dependencies
            .iter()
            .cloned()
            .map(GpuPreparedWorkDiagnostic::Dependency)
            .collect::<Vec<_>>();
        diagnostics.extend(initialization_diagnostics);
        diagnostics.extend(
            outputs
                .iter()
                .map(|output| GpuPreparedWorkDiagnostic::Output {
                    export_key: output.relationship().export_key().clone(),
                    resource: storage_identity(output.relationship().resource()),
                }),
        );

        Ok(Self {
            label,
            nodes: prepared_nodes,
            topological_order,
            dependencies,
            initialization,
            requirements,
            outputs,
            diagnostics,
        })
    }

    pub fn label(&self) -> &GpuResourceLabel {
        &self.label
    }

    pub fn nodes(&self) -> &[GpuPreparedWorkNode] {
        &self.nodes
    }

    pub fn topological_order(&self) -> &[GpuPreparedWorkNodeId] {
        &self.topological_order
    }

    pub fn dependencies(&self) -> &[GpuWorkDependency] {
        &self.dependencies
    }

    pub fn initialization(&self) -> &[GpuPreparedResourceInitialization] {
        &self.initialization
    }

    pub fn requirements(&self) -> &GpuCapabilityRequirements {
        &self.requirements
    }

    pub fn outputs(&self) -> &[GpuWorkOutput] {
        &self.outputs
    }

    pub fn diagnostics(&self) -> &[GpuPreparedWorkDiagnostic] {
        &self.diagnostics
    }
}

fn register_fragment_resources(
    graph_label: &str,
    fragment: &GpuWorkFragment,
    declared: &mut BTreeMap<GpuWorkResourceId, GpuResourceRef>,
    storage: &mut BTreeMap<GpuWorkResourceId, GpuResourceRef>,
) -> Result<(), GpuWorkGraphError> {
    for resource in fragment.resources() {
        let identity = resource.diagnostic_identity();
        if declared
            .get(&identity)
            .is_some_and(|existing| existing != resource)
        {
            return Err(graph_error(
                "register GPU work resource",
                graph_label,
                GraphErrorOrigin::new(Some(fragment), None),
                None,
                Some(identity),
                GpuWorkGraphCause::UnknownIdentity,
                "use one kind-preserving handle for each logical resource identity",
            ));
        }
        declared.entry(identity).or_insert_with(|| resource.clone());
        let storage_resource = canonical_storage_resource(resource);
        let storage_identity = storage_identity(&storage_resource);
        if storage
            .get(&storage_identity)
            .is_some_and(|existing| existing != &storage_resource)
        {
            return Err(graph_error(
                "register normalized GPU storage resource",
                graph_label,
                GraphErrorOrigin::new(Some(fragment), None),
                None,
                Some(storage_identity),
                GpuWorkGraphCause::UnknownIdentity,
                "retain one descriptor and kind for each normalized storage identity",
            ));
        }
        storage.entry(storage_identity).or_insert(storage_resource);
    }
    Ok(())
}

fn canonical_storage_resource(resource: &GpuResourceRef) -> GpuResourceRef {
    match resource {
        GpuResourceRef::TextureView(view) => {
            GpuResourceRef::Texture(view.descriptor().texture().clone())
        }
        _ => resource.clone(),
    }
}

fn validate_node_resources(
    graph_label: &str,
    fragment: &GpuWorkFragment,
    node: &GpuWorkNode,
) -> Result<(), GpuWorkGraphError> {
    node.operation().validate_shape().map_err(|source| {
        GpuWorkGraphError::with_source(
            "validate GPU work operation",
            GpuWorkGraphErrorContext::new(
                graph_label,
                Some(fragment.label().as_str().to_string()),
                Some(node.label().as_str().to_string()),
                None,
                source.resource(),
                None,
                Some(node.provenance().clone()),
            ),
            GpuWorkGraphCause::OperationAccessContradiction,
            "retain the checked operation shape accepted during fragment authoring",
            GpuWorkGraphErrorSource::Operation(source),
        )
    })?;
    for access in node.accesses() {
        let identity = access.declared_resource_identity();
        if !fragment
            .resources()
            .iter()
            .any(|resource| resource.diagnostic_identity() == identity)
        {
            return Err(graph_error(
                "validate GPU work-node resource identity",
                graph_label,
                GraphErrorOrigin::new(Some(fragment), Some(node)),
                None,
                Some(identity),
                GpuWorkGraphCause::UnknownIdentity,
                "declare every accessed typed resource in its immutable fragment",
            ));
        }
    }
    Ok(())
}

type OutputBindings = BTreeMap<GpuExportKey, (usize, usize)>;
type ImportBindings = BTreeMap<(usize, usize), (usize, usize)>;
type FragmentRelations = BTreeSet<(usize, usize, GpuWorkResourceId)>;

fn collect_output_bindings(
    graph_label: &str,
    fragments: &[GpuWorkFragment],
) -> Result<OutputBindings, GpuWorkGraphError> {
    let mut bindings = BTreeMap::new();
    for (fragment_index, fragment) in fragments.iter().enumerate() {
        for (output_index, output) in fragment.outputs().iter().enumerate() {
            let key = output.relationship().export_key().clone();
            if bindings
                .insert(key.clone(), (fragment_index, output_index))
                .is_some()
            {
                return Err(graph_error(
                    "bind GPU work output",
                    graph_label,
                    GraphErrorOrigin::new(Some(fragment), None),
                    None,
                    Some(output.relationship().resource().diagnostic_identity()),
                    GpuWorkGraphCause::DuplicateExportKey,
                    "use each typed export key for exactly one producer output",
                ));
            }
        }
    }
    Ok(bindings)
}

fn bind_imports(
    graph_label: &str,
    fragments: &[GpuWorkFragment],
    outputs: &OutputBindings,
) -> Result<(ImportBindings, FragmentRelations), GpuWorkGraphError> {
    let mut bindings = BTreeMap::new();
    let mut relations = BTreeSet::new();
    let mut consumer_sources = BTreeMap::<(usize, GpuWorkResourceId), usize>::new();
    for (consumer_index, fragment) in fragments.iter().enumerate() {
        for (import_index, import) in fragment.imports().iter().enumerate() {
            let Some(&(producer_index, output_index)) = outputs.get(import.producer()) else {
                return Err(graph_error(
                    "bind GPU work import",
                    graph_label,
                    GraphErrorOrigin::new(Some(fragment), None),
                    None,
                    Some(import.resource().diagnostic_identity()),
                    GpuWorkGraphCause::ImportExportMismatch,
                    "provide exactly one output with the imported typed export key",
                ));
            };
            let output = &fragments[producer_index].outputs()[output_index];
            if producer_index == consumer_index
                || output.relationship().resource() != import.resource()
            {
                return Err(graph_error(
                    "bind GPU work import",
                    graph_label,
                    GraphErrorOrigin::new(Some(fragment), None),
                    None,
                    Some(import.resource().diagnostic_identity()),
                    GpuWorkGraphCause::ImportExportMismatch,
                    "bind the exact kind-preserving producer resource from another fragment",
                ));
            }
            let resource = storage_identity(import.resource());
            if consumer_sources
                .insert((consumer_index, resource), producer_index)
                .is_some_and(|existing| existing != producer_index)
            {
                return Err(graph_error(
                    "bind GPU work import",
                    graph_label,
                    GraphErrorOrigin::new(Some(fragment), None),
                    None,
                    Some(resource),
                    GpuWorkGraphCause::AmbiguousWriter,
                    "select one typed producer for each imported storage resource",
                ));
            }
            bindings.insert(
                (consumer_index, import_index),
                (producer_index, output_index),
            );
            relations.insert((producer_index, consumer_index, resource));
        }
    }
    Ok((bindings, relations))
}

fn validate_boundary_access_intents(
    graph_label: &str,
    fragments: &[GpuWorkFragment],
) -> Result<(), GpuWorkGraphError> {
    for fragment in fragments {
        for import in fragment.imports() {
            let resource = storage_identity(import.resource());
            if first_fragment_access_intent(fragment, resource)
                != Some(import.required_initial_access())
            {
                return Err(graph_error(
                    "validate GPU work import access",
                    graph_label,
                    GraphErrorOrigin::new(Some(fragment), None),
                    None,
                    Some(resource),
                    GpuWorkGraphCause::ImportExportMismatch,
                    "make the import's required initial access match the consumer fragment's first actual access",
                ));
            }
        }
        for output in fragment.outputs() {
            let resource = storage_identity(output.relationship().resource());
            if last_fragment_access_intent(fragment, resource)
                != Some(output.relationship().required_final_access())
            {
                return Err(graph_error(
                    "validate GPU work output access",
                    graph_label,
                    GraphErrorOrigin::new(Some(fragment), None),
                    None,
                    Some(resource),
                    GpuWorkGraphCause::ImportExportMismatch,
                    "make the output's required final access match the producer fragment's last actual access",
                ));
            }
        }
    }
    Ok(())
}

fn first_fragment_access_intent(
    fragment: &GpuWorkFragment,
    resource: GpuWorkResourceId,
) -> Option<GpuResourceAccessIntent> {
    fragment.nodes().iter().find_map(|node| {
        combined_access_intent(
            node.accesses()
                .iter()
                .filter(|access| access.resource_identity() == resource),
        )
    })
}

fn last_fragment_access_intent(
    fragment: &GpuWorkFragment,
    resource: GpuWorkResourceId,
) -> Option<GpuResourceAccessIntent> {
    fragment.nodes().iter().rev().find_map(|node| {
        combined_access_intent(
            node.accesses()
                .iter()
                .filter(|access| access.resource_identity() == resource),
        )
    })
}

fn combined_access_intent<'a>(
    accesses: impl Iterator<Item = &'a GpuResourceAccess>,
) -> Option<GpuResourceAccessIntent> {
    let mut reads = false;
    let mut writes = false;
    let mut any = false;
    for access in accesses {
        any = true;
        reads |= access.reads();
        writes |= access.writes();
    }
    if !any {
        None
    } else if reads && writes {
        Some(GpuResourceAccessIntent::ReadWrite)
    } else if writes {
        Some(GpuResourceAccessIntent::Write)
    } else {
        Some(GpuResourceAccessIntent::Read)
    }
}

fn topological_fragment_order(
    graph_label: &str,
    fragments: &[GpuWorkFragment],
    imports: &ImportBindings,
) -> Result<Vec<usize>, GpuWorkGraphError> {
    let mut outgoing = vec![BTreeSet::<usize>::new(); fragments.len()];
    let mut indegree = vec![0_usize; fragments.len()];
    for &(producer, _) in imports.values() {
        // The consumer index is retained in the import-binding key.
        let consumers = imports
            .iter()
            .filter_map(|(&(consumer, _), &(bound_producer, _))| {
                (bound_producer == producer).then_some(consumer)
            })
            .collect::<BTreeSet<_>>();
        for consumer in consumers {
            if outgoing[producer].insert(consumer) {
                indegree[consumer] += 1;
            }
        }
    }
    let mut ready = indegree
        .iter()
        .enumerate()
        .filter_map(|(index, degree)| (*degree == 0).then_some(index))
        .collect::<BTreeSet<_>>();
    let mut order = Vec::with_capacity(fragments.len());
    while let Some(fragment) = ready.pop_first() {
        order.push(fragment);
        for &consumer in &outgoing[fragment] {
            indegree[consumer] -= 1;
            if indegree[consumer] == 0 {
                ready.insert(consumer);
            }
        }
    }
    if order.len() != fragments.len() {
        return Err(GpuWorkGraphError::invalid(
            "order GPU work-fragment imports",
            GpuWorkGraphErrorContext::new(graph_label, None, None, None, None, None, None),
            GpuWorkGraphCause::Cycle,
            "remove cyclic typed import/export relationships",
        ));
    }
    Ok(order)
}

fn validate_fragment_initialization(
    graph_label: &str,
    fragments: &[GpuWorkFragment],
    fragment_order: &[usize],
    import_bindings: &ImportBindings,
) -> Result<(), GpuWorkGraphError> {
    let mut prepared_outputs = BTreeMap::<(usize, usize), InitializedCoverage>::new();
    for &fragment_index in fragment_order {
        let fragment = &fragments[fragment_index];
        let mut state = fragment_entry_state(fragment);
        for (import_index, import) in fragment.imports().iter().enumerate() {
            let Some(binding) = import_bindings.get(&(fragment_index, import_index)) else {
                return Err(graph_error(
                    "resolve GPU work import coverage",
                    graph_label,
                    GraphErrorOrigin::new(Some(fragment), None),
                    None,
                    Some(storage_identity(import.resource())),
                    GpuWorkGraphCause::ImportExportMismatch,
                    "bind every import to one validated producer output",
                ));
            };
            let Some(coverage) = prepared_outputs.get(binding) else {
                return Err(graph_error(
                    "resolve GPU work import coverage",
                    graph_label,
                    GraphErrorOrigin::new(Some(fragment), None),
                    None,
                    Some(storage_identity(import.resource())),
                    GpuWorkGraphCause::ImportExportMismatch,
                    "order producer fragments before consumers through typed imports",
                ));
            };
            union_state_coverage(
                graph_label,
                fragment,
                &mut state,
                storage_identity(import.resource()),
                coverage,
            )?;
        }
        for node in fragment.nodes() {
            let id = GpuPreparedWorkNodeId::new(
                u32::try_from(fragment_index).map_err(|_| {
                    graph_error(
                        "assign prepared fragment identity",
                        graph_label,
                        GraphErrorOrigin::new(Some(fragment), Some(node)),
                        None,
                        None,
                        GpuWorkGraphCause::UnknownIdentity,
                        "prepare fewer fragments in one bounded graph",
                    )
                })?,
                node.id().local,
            );
            apply_node_initialization(graph_label, fragment, node, id, &mut state)?;
        }
        for (output_index, output) in fragment.outputs().iter().enumerate() {
            let resource = storage_identity(output.relationship().resource());
            let expected = initial_coverage_value(output.final_initialized_coverage());
            let mut actual = state.get(&resource).cloned().unwrap_or_else(|| {
                InitializedCoverage::empty_for(output.relationship().resource())
            });
            let domain = resource_domain_coverage(output.relationship().resource());
            if !intersect_coverage(&mut actual, &domain) || actual != expected {
                return Err(graph_error_with_region(
                    "validate GPU work output coverage",
                    graph_label,
                    GraphErrorOrigin::new(Some(fragment), None),
                    None,
                    (resource, &expected),
                    GpuWorkGraphCause::ImportExportMismatch,
                    "declare the producer fragment's exact final initialized coverage",
                ));
            }
            prepared_outputs.insert((fragment_index, output_index), expected);
        }
    }
    Ok(())
}

fn fragment_entry_state(
    fragment: &GpuWorkFragment,
) -> BTreeMap<GpuWorkResourceId, InitializedCoverage> {
    let mut state = BTreeMap::new();
    for resource in fragment.resources() {
        let storage = canonical_storage_resource(resource);
        let identity = storage_identity(&storage);
        let coverage = descriptor_coverage(resource);
        state
            .entry(identity)
            .and_modify(|existing: &mut InitializedCoverage| {
                let _ = existing.union(&coverage);
            })
            .or_insert(coverage);
    }
    for input in fragment.inputs() {
        let resource = input.initialized_coverage().storage_resource;
        let coverage = initial_coverage_value(input.initialized_coverage());
        state
            .entry(resource)
            .and_modify(|existing| {
                let _ = existing.union(&coverage);
            })
            .or_insert(coverage);
    }
    state
}

fn union_state_coverage(
    graph_label: &str,
    fragment: &GpuWorkFragment,
    state: &mut BTreeMap<GpuWorkResourceId, InitializedCoverage>,
    resource: GpuWorkResourceId,
    coverage: &InitializedCoverage,
) -> Result<(), GpuWorkGraphError> {
    let Some(existing) = state.get_mut(&resource) else {
        return Err(graph_error(
            "merge GPU initialization coverage",
            graph_label,
            GraphErrorOrigin::new(Some(fragment), None),
            None,
            Some(resource),
            GpuWorkGraphCause::UnknownIdentity,
            "declare the initialized resource in the fragment",
        ));
    };
    if !existing.union(coverage) {
        return Err(graph_error(
            "merge GPU initialization coverage",
            graph_label,
            GraphErrorOrigin::new(Some(fragment), None),
            None,
            Some(resource),
            GpuWorkGraphCause::ImportExportMismatch,
            "bind coverage with the same normalized resource kind",
        ));
    }
    Ok(())
}

fn resource_domain_coverage(resource: &GpuResourceRef) -> InitializedCoverage {
    match resource {
        GpuResourceRef::Buffer(buffer) => {
            InitializedCoverage::Buffer(vec![(0, buffer.descriptor().size_bytes())])
        }
        GpuResourceRef::Texture(texture) => whole_texture_coverage(texture, None),
        GpuResourceRef::TextureView(view) => whole_texture_coverage(
            view.descriptor().texture(),
            Some(view.descriptor().subresources()),
        ),
        GpuResourceRef::QuerySet(query_set) => {
            InitializedCoverage::Query(vec![(0, query_set.descriptor().count())])
        }
        GpuResourceRef::Sampler(_) => InitializedCoverage::Immutable,
    }
}

fn whole_texture_coverage(
    texture: &GpuTextureHandle,
    restriction: Option<GpuTextureSubresourceRange>,
) -> InitializedCoverage {
    if let Some(restriction) = restriction {
        return texture_range_coverage(texture, restriction);
    }
    let descriptor = texture.descriptor();
    let layers = match descriptor.dimension() {
        GpuTextureDimension::D2 => descriptor.extent().depth_or_layers(),
        GpuTextureDimension::D1 | GpuTextureDimension::D3 => 1,
    };
    let mut ranges = BTreeMap::new();
    for mip in 0..descriptor.mip_level_count() {
        ranges.insert((mip, texture_aspect(texture)), vec![(0, layers)]);
    }
    InitializedCoverage::Texture(ranges)
}

fn apply_node_initialization(
    graph_label: &str,
    fragment: &GpuWorkFragment,
    node: &GpuWorkNode,
    prepared_id: GpuPreparedWorkNodeId,
    state: &mut BTreeMap<GpuWorkResourceId, InitializedCoverage>,
) -> Result<(), GpuWorkGraphError> {
    for access in node.accesses().iter().filter(|access| access.reads()) {
        let resource = access.resource_identity();
        let required = coverage_for_access(access);
        if !state
            .get(&resource)
            .is_some_and(|coverage| coverage.contains(&required))
        {
            return Err(GpuWorkGraphError::invalid(
                "prepare GPU work initialization",
                GpuWorkGraphErrorContext::new(
                    graph_label,
                    Some(fragment.label().as_str().to_string()),
                    Some(node.label().as_str().to_string()),
                    Some(prepared_id),
                    Some(resource),
                    Some(access_region_description(access)),
                    Some(node.provenance().clone()),
                ),
                GpuWorkGraphCause::ReadBeforeInitialization,
                "provide descriptor/input/import coverage or preceding work that initializes the exact region",
            ));
        }
    }
    for access in node.accesses() {
        let resource = access.resource_identity();
        let affected = coverage_for_access(access);
        let Some(coverage) = state.get_mut(&resource) else {
            return Err(graph_error(
                "apply GPU work initialization",
                graph_label,
                GraphErrorOrigin::new(Some(fragment), Some(node)),
                Some(prepared_id),
                Some(resource),
                GpuWorkGraphCause::UnknownIdentity,
                "declare each normalized storage resource before use",
            ));
        };
        if attachment_discards(access) {
            if !coverage.remove(&affected) {
                return Err(graph_error(
                    "apply GPU attachment discard",
                    graph_label,
                    GraphErrorOrigin::new(Some(fragment), Some(node)),
                    Some(prepared_id),
                    Some(resource),
                    GpuWorkGraphCause::OperationAccessContradiction,
                    "discard coverage only from the matching texture storage",
                ));
            }
        } else if access.writes() && !coverage.union(&affected) {
            return Err(graph_error(
                "apply GPU write initialization",
                graph_label,
                GraphErrorOrigin::new(Some(fragment), Some(node)),
                Some(prepared_id),
                Some(resource),
                GpuWorkGraphCause::OperationAccessContradiction,
                "write coverage only to the matching normalized storage kind",
            ));
        }
    }
    Ok(())
}

fn attachment_discards(access: &GpuResourceAccess) -> bool {
    matches!(
        access,
        GpuResourceAccess::Texture(access)
            if matches!(
                access.kind(),
                GpuTextureAccessKind::ColorAttachment {
                    store: GpuAttachmentStore::Discard,
                    ..
                } | GpuTextureAccessKind::DepthStencilAttachment {
                    store: GpuAttachmentStore::Discard,
                    ..
                }
            )
    )
}

fn access_region_description(access: &GpuResourceAccess) -> String {
    match access {
        GpuResourceAccess::Buffer(access) => {
            format!(
                "bytes {}..{}",
                access.range().offset(),
                access.range().end()
            )
        }
        GpuResourceAccess::Texture(access) => {
            let range = access.normalized_subresources();
            format!(
                "mips {}..{}, layers {}..{}, {:?}",
                range.base_mip_level(),
                range.mip_end(),
                range.base_array_layer(),
                range.layer_end(),
                range.aspect()
            )
        }
        GpuResourceAccess::Query(access) => format!(
            "queries {}..{}",
            access.range().first(),
            access.range().end()
        ),
        GpuResourceAccess::Sampler(_) => "immutable sampler".to_string(),
    }
}

type DependencyEdges =
    BTreeMap<(GpuPreparedWorkNodeId, GpuPreparedWorkNodeId), BTreeSet<GpuDependencyReason>>;

fn infer_fragment_hazards(
    graph_label: &str,
    fragments: &[GpuWorkFragment],
    edges: &mut DependencyEdges,
) -> Result<(), GpuWorkGraphError> {
    for (fragment_index, fragment) in fragments.iter().enumerate() {
        for earlier_index in 0..fragment.nodes().len() {
            for later_index in (earlier_index + 1)..fragment.nodes().len() {
                let earlier = &fragment.nodes()[earlier_index];
                let later = &fragment.nodes()[later_index];
                let reasons = hazard_reasons(earlier, later);
                if reasons.is_empty() {
                    continue;
                }
                let before = prepared_node_id(graph_label, fragment_index, fragment, earlier)?;
                let after = prepared_node_id(graph_label, fragment_index, fragment, later)?;
                edges.entry((before, after)).or_default().extend(reasons);
            }
        }
    }
    Ok(())
}

fn infer_cross_fragment_hazards(
    graph_label: &str,
    fragments: &[GpuWorkFragment],
    relations: &FragmentRelations,
    edges: &mut DependencyEdges,
) -> Result<(), GpuWorkGraphError> {
    for left_fragment_index in 0..fragments.len() {
        for right_fragment_index in (left_fragment_index + 1)..fragments.len() {
            let left_fragment = &fragments[left_fragment_index];
            let right_fragment = &fragments[right_fragment_index];
            for left_node in left_fragment.nodes() {
                for right_node in right_fragment.nodes() {
                    for left_access in left_node.accesses() {
                        for right_access in right_node.accesses() {
                            if left_access.resource_identity() != right_access.resource_identity()
                                || !accesses_overlap(left_access, right_access)
                                || (!left_access.writes() && !right_access.writes())
                            {
                                continue;
                            }
                            let resource = left_access.resource_identity();
                            let left_to_right = relations.contains(&(
                                left_fragment_index,
                                right_fragment_index,
                                resource,
                            ));
                            let right_to_left = relations.contains(&(
                                right_fragment_index,
                                left_fragment_index,
                                resource,
                            ));
                            if left_to_right == right_to_left {
                                let cause = if left_access.writes() && right_access.writes() {
                                    GpuWorkGraphCause::AmbiguousWriter
                                } else {
                                    GpuWorkGraphCause::MissingCrossFragmentCausality
                                };
                                return Err(GpuWorkGraphError::invalid(
                                    "infer cross-fragment GPU hazard",
                                    GpuWorkGraphErrorContext::new(
                                        graph_label,
                                        Some(left_fragment.label().as_str().to_string()),
                                        Some(left_node.label().as_str().to_string()),
                                        None,
                                        Some(resource),
                                        Some(format!(
                                            "{} versus {}",
                                            access_region_description(left_access),
                                            access_region_description(right_access)
                                        )),
                                        Some(left_node.provenance().clone()),
                                    ),
                                    cause,
                                    "bind one unique typed producer output to the consumer import for this storage resource",
                                ));
                            }
                            let (
                                before_fragment,
                                before_node,
                                before_access,
                                after_fragment,
                                after_node,
                                after_access,
                            ) = if left_to_right {
                                (
                                    left_fragment_index,
                                    left_node,
                                    left_access,
                                    right_fragment_index,
                                    right_node,
                                    right_access,
                                )
                            } else {
                                (
                                    right_fragment_index,
                                    right_node,
                                    right_access,
                                    left_fragment_index,
                                    left_node,
                                    left_access,
                                )
                            };
                            let before = prepared_node_id(
                                graph_label,
                                before_fragment,
                                &fragments[before_fragment],
                                before_node,
                            )?;
                            let after = prepared_node_id(
                                graph_label,
                                after_fragment,
                                &fragments[after_fragment],
                                after_node,
                            )?;
                            edges
                                .entry((before, after))
                                .or_default()
                                .extend(access_pair_hazard_reasons(before_access, after_access));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn hazard_reasons(earlier: &GpuWorkNode, later: &GpuWorkNode) -> BTreeSet<GpuDependencyReason> {
    let mut reasons = BTreeSet::new();
    for earlier_access in earlier.accesses() {
        for later_access in later.accesses() {
            if earlier_access.resource_identity() == later_access.resource_identity()
                && accesses_overlap(earlier_access, later_access)
            {
                reasons.extend(access_pair_hazard_reasons(earlier_access, later_access));
            }
        }
    }
    reasons
}

fn access_pair_hazard_reasons(
    earlier: &GpuResourceAccess,
    later: &GpuResourceAccess,
) -> BTreeSet<GpuDependencyReason> {
    let mut reasons = BTreeSet::new();
    let resource = earlier.resource_identity();
    if earlier.writes() && later.reads() {
        reasons.insert(GpuDependencyReason::ReadAfterWrite { resource });
    }
    if earlier.reads() && later.writes() {
        reasons.insert(GpuDependencyReason::WriteAfterRead { resource });
    }
    if earlier.writes() && later.writes() {
        reasons.insert(GpuDependencyReason::WriteAfterWrite { resource });
    }
    reasons
}

fn prepared_node_id(
    graph_label: &str,
    fragment_index: usize,
    fragment: &GpuWorkFragment,
    node: &GpuWorkNode,
) -> Result<GpuPreparedWorkNodeId, GpuWorkGraphError> {
    let ordinal = u32::try_from(fragment_index).map_err(|_| {
        graph_error(
            "assign prepared GPU work-node identity",
            graph_label,
            GraphErrorOrigin::new(Some(fragment), Some(node)),
            None,
            None,
            GpuWorkGraphCause::UnknownIdentity,
            "prepare fewer fragments in one bounded graph",
        )
    })?;
    Ok(GpuPreparedWorkNodeId::new(ordinal, node.id().local))
}

fn add_explicit_orders(
    graph_label: &str,
    fragments: &[GpuWorkFragment],
    edges: &mut DependencyEdges,
) -> Result<(), GpuWorkGraphError> {
    for (fragment_index, fragment) in fragments.iter().enumerate() {
        for order in fragment.explicit_orders() {
            if !order.before().belongs_to(&fragment.identity)
                || !order.after().belongs_to(&fragment.identity)
            {
                return Err(graph_error(
                    "validate explicit GPU work order",
                    graph_label,
                    GraphErrorOrigin::new(Some(fragment), None),
                    None,
                    None,
                    GpuWorkGraphCause::ForeignIdentity,
                    "use endpoints allocated by the containing fragment",
                ));
            }
            let before_node = fragment
                .nodes()
                .iter()
                .find(|node| node.id() == order.before())
                .ok_or_else(|| {
                    graph_error(
                        "validate explicit GPU work-order endpoint",
                        graph_label,
                        GraphErrorOrigin::new(Some(fragment), None),
                        None,
                        None,
                        GpuWorkGraphCause::UnknownIdentity,
                        "retain both typed endpoints in the immutable fragment",
                    )
                })?;
            let after_node = fragment
                .nodes()
                .iter()
                .find(|node| node.id() == order.after())
                .ok_or_else(|| {
                    graph_error(
                        "validate explicit GPU work-order endpoint",
                        graph_label,
                        GraphErrorOrigin::new(Some(fragment), None),
                        None,
                        None,
                        GpuWorkGraphCause::UnknownIdentity,
                        "retain both typed endpoints in the immutable fragment",
                    )
                })?;
            let before = prepared_node_id(graph_label, fragment_index, fragment, before_node)?;
            let after = prepared_node_id(graph_label, fragment_index, fragment, after_node)?;
            if dependency_data_path_exists(edges, before, after) {
                return Err(graph_error(
                    "add explicit GPU work order",
                    graph_label,
                    GraphErrorOrigin::new(Some(fragment), Some(after_node)),
                    Some(after),
                    None,
                    GpuWorkGraphCause::RedundantExplicitDataOrder,
                    "remove the explicit edge and rely on typed access-derived dependency",
                ));
            }
            if dependency_data_path_exists(edges, after, before) {
                return Err(graph_error(
                    "add explicit GPU work order",
                    graph_label,
                    GraphErrorOrigin::new(Some(fragment), Some(after_node)),
                    Some(after),
                    None,
                    GpuWorkGraphCause::ExplicitOrderConflict,
                    "orient the non-data constraint consistently with inferred data order",
                ));
            }
            edges.entry((before, after)).or_default().insert(
                GpuDependencyReason::ExplicitNonData {
                    reason: order.reason().to_string(),
                },
            );
        }
    }
    Ok(())
}

fn dependency_data_path_exists(
    edges: &DependencyEdges,
    start: GpuPreparedWorkNodeId,
    target: GpuPreparedWorkNodeId,
) -> bool {
    let mut ready = vec![(start, false)];
    let mut visited = BTreeSet::new();
    while let Some((node, path_has_data)) = ready.pop() {
        if !visited.insert((node, path_has_data)) {
            continue;
        }
        for (&(before, after), reasons) in edges {
            if before != node {
                continue;
            }
            let edge_has_data = reasons
                .iter()
                .any(|reason| !matches!(reason, GpuDependencyReason::ExplicitNonData { .. }));
            let next_has_data = path_has_data || edge_has_data;
            if after == target && next_has_data {
                return true;
            }
            ready.push((after, next_has_data));
        }
    }
    false
}

fn topological_node_order(
    graph_label: &str,
    fragments: &[GpuWorkFragment],
    node_locations: &BTreeMap<GpuPreparedWorkNodeId, (usize, usize)>,
    edges: &DependencyEdges,
) -> Result<Vec<GpuPreparedWorkNodeId>, GpuWorkGraphError> {
    let mut outgoing = BTreeMap::<GpuPreparedWorkNodeId, BTreeSet<GpuPreparedWorkNodeId>>::new();
    let mut indegree = node_locations
        .keys()
        .copied()
        .map(|node| (node, 0_usize))
        .collect::<BTreeMap<_, _>>();
    for &(before, after) in edges.keys() {
        if !indegree.contains_key(&before) || !indegree.contains_key(&after) {
            return Err(GpuWorkGraphError::invalid(
                "validate GPU work dependency identity",
                GpuWorkGraphErrorContext::new(
                    graph_label,
                    None,
                    None,
                    Some(after),
                    None,
                    None,
                    None,
                ),
                GpuWorkGraphCause::UnknownIdentity,
                "retain dependency endpoints in the prepared node set",
            ));
        }
        if outgoing.entry(before).or_default().insert(after)
            && let Some(degree) = indegree.get_mut(&after)
        {
            *degree += 1;
        }
    }
    let mut ready = indegree
        .iter()
        .filter_map(|(&node, &degree)| (degree == 0).then_some(node))
        .collect::<BTreeSet<_>>();
    let mut ordered = Vec::with_capacity(node_locations.len());
    while let Some(node) = ready.pop_first() {
        ordered.push(node);
        if let Some(dependents) = outgoing.get(&node) {
            for &dependent in dependents {
                if let Some(degree) = indegree.get_mut(&dependent) {
                    *degree -= 1;
                    if *degree == 0 {
                        ready.insert(dependent);
                    }
                }
            }
        }
    }
    if ordered.len() != node_locations.len() {
        let cycle_node = indegree
            .iter()
            .find_map(|(&node, &degree)| (degree > 0).then_some(node));
        let fragment = cycle_node
            .and_then(|node| node_locations.get(&node))
            .map(|&(fragment, _)| &fragments[fragment]);
        return Err(GpuWorkGraphError::invalid(
            "topologically order GPU work",
            GpuWorkGraphErrorContext::new(
                graph_label,
                fragment.map(|fragment| fragment.label().as_str().to_string()),
                None,
                cycle_node,
                None,
                None,
                fragment.map(|fragment| fragment.provenance().clone()),
            ),
            GpuWorkGraphCause::Cycle,
            "remove cyclic explicit order or cyclic typed resource causality",
        ));
    }
    Ok(ordered)
}

fn simulate_prepared_initialization(
    graph_label: &str,
    fragments: &[GpuWorkFragment],
    storage_resources: &BTreeMap<GpuWorkResourceId, GpuResourceRef>,
    node_locations: &BTreeMap<GpuPreparedWorkNodeId, (usize, usize)>,
    topological_order: &[GpuPreparedWorkNodeId],
) -> Result<
    (
        Vec<GpuPreparedResourceInitialization>,
        Vec<GpuPreparedWorkDiagnostic>,
    ),
    GpuWorkGraphError,
> {
    let mut state = BTreeMap::<GpuWorkResourceId, InitializedCoverage>::new();
    for (identity, resource) in storage_resources {
        state.insert(*identity, descriptor_coverage(resource));
    }
    for fragment in fragments {
        for input in fragment.inputs() {
            union_state_coverage(
                graph_label,
                fragment,
                &mut state,
                input.initialized_coverage().storage_resource,
                &initial_coverage_value(input.initialized_coverage()),
            )?;
        }
    }
    let initial = state.clone();
    for &prepared_id in topological_order {
        let Some(&(fragment_index, node_index)) = node_locations.get(&prepared_id) else {
            return Err(GpuWorkGraphError::invalid(
                "simulate prepared GPU work initialization",
                GpuWorkGraphErrorContext::new(
                    graph_label,
                    None,
                    None,
                    Some(prepared_id),
                    None,
                    None,
                    None,
                ),
                GpuWorkGraphCause::UnknownIdentity,
                "retain every topological identity in the prepared node table",
            ));
        };
        let fragment = &fragments[fragment_index];
        let node = &fragment.nodes()[node_index];
        apply_node_initialization(graph_label, fragment, node, prepared_id, &mut state)?;
    }
    let mut summaries = Vec::new();
    for (identity, resource) in storage_resources {
        let initial_value = initial
            .get(identity)
            .cloned()
            .unwrap_or_else(|| InitializedCoverage::empty_for(resource));
        let final_value = state
            .get(identity)
            .cloned()
            .unwrap_or_else(|| InitializedCoverage::empty_for(resource));
        let summary = GpuPreparedResourceInitialization {
            resource: resource.clone(),
            initial: coverage_to_public(graph_label, resource, &initial_value)?,
            final_coverage: coverage_to_public(graph_label, resource, &final_value)?,
        };
        summaries.push(summary);
    }
    let diagnostics = summaries
        .iter()
        .cloned()
        .map(GpuPreparedWorkDiagnostic::ResourceInitialization)
        .collect();
    Ok((summaries, diagnostics))
}

fn coverage_to_public(
    graph_label: &str,
    resource: &GpuResourceRef,
    coverage: &InitializedCoverage,
) -> Result<Option<GpuInitialCoverage>, GpuWorkGraphError> {
    if coverage.is_empty() {
        return Ok(None);
    }
    let value = match (resource, coverage) {
        (GpuResourceRef::Buffer(buffer), InitializedCoverage::Buffer(intervals)) => {
            let ranges = intervals
                .iter()
                .map(|&(start, end)| {
                    GpuBufferRange::new(buffer, start, end - start).map_err(|source| {
                        GpuWorkGraphError::with_source(
                            "publish prepared buffer initialization",
                            GpuWorkGraphErrorContext::new(
                                graph_label,
                                None,
                                None,
                                None,
                                Some(buffer.diagnostic_identity()),
                                Some(format!("bytes {start}..{end}")),
                                Some(buffer.descriptor().common().provenance().clone()),
                            ),
                            GpuWorkGraphCause::OperationAccessContradiction,
                            "retain checked buffer coverage during preparation",
                            GpuWorkGraphErrorSource::Authoring(coverage_source_error(
                                "publish prepared buffer initialization",
                                buffer.diagnostic_identity(),
                                source,
                            )),
                        )
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            GpuInitialCoverage {
                resource: resource.clone(),
                storage_resource: buffer.diagnostic_identity(),
                data: GpuInitialCoverageData::BufferRanges(ranges),
            }
        }
        (GpuResourceRef::Texture(texture), InitializedCoverage::Texture(subresources)) => {
            let mut ranges = Vec::new();
            for (&(mip, aspect), intervals) in subresources {
                for &(layer_start, layer_end) in intervals {
                    ranges.push(
                        GpuTextureSubresourceRange::new(
                            texture.descriptor().common().label(),
                            mip,
                            1,
                            layer_start,
                            layer_end - layer_start,
                            aspect,
                        )
                        .map_err(|_| {
                            GpuWorkGraphError::invalid(
                                "publish prepared texture initialization",
                                GpuWorkGraphErrorContext::new(
                                    graph_label,
                                    None,
                                    None,
                                    None,
                                    Some(texture.diagnostic_identity()),
                                    Some(format!("{coverage:?}")),
                                    Some(texture.descriptor().common().provenance().clone()),
                                ),
                                GpuWorkGraphCause::OperationAccessContradiction,
                                "retain checked texture coverage during preparation",
                            )
                        })?,
                    );
                }
            }
            GpuInitialCoverage {
                resource: resource.clone(),
                storage_resource: texture.diagnostic_identity(),
                data: GpuInitialCoverageData::TextureSubresources(ranges),
            }
        }
        (GpuResourceRef::QuerySet(query_set), InitializedCoverage::Query(intervals)) => {
            let ranges = intervals
                .iter()
                .map(|&(start, end)| {
                    GpuQueryRange::new(query_set, start, end - start).map_err(|source| {
                        GpuWorkGraphError::with_source(
                            "publish prepared query initialization",
                            GpuWorkGraphErrorContext::new(
                                graph_label,
                                None,
                                None,
                                None,
                                Some(query_set.diagnostic_identity()),
                                Some(format!("queries {start}..{end}")),
                                Some(query_set.descriptor().common().provenance().clone()),
                            ),
                            GpuWorkGraphCause::OperationAccessContradiction,
                            "retain checked query coverage during preparation",
                            GpuWorkGraphErrorSource::Authoring(coverage_source_error(
                                "publish prepared query initialization",
                                query_set.diagnostic_identity(),
                                source,
                            )),
                        )
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            GpuInitialCoverage {
                resource: resource.clone(),
                storage_resource: query_set.diagnostic_identity(),
                data: GpuInitialCoverageData::QueryRanges(ranges),
            }
        }
        (GpuResourceRef::Sampler(_), InitializedCoverage::Immutable) => {
            GpuInitialCoverage::descriptor_initialization(resource.clone()).map_err(|source| {
                GpuWorkGraphError::with_source(
                    "publish prepared sampler initialization",
                    GpuWorkGraphErrorContext::new(
                        graph_label,
                        None,
                        None,
                        None,
                        Some(resource.diagnostic_identity()),
                        None,
                        Some(resource.common().provenance().clone()),
                    ),
                    GpuWorkGraphCause::OperationAccessContradiction,
                    "retain immutable sampler evidence",
                    GpuWorkGraphErrorSource::Authoring(source),
                )
            })?
        }
        _ => {
            return Err(GpuWorkGraphError::invalid(
                "publish prepared resource initialization",
                GpuWorkGraphErrorContext::new(
                    graph_label,
                    None,
                    None,
                    None,
                    Some(storage_identity(resource)),
                    Some(format!("{coverage:?}")),
                    Some(resource.common().provenance().clone()),
                ),
                GpuWorkGraphCause::OperationAccessContradiction,
                "retain coverage with the matching normalized resource kind",
            ));
        }
    };
    Ok(Some(value))
}

#[derive(Clone, Copy)]
struct GraphErrorOrigin<'a> {
    fragment: Option<&'a GpuWorkFragment>,
    node: Option<&'a GpuWorkNode>,
}

impl<'a> GraphErrorOrigin<'a> {
    const fn new(fragment: Option<&'a GpuWorkFragment>, node: Option<&'a GpuWorkNode>) -> Self {
        Self { fragment, node }
    }
}

fn graph_error(
    operation: &'static str,
    graph_label: &str,
    origin: GraphErrorOrigin<'_>,
    prepared_node: Option<GpuPreparedWorkNodeId>,
    resource: Option<GpuWorkResourceId>,
    cause: GpuWorkGraphCause,
    correction: &'static str,
) -> GpuWorkGraphError {
    GpuWorkGraphError::invalid(
        operation,
        GpuWorkGraphErrorContext::new(
            graph_label,
            origin
                .fragment
                .map(|fragment| fragment.label().as_str().to_string()),
            origin.node.map(|node| node.label().as_str().to_string()),
            prepared_node,
            resource,
            None,
            origin
                .node
                .map(|node| node.provenance().clone())
                .or_else(|| {
                    origin
                        .fragment
                        .map(|fragment| fragment.provenance().clone())
                }),
        ),
        cause,
        correction,
    )
}

fn graph_error_with_region(
    operation: &'static str,
    graph_label: &str,
    origin: GraphErrorOrigin<'_>,
    prepared_node: Option<GpuPreparedWorkNodeId>,
    region: (GpuWorkResourceId, &InitializedCoverage),
    cause: GpuWorkGraphCause,
    correction: &'static str,
) -> GpuWorkGraphError {
    let (resource, coverage) = region;
    GpuWorkGraphError::invalid(
        operation,
        GpuWorkGraphErrorContext::new(
            graph_label,
            origin
                .fragment
                .map(|fragment| fragment.label().as_str().to_string()),
            origin.node.map(|node| node.label().as_str().to_string()),
            prepared_node,
            Some(resource),
            Some(format!("{coverage:?}")),
            origin
                .node
                .map(|node| node.provenance().clone())
                .or_else(|| {
                    origin
                        .fragment
                        .map(|fragment| fragment.provenance().clone())
                }),
        ),
        cause,
        correction,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::gpu::{
        GpuBufferDescriptor, GpuBufferRegion, GpuBufferUsage, GpuBufferUsages,
        GpuCapabilityFeature, GpuCapabilityRequirement, GpuClearOperation, GpuColorAttachmentLoad,
        GpuColorClearValue, GpuComputeOperation, GpuCopyOperation, GpuDepthAttachmentLoad,
        GpuDepthClearValue, GpuDepthStencilAccess, GpuDispatchSize, GpuDrawIntent, GpuDrawRange,
        GpuMemoryIntent, GpuMultisampleResolveTarget, GpuPreparedTextureData, GpuQueryAccess,
        GpuQueryAccessKind, GpuQueryKind, GpuQueryResolveOperation, GpuQuerySetDescriptor,
        GpuReconstruction, GpuRenderColorAttachment, GpuRenderDepthStencilAttachment,
        GpuRenderOperation, GpuResourceCommon, GpuResourceLifetime, GpuTextureDescriptor,
        GpuTextureExtent, GpuTextureFormat, GpuTextureUsage, GpuTextureUsages,
        GpuWorkResourceIdAllocator, PreparedGpuData, TransferData,
    };

    fn label(value: &str) -> GpuResourceLabel {
        GpuResourceLabel::new(value).unwrap()
    }

    fn provenance(value: &str) -> GpuResourceProvenance {
        let label = label(value);
        GpuResourceProvenance::new(label, None, None)
    }

    fn common(value: &str) -> GpuResourceCommon {
        GpuResourceCommon::owned(
            label(value),
            GpuResourceLifetime::Transient,
            GpuMemoryIntent::Device,
            GpuReconstruction::SourceBacked,
            provenance(value),
        )
        .unwrap()
    }

    fn allocator() -> GpuWorkResourceIdAllocator {
        GpuWorkResourceIdAllocator::for_owner_scope(NonZeroU64::new(97).unwrap())
    }

    fn buffer(
        allocator: &mut GpuWorkResourceIdAllocator,
        name: &str,
        initialization: GpuBufferInitialization,
        usages: impl IntoIterator<Item = GpuBufferUsage>,
    ) -> GpuBufferHandle {
        let resource_label = label(name);
        allocator
            .allocate_buffer_handle(
                GpuBufferDescriptor::new(
                    common(name),
                    64,
                    GpuBufferUsages::new(&resource_label, usages).unwrap(),
                    initialization,
                )
                .unwrap(),
            )
            .unwrap()
    }

    fn texture(
        allocator: &mut GpuWorkResourceIdAllocator,
        name: &str,
        initialization: GpuTextureInitialization,
        mip_levels: u32,
        layers: u32,
        usages: impl IntoIterator<Item = GpuTextureUsage>,
    ) -> GpuTextureHandle {
        let resource_label = label(name);
        allocator
            .allocate_texture_handle(
                GpuTextureDescriptor::new(
                    common(name),
                    GpuTextureDimension::D2,
                    GpuTextureExtent::new(&resource_label, GpuTextureDimension::D2, 8, 8, layers)
                        .unwrap(),
                    mip_levels,
                    1,
                    GpuTextureFormat::Rgba8Unorm,
                    GpuTextureUsages::new(&resource_label, usages).unwrap(),
                    initialization,
                )
                .unwrap(),
            )
            .unwrap()
    }

    fn prepared_texture_initialization(name: &str) -> GpuTextureInitialization {
        let resource_label = label(name);
        let extent =
            GpuTextureExtent::new(&resource_label, GpuTextureDimension::D2, 8, 8, 1).unwrap();
        let data = PreparedGpuData::<TransferData>::from_pod_transfer(
            name,
            &[0_u8; 256],
            provenance(name),
        )
        .unwrap();
        GpuTextureInitialization::Prepared(
            GpuPreparedTextureData::new(
                &resource_label,
                data,
                GpuTextureFormat::Rgba8Unorm,
                extent,
                32,
                0,
            )
            .unwrap(),
        )
    }

    fn depth_texture(allocator: &mut GpuWorkResourceIdAllocator, name: &str) -> GpuTextureHandle {
        let resource_label = label(name);
        allocator
            .allocate_texture_handle(
                GpuTextureDescriptor::new(
                    common(name),
                    GpuTextureDimension::D2,
                    GpuTextureExtent::new(&resource_label, GpuTextureDimension::D2, 8, 8, 1)
                        .unwrap(),
                    1,
                    1,
                    GpuTextureFormat::Depth32Float,
                    GpuTextureUsages::new(
                        &resource_label,
                        [
                            GpuTextureUsage::DepthStencilAttachment,
                            GpuTextureUsage::Sampled,
                        ],
                    )
                    .unwrap(),
                    GpuTextureInitialization::Uninitialized,
                )
                .unwrap(),
            )
            .unwrap()
    }

    fn compute_operation() -> GpuWorkOperation {
        GpuWorkOperation::Compute(GpuComputeOperation::new(
            GpuDispatchSize::new(1, 1, 1).unwrap(),
        ))
    }

    fn builder(name: &str) -> GpuWorkFragmentBuilder {
        GpuWorkFragmentBuilder::new(label(name), provenance(name))
    }

    fn add_compute(
        builder: &mut GpuWorkFragmentBuilder,
        name: &str,
        accesses: impl IntoIterator<Item = GpuResourceAccess>,
    ) -> GpuWorkNodeId {
        builder
            .add_node(
                label(name),
                compute_operation(),
                accesses,
                GpuCapabilityRequirements::new(),
                GpuExecutionPreference::Automatic,
                provenance(name),
            )
            .unwrap()
    }

    fn buffer_access(
        buffer: &GpuBufferHandle,
        range: GpuBufferRange,
        kind: GpuBufferAccessKind,
    ) -> GpuResourceAccess {
        GpuResourceAccess::Buffer(GpuBufferAccess::new(buffer, range, kind).unwrap())
    }

    fn texture_access(
        texture: &GpuTextureHandle,
        range: GpuTextureSubresourceRange,
        kind: GpuTextureAccessKind,
    ) -> GpuResourceAccess {
        GpuResourceAccess::Texture(
            GpuTextureAccess::new(
                GpuTextureAccessResource::Texture(texture.clone()),
                range,
                kind,
            )
            .unwrap(),
        )
    }

    #[test]
    fn initial_coverage_is_checked_normalized_and_kind_preserving() {
        let mut allocator = allocator();
        let buffer = buffer(
            &mut allocator,
            "coverage",
            GpuBufferInitialization::Uninitialized,
            [GpuBufferUsage::Storage],
        );
        let coverage = GpuInitialCoverage::buffer_ranges(
            &buffer,
            [
                GpuBufferRange::new(&buffer, 16, 16).unwrap(),
                GpuBufferRange::new(&buffer, 0, 16).unwrap(),
                GpuBufferRange::new(&buffer, 8, 8).unwrap(),
            ],
        )
        .unwrap();
        assert_eq!(coverage.kind(), GpuInitialCoverageKind::BufferRanges);
        assert_eq!(coverage.buffer_range_values().unwrap().len(), 1);
        assert_eq!(coverage.buffer_range_values().unwrap()[0].size(), 32);

        let queries = allocator
            .allocate_query_set_handle(
                GpuQuerySetDescriptor::new(common("queries"), GpuQueryKind::Timestamp, 4).unwrap(),
            )
            .unwrap();
        assert!(
            GpuInitialCoverage::descriptor_initialization(GpuResourceRef::QuerySet(queries))
                .is_err()
        );
    }

    #[test]
    fn same_node_access_deduplicates_merges_and_rejects_incompatible_roles() {
        let mut allocator = allocator();
        let buffer = buffer(
            &mut allocator,
            "storage",
            GpuBufferInitialization::Uninitialized,
            [GpuBufferUsage::Storage, GpuBufferUsage::Uniform],
        );
        let range = GpuBufferRange::whole(&buffer).unwrap();
        let read = buffer_access(&buffer, range, GpuBufferAccessKind::StorageRead);
        let write = buffer_access(&buffer, range, GpuBufferAccessKind::StorageWrite);
        let mut fragment = builder("normalize");
        fragment
            .declare_resource(GpuResourceRef::Buffer(buffer.clone()))
            .unwrap();
        add_compute(&mut fragment, "read write", [read.clone(), read, write]);
        let fragment = fragment.finish().unwrap();
        assert_eq!(fragment.nodes()[0].accesses().len(), 1);
        assert!(matches!(
            fragment.nodes()[0].accesses()[0],
            GpuResourceAccess::Buffer(ref access)
                if access.kind() == GpuBufferAccessKind::StorageReadWrite
        ));

        let mut invalid = builder("invalid normalize");
        invalid
            .declare_resource(GpuResourceRef::Buffer(buffer.clone()))
            .unwrap();
        let error = invalid
            .add_node(
                label("contradiction"),
                compute_operation(),
                [
                    buffer_access(&buffer, range, GpuBufferAccessKind::UniformRead),
                    buffer_access(&buffer, range, GpuBufferAccessKind::StorageWrite),
                ],
                GpuCapabilityRequirements::new(),
                GpuExecutionPreference::Automatic,
                provenance("contradiction"),
            )
            .unwrap_err();
        assert_eq!(
            error.cause(),
            GpuWorkAuthoringCause::IncompatibleSameNodeAccess
        );
    }

    #[test]
    fn descriptor_and_partial_write_initialization_are_region_aware() {
        let mut allocator = allocator();
        let zeroed = buffer(
            &mut allocator,
            "zeroed",
            GpuBufferInitialization::Zeroed,
            [GpuBufferUsage::Storage],
        );
        let zeroed_range = GpuBufferRange::whole(&zeroed).unwrap();
        let mut readable = builder("descriptor initialized");
        readable
            .declare_resource(GpuResourceRef::Buffer(zeroed.clone()))
            .unwrap();
        add_compute(
            &mut readable,
            "read",
            [buffer_access(
                &zeroed,
                zeroed_range,
                GpuBufferAccessKind::StorageRead,
            )],
        );
        assert!(
            GpuPreparedWorkGraph::prepare(label("descriptor graph"), [readable.finish().unwrap()])
                .is_ok()
        );

        let prepared_data = PreparedGpuData::<TransferData>::from_pod_transfer(
            "prepared buffer bytes",
            &[0_u8; 64],
            provenance("prepared buffer bytes"),
        )
        .unwrap();
        let prepared = buffer(
            &mut allocator,
            "prepared",
            GpuBufferInitialization::Prepared(prepared_data),
            [GpuBufferUsage::Storage, GpuBufferUsage::CopyDestination],
        );
        let mut readable = builder("prepared descriptor initialized");
        readable
            .declare_resource(GpuResourceRef::Buffer(prepared.clone()))
            .unwrap();
        add_compute(
            &mut readable,
            "read prepared",
            [buffer_access(
                &prepared,
                GpuBufferRange::whole(&prepared).unwrap(),
                GpuBufferAccessKind::StorageRead,
            )],
        );
        assert!(
            GpuPreparedWorkGraph::prepare(
                label("prepared descriptor graph"),
                [readable.finish().unwrap()]
            )
            .is_ok()
        );

        let uninitialized = buffer(
            &mut allocator,
            "uninitialized",
            GpuBufferInitialization::Uninitialized,
            [GpuBufferUsage::Storage],
        );
        let written = GpuBufferRange::new(&uninitialized, 0, 16).unwrap();
        let disjoint = GpuBufferRange::new(&uninitialized, 32, 16).unwrap();
        let mut partial = builder("partial");
        partial
            .declare_resource(GpuResourceRef::Buffer(uninitialized.clone()))
            .unwrap();
        add_compute(
            &mut partial,
            "write",
            [buffer_access(
                &uninitialized,
                written,
                GpuBufferAccessKind::StorageWrite,
            )],
        );
        add_compute(
            &mut partial,
            "read written",
            [buffer_access(
                &uninitialized,
                written,
                GpuBufferAccessKind::StorageRead,
            )],
        );
        add_compute(
            &mut partial,
            "read disjoint",
            [buffer_access(
                &uninitialized,
                disjoint,
                GpuBufferAccessKind::StorageRead,
            )],
        );
        let error =
            GpuPreparedWorkGraph::prepare(label("partial graph"), [partial.finish().unwrap()])
                .unwrap_err();
        assert_eq!(error.cause(), GpuWorkGraphCause::ReadBeforeInitialization);
        assert_eq!(error.resource(), Some(uninitialized.diagnostic_identity()));
    }

    #[test]
    fn texture_descriptor_initialization_distinguishes_zeroed_and_prepared_coverage() {
        let mut allocator = allocator();
        let zeroed = texture(
            &mut allocator,
            "zeroed texture",
            GpuTextureInitialization::Zeroed,
            2,
            1,
            [GpuTextureUsage::Sampled, GpuTextureUsage::CopyDestination],
        );
        let prepared = texture(
            &mut allocator,
            "prepared texture",
            prepared_texture_initialization("prepared texture"),
            2,
            1,
            [GpuTextureUsage::Sampled, GpuTextureUsage::CopyDestination],
        );
        let prepared_base = GpuTextureSubresourceRange::new(
            prepared.descriptor().common().label(),
            0,
            1,
            0,
            1,
            GpuTextureAspect::Color,
        )
        .unwrap();
        let mut fragment = builder("initialized textures");
        for resource in [&zeroed, &prepared] {
            fragment
                .declare_resource(GpuResourceRef::Texture(resource.clone()))
                .unwrap();
        }
        add_compute(
            &mut fragment,
            "read zeroed",
            [texture_access(
                &zeroed,
                GpuTextureSubresourceRange::whole(&zeroed).unwrap(),
                GpuTextureAccessKind::SampledRead,
            )],
        );
        add_compute(
            &mut fragment,
            "read prepared base",
            [texture_access(
                &prepared,
                prepared_base,
                GpuTextureAccessKind::SampledRead,
            )],
        );
        let graph = GpuPreparedWorkGraph::prepare(
            label("initialized texture graph"),
            [fragment.finish().unwrap()],
        )
        .unwrap();
        let initial_mip_count = |identity| {
            graph
                .initialization()
                .iter()
                .find(|summary| summary.resource().diagnostic_identity() == identity)
                .unwrap()
                .initial()
                .unwrap()
                .texture_subresource_values()
                .unwrap()
                .len()
        };
        assert_eq!(initial_mip_count(zeroed.diagnostic_identity()), 2);
        assert_eq!(initial_mip_count(prepared.diagnostic_identity()), 1);
    }

    #[test]
    fn texture_reads_reject_uninitialized_or_unprepared_mips() {
        let mut allocator = allocator();
        let prepared = texture(
            &mut allocator,
            "partially prepared texture",
            prepared_texture_initialization("partially prepared texture"),
            2,
            1,
            [GpuTextureUsage::Sampled, GpuTextureUsage::CopyDestination],
        );
        let uninitialized = texture(
            &mut allocator,
            "uninitialized texture",
            GpuTextureInitialization::Uninitialized,
            1,
            1,
            [GpuTextureUsage::Sampled],
        );
        let prepared_mip_one = GpuTextureSubresourceRange::new(
            prepared.descriptor().common().label(),
            1,
            1,
            0,
            1,
            GpuTextureAspect::Color,
        )
        .unwrap();
        for (name, texture, range) in [
            ("unprepared mip", prepared, prepared_mip_one),
            (
                "uninitialized texture",
                uninitialized.clone(),
                GpuTextureSubresourceRange::whole(&uninitialized).unwrap(),
            ),
        ] {
            let mut fragment = builder(name);
            fragment
                .declare_resource(GpuResourceRef::Texture(texture.clone()))
                .unwrap();
            add_compute(
                &mut fragment,
                "invalid read",
                [texture_access(
                    &texture,
                    range,
                    GpuTextureAccessKind::SampledRead,
                )],
            );
            assert_eq!(
                GpuPreparedWorkGraph::prepare(label(name), [fragment.finish().unwrap()])
                    .unwrap_err()
                    .cause(),
                GpuWorkGraphCause::ReadBeforeInitialization
            );
        }
    }

    #[test]
    fn inferred_hazards_are_typed_and_disjoint_regions_remain_independent() {
        let mut allocator = allocator();
        let buffer = buffer(
            &mut allocator,
            "hazards",
            GpuBufferInitialization::Zeroed,
            [GpuBufferUsage::Storage],
        );
        let first = GpuBufferRange::new(&buffer, 0, 16).unwrap();
        let second = GpuBufferRange::new(&buffer, 32, 16).unwrap();
        let mut fragment = builder("hazards");
        fragment
            .declare_resource(GpuResourceRef::Buffer(buffer.clone()))
            .unwrap();
        add_compute(
            &mut fragment,
            "read",
            [buffer_access(
                &buffer,
                first,
                GpuBufferAccessKind::StorageRead,
            )],
        );
        add_compute(
            &mut fragment,
            "write",
            [buffer_access(
                &buffer,
                first,
                GpuBufferAccessKind::StorageWrite,
            )],
        );
        add_compute(
            &mut fragment,
            "read again",
            [buffer_access(
                &buffer,
                first,
                GpuBufferAccessKind::StorageRead,
            )],
        );
        add_compute(
            &mut fragment,
            "disjoint write",
            [buffer_access(
                &buffer,
                second,
                GpuBufferAccessKind::StorageWrite,
            )],
        );
        let graph =
            GpuPreparedWorkGraph::prepare(label("hazard graph"), [fragment.finish().unwrap()])
                .unwrap();
        let reasons = graph
            .dependencies()
            .iter()
            .flat_map(GpuWorkDependency::reasons)
            .collect::<Vec<_>>();
        assert!(
            reasons
                .iter()
                .any(|reason| matches!(reason, GpuDependencyReason::WriteAfterRead { .. }))
        );
        assert!(
            reasons
                .iter()
                .any(|reason| matches!(reason, GpuDependencyReason::ReadAfterWrite { .. }))
        );
        assert!(graph.dependencies().iter().all(|dependency| {
            dependency.before().local_node() != 4 && dependency.after().local_node() != 4
        }));
    }

    #[test]
    fn buffer_hazard_truth_table_is_lexically_oriented() {
        let mut allocator = allocator();
        let buffer = buffer(
            &mut allocator,
            "truth table",
            GpuBufferInitialization::Zeroed,
            [GpuBufferUsage::Storage],
        );
        let range = GpuBufferRange::whole(&buffer).unwrap();
        let mut fragment = builder("truth table");
        fragment
            .declare_resource(GpuResourceRef::Buffer(buffer.clone()))
            .unwrap();
        for (name, kind) in [
            ("read one", GpuBufferAccessKind::StorageRead),
            ("read two", GpuBufferAccessKind::StorageRead),
            ("write one", GpuBufferAccessKind::StorageWrite),
            ("write two", GpuBufferAccessKind::StorageWrite),
            ("read write", GpuBufferAccessKind::StorageReadWrite),
        ] {
            add_compute(&mut fragment, name, [buffer_access(&buffer, range, kind)]);
        }
        let graph =
            GpuPreparedWorkGraph::prepare(label("truth table graph"), [fragment.finish().unwrap()])
                .unwrap();
        let reasons = |before, after| {
            graph.dependencies().iter().find_map(|dependency| {
                (dependency.before().local_node() == before
                    && dependency.after().local_node() == after)
                    .then(|| dependency.reasons())
            })
        };
        assert!(reasons(1, 2).is_none());
        assert!(
            reasons(1, 3)
                .unwrap()
                .iter()
                .any(|reason| { matches!(reason, GpuDependencyReason::WriteAfterRead { .. }) })
        );
        assert!(
            reasons(3, 4)
                .unwrap()
                .iter()
                .any(|reason| { matches!(reason, GpuDependencyReason::WriteAfterWrite { .. }) })
        );
        let read_write = reasons(4, 5).unwrap();
        assert!(
            read_write
                .iter()
                .any(|reason| matches!(reason, GpuDependencyReason::ReadAfterWrite { .. }))
        );
        assert!(
            read_write
                .iter()
                .any(|reason| matches!(reason, GpuDependencyReason::WriteAfterWrite { .. }))
        );
    }

    #[test]
    fn disjoint_query_ranges_remain_independent() {
        let mut allocator = allocator();
        let queries = allocator
            .allocate_query_set_handle(
                GpuQuerySetDescriptor::new(common("disjoint queries"), GpuQueryKind::Timestamp, 4)
                    .unwrap(),
            )
            .unwrap();
        let operation = |range| {
            GpuWorkOperation::Render(
                GpuRenderOperation::new(
                    [],
                    None,
                    [],
                    [
                        GpuQueryAccess::new(&queries, range, GpuQueryAccessKind::WriteTimestamp)
                            .unwrap(),
                    ],
                )
                .unwrap(),
            )
        };
        let mut fragment = builder("disjoint queries");
        fragment
            .declare_resource(GpuResourceRef::QuerySet(queries.clone()))
            .unwrap();
        for (name, range) in [
            ("first queries", GpuQueryRange::new(&queries, 0, 2).unwrap()),
            (
                "second queries",
                GpuQueryRange::new(&queries, 2, 2).unwrap(),
            ),
        ] {
            fragment
                .add_node(
                    label(name),
                    operation(range),
                    [],
                    GpuCapabilityRequirements::new(),
                    GpuExecutionPreference::GraphicsRequired,
                    provenance(name),
                )
                .unwrap();
        }
        let graph = GpuPreparedWorkGraph::prepare(
            label("disjoint query graph"),
            [fragment.finish().unwrap()],
        )
        .unwrap();
        assert!(graph.dependencies().is_empty());
    }

    #[test]
    fn attachment_store_preserves_and_discard_invalidates_exact_coverage() {
        let mut allocator = allocator();
        let texture = texture(
            &mut allocator,
            "attachment",
            GpuTextureInitialization::Uninitialized,
            1,
            1,
            [GpuTextureUsage::ColorAttachment, GpuTextureUsage::Sampled],
        );
        let range = GpuTextureSubresourceRange::whole(&texture).unwrap();
        let render = |store| {
            GpuWorkOperation::Render(
                GpuRenderOperation::new(
                    [GpuRenderColorAttachment::new(
                        GpuTextureAccessResource::Texture(texture.clone()),
                        range,
                        GpuColorAttachmentLoad::Clear(
                            GpuColorClearValue::new(0.0, 0.0, 0.0, 1.0).unwrap(),
                        ),
                        store,
                        None,
                    )
                    .unwrap()],
                    None,
                    [],
                    [],
                )
                .unwrap(),
            )
        };
        let sampled = || {
            GpuResourceAccess::Texture(
                GpuTextureAccess::new(
                    GpuTextureAccessResource::Texture(texture.clone()),
                    range,
                    GpuTextureAccessKind::SampledRead,
                )
                .unwrap(),
            )
        };

        let mut stored = builder("stored");
        stored
            .declare_resource(GpuResourceRef::Texture(texture.clone()))
            .unwrap();
        stored
            .add_node(
                label("clear store"),
                render(GpuAttachmentStore::Store),
                [],
                GpuCapabilityRequirements::new(),
                GpuExecutionPreference::GraphicsRequired,
                provenance("clear store"),
            )
            .unwrap();
        add_compute(&mut stored, "sample", [sampled()]);
        assert!(
            GpuPreparedWorkGraph::prepare(label("stored graph"), [stored.finish().unwrap()])
                .is_ok()
        );

        let mut discarded = builder("discarded");
        discarded
            .declare_resource(GpuResourceRef::Texture(texture.clone()))
            .unwrap();
        discarded
            .add_node(
                label("clear discard"),
                render(GpuAttachmentStore::Discard),
                [],
                GpuCapabilityRequirements::new(),
                GpuExecutionPreference::GraphicsRequired,
                provenance("clear discard"),
            )
            .unwrap();
        add_compute(&mut discarded, "sample", [sampled()]);
        let error =
            GpuPreparedWorkGraph::prepare(label("discard graph"), [discarded.finish().unwrap()])
                .unwrap_err();
        assert_eq!(error.cause(), GpuWorkGraphCause::ReadBeforeInitialization);
    }

    #[test]
    fn depth_attachment_load_clear_store_and_discard_drive_initialization() {
        let mut allocator = allocator();
        let depth = depth_texture(&mut allocator, "depth attachment");
        let range = GpuTextureSubresourceRange::whole(&depth).unwrap();
        let render = |load, store, draws: Vec<GpuDrawIntent>| {
            GpuWorkOperation::Render(
                GpuRenderOperation::new(
                    [],
                    Some(
                        GpuRenderDepthStencilAttachment::new(
                            GpuTextureAccessResource::Texture(depth.clone()),
                            range,
                            GpuDepthStencilAccess::ReadWrite,
                            load,
                            store,
                        )
                        .unwrap(),
                    ),
                    draws,
                    [],
                )
                .unwrap(),
            )
        };
        let sampled = || texture_access(&depth, range, GpuTextureAccessKind::SampledRead);
        let clear = GpuDepthAttachmentLoad::Clear(GpuDepthClearValue::new(0.5).unwrap());
        for (name, store, succeeds) in [
            ("stored depth", GpuAttachmentStore::Store, true),
            ("discarded depth", GpuAttachmentStore::Discard, false),
        ] {
            let mut fragment = builder(name);
            fragment
                .declare_resource(GpuResourceRef::Texture(depth.clone()))
                .unwrap();
            fragment
                .add_node(
                    label("clear depth"),
                    render(clear, store, Vec::new()),
                    [],
                    GpuCapabilityRequirements::new(),
                    GpuExecutionPreference::GraphicsRequired,
                    provenance("clear depth"),
                )
                .unwrap();
            add_compute(&mut fragment, "sample depth", [sampled()]);
            let result = GpuPreparedWorkGraph::prepare(label(name), [fragment.finish().unwrap()]);
            assert_eq!(result.is_ok(), succeeds);
            if !succeeds {
                assert_eq!(
                    result.unwrap_err().cause(),
                    GpuWorkGraphCause::ReadBeforeInitialization
                );
            }
        }

        let mut load = builder("load depth");
        load.declare_resource(GpuResourceRef::Texture(depth.clone()))
            .unwrap();
        let draw = GpuDrawIntent::direct(
            GpuDrawRange::new(0, 3).unwrap(),
            GpuDrawRange::new(0, 1).unwrap(),
        );
        load.add_node(
            label("load depth"),
            render(
                GpuDepthAttachmentLoad::Load,
                GpuAttachmentStore::Store,
                vec![draw],
            ),
            [],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::GraphicsRequired,
            provenance("load depth"),
        )
        .unwrap();
        assert_eq!(
            GpuPreparedWorkGraph::prepare(label("load depth graph"), [load.finish().unwrap()])
                .unwrap_err()
                .cause(),
            GpuWorkGraphCause::ReadBeforeInitialization
        );
    }

    #[test]
    fn timestamp_resolve_and_copy_form_one_initialized_dependency_chain() {
        let mut allocator = allocator();
        let queries = allocator
            .allocate_query_set_handle(
                GpuQuerySetDescriptor::new(common("timestamps"), GpuQueryKind::Timestamp, 2)
                    .unwrap(),
            )
            .unwrap();
        let resolve = buffer(
            &mut allocator,
            "resolve",
            GpuBufferInitialization::Uninitialized,
            [GpuBufferUsage::QueryResolve, GpuBufferUsage::CopySource],
        );
        let readback = buffer(
            &mut allocator,
            "readback",
            GpuBufferInitialization::Uninitialized,
            [GpuBufferUsage::CopyDestination],
        );
        let query_range = GpuQueryRange::whole(&queries).unwrap();
        let timestamp_access =
            GpuQueryAccess::new(&queries, query_range, GpuQueryAccessKind::WriteTimestamp).unwrap();
        let render = GpuWorkOperation::Render(
            GpuRenderOperation::new([], None, [], [timestamp_access]).unwrap(),
        );
        let query_resolve =
            GpuQueryResolveOperation::new(&queries, query_range, &resolve, 0).unwrap();
        let resolve_range = query_resolve.destination_range();
        let mut unresolved = builder("unresolved timing");
        for resource in [
            GpuResourceRef::QuerySet(queries.clone()),
            GpuResourceRef::Buffer(resolve.clone()),
        ] {
            unresolved.declare_resource(resource).unwrap();
        }
        unresolved
            .add_node(
                label("resolve unwritten timestamps"),
                GpuWorkOperation::Resolve(query_resolve.clone()),
                [],
                GpuCapabilityRequirements::new(),
                GpuExecutionPreference::TransferPreferred,
                provenance("resolve unwritten timestamps"),
            )
            .unwrap();
        assert_eq!(
            GpuPreparedWorkGraph::prepare(
                label("unresolved timing graph"),
                [unresolved.finish().unwrap()],
            )
            .unwrap_err()
            .cause(),
            GpuWorkGraphCause::ReadBeforeInitialization
        );
        let copy = GpuCopyOperation::buffer_to_buffer(
            GpuBufferRegion::new(&resolve, resolve_range).unwrap(),
            GpuBufferRegion::new(
                &readback,
                GpuBufferRange::new(&readback, 0, resolve_range.size()).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();

        let mut fragment = builder("timing");
        for resource in [
            GpuResourceRef::QuerySet(queries),
            GpuResourceRef::Buffer(resolve),
            GpuResourceRef::Buffer(readback),
        ] {
            fragment.declare_resource(resource).unwrap();
        }
        fragment
            .add_node(
                label("write timestamps"),
                render,
                [],
                GpuCapabilityRequirements::new(),
                GpuExecutionPreference::GraphicsRequired,
                provenance("write timestamps"),
            )
            .unwrap();
        fragment
            .add_node(
                label("resolve timestamps"),
                GpuWorkOperation::Resolve(query_resolve),
                [],
                GpuCapabilityRequirements::new(),
                GpuExecutionPreference::TransferPreferred,
                provenance("resolve timestamps"),
            )
            .unwrap();
        fragment
            .add_node(
                label("copy readback"),
                GpuWorkOperation::Copy(copy),
                [],
                GpuCapabilityRequirements::new(),
                GpuExecutionPreference::TransferPreferred,
                provenance("copy readback"),
            )
            .unwrap();
        let fragment = fragment.finish().unwrap();
        let graph =
            GpuPreparedWorkGraph::prepare(label("timing graph"), [fragment.clone()]).unwrap();
        let repeated = GpuPreparedWorkGraph::prepare(label("timing graph"), [fragment]).unwrap();
        assert_eq!(graph.nodes(), repeated.nodes());
        assert_eq!(graph.topological_order(), repeated.topological_order());
        assert_eq!(graph.dependencies(), repeated.dependencies());
        assert_eq!(graph.initialization(), repeated.initialization());
        assert_eq!(graph.requirements(), repeated.requirements());
        assert_eq!(graph.outputs(), repeated.outputs());
        assert_eq!(graph.diagnostics(), repeated.diagnostics());
        assert_eq!(
            graph
                .nodes()
                .iter()
                .map(GpuPreparedWorkNode::id)
                .collect::<Vec<_>>(),
            graph.topological_order()
        );
        assert_eq!(
            graph
                .nodes()
                .iter()
                .map(|node| node.node().label().as_str())
                .collect::<Vec<_>>(),
            vec!["write timestamps", "resolve timestamps", "copy readback"]
        );
        assert!(graph.nodes().iter().all(|node| {
            node.node()
                .accesses()
                .windows(2)
                .all(|pair| pair[0] < pair[1])
        }));
        assert!(graph.dependencies().iter().any(|dependency| {
            dependency.before().local_node() == 1 && dependency.after().local_node() == 2
        }));
        assert!(graph.dependencies().iter().any(|dependency| {
            dependency.before().local_node() == 2 && dependency.after().local_node() == 3
        }));
        assert_eq!(
            graph
                .requirements()
                .get(GpuCapabilityFeature::TimestampQuery),
            Some(GpuCapabilityRequirement::Required(
                GpuCapabilityFeature::TimestampQuery
            ))
        );
        assert_eq!(
            graph
                .requirements()
                .iter()
                .map(GpuCapabilityRequirement::feature)
                .collect::<Vec<_>>(),
            vec![
                GpuCapabilityFeature::RenderPipeline,
                GpuCapabilityFeature::Copy,
                GpuCapabilityFeature::TimestampQuery,
            ]
        );
    }

    fn producer_fragment(
        buffer: &GpuBufferHandle,
        key: GpuExportKey,
        range: GpuBufferRange,
    ) -> GpuWorkFragment {
        let mut producer = builder("producer");
        producer
            .declare_resource(GpuResourceRef::Buffer(buffer.clone()))
            .unwrap();
        add_compute(
            &mut producer,
            "produce",
            [buffer_access(
                buffer,
                range,
                GpuBufferAccessKind::StorageWrite,
            )],
        );
        let coverage = GpuInitialCoverage::buffer_ranges(buffer, [range]).unwrap();
        producer
            .add_output(
                GpuWorkOutput::new(
                    GpuExportRelationship::new(
                        GpuResourceRef::Buffer(buffer.clone()),
                        key,
                        GpuResourceAccessIntent::Write,
                        provenance("producer output"),
                    ),
                    coverage,
                )
                .unwrap(),
            )
            .unwrap();
        producer.finish().unwrap()
    }

    fn consumer_fragment(
        buffer: &GpuBufferHandle,
        key: GpuExportKey,
        range: GpuBufferRange,
    ) -> GpuWorkFragment {
        let mut consumer = builder("consumer");
        consumer
            .declare_resource(GpuResourceRef::Buffer(buffer.clone()))
            .unwrap();
        consumer
            .add_import(GpuWorkImport::new(
                GpuResourceRef::Buffer(buffer.clone()),
                key,
                GpuResourceAccessIntent::Read,
                provenance("consumer import"),
            ))
            .unwrap();
        add_compute(
            &mut consumer,
            "consume",
            [buffer_access(
                buffer,
                range,
                GpuBufferAccessKind::StorageRead,
            )],
        );
        consumer.finish().unwrap()
    }

    fn semantic_dependencies(
        graph: &GpuPreparedWorkGraph,
    ) -> Vec<(String, String, Vec<GpuDependencyReason>)> {
        let node_label = |id| {
            graph
                .nodes()
                .iter()
                .find(|node| node.id() == id)
                .unwrap()
                .node()
                .label()
                .as_str()
                .to_string()
        };
        graph
            .dependencies()
            .iter()
            .map(|dependency| {
                (
                    node_label(dependency.before()),
                    node_label(dependency.after()),
                    dependency.reasons().to_vec(),
                )
            })
            .collect()
    }

    #[test]
    fn typed_import_export_causality_overrides_fragment_input_order() {
        let mut allocator = allocator();
        let shared = buffer(
            &mut allocator,
            "shared",
            GpuBufferInitialization::Uninitialized,
            [GpuBufferUsage::Storage],
        );
        let range = GpuBufferRange::new(&shared, 0, 32).unwrap();
        let key = GpuExportKey::new("shared.ready").unwrap();
        let producer = producer_fragment(&shared, key.clone(), range);
        let consumer = consumer_fragment(&shared, key, range);
        let producer_first = GpuPreparedWorkGraph::prepare(
            label("composition"),
            [producer.clone(), consumer.clone()],
        )
        .unwrap();
        let consumer_first =
            GpuPreparedWorkGraph::prepare(label("composition"), [consumer, producer]).unwrap();
        assert_eq!(
            semantic_dependencies(&producer_first),
            semantic_dependencies(&consumer_first)
        );
        assert_eq!(
            producer_first.initialization(),
            consumer_first.initialization()
        );
        assert_eq!(producer_first.requirements(), consumer_first.requirements());
        assert_eq!(producer_first.outputs(), consumer_first.outputs());
        assert_eq!(consumer_first.topological_order()[0].fragment_ordinal(), 1);
        assert_eq!(consumer_first.topological_order()[1].fragment_ordinal(), 0);
        assert!(
            consumer_first.dependencies()[0]
                .reasons()
                .iter()
                .any(|reason| matches!(reason, GpuDependencyReason::ReadAfterWrite { .. }))
        );
    }

    #[test]
    fn cross_fragment_conflict_without_typed_causality_fails_before_initialization() {
        let mut allocator = allocator();
        let shared = buffer(
            &mut allocator,
            "unbound shared",
            GpuBufferInitialization::Uninitialized,
            [GpuBufferUsage::Storage],
        );
        let range = GpuBufferRange::whole(&shared).unwrap();
        let mut writer = builder("writer");
        writer
            .declare_resource(GpuResourceRef::Buffer(shared.clone()))
            .unwrap();
        add_compute(
            &mut writer,
            "write",
            [buffer_access(
                &shared,
                range,
                GpuBufferAccessKind::StorageWrite,
            )],
        );
        let mut reader = builder("reader");
        reader
            .declare_resource(GpuResourceRef::Buffer(shared.clone()))
            .unwrap();
        add_compute(
            &mut reader,
            "read",
            [buffer_access(
                &shared,
                range,
                GpuBufferAccessKind::StorageRead,
            )],
        );
        let writer = writer.finish().unwrap();
        let reader = reader.finish().unwrap();
        let writer_first = GpuPreparedWorkGraph::prepare(
            label("missing causality"),
            [writer.clone(), reader.clone()],
        )
        .unwrap_err();
        let reader_first =
            GpuPreparedWorkGraph::prepare(label("missing causality"), [reader, writer])
                .unwrap_err();
        assert_eq!(
            writer_first.cause(),
            GpuWorkGraphCause::MissingCrossFragmentCausality
        );
        assert_eq!(writer_first.cause(), reader_first.cause());
        assert_eq!(writer_first.resource(), reader_first.resource());
    }

    #[test]
    fn imports_reject_mismatched_resources_and_insufficient_export_coverage() {
        let mut allocator = allocator();
        let produced = buffer(
            &mut allocator,
            "produced",
            GpuBufferInitialization::Uninitialized,
            [GpuBufferUsage::Storage],
        );
        let other = buffer(
            &mut allocator,
            "other",
            GpuBufferInitialization::Uninitialized,
            [GpuBufferUsage::Storage],
        );
        let produced_range = GpuBufferRange::new(&produced, 0, 16).unwrap();
        let key = GpuExportKey::new("produced.ready").unwrap();
        let producer = producer_fragment(&produced, key.clone(), produced_range);
        let mismatched_consumer = consumer_fragment(
            &other,
            key.clone(),
            GpuBufferRange::new(&other, 0, 16).unwrap(),
        );
        assert_eq!(
            GpuPreparedWorkGraph::prepare(
                label("mismatched resource"),
                [producer.clone(), mismatched_consumer],
            )
            .unwrap_err()
            .cause(),
            GpuWorkGraphCause::ImportExportMismatch
        );

        let oversized_consumer =
            consumer_fragment(&produced, key, GpuBufferRange::whole(&produced).unwrap());
        assert_eq!(
            GpuPreparedWorkGraph::prepare(
                label("insufficient coverage"),
                [producer, oversized_consumer],
            )
            .unwrap_err()
            .cause(),
            GpuWorkGraphCause::ReadBeforeInitialization
        );
    }

    #[test]
    fn duplicate_export_keys_and_ambiguous_writers_are_rejected() {
        let mut allocator = allocator();
        let shared = buffer(
            &mut allocator,
            "multi producer",
            GpuBufferInitialization::Uninitialized,
            [GpuBufferUsage::Storage],
        );
        let range = GpuBufferRange::whole(&shared).unwrap();
        let duplicate = GpuExportKey::new("duplicate").unwrap();
        let first = producer_fragment(&shared, duplicate.clone(), range);
        let second = producer_fragment(&shared, duplicate, range);
        let error =
            GpuPreparedWorkGraph::prepare(label("duplicates"), [first, second]).unwrap_err();
        assert_eq!(error.cause(), GpuWorkGraphCause::DuplicateExportKey);

        let first = producer_fragment(&shared, GpuExportKey::new("first").unwrap(), range);
        let second = producer_fragment(&shared, GpuExportKey::new("second").unwrap(), range);
        let error = GpuPreparedWorkGraph::prepare(label("ambiguous"), [first, second]).unwrap_err();
        assert_eq!(error.cause(), GpuWorkGraphCause::AmbiguousWriter);
    }

    #[test]
    fn explicit_non_data_order_succeeds_and_rejects_duplicate_or_unknown_endpoints() {
        let mut ordered = builder("ordered");
        let first = add_compute(&mut ordered, "first", []);
        let second = add_compute(&mut ordered, "second", []);
        ordered
            .add_explicit_order(GpuExplicitOrder::new(&first, &second, "phase order").unwrap())
            .unwrap();
        let graph =
            GpuPreparedWorkGraph::prepare(label("ordered graph"), [ordered.finish().unwrap()])
                .unwrap();
        assert_eq!(graph.dependencies().len(), 1);
        assert_eq!(
            graph.dependencies()[0].reasons(),
            [GpuDependencyReason::ExplicitNonData {
                reason: "phase order".to_string(),
            }]
        );

        let mut duplicate = builder("duplicate");
        let first = add_compute(&mut duplicate, "first", []);
        let second = add_compute(&mut duplicate, "second", []);
        let order = GpuExplicitOrder::new(&first, &second, "one edge").unwrap();
        duplicate.add_explicit_order(order.clone()).unwrap();
        assert_eq!(
            duplicate.add_explicit_order(order).unwrap_err().cause(),
            GpuWorkAuthoringCause::DuplicateExplicitOrder
        );

        let mut missing = builder("missing endpoint");
        let first = add_compute(&mut missing, "first", []);
        let second = add_compute(&mut missing, "second", []);
        missing
            .add_explicit_order(GpuExplicitOrder::new(&first, &second, "missing").unwrap())
            .unwrap();
        let mut missing = missing.finish().unwrap();
        missing.nodes.pop();
        assert_eq!(
            GpuPreparedWorkGraph::prepare(label("missing endpoint graph"), [missing])
                .unwrap_err()
                .cause(),
            GpuWorkGraphCause::UnknownIdentity
        );
    }

    #[test]
    fn explicit_order_rejects_redundancy_conflict_and_cycles() {
        let mut allocator = allocator();
        let resource = buffer(
            &mut allocator,
            "explicit",
            GpuBufferInitialization::Zeroed,
            [GpuBufferUsage::Storage],
        );
        let range = GpuBufferRange::whole(&resource).unwrap();

        let data_fragment = |reverse: bool| {
            let mut fragment = builder(if reverse { "conflict" } else { "redundant" });
            fragment
                .declare_resource(GpuResourceRef::Buffer(resource.clone()))
                .unwrap();
            let write = add_compute(
                &mut fragment,
                "write",
                [buffer_access(
                    &resource,
                    range,
                    GpuBufferAccessKind::StorageWrite,
                )],
            );
            let read = add_compute(
                &mut fragment,
                "read",
                [buffer_access(
                    &resource,
                    range,
                    GpuBufferAccessKind::StorageRead,
                )],
            );
            let order = if reverse {
                GpuExplicitOrder::new(&read, &write, "reverse data").unwrap()
            } else {
                GpuExplicitOrder::new(&write, &read, "duplicate data").unwrap()
            };
            fragment.add_explicit_order(order).unwrap();
            fragment.finish().unwrap()
        };
        assert_eq!(
            GpuPreparedWorkGraph::prepare(label("redundant graph"), [data_fragment(false)])
                .unwrap_err()
                .cause(),
            GpuWorkGraphCause::RedundantExplicitDataOrder
        );
        assert_eq!(
            GpuPreparedWorkGraph::prepare(label("conflict graph"), [data_fragment(true)])
                .unwrap_err()
                .cause(),
            GpuWorkGraphCause::ExplicitOrderConflict
        );

        let mut cycle = builder("cycle");
        let first = add_compute(&mut cycle, "first", []);
        let second = add_compute(&mut cycle, "second", []);
        let third = add_compute(&mut cycle, "third", []);
        for order in [
            GpuExplicitOrder::new(&first, &second, "one").unwrap(),
            GpuExplicitOrder::new(&second, &third, "two").unwrap(),
            GpuExplicitOrder::new(&third, &first, "three").unwrap(),
        ] {
            cycle.add_explicit_order(order).unwrap();
        }
        assert_eq!(
            GpuPreparedWorkGraph::prepare(label("cycle graph"), [cycle.finish().unwrap()])
                .unwrap_err()
                .cause(),
            GpuWorkGraphCause::Cycle
        );
    }

    #[test]
    fn foreign_node_identity_and_capability_contradiction_fail_structurally() {
        let mut first = builder("first");
        let first_node = add_compute(&mut first, "first node", []);
        let mut second = builder("second");
        let second_node = add_compute(&mut second, "second node", []);
        assert_eq!(
            GpuExplicitOrder::new(&first_node, &second_node, "foreign")
                .unwrap_err()
                .cause(),
            GpuWorkAuthoringCause::ForeignIdentity
        );

        let mut requirements = GpuCapabilityRequirements::new();
        requirements
            .insert(GpuCapabilityRequirement::Disabled(
                GpuCapabilityFeature::Compute,
            ))
            .unwrap();
        let error = first
            .add_node(
                label("disabled compute"),
                compute_operation(),
                [],
                requirements,
                GpuExecutionPreference::Automatic,
                provenance("disabled compute"),
            )
            .unwrap_err();
        assert_eq!(
            error.cause(),
            GpuWorkAuthoringCause::MechanicalCapabilityContradiction
        );
    }

    #[test]
    fn prepared_graph_rejects_a_foreign_resource_identity() {
        let mut allocator = allocator();
        let declared = buffer(
            &mut allocator,
            "declared",
            GpuBufferInitialization::Zeroed,
            [GpuBufferUsage::Storage],
        );
        let foreign = buffer(
            &mut allocator,
            "foreign",
            GpuBufferInitialization::Zeroed,
            [GpuBufferUsage::Storage],
        );
        let mut fragment = builder("foreign resource");
        fragment
            .declare_resource(GpuResourceRef::Buffer(declared.clone()))
            .unwrap();
        add_compute(
            &mut fragment,
            "read",
            [buffer_access(
                &declared,
                GpuBufferRange::whole(&declared).unwrap(),
                GpuBufferAccessKind::StorageRead,
            )],
        );
        let mut fragment = fragment.finish().unwrap();
        fragment.nodes[0].accesses = vec![buffer_access(
            &foreign,
            GpuBufferRange::whole(&foreign).unwrap(),
            GpuBufferAccessKind::StorageRead,
        )];
        let error =
            GpuPreparedWorkGraph::prepare(label("foreign resource graph"), [fragment]).unwrap_err();
        assert_eq!(error.cause(), GpuWorkGraphCause::UnknownIdentity);
        assert_eq!(error.resource(), Some(foreign.diagnostic_identity()));
    }

    #[test]
    fn disjoint_texture_subresources_do_not_create_false_dependencies() {
        let mut allocator = allocator();
        let texture = texture(
            &mut allocator,
            "subresources",
            GpuTextureInitialization::Uninitialized,
            2,
            2,
            [GpuTextureUsage::StorageWrite],
        );
        let first_range = GpuTextureSubresourceRange::new(
            texture.descriptor().common().label(),
            0,
            1,
            0,
            1,
            GpuTextureAspect::Color,
        )
        .unwrap();
        let second_range = GpuTextureSubresourceRange::new(
            texture.descriptor().common().label(),
            1,
            1,
            1,
            1,
            GpuTextureAspect::Color,
        )
        .unwrap();
        let access = |range| {
            GpuResourceAccess::Texture(
                GpuTextureAccess::new(
                    GpuTextureAccessResource::Texture(texture.clone()),
                    range,
                    GpuTextureAccessKind::StorageWrite,
                )
                .unwrap(),
            )
        };
        let mut fragment = builder("texture disjoint");
        fragment
            .declare_resource(GpuResourceRef::Texture(texture.clone()))
            .unwrap();
        add_compute(&mut fragment, "first mip", [access(first_range)]);
        add_compute(&mut fragment, "second mip", [access(second_range)]);
        let graph =
            GpuPreparedWorkGraph::prepare(label("texture graph"), [fragment.finish().unwrap()])
                .unwrap();
        assert!(graph.dependencies().is_empty());
        assert_eq!(graph.topological_order().len(), 2);
    }

    #[test]
    fn independent_ready_nodes_have_deterministic_inspection_order() {
        let mut first = builder("first fragment");
        add_compute(&mut first, "first one", []);
        add_compute(&mut first, "first two", []);
        let mut second = builder("second fragment");
        add_compute(&mut second, "second one", []);
        let graph = GpuPreparedWorkGraph::prepare(
            label("deterministic"),
            [first.finish().unwrap(), second.finish().unwrap()],
        )
        .unwrap();
        assert_eq!(
            graph
                .topological_order()
                .iter()
                .map(|id| (id.fragment_ordinal(), id.local_node()))
                .collect::<Vec<_>>(),
            vec![(0, 1), (0, 2), (1, 1)]
        );
    }

    #[test]
    fn buffer_zero_initializes_only_its_checked_range() {
        let mut allocator = allocator();
        let buffer = buffer(
            &mut allocator,
            "zero region",
            GpuBufferInitialization::Uninitialized,
            [GpuBufferUsage::CopyDestination, GpuBufferUsage::Storage],
        );
        let zeroed = GpuBufferRange::new(&buffer, 8, 16).unwrap();
        let clear =
            GpuClearOperation::buffer_zero(GpuBufferRegion::new(&buffer, zeroed).unwrap()).unwrap();
        let mut fragment = builder("buffer zero");
        fragment
            .declare_resource(GpuResourceRef::Buffer(buffer.clone()))
            .unwrap();
        fragment
            .add_node(
                label("zero"),
                GpuWorkOperation::Clear(clear),
                [],
                GpuCapabilityRequirements::new(),
                GpuExecutionPreference::TransferPreferred,
                provenance("zero"),
            )
            .unwrap();
        add_compute(
            &mut fragment,
            "read zeroed",
            [buffer_access(
                &buffer,
                zeroed,
                GpuBufferAccessKind::StorageRead,
            )],
        );
        let graph =
            GpuPreparedWorkGraph::prepare(label("buffer zero graph"), [fragment.finish().unwrap()])
                .unwrap();
        let summary = graph
            .initialization()
            .iter()
            .find(|summary| {
                summary.resource().diagnostic_identity() == buffer.diagnostic_identity()
            })
            .unwrap();
        let final_ranges = summary
            .final_coverage()
            .unwrap()
            .buffer_range_values()
            .unwrap();
        assert_eq!(final_ranges, [zeroed]);
    }

    #[test]
    fn explicit_graph_entry_input_initializes_only_declared_coverage() {
        let mut allocator = allocator();
        let imported_label = label("imported retained");
        let retained_label = label("owned retained");
        let commons = [
            GpuResourceCommon::imported(
                imported_label.clone(),
                GpuResourceLifetime::Retained,
                provenance("imported retained"),
            ),
            GpuResourceCommon::owned(
                retained_label.clone(),
                GpuResourceLifetime::Retained,
                GpuMemoryIntent::Device,
                GpuReconstruction::SourceBacked,
                provenance("owned retained"),
            )
            .unwrap(),
        ];
        for common in commons {
            let resource_label = common.label().clone();
            let resource = allocator
                .allocate_buffer_handle(
                    GpuBufferDescriptor::new(
                        common,
                        64,
                        GpuBufferUsages::new(&resource_label, [GpuBufferUsage::Storage]).unwrap(),
                        GpuBufferInitialization::Uninitialized,
                    )
                    .unwrap(),
                )
                .unwrap();
            let initialized = GpuBufferRange::new(&resource, 16, 16).unwrap();
            let input = GpuWorkResourceInput::new(
                GpuResourceRef::Buffer(resource.clone()),
                GpuInitialCoverage::buffer_ranges(&resource, [initialized]).unwrap(),
                provenance("external input"),
            )
            .unwrap();
            let prepare_read = |name, read_range| {
                let mut fragment = builder(name);
                fragment
                    .declare_resource(GpuResourceRef::Buffer(resource.clone()))
                    .unwrap();
                fragment.add_input(input.clone()).unwrap();
                add_compute(
                    &mut fragment,
                    "read input",
                    [buffer_access(
                        &resource,
                        read_range,
                        GpuBufferAccessKind::StorageRead,
                    )],
                );
                GpuPreparedWorkGraph::prepare(label(name), [fragment.finish().unwrap()])
            };
            assert!(prepare_read("initialized input", initialized).is_ok());
            assert_eq!(
                prepare_read(
                    "uninitialized input",
                    GpuBufferRange::new(&resource, 0, 16).unwrap(),
                )
                .unwrap_err()
                .cause(),
                GpuWorkGraphCause::ReadBeforeInitialization
            );
        }
    }

    #[test]
    fn output_access_and_coverage_mismatches_are_structured() {
        let mut allocator = allocator();
        let resource = buffer(
            &mut allocator,
            "mismatch",
            GpuBufferInitialization::Uninitialized,
            [GpuBufferUsage::Storage],
        );
        let written = GpuBufferRange::new(&resource, 0, 16).unwrap();
        let overstated = GpuBufferRange::new(&resource, 0, 32).unwrap();
        let make_fragment = |intent, coverage_range| {
            let mut fragment = builder("mismatch producer");
            fragment
                .declare_resource(GpuResourceRef::Buffer(resource.clone()))
                .unwrap();
            add_compute(
                &mut fragment,
                "write",
                [buffer_access(
                    &resource,
                    written,
                    GpuBufferAccessKind::StorageWrite,
                )],
            );
            fragment
                .add_output(
                    GpuWorkOutput::new(
                        GpuExportRelationship::new(
                            GpuResourceRef::Buffer(resource.clone()),
                            GpuExportKey::new("mismatch.output").unwrap(),
                            intent,
                            provenance("mismatch output"),
                        ),
                        GpuInitialCoverage::buffer_ranges(&resource, [coverage_range]).unwrap(),
                    )
                    .unwrap(),
                )
                .unwrap();
            fragment.finish().unwrap()
        };
        assert_eq!(
            GpuPreparedWorkGraph::prepare(
                label("intent mismatch"),
                [make_fragment(GpuResourceAccessIntent::Read, written)]
            )
            .unwrap_err()
            .cause(),
            GpuWorkGraphCause::ImportExportMismatch
        );
        assert_eq!(
            GpuPreparedWorkGraph::prepare(
                label("coverage mismatch"),
                [make_fragment(GpuResourceAccessIntent::Write, overstated)]
            )
            .unwrap_err()
            .cause(),
            GpuWorkGraphCause::ImportExportMismatch
        );
    }

    #[test]
    fn multisample_resolve_initializes_destination_despite_source_discard() {
        let mut allocator = allocator();
        let resource_label = label("msaa");
        let source = allocator
            .allocate_texture_handle(
                GpuTextureDescriptor::new(
                    common("msaa"),
                    GpuTextureDimension::D2,
                    GpuTextureExtent::new(&resource_label, GpuTextureDimension::D2, 8, 8, 1)
                        .unwrap(),
                    1,
                    4,
                    GpuTextureFormat::Rgba8Unorm,
                    GpuTextureUsages::new(&resource_label, [GpuTextureUsage::ColorAttachment])
                        .unwrap(),
                    GpuTextureInitialization::Uninitialized,
                )
                .unwrap(),
            )
            .unwrap();
        let destination = texture(
            &mut allocator,
            "resolved",
            GpuTextureInitialization::Uninitialized,
            1,
            1,
            [GpuTextureUsage::ColorAttachment, GpuTextureUsage::Sampled],
        );
        let source_range = GpuTextureSubresourceRange::whole(&source).unwrap();
        let destination_range = GpuTextureSubresourceRange::whole(&destination).unwrap();
        let resolve_target = GpuMultisampleResolveTarget::new(
            GpuTextureAccessResource::Texture(destination.clone()),
            destination_range,
        )
        .unwrap();
        let attachment = GpuRenderColorAttachment::new(
            GpuTextureAccessResource::Texture(source.clone()),
            source_range,
            GpuColorAttachmentLoad::Clear(GpuColorClearValue::new(0.0, 0.0, 0.0, 1.0).unwrap()),
            GpuAttachmentStore::Discard,
            Some(resolve_target),
        )
        .unwrap();
        let mut fragment = builder("resolve");
        for resource in [
            GpuResourceRef::Texture(source),
            GpuResourceRef::Texture(destination.clone()),
        ] {
            fragment.declare_resource(resource).unwrap();
        }
        fragment
            .add_node(
                label("render resolve"),
                GpuWorkOperation::Render(
                    GpuRenderOperation::new([attachment], None, [], []).unwrap(),
                ),
                [],
                GpuCapabilityRequirements::new(),
                GpuExecutionPreference::GraphicsRequired,
                provenance("render resolve"),
            )
            .unwrap();
        add_compute(
            &mut fragment,
            "sample resolve",
            [GpuResourceAccess::Texture(
                GpuTextureAccess::new(
                    GpuTextureAccessResource::Texture(destination),
                    destination_range,
                    GpuTextureAccessKind::SampledRead,
                )
                .unwrap(),
            )],
        );
        assert!(
            GpuPreparedWorkGraph::prepare(label("resolve graph"), [fragment.finish().unwrap()])
                .is_ok()
        );
    }
}
