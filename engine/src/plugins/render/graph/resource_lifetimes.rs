use crate::plugins::render::graph::{
    CompiledPassDescriptor, RenderExecutionGraphDiagnostic, RenderExecutionGraphDiagnosticKind,
};
use crate::plugins::render::resource::ResourceLifetime;
use crate::plugins::render::{RenderFlowId, RenderResourceDescriptor, RenderResourceId};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompiledResourceAccessKind {
    Read,
    Write,
    SampledTexture,
    StorageTextureWrite,
    UniformBuffer,
    StorageBuffer,
    VertexBuffer,
    IndexBuffer,
    InstanceBuffer,
    IndirectBuffer,
    DepthTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledResourceLifetimeWindow {
    pub resource_id: RenderResourceId,
    pub resource_label: Option<String>,
    pub lifetime: ResourceLifetime,
    pub first_use: Option<usize>,
    pub first_read: Option<usize>,
    pub first_write: Option<usize>,
    pub last_read: Option<usize>,
    pub last_write: Option<usize>,
    pub last_use: Option<usize>,
    pub access_kinds: BTreeSet<CompiledResourceAccessKind>,
}

impl CompiledResourceLifetimeWindow {
    fn new(
        resource_id: RenderResourceId,
        resource_label: Option<String>,
        lifetime: ResourceLifetime,
    ) -> Self {
        Self {
            resource_id,
            resource_label,
            lifetime,
            first_use: None,
            first_read: None,
            first_write: None,
            last_read: None,
            last_write: None,
            last_use: None,
            access_kinds: BTreeSet::new(),
        }
    }

    fn observe(&mut self, index: usize, access: CompiledResourceAccessKind) {
        self.first_use = Some(self.first_use.map_or(index, |existing| existing.min(index)));
        self.last_use = Some(self.last_use.map_or(index, |existing| existing.max(index)));
        self.access_kinds.insert(access);

        if is_write_access(access) {
            self.first_write = Some(
                self.first_write
                    .map_or(index, |existing| existing.min(index)),
            );
            self.last_write = Some(
                self.last_write
                    .map_or(index, |existing| existing.max(index)),
            );
        } else {
            self.first_read = Some(
                self.first_read
                    .map_or(index, |existing| existing.min(index)),
            );
            self.last_read = Some(self.last_read.map_or(index, |existing| existing.max(index)));
        }
    }

    pub fn is_read_before_first_write(&self) -> bool {
        matches!(
            (self.first_read, self.first_write),
            (Some(read), Some(write)) if read < write
        ) || matches!((self.first_read, self.first_write), (Some(_), None))
    }
}

pub fn compile_resource_lifetime_windows(
    resources: &[RenderResourceDescriptor],
    resource_labels: &BTreeMap<RenderResourceId, String>,
    pass_order: &[CompiledPassDescriptor],
) -> Vec<CompiledResourceLifetimeWindow> {
    let mut windows = resources
        .iter()
        .map(|resource| {
            (
                *resource.id(),
                CompiledResourceLifetimeWindow::new(
                    *resource.id(),
                    resource_labels.get(resource.id()).cloned(),
                    resource.lifetime(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();

    for pass in pass_order {
        let index = pass.order_index();
        let node = pass.node();
        for resource in &node.reads {
            observe(
                &mut windows,
                *resource,
                index,
                CompiledResourceAccessKind::Read,
            );
        }
        for resource in &node.writes {
            observe(
                &mut windows,
                *resource,
                index,
                CompiledResourceAccessKind::Write,
            );
        }
        for resource in &node.sampled_textures {
            observe(
                &mut windows,
                *resource,
                index,
                CompiledResourceAccessKind::SampledTexture,
            );
        }
        for resource in &node.write_textures {
            observe(
                &mut windows,
                *resource,
                index,
                CompiledResourceAccessKind::StorageTextureWrite,
            );
        }
        for binding in &node.uniform_bindings {
            observe(
                &mut windows,
                *binding.uniform_id(),
                index,
                CompiledResourceAccessKind::UniformBuffer,
            );
        }
        for resource in &node.vertex_buffers {
            observe(
                &mut windows,
                *resource,
                index,
                CompiledResourceAccessKind::VertexBuffer,
            );
        }
        for resource in &node.index_buffers {
            observe(
                &mut windows,
                *resource,
                index,
                CompiledResourceAccessKind::IndexBuffer,
            );
        }
        for resource in &node.instance_buffers {
            observe(
                &mut windows,
                *resource,
                index,
                CompiledResourceAccessKind::InstanceBuffer,
            );
        }
        for resource in &node.indirect_buffers {
            observe(
                &mut windows,
                *resource,
                index,
                CompiledResourceAccessKind::IndirectBuffer,
            );
        }
        if let Some(resource) = node.depth_target {
            observe(
                &mut windows,
                resource,
                index,
                CompiledResourceAccessKind::DepthTarget,
            );
        }
    }

    windows.into_values().collect()
}

pub fn diagnose_resource_lifetime_windows(
    flow_id: RenderFlowId,
    flow_label: &str,
    windows: &[CompiledResourceLifetimeWindow],
) -> Vec<RenderExecutionGraphDiagnostic> {
    windows
        .iter()
        .filter(|window| window.lifetime.is_transient() && window.is_read_before_first_write())
        .map(|window| {
            let label = window
                .resource_label
                .clone()
                .unwrap_or_else(|| window.resource_id.to_string());
            RenderExecutionGraphDiagnostic::error(
                RenderExecutionGraphDiagnosticKind::ResourceLifetimeUseBeforeWrite,
                format!(
                    "transient resource '{}' is read before its first write in compiled pass order",
                    label
                ),
            )
            .with_flow(flow_id, flow_label.to_string())
            .with_resource(window.resource_id, window.resource_label.clone())
        })
        .collect()
}

fn observe(
    windows: &mut BTreeMap<RenderResourceId, CompiledResourceLifetimeWindow>,
    resource_id: RenderResourceId,
    index: usize,
    access: CompiledResourceAccessKind,
) {
    if let Some(window) = windows.get_mut(&resource_id) {
        window.observe(index, access);
    }
}

fn is_write_access(access: CompiledResourceAccessKind) -> bool {
    matches!(
        access,
        CompiledResourceAccessKind::Write
            | CompiledResourceAccessKind::StorageTextureWrite
            | CompiledResourceAccessKind::DepthTarget
    )
}
