use super::{CompiledPassDescriptor, RenderPassKind, RenderPassNode, ResourceGraph};
use crate::plugins::render::api::ComputeDispatchDescriptor;
use crate::plugins::render::features::UI_RENDER_FEATURE_ID;
use crate::plugins::render::resource::ImportedTextureSemantic;
use crate::plugins::render::{RenderResourceDescriptor, RenderResourceId, RenderShaderReference};
use std::any::TypeId;
use std::collections::BTreeSet;

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
    pub pass_id: String,
    pub order_index: usize,
    pub feature_id: Option<String>,
    pub shader: Option<RenderShaderReference>,
    pub view_mask: CompiledViewMask,
    pub bindings: CompiledPassBindings,
    pub dispatch: Option<CompiledDispatchPlan>,
}

#[derive(Debug, Clone)]
pub struct CompiledRasterExecutionPlan {
    pub pass_id: String,
    pub order_index: usize,
    pub feature_id: Option<String>,
    pub shader: Option<RenderShaderReference>,
    pub view_mask: CompiledViewMask,
    pub bindings: CompiledPassBindings,
    pub targets: CompiledTargetPlan,
    pub draw_buffers: CompiledDrawBufferPlan,
    pub clear_color: Option<[f32; 4]>,
}

#[derive(Debug, Clone)]
pub struct CompiledCopyExecutionPlan {
    pub pass_id: String,
    pub order_index: usize,
    pub feature_id: Option<String>,
    pub view_mask: CompiledViewMask,
    pub source: Option<CompiledResourceRef>,
    pub destination: Option<CompiledResourceRef>,
}

#[derive(Debug, Clone)]
pub struct CompiledPresentExecutionPlan {
    pub pass_id: String,
    pub order_index: usize,
    pub feature_id: Option<String>,
    pub view_mask: CompiledViewMask,
    pub source: Option<CompiledResourceRef>,
}

#[derive(Debug, Clone)]
pub struct CompiledUiCompositeExecutionPlan {
    pub pass_id: String,
    pub order_index: usize,
    pub feature_id: String,
    pub view_mask: CompiledViewMask,
    pub ui_input: CompiledResourceRef,
    pub color_output: CompiledResourceRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CompiledViewMask {
    #[default]
    AllViews,
    Explicit(BTreeSet<String>),
}

impl CompiledViewMask {
    pub fn includes(&self, view_id: &str) -> bool {
        match self {
            Self::AllViews => true,
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
    pub vertex_buffers: Vec<CompiledResourceRef>,
    pub instance_buffers: Vec<CompiledResourceRef>,
    pub index_buffers: Vec<CompiledResourceRef>,
    pub indirect_buffers: Vec<CompiledResourceRef>,
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
    ImportedBuiltin(CompiledBuiltinImport),
    Imported(RenderResourceId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledBuiltinImport {
    SurfaceColor,
    SurfaceDepth,
    BuiltinUiDrawList,
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
    let pass_id = node.id.as_str().to_string();
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
                feature_id: UI_RENDER_FEATURE_ID.to_string(),
                view_mask: compile_view_mask(node),
                ui_input: CompiledResourceRef::ImportedBuiltin(
                    CompiledBuiltinImport::BuiltinUiDrawList,
                ),
                color_output: CompiledResourceRef::ImportedBuiltin(
                    CompiledBuiltinImport::SurfaceColor,
                ),
            })
        }
    }
}

fn compile_pass_bindings(node: &RenderPassNode, resources: &ResourceGraph) -> CompiledPassBindings {
    let uniform_order = dedupe_ids(
        node.uniform_bindings
            .iter()
            .map(|binding| binding.uniform_id()),
    );
    let write_ids = node
        .writes
        .iter()
        .map(|id| id.as_str().to_string())
        .collect::<BTreeSet<_>>();
    let mut storage_order = Vec::<CompiledStorageBinding>::new();
    let mut seen_storage = BTreeSet::<String>::new();
    for resource_id in node.reads.iter().chain(node.writes.iter()) {
        let Some(descriptor) = find_descriptor(resources, resource_id) else {
            continue;
        };
        if !matches!(
            descriptor,
            RenderResourceDescriptor::StorageBuffer(_)
                | RenderResourceDescriptor::ImportedBuffer(_)
        ) {
            continue;
        }
        let key = resource_id.as_str().to_string();
        if !seen_storage.insert(key) {
            continue;
        }
        let access = if write_ids.contains(resource_id.as_str()) {
            CompiledStorageAccess::ReadWrite
        } else {
            CompiledStorageAccess::ReadOnly
        };
        storage_order.push(CompiledStorageBinding {
            resource: compile_resource_ref(resource_id, resources),
            access,
        });
    }

    let mut bind_group = CompiledBindGroupPlan::default();
    for sampled in dedupe_ids(node.sampled_textures.iter()) {
        bind_group
            .entries
            .push(CompiledBindingEntry::SampledTexture {
                resource: compile_resource_ref(&sampled, resources),
            });
        bind_group.entries.push(CompiledBindingEntry::Sampler);
    }
    for written_texture in dedupe_ids(node.write_textures.iter()) {
        bind_group
            .entries
            .push(CompiledBindingEntry::StorageTexture {
                resource: compile_resource_ref(&written_texture, resources),
                access: CompiledStorageAccess::ReadWrite,
            });
    }
    for uniform_id in &uniform_order {
        bind_group
            .entries
            .push(CompiledBindingEntry::UniformBuffer {
                resource: uniform_id.clone(),
            });
    }
    for storage in &storage_order {
        bind_group
            .entries
            .push(CompiledBindingEntry::StorageBuffer {
                resource: storage.resource.clone(),
                access: storage.access,
            });
    }

    CompiledPassBindings {
        bind_group,
        uniform_order,
        storage_order,
    }
}

#[derive(Debug, Clone)]
pub struct CompiledStorageBinding {
    pub resource: CompiledResourceRef,
    pub access: CompiledStorageAccess,
}

fn compile_target_plan(node: &RenderPassNode, resources: &ResourceGraph) -> CompiledTargetPlan {
    let color_outputs = node
        .writes
        .iter()
        .map(|id| compile_resource_ref(id, resources))
        .collect();
    let depth_output = node
        .depth_target
        .as_ref()
        .map(|id| compile_resource_ref(id, resources));
    let reads = node
        .reads
        .iter()
        .map(|id| compile_resource_ref(id, resources))
        .collect();

    CompiledTargetPlan {
        color_outputs,
        depth_output,
        reads,
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
            .map(|id| compile_resource_ref(id, resources))
            .collect(),
        instance_buffers: node
            .instance_buffers
            .iter()
            .map(|id| compile_resource_ref(id, resources))
            .collect(),
        index_buffers: node
            .index_buffers
            .iter()
            .map(|id| compile_resource_ref(id, resources))
            .collect(),
        indirect_buffers: node
            .indirect_buffers
            .iter()
            .map(|id| compile_resource_ref(id, resources))
            .collect(),
    }
}

fn compile_dispatch_plan(node: &RenderPassNode) -> Option<CompiledDispatchPlan> {
    match &node.compute_dispatch {
        Some(ComputeDispatchDescriptor::Fixed(value)) => Some(CompiledDispatchPlan::Fixed(*value)),
        Some(ComputeDispatchDescriptor::State(binding)) => Some(CompiledDispatchPlan::FromState {
            state_type_id: binding.state_type_id(),
            state_type_name: binding.state_type_name(),
        }),
        None => None,
    }
}

fn compile_view_mask(_node: &RenderPassNode) -> CompiledViewMask {
    let mut values = BTreeSet::<String>::new();
    values.insert("main".to_string());
    CompiledViewMask::Explicit(values)
}

fn compile_feature_id(node: &RenderPassNode) -> Option<String> {
    node.feature_id
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

fn compile_resource_ref(id: &RenderResourceId, resources: &ResourceGraph) -> CompiledResourceRef {
    let Some(descriptor) = find_descriptor(resources, id) else {
        return CompiledResourceRef::FlowOwned(id.clone());
    };
    match descriptor {
        RenderResourceDescriptor::ImportedTexture(value) => match value.semantic {
            ImportedTextureSemantic::SurfaceColor => {
                CompiledResourceRef::ImportedBuiltin(CompiledBuiltinImport::SurfaceColor)
            }
            ImportedTextureSemantic::SurfaceDepth => {
                CompiledResourceRef::ImportedBuiltin(CompiledBuiltinImport::SurfaceDepth)
            }
            ImportedTextureSemantic::BuiltinUiDrawList => {
                CompiledResourceRef::ImportedBuiltin(CompiledBuiltinImport::BuiltinUiDrawList)
            }
            ImportedTextureSemantic::HistoryTexture | ImportedTextureSemantic::External => {
                CompiledResourceRef::Imported(id.clone())
            }
        },
        RenderResourceDescriptor::ImportedBuffer(_) => CompiledResourceRef::Imported(id.clone()),
        _ => CompiledResourceRef::FlowOwned(id.clone()),
    }
}

fn find_descriptor<'a>(
    resources: &'a ResourceGraph,
    id: &RenderResourceId,
) -> Option<&'a RenderResourceDescriptor> {
    resources
        .resources
        .iter()
        .find(|descriptor| descriptor.id() == id)
}

fn dedupe_ids<'a, I>(ids: I) -> Vec<RenderResourceId>
where
    I: IntoIterator<Item = &'a RenderResourceId>,
{
    let mut seen = BTreeSet::<String>::new();
    let mut ordered = Vec::<RenderResourceId>::new();
    for id in ids {
        let key = id.as_str().to_string();
        if seen.insert(key) {
            ordered.push(id.clone());
        }
    }
    ordered
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::{GpuStorage, GpuUniform, RenderFlow, compile_flow_plan};

    #[derive(Debug, Clone, Copy, GpuStorage)]
    struct Cell {
        alive: u32,
    }

    #[derive(Debug, Clone, Copy, GpuUniform)]
    struct SimParams {
        tick: u32,
        _pad: [u32; 3],
    }

    #[derive(Debug, Clone, ecs::Resource)]
    struct FlowState {
        tick: u32,
    }

    impl FlowState {
        fn params(&self) -> SimParams {
            SimParams {
                tick: self.tick,
                _pad: [0; 3],
            }
        }

        fn dispatch(&self) -> [u32; 3] {
            [1, 1, 1]
        }
    }

    fn build_flow() -> RenderFlow {
        RenderFlow::new("execution.plan.test")
            .with_state::<FlowState>()
            .with_surface_color()
            .with_builtin_ui()
            .double_buffer_storage_array::<Cell>("cells", 16)
            .compute_pass("simulate")
            .for_feature("world.draw")
            .uniform_from_state(FlowState::params)
            .bind_ping_pong_storage("cells")
            .dispatch_from_state(FlowState::dispatch)
            .finish()
            .fullscreen_pass("compose")
            .bind_ping_pong_storage("cells")
            .write_surface_color()
            .depends_on("simulate")
            .finish()
            .builtin_ui_composite_pass("ui")
            .depends_on("compose")
            .finish()
            .validate()
            .expect("flow should validate")
    }

    #[test]
    fn execution_plan_carries_state_and_dispatch_metadata() {
        let flow = build_flow();
        let compiled = compile_flow_plan(&flow).expect("flow should compile");
        let execution = &compiled.execution;

        assert!(
            execution
                .required_state_types
                .iter()
                .any(|value| value.type_name == std::any::type_name::<FlowState>())
        );

        let compute = execution
            .passes
            .iter()
            .find_map(|pass| match pass {
                CompiledPassExecutionPlan::Compute(value) => Some(value),
                _ => None,
            })
            .expect("compute pass should exist");

        assert_eq!(compute.pass_id, "simulate");
        assert!(
            !compute.bindings.uniform_order.is_empty(),
            "compute pass should preserve uniform binding order"
        );
        assert_eq!(
            compute.bindings.storage_order.len(),
            2,
            "ping-pong storage should lower into two storage bindings"
        );
        assert!(
            compute
                .bindings
                .storage_order
                .iter()
                .all(|value| value.access == CompiledStorageAccess::ReadWrite),
            "compute pass writes both ping-pong storage buffers"
        );
        assert!(matches!(
            compute.dispatch,
            Some(CompiledDispatchPlan::FromState {
                state_type_name, ..
            }) if state_type_name == std::any::type_name::<FlowState>()
        ));
        assert_eq!(
            compute.feature_id.as_deref(),
            Some("world.draw"),
            "compute pass should preserve explicit feature identity"
        );
        assert!(
            compute.view_mask.includes("main"),
            "compute pass should target the canonical single active view"
        );
        assert!(
            !compute.view_mask.includes("minimap"),
            "single-view execution plan should not claim support for deferred view ids"
        );

        let fullscreen = execution
            .passes
            .iter()
            .find_map(|pass| match pass {
                CompiledPassExecutionPlan::Fullscreen(value) => Some(value),
                _ => None,
            })
            .expect("fullscreen pass should exist");
        assert!(
            fullscreen.feature_id.is_none(),
            "passes without explicit for_feature(...) should keep feature id empty"
        );
    }

    #[test]
    fn execution_plan_carries_explicit_builtin_ui_imports() {
        let flow = build_flow();
        let compiled = compile_flow_plan(&flow).expect("flow should compile");
        let execution = &compiled.execution;

        let ui = execution
            .passes
            .iter()
            .find_map(|pass| match pass {
                CompiledPassExecutionPlan::BuiltinUiComposite(value) => Some(value),
                _ => None,
            })
            .expect("ui pass should exist");

        assert_eq!(ui.pass_id, "ui");
        assert_eq!(
            ui.ui_input,
            CompiledResourceRef::ImportedBuiltin(CompiledBuiltinImport::BuiltinUiDrawList)
        );
        assert_eq!(
            ui.color_output,
            CompiledResourceRef::ImportedBuiltin(CompiledBuiltinImport::SurfaceColor)
        );
        assert!(
            ui.view_mask.includes("main"),
            "ui pass should target the canonical single active view"
        );
    }
}
