use super::super::{
    GpuAttachmentStore, GpuBufferInitialization, GpuBufferRange, GpuQueryRange, GpuResourceAccess,
    GpuResourceRef, GpuTextureAccessKind, GpuTextureAspect, GpuTextureDimension, GpuTextureHandle,
    GpuTextureInitialization, GpuTextureSubresourceRange, GpuWorkGraphCause, GpuWorkGraphError,
    GpuWorkGraphErrorContext, GpuWorkGraphErrorSource, GpuWorkResourceId,
};
use super::{
    authoring::{GpuWorkFragment, GpuWorkNode},
    composition::ImportBindings,
    coverage::{
        GpuInitialCoverage, GpuInitialCoverageData, canonical_storage_resource,
        canonical_texture_aspect, coverage_source_error, normalize_u32_intervals,
        normalize_u64_intervals, storage_identity, texture_aspect,
    },
    diagnostics::{
        GpuPreparedWorkDiagnostic, GraphErrorOrigin, graph_error, graph_error_with_region,
    },
    identity::GpuPreparedWorkNodeId,
};
use std::collections::BTreeMap;

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

pub(super) fn validate_fragment_initialization(
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
                    (resource, format!("{expected:?}")),
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

pub(super) fn access_region_description(access: &GpuResourceAccess) -> String {
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

pub(super) fn simulate_prepared_initialization(
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
