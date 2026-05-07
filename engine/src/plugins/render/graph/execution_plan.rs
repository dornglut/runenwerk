use super::{
    CompiledPassDescriptor, RenderPassKind, RenderPassNode, RenderPassViewScope, ResourceGraph,
};
use crate::plugins::render::api::ComputeDispatchDescriptor;
use crate::plugins::render::api::ids::RenderFeatureId;
use crate::plugins::render::features::UI_RENDER_FEATURE_ID;
use crate::plugins::render::resource::ImportedTextureSemantic;
use crate::plugins::render::{
    RenderDrawDescriptor, RenderPassId, RenderResourceDescriptor, RenderResourceId,
    RenderShaderReference, RenderTargetAliasKind, RenderVertexAttribute, RenderVertexBufferLayout,
    RenderVertexStepMode,
};
use std::any::TypeId;
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Default)]
pub struct CompiledFlowExecutionPlan {
    pub required_state_types: Vec<CompiledStateRequirement>,
    pub passes: Vec<CompiledPassExecutionPlan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompiledStateRequirement {
    pub type_id: TypeId,
    pub type_name: &'static str,
}

#[derive(Debug, Clone)]
pub enum CompiledPassExecutionPlan {
    Compute(CompiledComputeExecutionPlan),
    Fullscreen(CompiledRasterExecutionPlan),
    Graphics(CompiledRasterExecutionPlan),
    Copy(CompiledCopyExecutionPlan),
    Present(CompiledPresentExecutionPlan),
    BuiltinUiComposite(CompiledUiCompositeExecutionPlan),
}

#[derive(Debug, Clone)]
pub struct CompiledComputeExecutionPlan {
    pub pass_id: RenderPassId,
    pub order_index: usize,
    pub feature_id: Option<RenderFeatureId>,
    pub shader: Option<RenderShaderReference>,
    pub view_mask: CompiledViewMask,
    pub bindings: CompiledPassBindings,
    pub dispatch: Option<CompiledDispatchPlan>,
}

#[derive(Debug, Clone)]
pub struct CompiledRasterExecutionPlan {
    pub pass_id: RenderPassId,
    pub order_index: usize,
    pub feature_id: Option<RenderFeatureId>,
    pub shader: Option<RenderShaderReference>,
    pub view_mask: CompiledViewMask,
    pub bindings: CompiledPassBindings,
    pub targets: CompiledTargetPlan,
    pub draw_buffers: CompiledDrawBufferPlan,
    pub clear_color: Option<[f32; 4]>,
    pub draw: Option<CompiledDrawPlan>,
}

#[derive(Debug, Clone)]
pub struct CompiledCopyExecutionPlan {
    pub pass_id: RenderPassId,
    pub order_index: usize,
    pub feature_id: Option<RenderFeatureId>,
    pub view_mask: CompiledViewMask,
    pub source: Option<CompiledResourceRef>,
    pub destination: Option<CompiledResourceRef>,
}

#[derive(Debug, Clone)]
pub struct CompiledPresentExecutionPlan {
    pub pass_id: RenderPassId,
    pub order_index: usize,
    pub feature_id: Option<RenderFeatureId>,
    pub view_mask: CompiledViewMask,
    pub source: Option<CompiledResourceRef>,
}

#[derive(Debug, Clone)]
pub struct CompiledUiCompositeExecutionPlan {
    pub pass_id: RenderPassId,
    pub order_index: usize,
    pub feature_id: RenderFeatureId,
    pub view_mask: CompiledViewMask,
    pub color_output: CompiledResourceRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CompiledViewMask {
    #[default]
    AllViews,
    MainSurfaceOnly,
    OffscreenProductsOnly,
    Explicit(BTreeSet<String>),
}

impl CompiledViewMask {
    pub fn includes(
        &self,
        view_id: &str,
        view_kind: crate::plugins::render::PreparedViewKind,
    ) -> bool {
        match self {
            Self::AllViews => true,
            Self::MainSurfaceOnly => matches!(
                view_kind,
                crate::plugins::render::PreparedViewKind::MainSurface
            ),
            Self::OffscreenProductsOnly => {
                matches!(
                    view_kind,
                    crate::plugins::render::PreparedViewKind::OffscreenProduct
                )
            }
            Self::Explicit(values) => values.contains(view_id),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CompiledPassBindings {
    pub bind_group: CompiledBindGroupPlan,
    pub uniform_order: Vec<RenderResourceId>,
    pub storage_order: Vec<CompiledStorageBinding>,
}

#[derive(Debug, Clone, Default)]
pub struct CompiledBindGroupPlan {
    pub entries: Vec<CompiledBindingEntry>,
}

#[derive(Debug, Clone)]
pub enum CompiledBindingEntry {
    SampledTexture {
        resource: CompiledResourceRef,
    },
    Sampler,
    StorageTexture {
        resource: CompiledResourceRef,
        access: CompiledStorageAccess,
    },
    UniformBuffer {
        resource: RenderResourceId,
    },
    StorageBuffer {
        resource: CompiledResourceRef,
        access: CompiledStorageAccess,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledStorageAccess {
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, Clone, Default)]
pub struct CompiledTargetPlan {
    pub color_outputs: Vec<CompiledResourceRef>,
    pub depth_output: Option<CompiledResourceRef>,
    pub reads: Vec<CompiledResourceRef>,
}

#[derive(Debug, Clone, Default)]
pub struct CompiledDrawBufferPlan {
    pub vertex_buffers: Vec<CompiledVertexBufferBinding>,
    pub instance_buffers: Vec<CompiledResourceRef>,
    pub instance_buffer_layouts: Vec<CompiledVertexBufferLayout>,
    pub index_buffers: Vec<CompiledResourceRef>,
    pub indirect_buffers: Vec<CompiledResourceRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledVertexBufferBinding {
    pub resource: CompiledResourceRef,
    pub layout: CompiledVertexBufferLayout,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompiledVertexBufferLayout {
    pub slot: u32,
    pub array_stride: u64,
    pub step_mode: RenderVertexStepMode,
    pub attributes: Vec<RenderVertexAttribute>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompiledDrawPlan {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledDispatchPlan {
    Fixed([u32; 3]),
    FromState {
        state_type_id: TypeId,
        state_type_name: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompiledResourceRef {
    FlowOwned(RenderResourceId),
    TargetAlias(CompiledTargetAliasRef),
    ImportedBuiltin(CompiledBuiltinImport),
    Imported(RenderResourceId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledTargetAliasRef {
    pub resource_id: RenderResourceId,
    pub label: String,
    pub kind: RenderTargetAliasKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledBuiltinImport {
    SurfaceColor,
    SurfaceDepth,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledStorageBinding {
    pub resource: CompiledResourceRef,
    pub access: CompiledStorageAccess,
}

pub fn compile_execution_plan(
    resources: &ResourceGraph,
    pass_order: &[CompiledPassDescriptor],
) -> CompiledFlowExecutionPlan {
    let mut required_state_types = Vec::<CompiledStateRequirement>::new();
    let mut seen_state_types = BTreeSet::<TypeId>::new();
    for state in &resources.state_resources {
        if seen_state_types.insert(state.type_id) {
            required_state_types.push(CompiledStateRequirement {
                type_id: state.type_id,
                type_name: state.type_name,
            });
        }
    }

    let passes = pass_order
        .iter()
        .map(|pass| compile_pass_execution(pass, resources))
        .collect();

    CompiledFlowExecutionPlan {
        required_state_types,
        passes,
    }
}

fn compile_pass_execution(
    pass: &CompiledPassDescriptor,
    resources: &ResourceGraph,
) -> CompiledPassExecutionPlan {
    let node = pass.node();
    let pass_id = node.id;
    let order_index = pass.order_index();

    let bindings = compile_pass_bindings(node, resources);

    match node.kind {
        RenderPassKind::Compute => {
            CompiledPassExecutionPlan::Compute(CompiledComputeExecutionPlan {
                pass_id,
                order_index,
                feature_id: compile_feature_id(node),
                shader: node.shader.clone(),
                view_mask: compile_view_mask(node),
                bindings,
                dispatch: compile_dispatch_plan(node),
            })
        }
        RenderPassKind::Fullscreen => {
            CompiledPassExecutionPlan::Fullscreen(CompiledRasterExecutionPlan {
                pass_id,
                order_index,
                feature_id: compile_feature_id(node),
                shader: node.shader.clone(),
                view_mask: compile_view_mask(node),
                bindings,
                targets: compile_target_plan(node, resources),
                draw_buffers: CompiledDrawBufferPlan::default(),
                clear_color: node.clear_color,
                draw: None,
            })
        }
        RenderPassKind::Graphics => {
            CompiledPassExecutionPlan::Graphics(CompiledRasterExecutionPlan {
                pass_id,
                order_index,
                feature_id: compile_feature_id(node),
                shader: node.shader.clone(),
                view_mask: compile_view_mask(node),
                bindings,
                targets: compile_target_plan(node, resources),
                draw_buffers: compile_draw_buffer_plan(node, resources),
                clear_color: node.clear_color,
                draw: node.draw.map(compile_draw_plan),
            })
        }
        RenderPassKind::Copy => CompiledPassExecutionPlan::Copy(CompiledCopyExecutionPlan {
            pass_id,
            order_index,
            feature_id: compile_feature_id(node),
            view_mask: compile_view_mask(node),
            source: node
                .reads
                .first()
                .map(|resource| compile_resource_ref(resource, resources)),
            destination: node
                .writes
                .first()
                .map(|resource| compile_resource_ref(resource, resources)),
        }),
        RenderPassKind::Present => {
            CompiledPassExecutionPlan::Present(CompiledPresentExecutionPlan {
                pass_id,
                order_index,
                feature_id: compile_feature_id(node),
                view_mask: compile_view_mask(node),
                source: node
                    .reads
                    .first()
                    .map(|resource| compile_resource_ref(resource, resources)),
            })
        }
        RenderPassKind::BuiltinUiComposite => {
            CompiledPassExecutionPlan::BuiltinUiComposite(CompiledUiCompositeExecutionPlan {
                pass_id,
                order_index,
                feature_id: UI_RENDER_FEATURE_ID,
                view_mask: compile_view_mask(node),
                color_output: CompiledResourceRef::ImportedBuiltin(
                    CompiledBuiltinImport::SurfaceColor,
                ),
            })
        }
    }
}

fn compile_feature_id(node: &RenderPassNode) -> Option<RenderFeatureId> {
    node.feature_id
}

fn compile_view_mask(node: &RenderPassNode) -> CompiledViewMask {
    match node.view_scope {
        RenderPassViewScope::AllViews => CompiledViewMask::AllViews,
        RenderPassViewScope::MainSurfaceOnly => CompiledViewMask::MainSurfaceOnly,
        RenderPassViewScope::OffscreenProductsOnly => CompiledViewMask::OffscreenProductsOnly,
    }
}

fn compile_dispatch_plan(node: &RenderPassNode) -> Option<CompiledDispatchPlan> {
    match node.compute_dispatch.as_ref() {
        None => None,
        Some(ComputeDispatchDescriptor::Fixed(value)) => Some(CompiledDispatchPlan::Fixed(*value)),
        Some(ComputeDispatchDescriptor::State(binding)) => Some(CompiledDispatchPlan::FromState {
            state_type_id: binding.state_type_id(),
            state_type_name: binding.state_type_name(),
        }),
    }
}

fn compile_pass_bindings(node: &RenderPassNode, resources: &ResourceGraph) -> CompiledPassBindings {
    let mut bind_group = CompiledBindGroupPlan::default();
    let mut uniform_order = Vec::<RenderResourceId>::new();
    let mut storage_order = Vec::<CompiledStorageBinding>::new();
    let mut seen_uniforms = BTreeSet::<RenderResourceId>::new();

    for resource in &node.sampled_textures {
        bind_group
            .entries
            .push(CompiledBindingEntry::SampledTexture {
                resource: compile_resource_ref(resource, resources),
            });
        bind_group.entries.push(CompiledBindingEntry::Sampler);
    }

    for resource in &node.write_textures {
        bind_group
            .entries
            .push(CompiledBindingEntry::StorageTexture {
                resource: compile_resource_ref(resource, resources),
                access: CompiledStorageAccess::ReadWrite,
            });
    }

    for binding in &node.uniform_bindings {
        let resource = *binding.uniform_id();
        bind_group
            .entries
            .push(CompiledBindingEntry::UniformBuffer { resource });
        if seen_uniforms.insert(resource) {
            uniform_order.push(resource);
        }
    }

    for (resource, access) in collect_storage_usage(node, resources) {
        let compiled = compile_resource_ref(&resource, resources);
        bind_group
            .entries
            .push(CompiledBindingEntry::StorageBuffer {
                resource: compiled.clone(),
                access,
            });
        storage_order.push(CompiledStorageBinding {
            resource: compiled,
            access,
        });
    }

    CompiledPassBindings {
        bind_group,
        uniform_order,
        storage_order,
    }
}

fn collect_storage_usage(
    node: &RenderPassNode,
    resources: &ResourceGraph,
) -> Vec<(RenderResourceId, CompiledStorageAccess)> {
    let writable_storage = node
        .writes
        .iter()
        .copied()
        .filter(|resource| is_buffer_like_resource(resources, resource))
        .collect::<BTreeSet<_>>();
    let mut seen_storage = BTreeSet::<RenderResourceId>::new();
    let mut usage = Vec::<(RenderResourceId, CompiledStorageAccess)>::new();
    for resource in node.reads.iter().chain(node.writes.iter()).copied() {
        if !is_buffer_like_resource(resources, &resource) {
            continue;
        }
        if !seen_storage.insert(resource) {
            continue;
        }
        let access = if writable_storage.contains(&resource) {
            CompiledStorageAccess::ReadWrite
        } else {
            CompiledStorageAccess::ReadOnly
        };
        usage.push((resource, access));
    }
    usage
}

fn is_buffer_like_resource(resources: &ResourceGraph, resource: &RenderResourceId) -> bool {
    matches!(
        resources
            .resources
            .iter()
            .find(|descriptor| descriptor.id() == resource),
        Some(
            RenderResourceDescriptor::UniformBuffer(_)
                | RenderResourceDescriptor::StorageBuffer(_)
                | RenderResourceDescriptor::ImportedBuffer(_)
        )
    )
}

fn compile_target_plan(node: &RenderPassNode, resources: &ResourceGraph) -> CompiledTargetPlan {
    CompiledTargetPlan {
        color_outputs: node
            .writes
            .iter()
            .map(|resource| compile_resource_ref(resource, resources))
            .collect(),
        depth_output: node
            .depth_target
            .as_ref()
            .map(|resource| compile_resource_ref(resource, resources)),
        reads: node
            .reads
            .iter()
            .map(|resource| compile_resource_ref(resource, resources))
            .collect(),
    }
}

fn compile_draw_buffer_plan(
    node: &RenderPassNode,
    resources: &ResourceGraph,
) -> CompiledDrawBufferPlan {
    CompiledDrawBufferPlan {
        vertex_buffers: node
            .vertex_buffers
            .iter()
            .zip(node.vertex_buffer_layouts.iter())
            .map(|(resource, layout)| CompiledVertexBufferBinding {
                resource: compile_resource_ref(resource, resources),
                layout: compile_vertex_buffer_layout(layout),
            })
            .collect(),
        instance_buffers: node
            .instance_buffers
            .iter()
            .map(|resource| compile_resource_ref(resource, resources))
            .collect(),
        instance_buffer_layouts: node
            .instance_buffer_layouts
            .iter()
            .map(compile_vertex_buffer_layout)
            .collect(),
        index_buffers: node
            .index_buffers
            .iter()
            .map(|resource| compile_resource_ref(resource, resources))
            .collect(),
        indirect_buffers: node
            .indirect_buffers
            .iter()
            .map(|resource| compile_resource_ref(resource, resources))
            .collect(),
    }
}

fn compile_vertex_buffer_layout(layout: &RenderVertexBufferLayout) -> CompiledVertexBufferLayout {
    CompiledVertexBufferLayout {
        slot: layout.slot,
        array_stride: layout.array_stride,
        step_mode: layout.step_mode,
        attributes: layout.attributes.clone(),
    }
}

fn compile_draw_plan(draw: RenderDrawDescriptor) -> CompiledDrawPlan {
    CompiledDrawPlan {
        vertex_count: draw.vertex_count,
        instance_count: draw.instance_count,
        first_vertex: draw.first_vertex,
        first_instance: draw.first_instance,
    }
}

impl CompiledDrawBufferPlan {
    pub fn vertex_layout_signature_hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for binding in &self.vertex_buffers {
            binding.layout.hash(&mut hasher);
        }
        for layout in &self.instance_buffer_layouts {
            layout.hash(&mut hasher);
        }
        hasher.finish()
    }
}

fn compile_resource_ref(
    resource: &RenderResourceId,
    resources: &ResourceGraph,
) -> CompiledResourceRef {
    match resources
        .resources
        .iter()
        .find(|descriptor| descriptor.id() == resource)
    {
        Some(RenderResourceDescriptor::TargetAlias(value)) => {
            CompiledResourceRef::TargetAlias(CompiledTargetAliasRef {
                resource_id: value.id,
                label: value.label.clone(),
                kind: value.kind,
            })
        }
        Some(RenderResourceDescriptor::ImportedTexture(value)) => match value.semantic {
            ImportedTextureSemantic::SurfaceColor => {
                CompiledResourceRef::ImportedBuiltin(CompiledBuiltinImport::SurfaceColor)
            }
            ImportedTextureSemantic::SurfaceDepth => {
                CompiledResourceRef::ImportedBuiltin(CompiledBuiltinImport::SurfaceDepth)
            }
            ImportedTextureSemantic::HistoryTexture | ImportedTextureSemantic::External => {
                CompiledResourceRef::Imported(*resource)
            }
        },
        Some(RenderResourceDescriptor::ImportedBuffer(_)) => {
            CompiledResourceRef::Imported(*resource)
        }
        Some(_) | None => CompiledResourceRef::FlowOwned(*resource),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::{RenderPassKind, RenderPassNode};

    fn resource(id: u64) -> RenderResourceId {
        RenderResourceId::try_from_raw(id).unwrap()
    }

    fn storage_read_write_pass() -> (RenderPassNode, ResourceGraph, RenderResourceId) {
        let storage_id = resource(7);
        let mut resources = ResourceGraph::default();
        resources.add_resource(RenderResourceDescriptor::imported_external_buffer(
            storage_id,
        ));
        let mut pass = RenderPassNode::new(
            RenderPassId::try_from_raw(1).unwrap(),
            "test.pass",
            RenderPassKind::Compute,
        );
        pass.reads.push(storage_id);
        pass.writes.push(storage_id);
        (pass, resources, storage_id)
    }

    #[test]
    fn storage_resource_in_reads_and_writes_emits_single_read_write_binding() {
        let (pass, resources, storage_id) = storage_read_write_pass();
        let bindings = compile_pass_bindings(&pass, &resources);

        let storage_bindings = bindings
            .bind_group
            .entries
            .iter()
            .filter_map(|entry| match entry {
                CompiledBindingEntry::StorageBuffer { resource, access } => {
                    Some((resource.clone(), *access))
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(storage_bindings.len(), 1);
        assert_eq!(
            storage_bindings[0],
            (
                CompiledResourceRef::Imported(storage_id),
                CompiledStorageAccess::ReadWrite
            )
        );
        assert_eq!(bindings.storage_order.len(), 1);
        assert_eq!(
            bindings.storage_order[0],
            CompiledStorageBinding {
                resource: CompiledResourceRef::Imported(storage_id),
                access: CompiledStorageAccess::ReadWrite,
            }
        );
    }

    #[test]
    fn storage_binding_order_is_stable_with_read_priority_and_write_only_appends() {
        let read_only = resource(1);
        let shared = resource(2);
        let write_only = resource(3);

        let mut resources = ResourceGraph::default();
        resources.add_resource(RenderResourceDescriptor::imported_external_buffer(
            read_only,
        ));
        resources.add_resource(RenderResourceDescriptor::imported_external_buffer(shared));
        resources.add_resource(RenderResourceDescriptor::imported_external_buffer(
            write_only,
        ));

        let mut pass = RenderPassNode::new(
            RenderPassId::try_from_raw(11).unwrap(),
            "test.order",
            RenderPassKind::Compute,
        );
        pass.reads.extend([read_only, shared]);
        pass.writes.extend([shared, write_only]);

        let bindings = compile_pass_bindings(&pass, &resources);
        let storage_bindings = bindings
            .bind_group
            .entries
            .iter()
            .filter_map(|entry| match entry {
                CompiledBindingEntry::StorageBuffer { resource, access } => {
                    Some((resource.clone(), *access))
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            storage_bindings,
            vec![
                (
                    CompiledResourceRef::Imported(read_only),
                    CompiledStorageAccess::ReadOnly,
                ),
                (
                    CompiledResourceRef::Imported(shared),
                    CompiledStorageAccess::ReadWrite,
                ),
                (
                    CompiledResourceRef::Imported(write_only),
                    CompiledStorageAccess::ReadWrite,
                ),
            ]
        );
    }

    #[test]
    fn collect_storage_usage_is_deduped_and_stable() {
        let first = resource(1);
        let second = resource(2);
        let third = resource(3);

        let mut resources = ResourceGraph::default();
        resources.add_resource(RenderResourceDescriptor::imported_external_buffer(first));
        resources.add_resource(RenderResourceDescriptor::imported_external_buffer(second));
        resources.add_resource(RenderResourceDescriptor::imported_external_buffer(third));

        let mut pass = RenderPassNode::new(
            RenderPassId::try_from_raw(12).unwrap(),
            "test.usage",
            RenderPassKind::Compute,
        );
        pass.reads.extend([first, second, first]);
        pass.writes.extend([second, third, second]);

        assert_eq!(
            collect_storage_usage(&pass, &resources),
            vec![
                (first, CompiledStorageAccess::ReadOnly),
                (second, CompiledStorageAccess::ReadWrite),
                (third, CompiledStorageAccess::ReadWrite),
            ]
        );
    }
}
