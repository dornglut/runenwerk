use crate::plugins::render::graph::{
    CompiledBindingEntry, CompiledPassExecutionPlan, CompiledRenderFlowPlan,
    RenderExecutionGraphDiagnostic, RenderExecutionGraphDiagnosticKind,
};
use crate::plugins::render::resource::{RenderTextureFormatPolicy, RenderTextureTargetFormat};
use crate::plugins::render::{RenderPassKind, RenderResourceDescriptor};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderBackendCapabilityProfile {
    pub key: String,
    pub supports_compute: bool,
    pub supports_fullscreen: bool,
    pub supports_graphics: bool,
    pub supports_copy: bool,
    pub supports_present: bool,
    pub supports_builtin_ui_composite: bool,
    pub supports_storage_textures: bool,
    pub supports_depth_targets: bool,
    pub max_color_attachments: usize,
    pub max_bind_group_entries: usize,
    pub max_vertex_buffers: usize,
    pub max_uniform_buffer_size: u64,
    pub max_storage_buffer_size: u64,
    pub color_formats: BTreeSet<RenderTextureTargetFormat>,
    pub storage_texture_formats: BTreeSet<RenderTextureTargetFormat>,
    pub depth_formats: BTreeSet<RenderTextureTargetFormat>,
}

impl Default for RenderBackendCapabilityProfile {
    fn default() -> Self {
        Self::runtime_default()
    }
}

impl RenderBackendCapabilityProfile {
    pub fn runtime_default() -> Self {
        Self {
            key: "wgpu-portable-v1".to_string(),
            supports_compute: true,
            supports_fullscreen: true,
            supports_graphics: true,
            supports_copy: true,
            supports_present: true,
            supports_builtin_ui_composite: true,
            supports_storage_textures: true,
            supports_depth_targets: true,
            max_color_attachments: 1,
            max_bind_group_entries: 16,
            max_vertex_buffers: 8,
            max_uniform_buffer_size: 64 * 1024,
            max_storage_buffer_size: 128 * 1024 * 1024,
            color_formats: [
                RenderTextureTargetFormat::Rgba8Unorm,
                RenderTextureTargetFormat::Rgba8UnormSrgb,
                RenderTextureTargetFormat::R32Uint,
            ]
            .into_iter()
            .collect(),
            storage_texture_formats: [
                RenderTextureTargetFormat::Rgba8Unorm,
                RenderTextureTargetFormat::R32Uint,
            ]
            .into_iter()
            .collect(),
            depth_formats: [RenderTextureTargetFormat::Depth32Float]
                .into_iter()
                .collect(),
        }
    }

    pub fn unsupported_for_tests(capability: impl Into<String>) -> Self {
        let capability = capability.into();
        let mut value = Self::runtime_default();
        match capability.as_str() {
            "compute" => value.supports_compute = false,
            "graphics" => value.supports_graphics = false,
            "storage_textures" => value.supports_storage_textures = false,
            "depth_targets" => value.supports_depth_targets = false,
            "bind_group_entries" => value.max_bind_group_entries = 0,
            _ => {}
        }
        value.key = format!("test-without-{capability}");
        value
    }

    pub fn supports_pass_kind(&self, kind: RenderPassKind) -> bool {
        match kind {
            RenderPassKind::Compute => self.supports_compute,
            RenderPassKind::Fullscreen => self.supports_fullscreen,
            RenderPassKind::Graphics => self.supports_graphics,
            RenderPassKind::Copy => self.supports_copy,
            RenderPassKind::Present => self.supports_present,
            RenderPassKind::BuiltinUiComposite => self.supports_builtin_ui_composite,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderBackendCapabilityInspection {
    pub profile_key: String,
    pub max_color_attachments: usize,
    pub max_bind_group_entries: usize,
    pub max_vertex_buffers: usize,
    pub supports_storage_textures: bool,
    pub supports_depth_targets: bool,
}

impl From<&RenderBackendCapabilityProfile> for RenderBackendCapabilityInspection {
    fn from(value: &RenderBackendCapabilityProfile) -> Self {
        Self {
            profile_key: value.key.clone(),
            max_color_attachments: value.max_color_attachments,
            max_bind_group_entries: value.max_bind_group_entries,
            max_vertex_buffers: value.max_vertex_buffers,
            supports_storage_textures: value.supports_storage_textures,
            supports_depth_targets: value.supports_depth_targets,
        }
    }
}

pub fn validate_compiled_flow_capabilities(
    flow: &CompiledRenderFlowPlan,
    profile: &RenderBackendCapabilityProfile,
) -> Vec<RenderExecutionGraphDiagnostic> {
    let mut diagnostics = Vec::<RenderExecutionGraphDiagnostic>::new();

    for pass in &flow.pass_order {
        if !profile.supports_pass_kind(pass.node().kind) {
            diagnostics.push(
                RenderExecutionGraphDiagnostic::error(
                    RenderExecutionGraphDiagnosticKind::BackendCapabilityMismatch,
                    format!(
                        "backend capability profile '{}' does not support {:?} pass '{}'",
                        profile.key,
                        pass.node().kind,
                        pass.pass_label()
                    ),
                )
                .with_flow(flow.flow_id, flow.flow_label.clone())
                .with_pass(pass.pass_id(), pass.pass_label().to_string())
                .with_capability(format!("pass_kind::{:?}", pass.node().kind)),
            );
        }
    }

    for pass in &flow.execution.passes {
        validate_execution_pass_capabilities(flow, pass, profile, &mut diagnostics);
    }

    for descriptor in &flow.resources.resources {
        validate_resource_descriptor_capabilities(flow, descriptor, profile, &mut diagnostics);
    }

    diagnostics
}

fn validate_execution_pass_capabilities(
    flow: &CompiledRenderFlowPlan,
    pass: &CompiledPassExecutionPlan,
    profile: &RenderBackendCapabilityProfile,
    diagnostics: &mut Vec<RenderExecutionGraphDiagnostic>,
) {
    match pass {
        CompiledPassExecutionPlan::Compute(value) => {
            validate_bind_group_limit(
                flow,
                value.pass_id,
                value.bindings.bind_group.entries.len(),
                profile,
                diagnostics,
            );
        }
        CompiledPassExecutionPlan::Fullscreen(value)
        | CompiledPassExecutionPlan::Graphics(value) => {
            validate_bind_group_limit(
                flow,
                value.pass_id,
                value.bindings.bind_group.entries.len(),
                profile,
                diagnostics,
            );
            if value.targets.color_outputs.len() > profile.max_color_attachments {
                diagnostics.push(
                    RenderExecutionGraphDiagnostic::error(
                        RenderExecutionGraphDiagnosticKind::BackendCapabilityMismatch,
                        format!(
                            "pass '{}' declares {} color attachments but backend profile '{}' supports {}",
                            value.pass_id,
                            value.targets.color_outputs.len(),
                            profile.key,
                            profile.max_color_attachments
                        ),
                    )
                    .with_flow(flow.flow_id, flow.flow_label.clone())
                    .with_pass(value.pass_id, value.pass_id.to_string())
                    .with_capability("max_color_attachments"),
                );
            }
            if value.draw_buffers.vertex_buffers.len() > profile.max_vertex_buffers {
                diagnostics.push(
                    RenderExecutionGraphDiagnostic::error(
                        RenderExecutionGraphDiagnosticKind::BackendCapabilityMismatch,
                        format!(
                            "pass '{}' declares {} vertex buffers but backend profile '{}' supports {}",
                            value.pass_id,
                            value.draw_buffers.vertex_buffers.len(),
                            profile.key,
                            profile.max_vertex_buffers
                        ),
                    )
                    .with_flow(flow.flow_id, flow.flow_label.clone())
                    .with_pass(value.pass_id, value.pass_id.to_string())
                    .with_capability("max_vertex_buffers"),
                );
            }
        }
        CompiledPassExecutionPlan::Copy(_)
        | CompiledPassExecutionPlan::Present(_)
        | CompiledPassExecutionPlan::BuiltinUiComposite(_) => {}
    }
}

fn validate_bind_group_limit(
    flow: &CompiledRenderFlowPlan,
    pass_id: crate::plugins::render::RenderPassId,
    entries: usize,
    profile: &RenderBackendCapabilityProfile,
    diagnostics: &mut Vec<RenderExecutionGraphDiagnostic>,
) {
    if entries <= profile.max_bind_group_entries {
        return;
    }
    diagnostics.push(
        RenderExecutionGraphDiagnostic::error(
            RenderExecutionGraphDiagnosticKind::BackendCapabilityMismatch,
            format!(
                "pass '{}' declares {} bind group entries but backend profile '{}' supports {}",
                pass_id, entries, profile.key, profile.max_bind_group_entries
            ),
        )
        .with_flow(flow.flow_id, flow.flow_label.clone())
        .with_pass(pass_id, pass_id.to_string())
        .with_capability("max_bind_group_entries"),
    );
}

fn validate_resource_descriptor_capabilities(
    flow: &CompiledRenderFlowPlan,
    descriptor: &RenderResourceDescriptor,
    profile: &RenderBackendCapabilityProfile,
    diagnostics: &mut Vec<RenderExecutionGraphDiagnostic>,
) {
    match descriptor {
        RenderResourceDescriptor::UniformBuffer(value)
            if value.size_bytes > profile.max_uniform_buffer_size =>
        {
            diagnostics.push(resource_capability_diagnostic(
                flow,
                *descriptor.id(),
                "max_uniform_buffer_size",
                format!(
                    "uniform buffer '{}' requires {} bytes but backend profile '{}' supports {}",
                    value.id, value.size_bytes, profile.key, profile.max_uniform_buffer_size
                ),
            ));
        }
        RenderResourceDescriptor::StorageBuffer(value)
            if value.size_bytes > profile.max_storage_buffer_size =>
        {
            diagnostics.push(resource_capability_diagnostic(
                flow,
                *descriptor.id(),
                "max_storage_buffer_size",
                format!(
                    "storage buffer '{}' requires {} bytes but backend profile '{}' supports {}",
                    value.id, value.size_bytes, profile.key, profile.max_storage_buffer_size
                ),
            ));
        }
        RenderResourceDescriptor::StorageTexture(value) => {
            if !profile.supports_storage_textures {
                diagnostics.push(resource_capability_diagnostic(
                    flow,
                    *descriptor.id(),
                    "supports_storage_textures",
                    format!(
                        "storage texture '{}' is not supported by backend profile '{}'",
                        value.id, profile.key
                    ),
                ));
            }
            validate_texture_format_capability(
                flow,
                *descriptor.id(),
                value.texture.format,
                &profile.storage_texture_formats,
                profile,
                "storage_texture_formats",
                diagnostics,
            );
        }
        RenderResourceDescriptor::ColorTarget(value) => validate_texture_format_capability(
            flow,
            *descriptor.id(),
            value.texture.format,
            &profile.color_formats,
            profile,
            "color_formats",
            diagnostics,
        ),
        RenderResourceDescriptor::DepthTarget(value) => {
            if !profile.supports_depth_targets {
                diagnostics.push(resource_capability_diagnostic(
                    flow,
                    *descriptor.id(),
                    "supports_depth_targets",
                    format!(
                        "depth target '{}' is not supported by backend profile '{}'",
                        value.id, profile.key
                    ),
                ));
            }
            validate_texture_format_capability(
                flow,
                *descriptor.id(),
                value.texture.format,
                &profile.depth_formats,
                profile,
                "depth_formats",
                diagnostics,
            );
        }
        RenderResourceDescriptor::SampledTexture(value) => validate_texture_format_capability(
            flow,
            *descriptor.id(),
            value.texture.format,
            &profile.color_formats,
            profile,
            "sampled_texture_formats",
            diagnostics,
        ),
        RenderResourceDescriptor::HistoryTexture(value) => validate_texture_format_capability(
            flow,
            *descriptor.id(),
            value.texture.format,
            &profile.color_formats,
            profile,
            "history_texture_formats",
            diagnostics,
        ),
        RenderResourceDescriptor::UniformBuffer(_)
        | RenderResourceDescriptor::StorageBuffer(_)
        | RenderResourceDescriptor::TargetAlias(_)
        | RenderResourceDescriptor::ImportedTexture(_)
        | RenderResourceDescriptor::ImportedBuffer(_) => {}
    }
}

fn validate_texture_format_capability(
    flow: &CompiledRenderFlowPlan,
    resource_id: crate::plugins::render::RenderResourceId,
    format: RenderTextureFormatPolicy,
    supported: &BTreeSet<RenderTextureTargetFormat>,
    profile: &RenderBackendCapabilityProfile,
    capability: &'static str,
    diagnostics: &mut Vec<RenderExecutionGraphDiagnostic>,
) {
    let RenderTextureFormatPolicy::Exact(format) = format else {
        return;
    };
    if supported.contains(&format) {
        return;
    }
    diagnostics.push(resource_capability_diagnostic(
        flow,
        resource_id,
        capability,
        format!(
            "resource '{}' requires format {:?} but backend profile '{}' does not list it in {}",
            resource_id, format, profile.key, capability
        ),
    ));
}

fn resource_capability_diagnostic(
    flow: &CompiledRenderFlowPlan,
    resource_id: crate::plugins::render::RenderResourceId,
    capability: impl Into<String>,
    message: impl Into<String>,
) -> RenderExecutionGraphDiagnostic {
    RenderExecutionGraphDiagnostic::error(
        RenderExecutionGraphDiagnosticKind::BackendCapabilityMismatch,
        message,
    )
    .with_flow(flow.flow_id, flow.flow_label.clone())
    .with_resource(resource_id, flow.resource_label(resource_id))
    .with_capability(capability)
}

pub fn bind_group_entry_count(pass: &CompiledPassExecutionPlan) -> usize {
    match pass {
        CompiledPassExecutionPlan::Compute(value) => value.bindings.bind_group.entries.len(),
        CompiledPassExecutionPlan::Fullscreen(value)
        | CompiledPassExecutionPlan::Graphics(value) => value.bindings.bind_group.entries.len(),
        CompiledPassExecutionPlan::Copy(_)
        | CompiledPassExecutionPlan::Present(_)
        | CompiledPassExecutionPlan::BuiltinUiComposite(_) => 0,
    }
}

pub fn binding_entry_resource_label(entry: &CompiledBindingEntry) -> &'static str {
    match entry {
        CompiledBindingEntry::SampledTexture { .. } => "sampled_texture",
        CompiledBindingEntry::Sampler => "sampler",
        CompiledBindingEntry::StorageTexture { .. } => "storage_texture",
        CompiledBindingEntry::UniformBuffer { .. } => "uniform_buffer",
        CompiledBindingEntry::StorageBuffer { .. } => "storage_buffer",
    }
}
