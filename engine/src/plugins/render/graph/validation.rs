use crate::plugins::render::RenderResourceDescriptor;
use crate::plugins::render::graph::{RenderFlowGraph, RenderPassKind, RenderPassNode};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FlowValidationReport {
    pub pass_order: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderFlowValidationError {
    pub issues: Vec<String>,
}

impl std::fmt::Display for RenderFlowValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.issues.join("; "))
    }
}

impl std::error::Error for RenderFlowValidationError {}

pub fn validate_flow_graph(
    graph: &RenderFlowGraph,
) -> Result<FlowValidationReport, RenderFlowValidationError> {
    let mut issues = Vec::<String>::new();

    let mut resource_ids = BTreeSet::<String>::new();
    let mut resources_by_id = BTreeMap::<String, &RenderResourceDescriptor>::new();
    for resource in &graph.resources.resources {
        let resource_id = resource.id().as_str().trim();
        if !resource_ids.insert(resource_id.to_string()) {
            issues.push(format!("duplicate resource id '{}'", resource_id));
        }
        if let RenderResourceDescriptor::StorageBuffer(value) = resource
            && value.element_count == 0
        {
            issues.push(format!(
                "storage_buffer '{}' declares zero elements; element_count must be greater than zero",
                resource_id
            ));
        }
        resources_by_id.insert(resource_id.to_string(), resource);
    }

    let mut pass_ids = BTreeSet::<String>::new();
    for pass in &graph.passes.passes {
        let pass_id = pass.id.as_str().trim();
        if !pass_ids.insert(pass_id.to_string()) {
            issues.push(format!("duplicate pass id '{}'", pass_id));
        }
    }

    let pass_lookup: BTreeMap<_, _> = graph
        .passes
        .passes
        .iter()
        .map(|pass| (pass.id.as_str().to_string(), pass))
        .collect();

    for pass in &graph.passes.passes {
        validate_pass_shape(pass, &mut issues);

        for dependency in &pass.depends_on {
            let dep_id = dependency.as_str().trim();
            if !pass_lookup.contains_key(dep_id) {
                issues.push(format!(
                    "pass '{}' depends on unknown pass '{}'",
                    pass.id.as_str(),
                    dep_id
                ));
            }
        }

        for resource in pass_resource_refs(pass) {
            let resource_id = resource.as_str().trim();
            if !resource_ids.contains(resource_id) {
                issues.push(format!(
                    "pass '{}' references unknown resource '{}'",
                    pass.id.as_str(),
                    resource_id
                ));
            }
        }

        validate_pass_resource_usage(pass, &resources_by_id, &mut issues);

        for binding in &pass.uniform_bindings {
            if !graph.resources.has_state_resource(binding.state_type_id()) {
                issues.push(format!(
                    "pass '{}' uses uniform projection for state '{}' but with_state::<...>() was not declared",
                    pass.id.as_str(),
                    binding.state_type_name()
                ));
            }

            if !graph.resources.has_uniform_buffer(binding.uniform_id()) {
                issues.push(format!(
                    "pass '{}' references missing uniform buffer '{}'",
                    pass.id.as_str(),
                    binding.uniform_id().as_str()
                ));
            }
        }

        if let Some(dispatch) = &pass.compute_dispatch
            && let crate::plugins::render::api::ComputeDispatchDescriptor::State(binding) = dispatch
            && !graph.resources.has_state_resource(binding.state_type_id())
        {
            issues.push(format!(
                "pass '{}' uses dispatch_from_state for resource '{}' but with_state::<...>() was not declared",
                pass.id.as_str(),
                binding.state_type_name()
            ));
        }
    }

    let present_pass_ids = graph
        .passes
        .passes
        .iter()
        .filter(|pass| matches!(pass.kind, RenderPassKind::Present))
        .map(|pass| pass.id.as_str().to_string())
        .collect::<Vec<_>>();
    if present_pass_ids.len() > 1 {
        issues.push(format!(
            "flow declares {} present passes ({}); exactly zero or one present pass is allowed",
            present_pass_ids.len(),
            present_pass_ids.join(", ")
        ));
    }

    let pass_order = topological_sort(&graph.passes.passes, &mut issues);
    if present_pass_ids.len() == 1 {
        let present_id = &present_pass_ids[0];
        let dependent_passes = graph
            .passes
            .passes
            .iter()
            .filter(|pass| pass.depends_on.iter().any(|dep| dep.as_str() == present_id))
            .map(|pass| pass.id.as_str().to_string())
            .collect::<Vec<_>>();
        if !dependent_passes.is_empty() {
            issues.push(format!(
                "present pass '{}' must be terminal but is a dependency for ({})",
                present_id,
                dependent_passes.join(", ")
            ));
        }
        if pass_order.last().is_some_and(|id| id != present_id) {
            issues.push(format!(
                "present pass '{}' must be the final execution node; add explicit depends_on edges so it orders last",
                present_id
            ));
        }
    }

    if issues.is_empty() {
        Ok(FlowValidationReport { pass_order })
    } else {
        Err(RenderFlowValidationError { issues })
    }
}

fn validate_pass_shape(pass: &RenderPassNode, issues: &mut Vec<String>) {
    match pass.kind {
        RenderPassKind::Compute => {
            if pass.compute_dispatch.is_none() {
                issues.push(format!(
                    "compute pass '{}' must declare explicit dispatch(...) or dispatch_from_state(...)",
                    pass.id.as_str()
                ));
            }
            if pass.depth_target.is_some() {
                issues.push(format!(
                    "compute pass '{}' cannot declare a depth target",
                    pass.id.as_str()
                ));
            }
            if !pass.vertex_buffers.is_empty()
                || !pass.index_buffers.is_empty()
                || !pass.instance_buffers.is_empty()
                || !pass.indirect_buffers.is_empty()
            {
                issues.push(format!(
                    "compute pass '{}' cannot declare vertex/index/instance/indirect buffers",
                    pass.id.as_str()
                ));
            }
            if pass.clear_color.is_some() {
                issues.push(format!(
                    "compute pass '{}' cannot declare clear_color",
                    pass.id.as_str()
                ));
            }
            if let Some(crate::plugins::render::api::ComputeDispatchDescriptor::Fixed(dims)) =
                &pass.compute_dispatch
                && (dims[0] == 0 || dims[1] == 0 || dims[2] == 0)
            {
                issues.push(format!(
                    "compute pass '{}' declares invalid dispatch_workgroups({}, {}, {})",
                    pass.id.as_str(),
                    dims[0],
                    dims[1],
                    dims[2]
                ));
            }
        }
        RenderPassKind::Fullscreen => {
            if pass.workgroup_size.is_some() {
                issues.push(format!(
                    "fullscreen pass '{}' cannot declare workgroup_size",
                    pass.id.as_str()
                ));
            }
            if pass.compute_dispatch.is_some() {
                issues.push(format!(
                    "fullscreen pass '{}' cannot declare compute dispatch",
                    pass.id.as_str()
                ));
            }
            if pass.depth_target.is_some() {
                issues.push(format!(
                    "fullscreen pass '{}' cannot declare a depth target",
                    pass.id.as_str()
                ));
            }
            if !pass.vertex_buffers.is_empty()
                || !pass.index_buffers.is_empty()
                || !pass.instance_buffers.is_empty()
                || !pass.indirect_buffers.is_empty()
            {
                issues.push(format!(
                    "fullscreen pass '{}' cannot declare vertex/index/instance/indirect buffers",
                    pass.id.as_str()
                ));
            }
        }
        RenderPassKind::BuiltinUiComposite => {
            if pass.shader.is_some() {
                issues.push(format!(
                    "builtin_ui_composite pass '{}' cannot declare shader",
                    pass.id.as_str()
                ));
            }
            if pass.workgroup_size.is_some() {
                issues.push(format!(
                    "builtin_ui_composite pass '{}' cannot declare workgroup_size",
                    pass.id.as_str()
                ));
            }
            if pass.clear_color.is_some() {
                issues.push(format!(
                    "builtin_ui_composite pass '{}' cannot declare clear_color",
                    pass.id.as_str()
                ));
            }
            if pass.compute_dispatch.is_some() {
                issues.push(format!(
                    "builtin_ui_composite pass '{}' cannot declare compute dispatch",
                    pass.id.as_str()
                ));
            }
            if pass.depth_target.is_some() {
                issues.push(format!(
                    "builtin_ui_composite pass '{}' cannot declare depth target",
                    pass.id.as_str()
                ));
            }
            if !pass.uniform_bindings.is_empty() {
                issues.push(format!(
                    "builtin_ui_composite pass '{}' cannot declare uniform bindings",
                    pass.id.as_str()
                ));
            }
            if !pass.sampled_textures.is_empty()
                || !pass.write_textures.is_empty()
                || !pass.vertex_buffers.is_empty()
                || !pass.index_buffers.is_empty()
                || !pass.instance_buffers.is_empty()
                || !pass.indirect_buffers.is_empty()
            {
                issues.push(format!(
                    "builtin_ui_composite pass '{}' only supports reads/writes/depends_on",
                    pass.id.as_str()
                ));
            }
        }
        RenderPassKind::Graphics => {
            if pass.workgroup_size.is_some() {
                issues.push(format!(
                    "graphics pass '{}' cannot declare workgroup_size",
                    pass.id.as_str()
                ));
            }
            if pass.compute_dispatch.is_some() {
                issues.push(format!(
                    "graphics pass '{}' cannot declare compute dispatch",
                    pass.id.as_str()
                ));
            }
        }
        RenderPassKind::Copy => {
            if pass.reads.len() != 1 || pass.writes.len() != 1 {
                issues.push(format!(
                    "copy pass '{}' must declare exactly one reads(...) and one writes(...) resource",
                    pass.id.as_str()
                ));
            }
            if pass.shader.is_some()
                || pass.workgroup_size.is_some()
                || pass.compute_dispatch.is_some()
                || pass.clear_color.is_some()
                || pass.depth_target.is_some()
                || !pass.uniform_bindings.is_empty()
                || !pass.sampled_textures.is_empty()
                || !pass.write_textures.is_empty()
                || !pass.vertex_buffers.is_empty()
                || !pass.index_buffers.is_empty()
                || !pass.instance_buffers.is_empty()
                || !pass.indirect_buffers.is_empty()
            {
                issues.push(format!(
                    "copy pass '{}' only supports reads/writes/depends_on",
                    pass.id.as_str()
                ));
            }
        }
        RenderPassKind::Present => {
            if pass.reads.len() != 1 {
                issues.push(format!(
                    "present pass '{}' must declare exactly one reads(...) resource",
                    pass.id.as_str()
                ));
            }
            if !pass.writes.is_empty() {
                issues.push(format!(
                    "present pass '{}' cannot declare writes(...) resources",
                    pass.id.as_str()
                ));
            }
            if pass.shader.is_some()
                || pass.workgroup_size.is_some()
                || pass.compute_dispatch.is_some()
                || pass.clear_color.is_some()
                || pass.depth_target.is_some()
                || !pass.uniform_bindings.is_empty()
                || !pass.sampled_textures.is_empty()
                || !pass.write_textures.is_empty()
                || !pass.vertex_buffers.is_empty()
                || !pass.index_buffers.is_empty()
                || !pass.instance_buffers.is_empty()
                || !pass.indirect_buffers.is_empty()
            {
                issues.push(format!(
                    "present pass '{}' only supports reads/depends_on",
                    pass.id.as_str()
                ));
            }
        }
    }
}

fn validate_pass_resource_usage(
    pass: &RenderPassNode,
    resources_by_id: &BTreeMap<String, &RenderResourceDescriptor>,
    issues: &mut Vec<String>,
) {
    for sampled in &pass.sampled_textures {
        let Some(resource) = resources_by_id.get(sampled.as_str()) else {
            continue;
        };
        if !matches!(
            resource,
            RenderResourceDescriptor::SampledTexture(_)
                | RenderResourceDescriptor::StorageTexture(_)
                | RenderResourceDescriptor::ColorTarget(_)
                | RenderResourceDescriptor::DepthTarget(_)
                | RenderResourceDescriptor::HistoryTexture(_)
                | RenderResourceDescriptor::ImportedTexture(_)
        ) {
            issues.push(format!(
                "pass '{}' samples resource '{}' which is not texture-like (kind: {})",
                pass.id.as_str(),
                sampled.as_str(),
                resource_kind_name(resource)
            ));
        }
    }

    for written in &pass.write_textures {
        let Some(resource) = resources_by_id.get(written.as_str()) else {
            continue;
        };
        if !matches!(
            resource,
            RenderResourceDescriptor::StorageTexture(_)
                | RenderResourceDescriptor::HistoryTexture(_)
        ) {
            issues.push(format!(
                "pass '{}' writes texture resource '{}' via write_texture(...) but kind '{}' is not storage/history texture",
                pass.id.as_str(),
                written.as_str(),
                resource_kind_name(resource)
            ));
        }
    }

    for id in &pass.vertex_buffers {
        validate_buffer_role_resource(pass, id.as_str(), "vertex_buffer", resources_by_id, issues);
    }
    for id in &pass.index_buffers {
        validate_buffer_role_resource(pass, id.as_str(), "index_buffer", resources_by_id, issues);
    }
    for id in &pass.instance_buffers {
        validate_buffer_role_resource(
            pass,
            id.as_str(),
            "instance_buffer",
            resources_by_id,
            issues,
        );
    }
    for id in &pass.indirect_buffers {
        validate_buffer_role_resource(
            pass,
            id.as_str(),
            "indirect_buffer",
            resources_by_id,
            issues,
        );
    }

    if let Some(depth_target) = &pass.depth_target {
        if let Some(resource) = resources_by_id.get(depth_target.as_str()) {
            if !matches!(
                resource,
                RenderResourceDescriptor::DepthTarget(_)
                    | RenderResourceDescriptor::ImportedTexture(_)
            ) {
                issues.push(format!(
                    "graphics pass '{}' uses depth_target '{}' but kind '{}' is not depth/imported texture",
                    pass.id.as_str(),
                    depth_target.as_str(),
                    resource_kind_name(resource)
                ));
            }
        }
    }

    if matches!(pass.kind, RenderPassKind::Copy) && pass.reads.len() == 1 && pass.writes.len() == 1
    {
        let read = &pass.reads[0];
        let write = &pass.writes[0];
        if let (Some(read_resource), Some(write_resource)) = (
            resources_by_id.get(read.as_str()),
            resources_by_id.get(write.as_str()),
        ) {
            let read_texture = is_texture_resource(read_resource);
            let write_texture = is_texture_resource(write_resource);
            let read_buffer = is_buffer_resource(read_resource);
            let write_buffer = is_buffer_resource(write_resource);
            if (read_texture && write_buffer) || (read_buffer && write_texture) {
                issues.push(format!(
                    "copy pass '{}' mixes incompatible resource classes: '{}' ({}) -> '{}' ({})",
                    pass.id.as_str(),
                    read.as_str(),
                    resource_kind_name(read_resource),
                    write.as_str(),
                    resource_kind_name(write_resource)
                ));
            }
        }
    }

    if matches!(pass.kind, RenderPassKind::Present) && pass.reads.len() == 1 {
        let read = &pass.reads[0];
        if let Some(resource) = resources_by_id.get(read.as_str()) {
            if !is_texture_resource(resource) {
                issues.push(format!(
                    "present pass '{}' must read a texture-like resource; '{}' is '{}'",
                    pass.id.as_str(),
                    read.as_str(),
                    resource_kind_name(resource)
                ));
            }
        }
    }

    for write in &pass.writes {
        let Some(resource) = resources_by_id.get(write.as_str()) else {
            continue;
        };
        if matches!(resource, RenderResourceDescriptor::ImportedTexture(_))
            && !matches!(
                pass.kind,
                RenderPassKind::Fullscreen
                    | RenderPassKind::Graphics
                    | RenderPassKind::BuiltinUiComposite
                    | RenderPassKind::Copy
            )
        {
            issues.push(format!(
                "pass '{}' writes imported texture '{}' but pass kind '{:?}' is not supported for imported texture writes",
                pass.id.as_str(),
                write.as_str(),
                pass.kind
            ));
        }
    }
}

fn validate_buffer_role_resource(
    pass: &RenderPassNode,
    resource_id: &str,
    role: &str,
    resources_by_id: &BTreeMap<String, &RenderResourceDescriptor>,
    issues: &mut Vec<String>,
) {
    let Some(resource) = resources_by_id.get(resource_id) else {
        return;
    };
    if !is_buffer_resource(resource) {
        issues.push(format!(
            "pass '{}' uses '{}' in {}(...) but kind '{}' is not buffer-like",
            pass.id.as_str(),
            resource_id,
            role,
            resource_kind_name(resource)
        ));
    }
}

fn is_texture_resource(resource: &RenderResourceDescriptor) -> bool {
    matches!(
        resource,
        RenderResourceDescriptor::SampledTexture(_)
            | RenderResourceDescriptor::StorageTexture(_)
            | RenderResourceDescriptor::ColorTarget(_)
            | RenderResourceDescriptor::DepthTarget(_)
            | RenderResourceDescriptor::HistoryTexture(_)
            | RenderResourceDescriptor::ImportedTexture(_)
    )
}

fn is_buffer_resource(resource: &RenderResourceDescriptor) -> bool {
    matches!(
        resource,
        RenderResourceDescriptor::UniformBuffer(_)
            | RenderResourceDescriptor::StorageBuffer(_)
            | RenderResourceDescriptor::ImportedBuffer(_)
    )
}

fn resource_kind_name(resource: &RenderResourceDescriptor) -> &'static str {
    match resource {
        RenderResourceDescriptor::UniformBuffer(_) => "uniform_buffer",
        RenderResourceDescriptor::StorageBuffer(_) => "storage_buffer",
        RenderResourceDescriptor::SampledTexture(_) => "sampled_texture",
        RenderResourceDescriptor::StorageTexture(_) => "storage_texture",
        RenderResourceDescriptor::ColorTarget(_) => "color_target",
        RenderResourceDescriptor::DepthTarget(_) => "depth_target",
        RenderResourceDescriptor::HistoryTexture(_) => "history_texture",
        RenderResourceDescriptor::ImportedTexture(_) => "imported_texture",
        RenderResourceDescriptor::ImportedBuffer(_) => "imported_buffer",
    }
}

fn topological_sort(passes: &[RenderPassNode], issues: &mut Vec<String>) -> Vec<String> {
    let mut by_id = BTreeMap::<String, usize>::new();
    for (index, pass) in passes.iter().enumerate() {
        by_id.insert(pass.id.as_str().to_string(), index);
    }

    let mut indegree = vec![0usize; passes.len()];
    let mut outgoing = vec![Vec::<usize>::new(); passes.len()];

    for (index, pass) in passes.iter().enumerate() {
        for dependency in &pass.depends_on {
            if let Some(dep_index) = by_id.get(dependency.as_str()) {
                indegree[index] = indegree[index].saturating_add(1);
                outgoing[*dep_index].push(index);
            }
        }
    }

    let mut queue = VecDeque::<usize>::new();
    for (index, degree) in indegree.iter().enumerate() {
        if *degree == 0 {
            queue.push_back(index);
        }
    }

    let mut order = Vec::<String>::with_capacity(passes.len());
    while let Some(index) = queue.pop_front() {
        order.push(passes[index].id.as_str().to_string());
        for next in &outgoing[index] {
            indegree[*next] = indegree[*next].saturating_sub(1);
            if indegree[*next] == 0 {
                queue.push_back(*next);
            }
        }
    }

    if order.len() != passes.len() {
        let cycle_nodes = indegree
            .iter()
            .enumerate()
            .filter(|(_, degree)| **degree > 0)
            .map(|(index, _)| passes[index].id.as_str().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        issues.push(format!("pass dependency cycle detected: {}", cycle_nodes));
    }

    order
}

fn pass_resource_refs(
    pass: &RenderPassNode,
) -> impl Iterator<Item = &crate::plugins::render::RenderResourceId> {
    pass.reads
        .iter()
        .chain(pass.writes.iter())
        .chain(pass.sampled_textures.iter())
        .chain(pass.write_textures.iter())
        .chain(pass.vertex_buffers.iter())
        .chain(pass.index_buffers.iter())
        .chain(pass.instance_buffers.iter())
        .chain(pass.indirect_buffers.iter())
        .chain(pass.depth_target.iter())
}
