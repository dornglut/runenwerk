use super::*;
use crate::plugins::render::{RenderFlowId, RenderPassId};

#[derive(Debug, Clone)]
pub struct EncodedPassEvidence {
    pub dispatch_workgroups: Option<[u32; 3]>,
    pub shader_id: String,
    pub shader_revision: u64,
    pub fallback_used: bool,
    pub pipeline_key: Option<FlowPassPipelineKey>,
}

#[derive(Debug, Clone)]
pub struct EncodedPipelinePass {
    pub dispatch_workgroups: Option<[u32; 3]>,
    pub shader_id: String,
    pub shader_revision: u64,
    pub fallback_used: bool,
    pub pipeline_key: FlowPassPipelineKey,
}

#[derive(Debug, Default)]
pub struct PassResourceTruth {
    pub render_targets: Vec<String>,
    pub sampled_textures: Vec<String>,
    pub storage_textures: Vec<String>,
    pub depth_targets: Vec<String>,
    pub capture_points_available: Vec<RenderCapturePointIdentity>,
}

pub fn collect_pass_resource_truth(
    flow_id: RenderFlowId,
    pass: &CompiledPassExecutionPlan,
    runtime_resources: &FlowRuntimeResources,
) -> PassResourceTruth {
    let pass_id = execution_pass_id(pass);
    let pass_label = pass_id.to_string();
    let mut render_targets = Vec::<String>::new();
    let mut sampled_textures = Vec::<String>::new();
    let mut storage_textures = Vec::<String>::new();
    let mut depth_targets = Vec::<String>::new();
    let mut render_target_seen = BTreeSet::<String>::new();
    let mut sampled_seen = BTreeSet::<String>::new();
    let mut storage_seen = BTreeSet::<String>::new();
    let mut depth_seen = BTreeSet::<String>::new();

    match pass {
        CompiledPassExecutionPlan::Compute(plan) => {
            for entry in &plan.bindings.bind_group.entries {
                match entry {
                    CompiledBindingEntry::SampledTexture { resource, .. } => {
                        push_resolved_resource_id(
                            pass_id,
                            resource,
                            "sampled_texture",
                            runtime_resources,
                            &mut sampled_seen,
                            &mut sampled_textures,
                        );
                    }
                    CompiledBindingEntry::StorageTexture { resource, .. } => {
                        push_resolved_resource_id(
                            pass_id,
                            resource,
                            "storage_texture",
                            runtime_resources,
                            &mut storage_seen,
                            &mut storage_textures,
                        );
                    }
                    _ => {}
                }
            }
        }
        CompiledPassExecutionPlan::Fullscreen(plan) | CompiledPassExecutionPlan::Graphics(plan) => {
            for target in &plan.targets.color_outputs {
                push_resolved_resource_id(
                    pass_id,
                    target,
                    "color_target",
                    runtime_resources,
                    &mut render_target_seen,
                    &mut render_targets,
                );
            }
            if let Some(depth_output) = plan.targets.depth_output.as_ref() {
                push_resolved_resource_id(
                    pass_id,
                    depth_output,
                    "depth_target",
                    runtime_resources,
                    &mut depth_seen,
                    &mut depth_targets,
                );
            }
            for entry in &plan.bindings.bind_group.entries {
                match entry {
                    CompiledBindingEntry::SampledTexture { resource, .. } => {
                        push_resolved_resource_id(
                            pass_id,
                            resource,
                            "sampled_texture",
                            runtime_resources,
                            &mut sampled_seen,
                            &mut sampled_textures,
                        );
                    }
                    CompiledBindingEntry::StorageTexture { resource, .. } => {
                        push_resolved_resource_id(
                            pass_id,
                            resource,
                            "storage_texture",
                            runtime_resources,
                            &mut storage_seen,
                            &mut storage_textures,
                        );
                    }
                    _ => {}
                }
            }
        }
        CompiledPassExecutionPlan::Copy(plan) => {
            if let Some(source) = plan.source.as_ref() {
                push_resolved_resource_id(
                    pass_id,
                    source,
                    "copy_source",
                    runtime_resources,
                    &mut render_target_seen,
                    &mut render_targets,
                );
            }
            if let Some(destination) = plan.destination.as_ref() {
                push_resolved_resource_id(
                    pass_id,
                    destination,
                    "copy_destination",
                    runtime_resources,
                    &mut render_target_seen,
                    &mut render_targets,
                );
            }
        }
        CompiledPassExecutionPlan::Present(plan) => {
            if let Some(source) = plan.source.as_ref() {
                push_resolved_resource_id(
                    pass_id,
                    source,
                    "present_source",
                    runtime_resources,
                    &mut render_target_seen,
                    &mut render_targets,
                );
            }
        }
        CompiledPassExecutionPlan::BuiltinUiComposite(_) => {}
    }

    let capture_points_available = runtime_resources
        .capture_points_for_pass(pass_id)
        .into_iter()
        .map(|point| RenderCapturePointIdentity {
            flow_id,
            pass_id,
            pass_label: pass_label.clone(),
            resource_identity: point.resource_identity,
            semantic: point.semantic,
        })
        .collect::<Vec<_>>();

    PassResourceTruth {
        render_targets,
        sampled_textures,
        storage_textures,
        depth_targets,
        capture_points_available,
    }
}

fn push_resolved_resource_id(
    pass_id: RenderPassId,
    resource: &CompiledResourceRef,
    role: &'static str,
    runtime_resources: &FlowRuntimeResources,
    seen: &mut BTreeSet<String>,
    values: &mut Vec<String>,
) {
    let value = runtime_resources
        .resolve_resource_key(pass_id, resource, role)
        .map(|key| key.to_string())
        .unwrap_or_else(|_| format!("unresolved:{resource:?}"));
    if seen.insert(value.clone()) {
        values.push(value);
    }
}
