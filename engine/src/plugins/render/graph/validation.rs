use crate::plugins::render::RenderResourceDescriptor;
use crate::plugins::render::api::{SURFACE_COLOR_RESOURCE_LABEL, SURFACE_DEPTH_RESOURCE_LABEL};
use crate::plugins::render::graph::{
    RenderFlowGraph, RenderPassKind, RenderPassNode, validate_builtin_ui_pass_shape,
};
use crate::plugins::render::resource::{ImportedBufferSemantic, ImportedTextureSemantic};
use crate::plugins::render::{RenderPassId, RenderResourceId, RenderVertexStepMode};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use thiserror::Error;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FlowValidationReport {
    pub pass_order: Vec<RenderPassId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RenderFlowValidationIssue {
    #[error("duplicate resource id '{resource_id:?}'")]
    DuplicateResourceId { resource_id: RenderResourceId },

    #[error(
        "storage_buffer '{resource_id:?}' declares zero elements; element_count must be greater than zero"
    )]
    ZeroLengthStorageBuffer { resource_id: RenderResourceId },

    #[error("duplicate pass id '{pass_label}' ({pass_id:?})")]
    DuplicatePassId {
        pass_id: RenderPassId,
        pass_label: String,
    },

    #[error("pass '{pass_label}' depends on unknown pass '{dependency_id:?}'")]
    UnknownPassDependency {
        pass_label: String,
        dependency_id: RenderPassId,
    },

    #[error("pass '{pass_label}' references unknown resource '{resource_id:?}'")]
    UnknownResourceReference {
        pass_label: String,
        resource_id: RenderResourceId,
    },

    #[error(
        "pass '{pass_label}' uses uniform projection for state '{state_type_name}' but with_state::<...>() was not declared"
    )]
    MissingProjectedStateDeclaration {
        pass_label: String,
        state_type_name: &'static str,
    },

    #[error("pass '{pass_label}' references missing uniform buffer '{uniform_id:?}'")]
    MissingUniformBuffer {
        pass_label: String,
        uniform_id: RenderResourceId,
    },

    #[error(
        "pass '{pass_label}' uses dispatch_from_state for resource '{state_type_name}' but with_state::<...>() was not declared"
    )]
    MissingDispatchStateDeclaration {
        pass_label: String,
        state_type_name: &'static str,
    },

    #[error(
        "flow declares {count} present passes ({labels}); exactly zero or one present pass is allowed"
    )]
    MultiplePresentPasses { count: usize, labels: String },

    #[error(
        "present pass '{present_label}' must be terminal but is a dependency for ({dependent_labels})"
    )]
    PresentPassNotTerminal {
        present_label: String,
        dependent_labels: String,
    },

    #[error(
        "present pass '{present_label}' must be the final execution node; add explicit depends_on edges so it orders last"
    )]
    PresentPassNotLast { present_label: String },

    #[error("pass dependency cycle detected: {cycle_labels}")]
    PassDependencyCycleDetected { cycle_labels: String },

    #[error(
        "compute pass '{pass_label}' must declare explicit dispatch(...) or dispatch_from_state(...)"
    )]
    ComputePassMissingDispatch { pass_label: String },

    #[error("compute pass '{pass_label}' cannot declare a depth target")]
    ComputePassHasDepthTarget { pass_label: String },

    #[error("compute pass '{pass_label}' cannot declare vertex/index/instance/indirect buffers")]
    ComputePassHasGraphicsBuffers { pass_label: String },

    #[error("compute pass '{pass_label}' cannot declare clear_color")]
    ComputePassHasClearColor { pass_label: String },

    #[error("compute pass '{pass_label}' cannot declare draw parameters")]
    ComputePassHasDraw { pass_label: String },

    #[error("compute pass '{pass_label}' declares invalid dispatch_workgroups({x}, {y}, {z})")]
    InvalidComputeDispatch {
        pass_label: String,
        x: u32,
        y: u32,
        z: u32,
    },

    #[error("fullscreen pass '{pass_label}' cannot declare workgroup_size")]
    FullscreenPassHasWorkgroupSize { pass_label: String },

    #[error("fullscreen pass '{pass_label}' cannot declare compute dispatch")]
    FullscreenPassHasComputeDispatch { pass_label: String },

    #[error("fullscreen pass '{pass_label}' cannot declare a depth target")]
    FullscreenPassHasDepthTarget { pass_label: String },

    #[error("fullscreen pass '{pass_label}' cannot declare vertex/index/instance/indirect buffers")]
    FullscreenPassHasGraphicsBuffers { pass_label: String },

    #[error("fullscreen pass '{pass_label}' cannot declare vertex buffer layouts")]
    FullscreenPassHasVertexLayouts { pass_label: String },

    #[error("fullscreen pass '{pass_label}' cannot declare draw parameters")]
    FullscreenPassHasDraw { pass_label: String },

    #[error(
        "fullscreen pass '{pass_label}' must declare exactly one color output; found {write_count}"
    )]
    FullscreenPassInvalidColorOutputArity {
        pass_label: String,
        write_count: usize,
    },

    #[error("graphics pass '{pass_label}' cannot declare workgroup_size")]
    GraphicsPassHasWorkgroupSize { pass_label: String },

    #[error("graphics pass '{pass_label}' cannot declare compute dispatch")]
    GraphicsPassHasComputeDispatch { pass_label: String },

    #[error(
        "graphics pass '{pass_label}' must declare exactly one color output; found {write_count}"
    )]
    GraphicsPassInvalidColorOutputArity {
        pass_label: String,
        write_count: usize,
    },

    #[error(
        "{pass_kind} pass '{pass_label}' writes raster color output '{resource_id:?}' but kind '{resource_kind}' is not a runtime-supported color attachment"
    )]
    InvalidRasterColorOutputResource {
        pass_kind: &'static str,
        pass_label: String,
        resource_id: RenderResourceId,
        resource_kind: &'static str,
    },

    #[error("graphics pass '{pass_label}' must declare draw(...) or draw_with_offsets(...)")]
    GraphicsPassMissingDraw { pass_label: String },

    #[error(
        "graphics pass '{pass_label}' declares invalid draw(vertex_count={vertex_count}, instance_count={instance_count})"
    )]
    GraphicsPassInvalidDraw {
        pass_label: String,
        vertex_count: u32,
        instance_count: u32,
    },

    #[error(
        "graphics pass '{pass_label}' declares {buffer_count} {role} buffers but {layout_count} layouts"
    )]
    GraphicsPassBufferLayoutCountMismatch {
        pass_label: String,
        role: &'static str,
        buffer_count: usize,
        layout_count: usize,
    },

    #[error(
        "graphics pass '{pass_label}' declares {role} buffer layout slot {slot} with step mode '{step_mode}', expected '{expected}'"
    )]
    GraphicsPassBufferLayoutStepModeMismatch {
        pass_label: String,
        role: &'static str,
        slot: u32,
        step_mode: &'static str,
        expected: &'static str,
    },

    #[error("graphics pass '{pass_label}' declares duplicate vertex buffer slot {slot}")]
    GraphicsPassDuplicateVertexBufferSlot { pass_label: String, slot: u32 },

    #[error(
        "graphics pass '{pass_label}' declares vertex buffer slots that must be dense from 0; expected slot {expected}, found {found}"
    )]
    GraphicsPassNonDenseVertexBufferSlots {
        pass_label: String,
        expected: u32,
        found: u32,
    },

    #[error("graphics pass '{pass_label}' declares vertex buffer slot {slot} with zero stride")]
    GraphicsPassInvalidVertexStride { pass_label: String, slot: u32 },

    #[error("graphics pass '{pass_label}' declares vertex buffer slot {slot} with no attributes")]
    GraphicsPassMissingVertexAttributes { pass_label: String, slot: u32 },

    #[error("graphics pass '{pass_label}' declares duplicate shader location {shader_location}")]
    GraphicsPassDuplicateVertexShaderLocation {
        pass_label: String,
        shader_location: u32,
    },

    #[error(
        "graphics pass '{pass_label}' declares vertex attribute at location {shader_location} beyond stride for slot {slot}: offset {offset} + size {size} > stride {stride}"
    )]
    GraphicsPassInvalidVertexAttributeRange {
        pass_label: String,
        slot: u32,
        shader_location: u32,
        offset: u64,
        size: u64,
        stride: u64,
    },

    #[error(
        "copy pass '{pass_label}' must declare exactly one reads(...) and one writes(...) resource"
    )]
    CopyPassInvalidArity { pass_label: String },

    #[error("copy pass '{pass_label}' only supports reads/writes/depends_on")]
    CopyPassInvalidFields { pass_label: String },

    #[error("present pass '{pass_label}' must declare exactly one reads(...) resource")]
    PresentPassInvalidReadArity { pass_label: String },

    #[error("present pass '{pass_label}' cannot declare writes(...) resources")]
    PresentPassHasWrites { pass_label: String },

    #[error("present pass '{pass_label}' only supports reads/depends_on")]
    PresentPassInvalidFields { pass_label: String },

    #[error(
        "pass '{pass_label}' samples resource '{resource_id:?}' which is not texture-like (kind: {resource_kind})"
    )]
    SampledNonTextureResource {
        pass_label: String,
        resource_id: RenderResourceId,
        resource_kind: &'static str,
    },

    #[error(
        "pass '{pass_label}' writes texture resource '{resource_id:?}' via write_texture(...) but kind '{resource_kind}' is not storage/history texture"
    )]
    WriteTextureOnInvalidResource {
        pass_label: String,
        resource_id: RenderResourceId,
        resource_kind: &'static str,
    },

    #[error(
        "graphics pass '{pass_label}' uses depth_target '{resource_id:?}' but kind '{resource_kind}' is not a flow-owned depth target"
    )]
    InvalidDepthTargetResource {
        pass_label: String,
        resource_id: RenderResourceId,
        resource_kind: &'static str,
    },

    #[error(
        "copy pass '{pass_label}' mixes incompatible resource classes: '{read_id:?}' ({read_kind}) -> '{write_id:?}' ({write_kind})"
    )]
    CopyPassMixedResourceClasses {
        pass_label: String,
        read_id: RenderResourceId,
        read_kind: &'static str,
        write_id: RenderResourceId,
        write_kind: &'static str,
    },

    #[error(
        "present pass '{pass_label}' must read a texture-like resource; '{resource_id:?}' is '{resource_kind}'"
    )]
    PresentPassReadsNonTexture {
        pass_label: String,
        resource_id: RenderResourceId,
        resource_kind: &'static str,
    },

    #[error(
        "pass '{pass_label}' writes imported texture '{resource_id:?}' with semantic '{semantic}'; only '{allowed}' is writable in core runtime"
    )]
    InvalidImportedTextureWriteSemantic {
        pass_label: String,
        resource_id: RenderResourceId,
        semantic: &'static str,
        allowed: &'static str,
    },

    #[error(
        "pass '{pass_label}' writes imported texture '{resource_id:?}' but pass kind '{pass_kind:?}' is not supported for imported texture writes"
    )]
    UnsupportedImportedTextureWriteKind {
        pass_label: String,
        resource_id: RenderResourceId,
        pass_kind: RenderPassKind,
    },

    #[error(
        "pass '{pass_label}' uses '{resource_id:?}' in {role}(...) but kind '{resource_kind}' is not buffer-like"
    )]
    InvalidBufferRoleResource {
        pass_label: String,
        resource_id: RenderResourceId,
        role: &'static str,
        resource_kind: &'static str,
    },

    #[error(
        "resource '{resource_id:?}' uses external imported texture semantics; external imports are not supported in active runtime flows"
    )]
    UnsupportedExternalImportedTexture { resource_id: RenderResourceId },

    #[error(
        "resource '{resource_id:?}' uses external imported buffer semantics; external imports are not supported in active runtime flows"
    )]
    UnsupportedExternalImportedBuffer { resource_id: RenderResourceId },

    #[error(
        "multiple imported surface-color textures detected; expected zero or one canonical '{canonical_label}' import"
    )]
    MultipleSurfaceColorImports { canonical_label: &'static str },

    #[error(
        "multiple imported surface-depth textures detected; expected zero or one canonical '{canonical_label}' import"
    )]
    MultipleSurfaceDepthImports { canonical_label: &'static str },

    #[error(
        "builtin_ui_composite pass '{pass_label}' cannot declare explicit feature id; feature is fixed to 'ui'"
    )]
    BuiltinUiExplicitFeatureId { pass_label: String },

    #[error("builtin_ui_composite pass '{pass_label}' cannot declare shader")]
    BuiltinUiHasShader { pass_label: String },

    #[error("builtin_ui_composite pass '{pass_label}' cannot declare workgroup_size")]
    BuiltinUiHasWorkgroupSize { pass_label: String },

    #[error("builtin_ui_composite pass '{pass_label}' cannot declare clear_color")]
    BuiltinUiHasClearColor { pass_label: String },

    #[error("builtin_ui_composite pass '{pass_label}' cannot declare compute dispatch")]
    BuiltinUiHasComputeDispatch { pass_label: String },

    #[error("builtin_ui_composite pass '{pass_label}' cannot declare depth target")]
    BuiltinUiHasDepthTarget { pass_label: String },

    #[error("builtin_ui_composite pass '{pass_label}' cannot declare uniform bindings")]
    BuiltinUiHasUniformBindings { pass_label: String },

    #[error("builtin_ui_composite pass '{pass_label}' only supports writes/depends_on")]
    BuiltinUiInvalidResourceBindings { pass_label: String },

    #[error(
        "builtin_ui_composite pass '{pass_label}' must not declare reads(...); UI input comes from PreparedRenderFrame::ui()"
    )]
    BuiltinUiHasReads { pass_label: String },

    #[error(
        "builtin_ui_composite pass '{pass_label}' must declare exactly one writes(...) resource; found {write_count}"
    )]
    BuiltinUiInvalidWriteArity {
        pass_label: String,
        write_count: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct RenderFlowValidationError {
    pub issues: Vec<RenderFlowValidationIssue>,
    pub message: String,
}

impl From<Vec<RenderFlowValidationIssue>> for RenderFlowValidationError {
    fn from(issues: Vec<RenderFlowValidationIssue>) -> Self {
        let message = issues
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ");
        Self { issues, message }
    }
}

pub fn validate_flow_graph(
    graph: &RenderFlowGraph,
) -> Result<FlowValidationReport, RenderFlowValidationError> {
    let mut issues = Vec::<RenderFlowValidationIssue>::new();

    let mut resource_ids = BTreeSet::<RenderResourceId>::new();
    let mut resources_by_id = BTreeMap::<RenderResourceId, &RenderResourceDescriptor>::new();
    for resource in &graph.resources.resources {
        let resource_id = *resource.id();
        if !resource_ids.insert(resource_id) {
            issues.push(RenderFlowValidationIssue::DuplicateResourceId { resource_id });
        }
        if let RenderResourceDescriptor::StorageBuffer(value) = resource
            && value.element_count == 0
        {
            issues.push(RenderFlowValidationIssue::ZeroLengthStorageBuffer { resource_id });
        }
        resources_by_id.insert(resource_id, resource);
    }
    validate_imported_resource_descriptors(&resources_by_id, &mut issues);

    let mut pass_ids = BTreeSet::<RenderPassId>::new();
    for pass in &graph.passes.passes {
        if !pass_ids.insert(pass.id) {
            issues.push(RenderFlowValidationIssue::DuplicatePassId {
                pass_id: pass.id,
                pass_label: pass.label.clone(),
            });
        }
    }

    let pass_lookup: BTreeMap<RenderPassId, &RenderPassNode> = graph
        .passes
        .passes
        .iter()
        .map(|pass| (pass.id, pass))
        .collect();

    for pass in &graph.passes.passes {
        validate_pass_shape(pass, &mut issues);

        for dependency in &pass.depends_on {
            if !pass_lookup.contains_key(dependency) {
                issues.push(RenderFlowValidationIssue::UnknownPassDependency {
                    pass_label: pass.label.clone(),
                    dependency_id: *dependency,
                });
            }
        }

        for resource in pass_resource_refs(pass) {
            if !resource_ids.contains(resource) {
                issues.push(RenderFlowValidationIssue::UnknownResourceReference {
                    pass_label: pass.label.clone(),
                    resource_id: *resource,
                });
            }
        }

        validate_pass_resource_usage(pass, &resources_by_id, &mut issues);

        for binding in &pass.uniform_bindings {
            if !graph.resources.has_state_resource(binding.state_type_id()) {
                issues.push(
                    RenderFlowValidationIssue::MissingProjectedStateDeclaration {
                        pass_label: pass.label.clone(),
                        state_type_name: binding.state_type_name(),
                    },
                );
            }

            if !graph.resources.has_uniform_buffer(binding.uniform_id()) {
                issues.push(RenderFlowValidationIssue::MissingUniformBuffer {
                    pass_label: pass.label.clone(),
                    uniform_id: *binding.uniform_id(),
                });
            }
        }

        if let Some(dispatch) = &pass.compute_dispatch
            && let crate::plugins::render::api::ComputeDispatchDescriptor::State(binding) = dispatch
            && !graph.resources.has_state_resource(binding.state_type_id())
        {
            issues.push(RenderFlowValidationIssue::MissingDispatchStateDeclaration {
                pass_label: pass.label.clone(),
                state_type_name: binding.state_type_name(),
            });
        }
    }

    let present_passes = graph
        .passes
        .passes
        .iter()
        .filter(|pass| matches!(pass.kind, RenderPassKind::Present))
        .collect::<Vec<_>>();

    if present_passes.len() > 1 {
        issues.push(RenderFlowValidationIssue::MultiplePresentPasses {
            count: present_passes.len(),
            labels: present_passes
                .iter()
                .map(|pass| pass.label.clone())
                .collect::<Vec<_>>()
                .join(", "),
        });
    }

    let pass_order = topological_sort(&graph.passes.passes, &mut issues);

    if present_passes.len() == 1 {
        let present_pass = present_passes[0];
        let dependent_passes = graph
            .passes
            .passes
            .iter()
            .filter(|pass| pass.depends_on.contains(&present_pass.id))
            .map(|pass| pass.label.clone())
            .collect::<Vec<_>>();

        if !dependent_passes.is_empty() {
            issues.push(RenderFlowValidationIssue::PresentPassNotTerminal {
                present_label: present_pass.label.clone(),
                dependent_labels: dependent_passes.join(", "),
            });
        }

        if pass_order
            .last()
            .copied()
            .is_some_and(|id| id != present_pass.id)
        {
            issues.push(RenderFlowValidationIssue::PresentPassNotLast {
                present_label: present_pass.label.clone(),
            });
        }
    }

    if issues.is_empty() {
        Ok(FlowValidationReport { pass_order })
    } else {
        Err(RenderFlowValidationError::from(issues))
    }
}

fn validate_pass_shape(pass: &RenderPassNode, issues: &mut Vec<RenderFlowValidationIssue>) {
    match pass.kind {
        RenderPassKind::Compute => {
            if pass.compute_dispatch.is_none() {
                issues.push(RenderFlowValidationIssue::ComputePassMissingDispatch {
                    pass_label: pass.label.clone(),
                });
            }
            if pass.depth_target.is_some() {
                issues.push(RenderFlowValidationIssue::ComputePassHasDepthTarget {
                    pass_label: pass.label.clone(),
                });
            }
            if !pass.vertex_buffers.is_empty()
                || !pass.index_buffers.is_empty()
                || !pass.instance_buffers.is_empty()
                || !pass.indirect_buffers.is_empty()
                || !pass.vertex_buffer_layouts.is_empty()
                || !pass.instance_buffer_layouts.is_empty()
            {
                issues.push(RenderFlowValidationIssue::ComputePassHasGraphicsBuffers {
                    pass_label: pass.label.clone(),
                });
            }
            if pass.clear_color.is_some() {
                issues.push(RenderFlowValidationIssue::ComputePassHasClearColor {
                    pass_label: pass.label.clone(),
                });
            }
            if pass.draw.is_some() {
                issues.push(RenderFlowValidationIssue::ComputePassHasDraw {
                    pass_label: pass.label.clone(),
                });
            }
            if let Some(crate::plugins::render::api::ComputeDispatchDescriptor::Fixed(dims)) =
                &pass.compute_dispatch
                && (dims[0] == 0 || dims[1] == 0 || dims[2] == 0)
            {
                issues.push(RenderFlowValidationIssue::InvalidComputeDispatch {
                    pass_label: pass.label.clone(),
                    x: dims[0],
                    y: dims[1],
                    z: dims[2],
                });
            }
        }
        RenderPassKind::Fullscreen => {
            if pass.workgroup_size.is_some() {
                issues.push(RenderFlowValidationIssue::FullscreenPassHasWorkgroupSize {
                    pass_label: pass.label.clone(),
                });
            }
            if pass.compute_dispatch.is_some() {
                issues.push(
                    RenderFlowValidationIssue::FullscreenPassHasComputeDispatch {
                        pass_label: pass.label.clone(),
                    },
                );
            }
            if pass.depth_target.is_some() {
                issues.push(RenderFlowValidationIssue::FullscreenPassHasDepthTarget {
                    pass_label: pass.label.clone(),
                });
            }
            if !pass.vertex_buffers.is_empty()
                || !pass.index_buffers.is_empty()
                || !pass.instance_buffers.is_empty()
                || !pass.indirect_buffers.is_empty()
            {
                issues.push(
                    RenderFlowValidationIssue::FullscreenPassHasGraphicsBuffers {
                        pass_label: pass.label.clone(),
                    },
                );
            }
            if !pass.vertex_buffer_layouts.is_empty() || !pass.instance_buffer_layouts.is_empty() {
                issues.push(RenderFlowValidationIssue::FullscreenPassHasVertexLayouts {
                    pass_label: pass.label.clone(),
                });
            }
            if pass.draw.is_some() {
                issues.push(RenderFlowValidationIssue::FullscreenPassHasDraw {
                    pass_label: pass.label.clone(),
                });
            }
            if pass.writes.len() != 1 {
                issues.push(
                    RenderFlowValidationIssue::FullscreenPassInvalidColorOutputArity {
                        pass_label: pass.label.clone(),
                        write_count: pass.writes.len(),
                    },
                );
            }
        }
        RenderPassKind::BuiltinUiComposite => {
            validate_builtin_ui_pass_shape(pass, issues);
        }
        RenderPassKind::Graphics => {
            if pass.workgroup_size.is_some() {
                issues.push(RenderFlowValidationIssue::GraphicsPassHasWorkgroupSize {
                    pass_label: pass.label.clone(),
                });
            }
            if pass.compute_dispatch.is_some() {
                issues.push(RenderFlowValidationIssue::GraphicsPassHasComputeDispatch {
                    pass_label: pass.label.clone(),
                });
            }
            if pass.writes.len() != 1 {
                issues.push(
                    RenderFlowValidationIssue::GraphicsPassInvalidColorOutputArity {
                        pass_label: pass.label.clone(),
                        write_count: pass.writes.len(),
                    },
                );
            }
            match pass.draw {
                Some(draw) if draw.vertex_count == 0 || draw.instance_count == 0 => {
                    issues.push(RenderFlowValidationIssue::GraphicsPassInvalidDraw {
                        pass_label: pass.label.clone(),
                        vertex_count: draw.vertex_count,
                        instance_count: draw.instance_count,
                    });
                }
                Some(_) => {}
                None => issues.push(RenderFlowValidationIssue::GraphicsPassMissingDraw {
                    pass_label: pass.label.clone(),
                }),
            }
            validate_graphics_vertex_layouts(pass, issues);
        }
        RenderPassKind::Copy => {
            if pass.reads.len() != 1 || pass.writes.len() != 1 {
                issues.push(RenderFlowValidationIssue::CopyPassInvalidArity {
                    pass_label: pass.label.clone(),
                });
            }
            if pass.shader.is_some()
                || pass.workgroup_size.is_some()
                || pass.compute_dispatch.is_some()
                || pass.clear_color.is_some()
                || pass.depth_target.is_some()
                || pass.draw.is_some()
                || !pass.uniform_bindings.is_empty()
                || !pass.sampled_textures.is_empty()
                || !pass.write_textures.is_empty()
                || !pass.vertex_buffers.is_empty()
                || !pass.vertex_buffer_layouts.is_empty()
                || !pass.index_buffers.is_empty()
                || !pass.instance_buffers.is_empty()
                || !pass.instance_buffer_layouts.is_empty()
                || !pass.indirect_buffers.is_empty()
            {
                issues.push(RenderFlowValidationIssue::CopyPassInvalidFields {
                    pass_label: pass.label.clone(),
                });
            }
        }
        RenderPassKind::Present => {
            if pass.reads.len() != 1 {
                issues.push(RenderFlowValidationIssue::PresentPassInvalidReadArity {
                    pass_label: pass.label.clone(),
                });
            }
            if !pass.writes.is_empty() {
                issues.push(RenderFlowValidationIssue::PresentPassHasWrites {
                    pass_label: pass.label.clone(),
                });
            }
            if pass.shader.is_some()
                || pass.workgroup_size.is_some()
                || pass.compute_dispatch.is_some()
                || pass.clear_color.is_some()
                || pass.depth_target.is_some()
                || pass.draw.is_some()
                || !pass.uniform_bindings.is_empty()
                || !pass.sampled_textures.is_empty()
                || !pass.write_textures.is_empty()
                || !pass.vertex_buffers.is_empty()
                || !pass.vertex_buffer_layouts.is_empty()
                || !pass.index_buffers.is_empty()
                || !pass.instance_buffers.is_empty()
                || !pass.instance_buffer_layouts.is_empty()
                || !pass.indirect_buffers.is_empty()
            {
                issues.push(RenderFlowValidationIssue::PresentPassInvalidFields {
                    pass_label: pass.label.clone(),
                });
            }
        }
    }
}

fn validate_pass_resource_usage(
    pass: &RenderPassNode,
    resources_by_id: &BTreeMap<RenderResourceId, &RenderResourceDescriptor>,
    issues: &mut Vec<RenderFlowValidationIssue>,
) {
    for sampled in &pass.sampled_textures {
        let Some(resource) = resources_by_id.get(sampled) else {
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
            issues.push(RenderFlowValidationIssue::SampledNonTextureResource {
                pass_label: pass.label.clone(),
                resource_id: *sampled,
                resource_kind: resource_kind_name(resource),
            });
        }
    }

    for written in &pass.write_textures {
        let Some(resource) = resources_by_id.get(written) else {
            continue;
        };
        if !matches!(
            resource,
            RenderResourceDescriptor::StorageTexture(_)
                | RenderResourceDescriptor::HistoryTexture(_)
        ) {
            issues.push(RenderFlowValidationIssue::WriteTextureOnInvalidResource {
                pass_label: pass.label.clone(),
                resource_id: *written,
                resource_kind: resource_kind_name(resource),
            });
        }
    }

    if matches!(
        pass.kind,
        RenderPassKind::Fullscreen | RenderPassKind::Graphics
    ) && pass.writes.len() == 1
    {
        let output = pass.writes[0];
        if let Some(resource) = resources_by_id.get(&output)
            && !is_raster_color_output_resource(resource)
        {
            issues.push(
                RenderFlowValidationIssue::InvalidRasterColorOutputResource {
                    pass_kind: render_pass_kind_name(pass.kind),
                    pass_label: pass.label.clone(),
                    resource_id: output,
                    resource_kind: resource_kind_name(resource),
                },
            );
        }
    }

    for id in &pass.vertex_buffers {
        validate_buffer_role_resource(pass, *id, "vertex_buffer", resources_by_id, issues);
    }
    for id in &pass.index_buffers {
        validate_buffer_role_resource(pass, *id, "index_buffer", resources_by_id, issues);
    }
    for id in &pass.instance_buffers {
        validate_buffer_role_resource(pass, *id, "instance_buffer", resources_by_id, issues);
    }
    for id in &pass.indirect_buffers {
        validate_buffer_role_resource(pass, *id, "indirect_buffer", resources_by_id, issues);
    }

    if let Some(depth_target) = &pass.depth_target
        && let Some(resource) = resources_by_id.get(depth_target)
    {
        let depth_ok = matches!(resource, RenderResourceDescriptor::DepthTarget(_));
        if !depth_ok {
            issues.push(RenderFlowValidationIssue::InvalidDepthTargetResource {
                pass_label: pass.label.clone(),
                resource_id: *depth_target,
                resource_kind: resource_kind_name(resource),
            });
        }
    }

    if matches!(pass.kind, RenderPassKind::Copy) && pass.reads.len() == 1 && pass.writes.len() == 1
    {
        let read = pass.reads[0];
        let write = pass.writes[0];
        if let (Some(read_resource), Some(write_resource)) =
            (resources_by_id.get(&read), resources_by_id.get(&write))
        {
            let read_texture = is_texture_resource(read_resource);
            let write_texture = is_texture_resource(write_resource);
            let read_buffer = is_buffer_resource(read_resource);
            let write_buffer = is_buffer_resource(write_resource);
            if (read_texture && write_buffer) || (read_buffer && write_texture) {
                issues.push(RenderFlowValidationIssue::CopyPassMixedResourceClasses {
                    pass_label: pass.label.clone(),
                    read_id: read,
                    read_kind: resource_kind_name(read_resource),
                    write_id: write,
                    write_kind: resource_kind_name(write_resource),
                });
            }
        }
    }

    if matches!(pass.kind, RenderPassKind::Present) && pass.reads.len() == 1 {
        let read = pass.reads[0];
        if let Some(resource) = resources_by_id.get(&read)
            && !is_texture_resource(resource)
        {
            issues.push(RenderFlowValidationIssue::PresentPassReadsNonTexture {
                pass_label: pass.label.clone(),
                resource_id: read,
                resource_kind: resource_kind_name(resource),
            });
        }
    }

    for write in &pass.writes {
        let Some(resource) = resources_by_id.get(write) else {
            continue;
        };
        if let RenderResourceDescriptor::ImportedTexture(value) = resource {
            if value.semantic != ImportedTextureSemantic::SurfaceColor {
                issues.push(
                    RenderFlowValidationIssue::InvalidImportedTextureWriteSemantic {
                        pass_label: pass.label.clone(),
                        resource_id: *write,
                        semantic: value.semantic.as_str(),
                        allowed: ImportedTextureSemantic::SurfaceColor.as_str(),
                    },
                );
                continue;
            }
            if !matches!(
                pass.kind,
                RenderPassKind::Fullscreen
                    | RenderPassKind::Graphics
                    | RenderPassKind::BuiltinUiComposite
                    | RenderPassKind::Copy
            ) {
                issues.push(
                    RenderFlowValidationIssue::UnsupportedImportedTextureWriteKind {
                        pass_label: pass.label.clone(),
                        resource_id: *write,
                        pass_kind: pass.kind,
                    },
                );
            }
        }
    }
}

fn validate_graphics_vertex_layouts(
    pass: &RenderPassNode,
    issues: &mut Vec<RenderFlowValidationIssue>,
) {
    validate_graphics_buffer_layout_counts(
        pass,
        "vertex",
        pass.vertex_buffers.len(),
        pass.vertex_buffer_layouts.len(),
        issues,
    );
    validate_graphics_buffer_layout_counts(
        pass,
        "instance",
        pass.instance_buffers.len(),
        pass.instance_buffer_layouts.len(),
        issues,
    );

    let mut slots = BTreeSet::<u32>::new();
    let mut ordered_slots = Vec::<u32>::new();
    let mut shader_locations = BTreeSet::<u32>::new();

    for layout in &pass.vertex_buffer_layouts {
        validate_graphics_layout_step_mode(
            pass,
            "vertex",
            layout.slot,
            layout.step_mode,
            RenderVertexStepMode::Vertex,
            issues,
        );
        validate_graphics_layout_shape(pass, layout, &mut shader_locations, issues);
        if !slots.insert(layout.slot) {
            issues.push(
                RenderFlowValidationIssue::GraphicsPassDuplicateVertexBufferSlot {
                    pass_label: pass.label.clone(),
                    slot: layout.slot,
                },
            );
        }
        ordered_slots.push(layout.slot);
    }

    for layout in &pass.instance_buffer_layouts {
        validate_graphics_layout_step_mode(
            pass,
            "instance",
            layout.slot,
            layout.step_mode,
            RenderVertexStepMode::Instance,
            issues,
        );
        validate_graphics_layout_shape(pass, layout, &mut shader_locations, issues);
        if !slots.insert(layout.slot) {
            issues.push(
                RenderFlowValidationIssue::GraphicsPassDuplicateVertexBufferSlot {
                    pass_label: pass.label.clone(),
                    slot: layout.slot,
                },
            );
        }
        ordered_slots.push(layout.slot);
    }

    ordered_slots.sort_unstable();
    for (expected, found) in ordered_slots.iter().copied().enumerate() {
        if expected as u32 != found {
            issues.push(
                RenderFlowValidationIssue::GraphicsPassNonDenseVertexBufferSlots {
                    pass_label: pass.label.clone(),
                    expected: expected as u32,
                    found,
                },
            );
        }
    }
}

fn validate_graphics_buffer_layout_counts(
    pass: &RenderPassNode,
    role: &'static str,
    buffer_count: usize,
    layout_count: usize,
    issues: &mut Vec<RenderFlowValidationIssue>,
) {
    if buffer_count != layout_count {
        issues.push(
            RenderFlowValidationIssue::GraphicsPassBufferLayoutCountMismatch {
                pass_label: pass.label.clone(),
                role,
                buffer_count,
                layout_count,
            },
        );
    }
}

fn validate_graphics_layout_step_mode(
    pass: &RenderPassNode,
    role: &'static str,
    slot: u32,
    step_mode: RenderVertexStepMode,
    expected: RenderVertexStepMode,
    issues: &mut Vec<RenderFlowValidationIssue>,
) {
    if step_mode != expected {
        issues.push(
            RenderFlowValidationIssue::GraphicsPassBufferLayoutStepModeMismatch {
                pass_label: pass.label.clone(),
                role,
                slot,
                step_mode: vertex_step_mode_name(step_mode),
                expected: vertex_step_mode_name(expected),
            },
        );
    }
}

fn validate_graphics_layout_shape(
    pass: &RenderPassNode,
    layout: &crate::plugins::render::RenderVertexBufferLayout,
    shader_locations: &mut BTreeSet<u32>,
    issues: &mut Vec<RenderFlowValidationIssue>,
) {
    if layout.array_stride == 0 {
        issues.push(RenderFlowValidationIssue::GraphicsPassInvalidVertexStride {
            pass_label: pass.label.clone(),
            slot: layout.slot,
        });
    }

    if layout.attributes.is_empty() {
        issues.push(
            RenderFlowValidationIssue::GraphicsPassMissingVertexAttributes {
                pass_label: pass.label.clone(),
                slot: layout.slot,
            },
        );
    }

    for attribute in &layout.attributes {
        if !shader_locations.insert(attribute.shader_location) {
            issues.push(
                RenderFlowValidationIssue::GraphicsPassDuplicateVertexShaderLocation {
                    pass_label: pass.label.clone(),
                    shader_location: attribute.shader_location,
                },
            );
        }

        let size = attribute.format.size_bytes();
        if attribute.offset.saturating_add(size) > layout.array_stride {
            issues.push(
                RenderFlowValidationIssue::GraphicsPassInvalidVertexAttributeRange {
                    pass_label: pass.label.clone(),
                    slot: layout.slot,
                    shader_location: attribute.shader_location,
                    offset: attribute.offset,
                    size,
                    stride: layout.array_stride,
                },
            );
        }
    }
}

fn vertex_step_mode_name(value: RenderVertexStepMode) -> &'static str {
    match value {
        RenderVertexStepMode::Vertex => "vertex",
        RenderVertexStepMode::Instance => "instance",
    }
}

fn is_raster_color_output_resource(resource: &RenderResourceDescriptor) -> bool {
    match resource {
        RenderResourceDescriptor::ColorTarget(_) => true,
        RenderResourceDescriptor::ImportedTexture(value) => {
            value.semantic == ImportedTextureSemantic::SurfaceColor
        }
        _ => false,
    }
}

fn validate_buffer_role_resource(
    pass: &RenderPassNode,
    resource_id: RenderResourceId,
    role: &'static str,
    resources_by_id: &BTreeMap<RenderResourceId, &RenderResourceDescriptor>,
    issues: &mut Vec<RenderFlowValidationIssue>,
) {
    let Some(resource) = resources_by_id.get(&resource_id) else {
        return;
    };
    if !is_buffer_resource(resource) {
        issues.push(RenderFlowValidationIssue::InvalidBufferRoleResource {
            pass_label: pass.label.clone(),
            resource_id,
            role,
            resource_kind: resource_kind_name(resource),
        });
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
        RenderResourceDescriptor::ImportedTexture(value) => match value.semantic {
            ImportedTextureSemantic::SurfaceColor => "imported_texture(surface_color)",
            ImportedTextureSemantic::SurfaceDepth => "imported_texture(surface_depth)",
            ImportedTextureSemantic::HistoryTexture => "imported_texture(history_texture)",
            ImportedTextureSemantic::External => "imported_texture(external)",
        },
        RenderResourceDescriptor::ImportedBuffer(value) => match value.semantic {
            ImportedBufferSemantic::HistoryBuffer => "imported_buffer(history_buffer)",
            ImportedBufferSemantic::External => "imported_buffer(external)",
        },
    }
}

fn render_pass_kind_name(kind: RenderPassKind) -> &'static str {
    match kind {
        RenderPassKind::Compute => "compute",
        RenderPassKind::Fullscreen => "fullscreen",
        RenderPassKind::BuiltinUiComposite => "builtin_ui_composite",
        RenderPassKind::Graphics => "graphics",
        RenderPassKind::Copy => "copy",
        RenderPassKind::Present => "present",
    }
}

fn validate_imported_resource_descriptors(
    resources_by_id: &BTreeMap<RenderResourceId, &RenderResourceDescriptor>,
    issues: &mut Vec<RenderFlowValidationIssue>,
) {
    let mut surface_color_count = 0usize;
    let mut surface_depth_count = 0usize;

    for (id, descriptor) in resources_by_id {
        match descriptor {
            RenderResourceDescriptor::ImportedTexture(value) => match value.semantic {
                ImportedTextureSemantic::SurfaceColor => {
                    surface_color_count = surface_color_count.saturating_add(1);
                }
                ImportedTextureSemantic::SurfaceDepth => {
                    surface_depth_count = surface_depth_count.saturating_add(1);
                }
                ImportedTextureSemantic::HistoryTexture => {}
                ImportedTextureSemantic::External => {
                    issues.push(
                        RenderFlowValidationIssue::UnsupportedExternalImportedTexture {
                            resource_id: *id,
                        },
                    );
                }
            },
            RenderResourceDescriptor::ImportedBuffer(value) => match value.semantic {
                ImportedBufferSemantic::HistoryBuffer => {}
                ImportedBufferSemantic::External => {
                    issues.push(
                        RenderFlowValidationIssue::UnsupportedExternalImportedBuffer {
                            resource_id: *id,
                        },
                    );
                }
            },
            _ => {}
        }
    }

    if surface_color_count > 1 {
        issues.push(RenderFlowValidationIssue::MultipleSurfaceColorImports {
            canonical_label: SURFACE_COLOR_RESOURCE_LABEL,
        });
    }

    if surface_depth_count > 1 {
        issues.push(RenderFlowValidationIssue::MultipleSurfaceDepthImports {
            canonical_label: SURFACE_DEPTH_RESOURCE_LABEL,
        });
    }
}

fn topological_sort(
    passes: &[RenderPassNode],
    issues: &mut Vec<RenderFlowValidationIssue>,
) -> Vec<RenderPassId> {
    let mut by_id = BTreeMap::<RenderPassId, usize>::new();
    for (index, pass) in passes.iter().enumerate() {
        by_id.insert(pass.id, index);
    }

    let mut indegree = vec![0usize; passes.len()];
    let mut outgoing = vec![Vec::<usize>::new(); passes.len()];

    for (index, pass) in passes.iter().enumerate() {
        for dependency in &pass.depends_on {
            if let Some(dep_index) = by_id.get(dependency) {
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

    let mut order = Vec::<RenderPassId>::with_capacity(passes.len());
    while let Some(index) = queue.pop_front() {
        order.push(passes[index].id);
        for next in &outgoing[index] {
            indegree[*next] = indegree[*next].saturating_sub(1);
            if indegree[*next] == 0 {
                queue.push_back(*next);
            }
        }
    }

    if order.len() != passes.len() {
        let cycle_labels = indegree
            .iter()
            .enumerate()
            .filter(|(_, degree)| **degree > 0)
            .map(|(index, _)| passes[index].label.clone())
            .collect::<Vec<_>>()
            .join(", ");
        issues.push(RenderFlowValidationIssue::PassDependencyCycleDetected { cycle_labels });
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
