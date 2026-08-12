use super::G4C3_WGPU_PIPELINE_COMPATIBILITY_REVISION;
use super::records::RenderStageIoEvidence;
use crate::plugins::gpu::{
    GpuAlignmentFacts, GpuAlignmentKind, GpuCapabilityFeature, GpuContext, GpuContextAffinity,
    GpuFormatRole, GpuLimits, GpuPipelineLayoutDescriptor, GpuPipelineRealizationError,
    GpuPipelineRealizationErrorCategory, GpuProgramBindingRealizationError,
    GpuProgramBindingRealizationErrorCategory, GpuRealizedProgram, GpuRenderPipelineDescriptor,
    GpuShaderStage, GpuSpecializationValue, GpuTextureFormat, compare_fragment_output_signatures,
    compare_vertex_input_signatures,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct RenderPipelineRequestKey {
    affinity: GpuContextAffinity,
    descriptor: GpuRenderPipelineDescriptor,
    stage_io: RenderStageIoEvidence,
    admitted_format_roles: Vec<(GpuTextureFormat, GpuFormatRole)>,
    enabled_features: Vec<GpuCapabilityFeature>,
    device_limits: GpuLimits,
    device_alignments: GpuAlignmentFacts,
    workload_limits: GpuLimits,
    workload_alignment_maximums: Vec<(GpuAlignmentKind, u64)>,
    wgpu_pipeline_compatibility_revision: u32,
}

impl RenderPipelineRequestKey {
    pub(super) fn new(
        context: &GpuContext,
        descriptor: &GpuRenderPipelineDescriptor,
        stage_io: RenderStageIoEvidence,
    ) -> Self {
        let device = context.device_facts();
        Self {
            affinity: context.affinity(),
            descriptor: descriptor.clone(),
            stage_io,
            admitted_format_roles: relevant_format_roles(descriptor),
            enabled_features: device.enabled_features().collect(),
            device_limits: device.device_limits().values(),
            device_alignments: device.device_limits().alignments(),
            workload_limits: device.workload_budget().limits(),
            workload_alignment_maximums: device.workload_budget().alignment_maximums().collect(),
            wgpu_pipeline_compatibility_revision: G4C3_WGPU_PIPELINE_COMPATIBILITY_REVISION,
        }
    }
}

pub(super) fn validate_render_descriptor(
    descriptor: &GpuRenderPipelineDescriptor,
    request: String,
) -> Result<(), GpuPipelineRealizationError> {
    if descriptor
        .program()
        .entry_point(GpuShaderStage::Vertex, descriptor.entry_points().vertex())
        .is_none()
    {
        return Err(GpuPipelineRealizationError::new(
            GpuPipelineRealizationErrorCategory::EntryPointStageMismatch,
            request,
            "the vertex entry point is absent from the descriptor-owned program",
        ));
    }
    if let Some(fragment) = descriptor.entry_points().fragment()
        && descriptor
            .program()
            .entry_point(GpuShaderStage::Fragment, fragment)
            .is_none()
    {
        return Err(GpuPipelineRealizationError::new(
            GpuPipelineRealizationErrorCategory::EntryPointStageMismatch,
            render_request_name(descriptor),
            "the fragment entry point is absent from the descriptor-owned program",
        ));
    }
    let expected_layout = GpuPipelineLayoutDescriptor::from_interface(
        descriptor.program().interface(),
    )
    .map_err(|error| {
        GpuPipelineRealizationError::new(
            GpuPipelineRealizationErrorCategory::PipelineDescriptorInvalid,
            render_request_name(descriptor),
            error.to_string(),
        )
    })?;
    if descriptor.layout() != &expected_layout {
        return Err(GpuPipelineRealizationError::new(
            GpuPipelineRealizationErrorCategory::ProgramInterfaceMismatch,
            render_request_name(descriptor),
            "the render pipeline layout does not match the program's explicit resource interface",
        ));
    }
    descriptor
        .specialization()
        .validate_override_support(true)
        .map_err(|error| {
            GpuPipelineRealizationError::new(
                GpuPipelineRealizationErrorCategory::PipelineDescriptorInvalid,
                render_request_name(descriptor),
                error.to_string(),
            )
        })
}

pub(super) fn validate_stage_io(
    descriptor: &GpuRenderPipelineDescriptor,
    program: &GpuRealizedProgram,
) -> Result<RenderStageIoEvidence, GpuPipelineRealizationError> {
    let expected_vertex = descriptor
        .expected_vertex_input_signature()
        .map_err(|error| stage_descriptor_error(descriptor, error.to_string()))?;
    let observed_vertex = program
        .record
        .vertex_inputs()
        .iter()
        .find(|signature| signature.entry_point() == descriptor.entry_points().vertex())
        .cloned()
        .ok_or_else(|| {
            GpuPipelineRealizationError::new(
                GpuPipelineRealizationErrorCategory::ProgramInterfaceMismatch,
                render_request_name(descriptor),
                "the realized program has no observed vertex-input signature for the selected entry point",
            )
        })?;
    compare_vertex_input_signatures(&expected_vertex, &observed_vertex).map_err(|error| {
        GpuPipelineRealizationError::new(
            GpuPipelineRealizationErrorCategory::PipelineStageIoMismatch,
            render_request_name(descriptor),
            error.to_string(),
        )
    })?;

    let expected_fragment = descriptor
        .expected_fragment_output_signature()
        .map_err(|error| stage_descriptor_error(descriptor, error.to_string()))?;
    let observed_fragment = match descriptor.entry_points().fragment() {
        Some(fragment) => Some(
            program
                .record
                .fragment_outputs()
                .iter()
                .find(|signature| signature.entry_point() == fragment)
                .cloned()
                .ok_or_else(|| {
                    GpuPipelineRealizationError::new(
                        GpuPipelineRealizationErrorCategory::ProgramInterfaceMismatch,
                        render_request_name(descriptor),
                        "the realized program has no observed fragment-output signature for the selected entry point",
                    )
                })?,
        ),
        None => None,
    };
    match (&expected_fragment, &observed_fragment) {
        (Some(expected), Some(observed)) => {
            compare_fragment_output_signatures(expected, observed).map_err(|error| {
                GpuPipelineRealizationError::new(
                    GpuPipelineRealizationErrorCategory::PipelineStageIoMismatch,
                    render_request_name(descriptor),
                    error.to_string(),
                )
            })?;
        }
        (None, None) => {}
        _ => {
            return Err(GpuPipelineRealizationError::new(
                GpuPipelineRealizationErrorCategory::ProgramInterfaceMismatch,
                render_request_name(descriptor),
                "fragment-stage presence disagrees between the accepted descriptor and observed program evidence",
            ));
        }
    }

    Ok(RenderStageIoEvidence {
        expected_vertex,
        observed_vertex,
        expected_fragment,
        observed_fragment,
    })
}

pub(super) fn validate_admitted_format_roles(
    context: &GpuContext,
    descriptor: &GpuRenderPipelineDescriptor,
    request: &str,
) -> Result<(), GpuPipelineRealizationError> {
    let admitted = context
        .device_facts()
        .admission_contract()
        .format_roles()
        .collect::<std::collections::BTreeSet<_>>();
    for role in relevant_format_roles(descriptor) {
        if !admitted.contains(&role) {
            return Err(GpuPipelineRealizationError::new(
                GpuPipelineRealizationErrorCategory::FormatOrAlignmentNotAdmitted,
                request,
                format!(
                    "render attachment format role was not admitted: {:?}::{:?}",
                    role.0, role.1
                ),
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_dependency_affinity(
    expected: GpuContextAffinity,
    observed: GpuContextAffinity,
    request: String,
) -> Result<(), GpuPipelineRealizationError> {
    if observed.context() != expected.context() {
        return Err(GpuPipelineRealizationError::affinity(
            GpuPipelineRealizationErrorCategory::ForeignContext,
            request,
            expected,
            observed,
        ));
    }
    if observed.generation() != expected.generation() {
        return Err(GpuPipelineRealizationError::affinity(
            GpuPipelineRealizationErrorCategory::StaleDeviceGeneration,
            request,
            expected,
            observed,
        ));
    }
    Ok(())
}

pub(super) fn map_dependency_error(
    request: String,
    error: GpuProgramBindingRealizationError,
) -> GpuPipelineRealizationError {
    let category = match error.category() {
        GpuProgramBindingRealizationErrorCategory::ForeignContext => {
            GpuPipelineRealizationErrorCategory::ForeignContext
        }
        GpuProgramBindingRealizationErrorCategory::StaleDeviceGeneration => {
            GpuPipelineRealizationErrorCategory::StaleDeviceGeneration
        }
        GpuProgramBindingRealizationErrorCategory::BackendResourceExhaustion => {
            GpuPipelineRealizationErrorCategory::BackendResourceExhaustion
        }
        GpuProgramBindingRealizationErrorCategory::ContextOrDeviceUnavailableOrLost => {
            GpuPipelineRealizationErrorCategory::ContextOrDeviceUnavailableOrLost
        }
        _ => GpuPipelineRealizationErrorCategory::UnknownProgramOrLayoutRealization,
    };
    GpuPipelineRealizationError::new(category, request, error.to_string())
}

pub(super) fn render_request_name(descriptor: &GpuRenderPipelineDescriptor) -> String {
    match descriptor.entry_points().fragment() {
        Some(fragment) => format!(
            "render pipeline vertex={} fragment={fragment}",
            descriptor.entry_points().vertex()
        ),
        None => format!(
            "render pipeline vertex={} fragment=<none>",
            descriptor.entry_points().vertex()
        ),
    }
}

pub(super) fn wgpu_specialization_constants(
    descriptor: &GpuRenderPipelineDescriptor,
) -> Vec<(&str, f64)> {
    descriptor
        .specialization()
        .entries()
        .map(|entry| {
            let value = match entry.value() {
                GpuSpecializationValue::Bool(value) => {
                    if value {
                        1.0
                    } else {
                        0.0
                    }
                }
                GpuSpecializationValue::U32(value) => f64::from(value),
                GpuSpecializationValue::I32(value) => f64::from(value),
                GpuSpecializationValue::F32(value) => f64::from(value.get()),
            };
            (entry.key().as_str(), value)
        })
        .collect()
}

fn relevant_format_roles(
    descriptor: &GpuRenderPipelineDescriptor,
) -> Vec<(GpuTextureFormat, GpuFormatRole)> {
    let mut roles = descriptor
        .state()
        .fragment_output()
        .into_iter()
        .flat_map(|output| {
            output
                .color_targets()
                .map(|target| (target.format(), GpuFormatRole::ColorAttachment))
        })
        .collect::<Vec<_>>();
    if let Some(depth) = descriptor.state().depth_stencil() {
        roles.push((depth.format(), GpuFormatRole::DepthStencil));
    }
    roles.sort_unstable();
    roles.dedup();
    roles
}

fn stage_descriptor_error(
    descriptor: &GpuRenderPipelineDescriptor,
    detail: String,
) -> GpuPipelineRealizationError {
    GpuPipelineRealizationError::new(
        GpuPipelineRealizationErrorCategory::PipelineDescriptorInvalid,
        render_request_name(descriptor),
        detail,
    )
}
