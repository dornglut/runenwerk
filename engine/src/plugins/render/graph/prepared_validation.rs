use crate::plugins::render::features::FeatureFallbackPolicy;
use crate::plugins::render::graph::{
    CompiledBindingEntry, CompiledBuiltinImport, CompiledPassExecutionPlan, CompiledRenderFlowPlan,
    CompiledResourceRef, CompiledStorageAccess, CompiledTargetAliasRef,
    RenderBackendCapabilityProfile, RenderExecutionGraphDiagnostic,
    RenderExecutionGraphDiagnosticKind, RenderExecutionGraphPreparedError,
    validate_compiled_flow_capabilities,
};
use crate::plugins::render::{
    PreparedFlowInvocation, PreparedFlowInvocationId, PreparedRenderFrame, PreparedTargetBinding,
    PreparedViewFrame, RenderDynamicTextureTargetDescriptor, RenderDynamicTextureTargetKey,
    RenderResourceDescriptor, RenderResourceId, RenderTargetAliasKind,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RenderExecutionGraphPreparedReport {
    pub diagnostics: Vec<RenderExecutionGraphDiagnostic>,
}

impl RenderExecutionGraphPreparedReport {
    pub fn new(diagnostics: Vec<RenderExecutionGraphDiagnostic>) -> Self {
        Self { diagnostics }
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(RenderExecutionGraphDiagnostic::is_error)
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.is_error())
            .count()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AliasUseRole {
    ColorOutput,
    DepthOutput,
    SampledTexture,
    StorageTextureWrite,
    StorageBuffer,
    CopySource,
    CopyDestination,
    PresentSource,
}

#[derive(Debug, Clone)]
struct AliasRequirement {
    alias: CompiledTargetAliasRef,
    role: AliasUseRole,
}

pub fn preflight_prepared_render_frame(
    frame: &PreparedRenderFrame,
    compiled_flows: &[CompiledRenderFlowPlan],
    profile: &RenderBackendCapabilityProfile,
) -> Result<RenderExecutionGraphPreparedReport, RenderExecutionGraphPreparedError> {
    let report = validate_prepared_render_frame(frame, compiled_flows, profile);
    if report.has_errors() {
        Err(RenderExecutionGraphPreparedError::new(report.diagnostics))
    } else {
        Ok(report)
    }
}

pub fn validate_prepared_render_frame(
    frame: &PreparedRenderFrame,
    compiled_flows: &[CompiledRenderFlowPlan],
    profile: &RenderBackendCapabilityProfile,
) -> RenderExecutionGraphPreparedReport {
    let mut diagnostics = Vec::<RenderExecutionGraphDiagnostic>::new();
    let flows_by_id = compiled_flows
        .iter()
        .map(|flow| (flow.flow_id, flow))
        .collect::<BTreeMap<_, _>>();
    let views_by_id = frame
        .views
        .iter()
        .map(|view| (view.view_id.as_str(), view))
        .collect::<BTreeMap<_, _>>();
    let dynamic_targets = collect_dynamic_target_descriptors(frame, &mut diagnostics);

    diagnose_duplicate_invocation_ids(frame, &mut diagnostics);
    diagnose_dynamic_target_history_conflicts(frame, &views_by_id, &mut diagnostics);

    for flow in compiled_flows {
        diagnostics.extend(validate_compiled_flow_capabilities(flow, profile));
    }

    for invocation in &frame.flow_invocations {
        let Some(flow) = flows_by_id.get(&invocation.flow_id).copied() else {
            diagnostics.push(
                RenderExecutionGraphDiagnostic::error(
                    RenderExecutionGraphDiagnosticKind::FlowValidationIssue,
                    format!(
                        "prepared invocation '{}' references unknown compiled flow '{:?}'",
                        invocation.invocation_id.0, invocation.flow_id
                    ),
                )
                .with_invocation(invocation.invocation_id.clone())
                .with_view(invocation.view_id.clone()),
            );
            continue;
        };
        let Some(view) = views_by_id.get(invocation.view_id.as_str()).copied() else {
            diagnostics.push(
                RenderExecutionGraphDiagnostic::error(
                    RenderExecutionGraphDiagnosticKind::PreparedViewMissing,
                    format!(
                        "prepared invocation '{}' references missing view '{}'",
                        invocation.invocation_id.0, invocation.view_id
                    ),
                )
                .with_flow(flow.flow_id, flow.flow_label.clone())
                .with_invocation(invocation.invocation_id.clone())
                .with_view(invocation.view_id.clone()),
            );
            continue;
        };
        validate_invocation(
            flow,
            invocation,
            view,
            &dynamic_targets,
            frame,
            &mut diagnostics,
        );
    }

    RenderExecutionGraphPreparedReport::new(diagnostics)
}

fn collect_dynamic_target_descriptors<'a>(
    frame: &'a PreparedRenderFrame,
    diagnostics: &mut Vec<RenderExecutionGraphDiagnostic>,
) -> BTreeMap<&'a RenderDynamicTextureTargetKey, &'a RenderDynamicTextureTargetDescriptor> {
    let mut descriptors =
        BTreeMap::<&RenderDynamicTextureTargetKey, &RenderDynamicTextureTargetDescriptor>::new();
    for descriptor in &frame.dynamic_texture_targets {
        if let Err(error) = descriptor.validate() {
            diagnostics.push(
                RenderExecutionGraphDiagnostic::error(
                    RenderExecutionGraphDiagnosticKind::DynamicTargetInvalidDescriptor,
                    format!(
                        "dynamic target '{}' has invalid descriptor: {}",
                        descriptor.key, error
                    ),
                )
                .with_dynamic_target(descriptor.key.clone()),
            );
        }
        if let Some(existing) = descriptors.insert(&descriptor.key, descriptor)
            && existing.signature() != descriptor.signature()
        {
            diagnostics.push(
                RenderExecutionGraphDiagnostic::error(
                    RenderExecutionGraphDiagnosticKind::DynamicTargetInvalidDescriptor,
                    format!(
                        "dynamic target '{}' has conflicting descriptors in one prepared frame",
                        descriptor.key
                    ),
                )
                .with_dynamic_target(descriptor.key.clone()),
            );
        }
    }
    descriptors
}

fn diagnose_duplicate_invocation_ids(
    frame: &PreparedRenderFrame,
    diagnostics: &mut Vec<RenderExecutionGraphDiagnostic>,
) {
    let mut seen = BTreeSet::<&PreparedFlowInvocationId>::new();
    for invocation in &frame.flow_invocations {
        if seen.insert(&invocation.invocation_id) {
            continue;
        }
        diagnostics.push(
            RenderExecutionGraphDiagnostic::error(
                RenderExecutionGraphDiagnosticKind::PreparedInvocationDuplicate,
                format!(
                    "prepared invocation '{}' appears more than once in the prepared frame",
                    invocation.invocation_id.0
                ),
            )
            .with_invocation(invocation.invocation_id.clone())
            .with_view(invocation.view_id.clone()),
        );
    }
}

fn diagnose_dynamic_target_history_conflicts(
    frame: &PreparedRenderFrame,
    views_by_id: &BTreeMap<&str, &PreparedViewFrame>,
    diagnostics: &mut Vec<RenderExecutionGraphDiagnostic>,
) {
    let mut signatures = BTreeMap::<RenderDynamicTextureTargetKey, String>::new();
    for invocation in &frame.flow_invocations {
        let effective_signature = invocation.history_signature.as_ref().or_else(|| {
            views_by_id
                .get(invocation.view_id.as_str())
                .and_then(|view| view.history_signature.as_ref())
        });
        let Some(signature) = effective_signature else {
            continue;
        };
        for binding in invocation.target_alias_bindings.values() {
            let PreparedTargetBinding::DynamicTexture(key) = binding else {
                continue;
            };
            if let Some(existing) = signatures.get(key)
                && existing != signature
            {
                diagnostics.push(
                    RenderExecutionGraphDiagnostic::error(
                        RenderExecutionGraphDiagnosticKind::HistorySignatureConflict,
                        format!(
                            "dynamic target '{}' has incompatible history signatures '{}' and '{}'",
                            key, existing, signature
                        ),
                    )
                    .with_dynamic_target(key.clone())
                    .with_invocation(invocation.invocation_id.clone())
                    .with_view(invocation.view_id.clone())
                    .with_history_signature(signature.clone()),
                );
                continue;
            }
            signatures.insert(key.clone(), signature.clone());
        }
    }
}

fn validate_invocation(
    flow: &CompiledRenderFlowPlan,
    invocation: &PreparedFlowInvocation,
    view: &PreparedViewFrame,
    dynamic_targets: &BTreeMap<
        &RenderDynamicTextureTargetKey,
        &RenderDynamicTextureTargetDescriptor,
    >,
    frame: &PreparedRenderFrame,
    diagnostics: &mut Vec<RenderExecutionGraphDiagnostic>,
) {
    for pass in &flow.execution.passes {
        if !pass_targets_view(pass, view) {
            continue;
        }
        validate_pass_feature_gate(flow, pass, invocation, frame, diagnostics);
        validate_pass_dispatch(flow, pass, invocation, diagnostics);
        validate_pass_uniforms(flow, pass, invocation, diagnostics);
        for requirement in alias_requirements(pass) {
            validate_alias_requirement(
                flow,
                pass,
                invocation,
                &requirement,
                dynamic_targets,
                diagnostics,
            );
        }
    }
}

fn validate_pass_feature_gate(
    flow: &CompiledRenderFlowPlan,
    pass: &CompiledPassExecutionPlan,
    invocation: &PreparedFlowInvocation,
    frame: &PreparedRenderFrame,
    diagnostics: &mut Vec<RenderExecutionGraphDiagnostic>,
) {
    let Some(feature_id) = pass_feature_id(pass) else {
        return;
    };
    let Some(gate) = frame.contributions.feature_gate(&feature_id) else {
        diagnostics.push(
            RenderExecutionGraphDiagnostic::error(
                RenderExecutionGraphDiagnosticKind::FeatureGateMissing,
                format!(
                    "pass '{}' requires feature '{:?}' but the prepared frame has no feature gate",
                    pass_id(pass),
                    feature_id
                ),
            )
            .with_flow(flow.flow_id, flow.flow_label.clone())
            .with_pass(pass_id(pass), pass_id(pass).to_string())
            .with_invocation(invocation.invocation_id.clone()),
        );
        return;
    };

    if gate.fallback_policy == FeatureFallbackPolicy::FailFrame
        && !matches!(
            gate.status,
            crate::plugins::render::FeatureContributionStatus::Ready
        )
    {
        diagnostics.push(
            RenderExecutionGraphDiagnostic::error(
                RenderExecutionGraphDiagnosticKind::FeatureGateMissing,
                format!(
                    "pass '{}' feature '{:?}' is {:?} and fallback policy is fail-frame",
                    pass_id(pass),
                    feature_id,
                    gate.status
                ),
            )
            .with_flow(flow.flow_id, flow.flow_label.clone())
            .with_pass(pass_id(pass), pass_id(pass).to_string())
            .with_invocation(invocation.invocation_id.clone()),
        );
    }
}

fn validate_pass_dispatch(
    flow: &CompiledRenderFlowPlan,
    pass: &CompiledPassExecutionPlan,
    invocation: &PreparedFlowInvocation,
    diagnostics: &mut Vec<RenderExecutionGraphDiagnostic>,
) {
    let CompiledPassExecutionPlan::Compute(value) = pass else {
        return;
    };
    if value.dispatch.is_none() {
        return;
    }
    let Some(dispatch) = invocation
        .inputs
        .projected_dispatch_workgroups
        .get(&value.pass_id)
        .copied()
    else {
        diagnostics.push(
            RenderExecutionGraphDiagnostic::error(
                RenderExecutionGraphDiagnosticKind::DispatchMissing,
                format!(
                    "prepared invocation '{}' is missing dispatch workgroups for compute pass '{}'",
                    invocation.invocation_id.0, value.pass_id
                ),
            )
            .with_flow(flow.flow_id, flow.flow_label.clone())
            .with_pass(value.pass_id, value.pass_id.to_string())
            .with_invocation(invocation.invocation_id.clone())
            .with_view(invocation.view_id.clone()),
        );
        return;
    };
    if dispatch[0] == 0 || dispatch[1] == 0 || dispatch[2] == 0 {
        diagnostics.push(
            RenderExecutionGraphDiagnostic::error(
                RenderExecutionGraphDiagnosticKind::DispatchInvalid,
                format!(
                    "prepared invocation '{}' resolved invalid dispatch ({}, {}, {}) for compute pass '{}'",
                    invocation.invocation_id.0,
                    dispatch[0],
                    dispatch[1],
                    dispatch[2],
                    value.pass_id
                ),
            )
            .with_flow(flow.flow_id, flow.flow_label.clone())
            .with_pass(value.pass_id, value.pass_id.to_string())
            .with_invocation(invocation.invocation_id.clone())
            .with_view(invocation.view_id.clone()),
        );
    }
}

fn validate_pass_uniforms(
    flow: &CompiledRenderFlowPlan,
    pass: &CompiledPassExecutionPlan,
    invocation: &PreparedFlowInvocation,
    diagnostics: &mut Vec<RenderExecutionGraphDiagnostic>,
) {
    for uniform_id in pass_uniform_order(pass) {
        match invocation.inputs.projected_uniform_bytes.get(uniform_id) {
            Some(bytes) if !bytes.is_empty() => {}
            Some(_) => diagnostics.push(
                RenderExecutionGraphDiagnostic::error(
                    RenderExecutionGraphDiagnosticKind::UniformOverrideEmpty,
                    format!(
                        "prepared invocation '{}' supplies empty bytes for uniform buffer '{}'",
                        invocation.invocation_id.0, uniform_id
                    ),
                )
                .with_flow(flow.flow_id, flow.flow_label.clone())
                .with_pass(pass_id(pass), pass_id(pass).to_string())
                .with_resource(*uniform_id, flow.resource_label(*uniform_id))
                .with_invocation(invocation.invocation_id.clone()),
            ),
            None => diagnostics.push(
                RenderExecutionGraphDiagnostic::error(
                    RenderExecutionGraphDiagnosticKind::UniformMissing,
                    format!(
                        "prepared invocation '{}' is missing uniform buffer '{}' required by pass '{}'",
                        invocation.invocation_id.0,
                        uniform_id,
                        pass_id(pass)
                    ),
                )
                .with_flow(flow.flow_id, flow.flow_label.clone())
                .with_pass(pass_id(pass), pass_id(pass).to_string())
                .with_resource(*uniform_id, flow.resource_label(*uniform_id))
                .with_invocation(invocation.invocation_id.clone()),
            ),
        }
    }
}

fn validate_alias_requirement(
    flow: &CompiledRenderFlowPlan,
    pass: &CompiledPassExecutionPlan,
    invocation: &PreparedFlowInvocation,
    requirement: &AliasRequirement,
    dynamic_targets: &BTreeMap<
        &RenderDynamicTextureTargetKey,
        &RenderDynamicTextureTargetDescriptor,
    >,
    diagnostics: &mut Vec<RenderExecutionGraphDiagnostic>,
) {
    let Some(binding) = invocation
        .target_alias_bindings
        .get(requirement.alias.label.as_str())
    else {
        diagnostics.push(
            RenderExecutionGraphDiagnostic::error(
                RenderExecutionGraphDiagnosticKind::TargetAliasMissingBinding,
                format!(
                    "prepared invocation '{}' does not bind target alias '{}' required by pass '{}'",
                    invocation.invocation_id.0,
                    requirement.alias.label,
                    pass_id(pass)
                ),
            )
            .with_flow(flow.flow_id, flow.flow_label.clone())
            .with_pass(pass_id(pass), pass_id(pass).to_string())
            .with_alias(requirement.alias.label.clone(), requirement.alias.kind)
            .with_invocation(invocation.invocation_id.clone())
            .with_view(invocation.view_id.clone()),
        );
        return;
    };

    validate_alias_kind_binding(flow, pass, invocation, requirement, binding, diagnostics);
    match binding {
        PreparedTargetBinding::DynamicTexture(key) => {
            let Some(descriptor) = dynamic_targets.get(key) else {
                diagnostics.push(
                    RenderExecutionGraphDiagnostic::error(
                        RenderExecutionGraphDiagnosticKind::DynamicTargetMissingDescriptor,
                        format!(
                            "prepared invocation '{}' binds alias '{}' to dynamic target '{}' but no descriptor was requested this frame",
                            invocation.invocation_id.0,
                            requirement.alias.label,
                            key
                        ),
                    )
                    .with_flow(flow.flow_id, flow.flow_label.clone())
                    .with_pass(pass_id(pass), pass_id(pass).to_string())
                    .with_alias(requirement.alias.label.clone(), requirement.alias.kind)
                    .with_dynamic_target(key.clone())
                    .with_invocation(invocation.invocation_id.clone())
                    .with_view(invocation.view_id.clone()),
                );
                return;
            };
            validate_dynamic_target_usage(
                flow,
                pass,
                invocation,
                requirement,
                descriptor,
                diagnostics,
            );
        }
        PreparedTargetBinding::FlowOwned(resource_id) => validate_flow_owned_alias_binding(
            flow,
            pass,
            invocation,
            requirement,
            *resource_id,
            diagnostics,
        ),
        PreparedTargetBinding::SurfaceDepth => diagnostics.push(
            RenderExecutionGraphDiagnostic::error(
                RenderExecutionGraphDiagnosticKind::UnsupportedImportedResource,
                format!(
                    "prepared invocation '{}' binds alias '{}' to surface depth, but surface-depth imports are not supported by active runtime execution",
                    invocation.invocation_id.0,
                    requirement.alias.label
                ),
            )
            .with_flow(flow.flow_id, flow.flow_label.clone())
            .with_pass(pass_id(pass), pass_id(pass).to_string())
            .with_alias(requirement.alias.label.clone(), requirement.alias.kind)
            .with_invocation(invocation.invocation_id.clone())
            .with_view(invocation.view_id.clone()),
        ),
        PreparedTargetBinding::SurfaceColor => {}
    }
}

fn validate_alias_kind_binding(
    flow: &CompiledRenderFlowPlan,
    pass: &CompiledPassExecutionPlan,
    invocation: &PreparedFlowInvocation,
    requirement: &AliasRequirement,
    binding: &PreparedTargetBinding,
    diagnostics: &mut Vec<RenderExecutionGraphDiagnostic>,
) {
    let compatible = match (requirement.alias.kind, requirement.role, binding) {
        (
            RenderTargetAliasKind::Color,
            AliasUseRole::ColorOutput
            | AliasUseRole::SampledTexture
            | AliasUseRole::CopySource
            | AliasUseRole::CopyDestination
            | AliasUseRole::PresentSource,
            PreparedTargetBinding::SurfaceColor
            | PreparedTargetBinding::DynamicTexture(_)
            | PreparedTargetBinding::FlowOwned(_),
        ) => true,
        (
            RenderTargetAliasKind::Depth,
            AliasUseRole::DepthOutput | AliasUseRole::SampledTexture,
            PreparedTargetBinding::SurfaceDepth
            | PreparedTargetBinding::DynamicTexture(_)
            | PreparedTargetBinding::FlowOwned(_),
        ) => true,
        (
            RenderTargetAliasKind::Texture,
            AliasUseRole::SampledTexture
            | AliasUseRole::StorageTextureWrite
            | AliasUseRole::CopySource
            | AliasUseRole::CopyDestination
            | AliasUseRole::PresentSource,
            PreparedTargetBinding::DynamicTexture(_)
            | PreparedTargetBinding::FlowOwned(_)
            | PreparedTargetBinding::SurfaceColor,
        ) => true,
        _ => false,
    };
    if compatible {
        return;
    }
    diagnostics.push(
        RenderExecutionGraphDiagnostic::error(
            RenderExecutionGraphDiagnosticKind::TargetAliasKindMismatch,
            format!(
                "prepared invocation '{}' binds alias '{}' with incompatible role {:?} and binding {:?}",
                invocation.invocation_id.0,
                requirement.alias.label,
                requirement.role,
                binding
            ),
        )
        .with_flow(flow.flow_id, flow.flow_label.clone())
        .with_pass(pass_id(pass), pass_id(pass).to_string())
        .with_alias(requirement.alias.label.clone(), requirement.alias.kind)
        .with_invocation(invocation.invocation_id.clone())
        .with_view(invocation.view_id.clone()),
    );
}

fn validate_dynamic_target_usage(
    flow: &CompiledRenderFlowPlan,
    pass: &CompiledPassExecutionPlan,
    invocation: &PreparedFlowInvocation,
    requirement: &AliasRequirement,
    descriptor: &RenderDynamicTextureTargetDescriptor,
    diagnostics: &mut Vec<RenderExecutionGraphDiagnostic>,
) {
    let usage_ok = match requirement.role {
        AliasUseRole::ColorOutput => {
            descriptor.usage.color_attachment && !descriptor.format.is_depth()
        }
        AliasUseRole::DepthOutput => {
            descriptor.usage.depth_attachment && descriptor.format.is_depth()
        }
        AliasUseRole::SampledTexture => {
            descriptor.usage.sampled && descriptor.sample_mode.is_sampled()
        }
        AliasUseRole::PresentSource => descriptor.usage.copy_src,
        AliasUseRole::StorageTextureWrite => descriptor.usage.storage,
        AliasUseRole::CopySource => descriptor.usage.copy_src,
        AliasUseRole::CopyDestination => descriptor.usage.copy_dst,
        AliasUseRole::StorageBuffer => false,
    };
    if usage_ok {
        return;
    }
    diagnostics.push(
        RenderExecutionGraphDiagnostic::error(
            RenderExecutionGraphDiagnosticKind::DynamicTargetUsageMismatch,
            format!(
                "dynamic target '{}' is incompatible with alias '{}' role {:?} in pass '{}'",
                descriptor.key,
                requirement.alias.label,
                requirement.role,
                pass_id(pass)
            ),
        )
        .with_flow(flow.flow_id, flow.flow_label.clone())
        .with_pass(pass_id(pass), pass_id(pass).to_string())
        .with_alias(requirement.alias.label.clone(), requirement.alias.kind)
        .with_dynamic_target(descriptor.key.clone())
        .with_invocation(invocation.invocation_id.clone())
        .with_view(invocation.view_id.clone()),
    );
}

fn validate_flow_owned_alias_binding(
    flow: &CompiledRenderFlowPlan,
    pass: &CompiledPassExecutionPlan,
    invocation: &PreparedFlowInvocation,
    requirement: &AliasRequirement,
    resource_id: RenderResourceId,
    diagnostics: &mut Vec<RenderExecutionGraphDiagnostic>,
) {
    let Some(descriptor) = flow.resource_descriptor(resource_id) else {
        diagnostics.push(
            RenderExecutionGraphDiagnostic::error(
                RenderExecutionGraphDiagnosticKind::InvalidResource,
                format!(
                    "prepared invocation '{}' binds alias '{}' to unknown flow-owned resource '{}'",
                    invocation.invocation_id.0, requirement.alias.label, resource_id
                ),
            )
            .with_flow(flow.flow_id, flow.flow_label.clone())
            .with_pass(pass_id(pass), pass_id(pass).to_string())
            .with_resource(resource_id, Option::<String>::None)
            .with_alias(requirement.alias.label.clone(), requirement.alias.kind)
            .with_invocation(invocation.invocation_id.clone())
            .with_view(invocation.view_id.clone()),
        );
        return;
    };

    let compatible = match requirement.role {
        AliasUseRole::ColorOutput => matches!(
            descriptor,
            RenderResourceDescriptor::ColorTarget(_) | RenderResourceDescriptor::ImportedTexture(_)
        ),
        AliasUseRole::DepthOutput => matches!(descriptor, RenderResourceDescriptor::DepthTarget(_)),
        AliasUseRole::SampledTexture | AliasUseRole::PresentSource => {
            is_texture_descriptor(descriptor)
        }
        AliasUseRole::StorageTextureWrite => {
            matches!(descriptor, RenderResourceDescriptor::StorageTexture(_))
        }
        AliasUseRole::CopySource | AliasUseRole::CopyDestination => true,
        AliasUseRole::StorageBuffer => is_buffer_descriptor(descriptor),
    };
    if compatible {
        return;
    }
    diagnostics.push(
        RenderExecutionGraphDiagnostic::error(
            RenderExecutionGraphDiagnosticKind::TargetAliasKindMismatch,
            format!(
                "prepared invocation '{}' binds alias '{}' to resource '{}' with incompatible role {:?}",
                invocation.invocation_id.0,
                requirement.alias.label,
                resource_id,
                requirement.role
            ),
        )
        .with_flow(flow.flow_id, flow.flow_label.clone())
        .with_pass(pass_id(pass), pass_id(pass).to_string())
        .with_resource(resource_id, flow.resource_label(resource_id))
        .with_alias(requirement.alias.label.clone(), requirement.alias.kind)
        .with_invocation(invocation.invocation_id.clone())
        .with_view(invocation.view_id.clone()),
    );
}

fn alias_requirements(pass: &CompiledPassExecutionPlan) -> Vec<AliasRequirement> {
    let mut requirements = Vec::<AliasRequirement>::new();
    match pass {
        CompiledPassExecutionPlan::Compute(value) => {
            collect_binding_alias_requirements(
                &value.bindings.bind_group.entries,
                &mut requirements,
            );
        }
        CompiledPassExecutionPlan::Fullscreen(value)
        | CompiledPassExecutionPlan::Graphics(value) => {
            collect_binding_alias_requirements(
                &value.bindings.bind_group.entries,
                &mut requirements,
            );
            for resource in &value.targets.color_outputs {
                collect_alias_requirement(resource, AliasUseRole::ColorOutput, &mut requirements);
            }
            if let Some(resource) = &value.targets.depth_output {
                collect_alias_requirement(resource, AliasUseRole::DepthOutput, &mut requirements);
            }
            for resource in &value.targets.reads {
                collect_alias_requirement(
                    resource,
                    AliasUseRole::SampledTexture,
                    &mut requirements,
                );
            }
        }
        CompiledPassExecutionPlan::Copy(value) => {
            if let Some(resource) = &value.source {
                collect_alias_requirement(resource, AliasUseRole::CopySource, &mut requirements);
            }
            if let Some(resource) = &value.destination {
                collect_alias_requirement(
                    resource,
                    AliasUseRole::CopyDestination,
                    &mut requirements,
                );
            }
        }
        CompiledPassExecutionPlan::Present(value) => {
            if let Some(resource) = &value.source {
                collect_alias_requirement(resource, AliasUseRole::PresentSource, &mut requirements);
            }
        }
        CompiledPassExecutionPlan::BuiltinUiComposite(_) => {}
    }
    requirements
}

fn collect_binding_alias_requirements(
    entries: &[CompiledBindingEntry],
    requirements: &mut Vec<AliasRequirement>,
) {
    for entry in entries {
        match entry {
            CompiledBindingEntry::SampledTexture { resource } => {
                collect_alias_requirement(resource, AliasUseRole::SampledTexture, requirements);
            }
            CompiledBindingEntry::StorageTexture { resource, .. } => {
                collect_alias_requirement(
                    resource,
                    AliasUseRole::StorageTextureWrite,
                    requirements,
                );
            }
            CompiledBindingEntry::StorageBuffer { resource, access } => {
                if matches!(
                    access,
                    CompiledStorageAccess::ReadOnly
                        | CompiledStorageAccess::ReadWrite
                        | CompiledStorageAccess::WriteOnly
                ) {
                    collect_alias_requirement(resource, AliasUseRole::StorageBuffer, requirements);
                }
            }
            CompiledBindingEntry::Sampler | CompiledBindingEntry::UniformBuffer { .. } => {}
        }
    }
}

fn collect_alias_requirement(
    resource: &CompiledResourceRef,
    role: AliasUseRole,
    requirements: &mut Vec<AliasRequirement>,
) {
    if let CompiledResourceRef::TargetAlias(alias) = resource {
        requirements.push(AliasRequirement {
            alias: alias.clone(),
            role,
        });
    }
}

fn pass_targets_view(pass: &CompiledPassExecutionPlan, view: &PreparedViewFrame) -> bool {
    match pass {
        CompiledPassExecutionPlan::Compute(value) => {
            value.view_mask.includes(view.view_id.as_str(), view.kind)
        }
        CompiledPassExecutionPlan::Fullscreen(value)
        | CompiledPassExecutionPlan::Graphics(value) => {
            value.view_mask.includes(view.view_id.as_str(), view.kind)
        }
        CompiledPassExecutionPlan::Copy(value) => {
            value.view_mask.includes(view.view_id.as_str(), view.kind)
        }
        CompiledPassExecutionPlan::Present(value) => {
            value.view_mask.includes(view.view_id.as_str(), view.kind)
        }
        CompiledPassExecutionPlan::BuiltinUiComposite(value) => {
            value.view_mask.includes(view.view_id.as_str(), view.kind)
        }
    }
}

fn pass_id(pass: &CompiledPassExecutionPlan) -> crate::plugins::render::RenderPassId {
    match pass {
        CompiledPassExecutionPlan::Compute(value) => value.pass_id,
        CompiledPassExecutionPlan::Fullscreen(value)
        | CompiledPassExecutionPlan::Graphics(value) => value.pass_id,
        CompiledPassExecutionPlan::Copy(value) => value.pass_id,
        CompiledPassExecutionPlan::Present(value) => value.pass_id,
        CompiledPassExecutionPlan::BuiltinUiComposite(value) => value.pass_id,
    }
}

fn pass_feature_id(
    pass: &CompiledPassExecutionPlan,
) -> Option<crate::plugins::render::RenderFeatureId> {
    match pass {
        CompiledPassExecutionPlan::Compute(value) => value.feature_id,
        CompiledPassExecutionPlan::Fullscreen(value)
        | CompiledPassExecutionPlan::Graphics(value) => value.feature_id,
        CompiledPassExecutionPlan::Copy(value) => value.feature_id,
        CompiledPassExecutionPlan::Present(value) => value.feature_id,
        CompiledPassExecutionPlan::BuiltinUiComposite(value) => Some(value.feature_id),
    }
}

fn pass_uniform_order(pass: &CompiledPassExecutionPlan) -> &[RenderResourceId] {
    match pass {
        CompiledPassExecutionPlan::Compute(value) => &value.bindings.uniform_order,
        CompiledPassExecutionPlan::Fullscreen(value)
        | CompiledPassExecutionPlan::Graphics(value) => &value.bindings.uniform_order,
        CompiledPassExecutionPlan::Copy(_)
        | CompiledPassExecutionPlan::Present(_)
        | CompiledPassExecutionPlan::BuiltinUiComposite(_) => &[],
    }
}

fn is_texture_descriptor(descriptor: &RenderResourceDescriptor) -> bool {
    matches!(
        descriptor,
        RenderResourceDescriptor::SampledTexture(_)
            | RenderResourceDescriptor::StorageTexture(_)
            | RenderResourceDescriptor::ColorTarget(_)
            | RenderResourceDescriptor::DepthTarget(_)
            | RenderResourceDescriptor::HistoryTexture(_)
            | RenderResourceDescriptor::TargetAlias(_)
            | RenderResourceDescriptor::ImportedTexture(_)
    )
}

fn is_buffer_descriptor(descriptor: &RenderResourceDescriptor) -> bool {
    matches!(
        descriptor,
        RenderResourceDescriptor::UniformBuffer(_)
            | RenderResourceDescriptor::StorageBuffer(_)
            | RenderResourceDescriptor::ImportedBuffer(_)
    )
}

#[allow(dead_code)]
fn imported_builtin_label(value: CompiledBuiltinImport) -> &'static str {
    match value {
        CompiledBuiltinImport::SurfaceColor => "surface.color",
        CompiledBuiltinImport::SurfaceDepth => "surface.depth",
    }
}
