//! Owner-controlled ordinary execution for the maintained deterministic RunenRender evaluator.
//!
//! This module lowers one exact [`AdmittedDeterministicRender`] into backend-neutral public
//! RunenGPU work and submits every admitted output through one exact `GpuSubmission`. It does not
//! re-plan renderer semantics, expose caller-authored GPU fragments, or make physical encoding part
//! of renderer-semantic result identity.
//!
//! The maintained carrier is one 32-bit word per semantic sample. Radiance and forward depth use
//! `f32` bits. Object identity uses an execution-local non-zero `u32` code decoded only through the
//! returned [`RenderObjectIdentityDecoder`]. Renderer-private definedness and evaluator-status words
//! remain distinct from payload bits. For this maintained direct/no-environment method only, a
//! primary radiance miss is the defined value zero; generic R2 radiance-miss semantics remain wider.

use super::admission::{AdmittedRenderPlan, RenderOutputDestination};
use super::derived_transform::{RenderCompiledObjectTransform, RenderCompiledObjectTransformError};
use super::deterministic_admission::AdmittedDeterministicRender;
use super::lowering::RenderWorkSet;
use super::representation::RenderRepresentationId;
use super::request::{RenderDistanceConvention, RenderObservationSpec, RenderOutputValue};
use super::scene::RenderObjectId;
use super::surface_input::RenderSurfaceSemanticInputView;
use runen_gpu::{
    GpuBufferDescriptor, GpuBufferInitialization, GpuBufferRegion, GpuBufferTextureLayout,
    GpuBufferUsage, GpuClearOperation, GpuComputeOperation, GpuComputePipelineDescriptor,
    GpuContext, GpuContextAffinity, GpuCopyOperation, GpuDispatchIntent, GpuDispatchSize,
    GpuReadbackId, GpuReadbackOperation, GpuReconstruction, GpuResourceLifetime, GpuResourceScope,
    GpuRuntimeBindingValue, GpuSubmission, GpuTextureCopyRegion, GpuUploadOperation,
    GpuWorkFragment, GpuWorkSubmissionError, PreparedGpuData, TransferData,
    admit_static_wgsl_sources,
};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

const WORD_BYTES: u64 = 4;
const HEADER_WORDS: usize = 24;
const GEOMETRY_WORDS: usize = 32;
const EMITTER_WORDS: usize = 4;
const WORKGROUP_SIZE: u32 = 64;
const OUTPUT_RADIANCE: u32 = 1;
const OUTPUT_FORWARD_DEPTH: u32 = 2;
const OUTPUT_OBJECT_IDENTITY: u32 = 3;
const OBSERVATION_PERSPECTIVE: u32 = 1;
const OBSERVATION_PROBE: u32 = 2;
const SHAPE_SPHERE: u32 = 1;
const SHAPE_PLANE: u32 = 2;
const MAINTAINED_WGSL: &str = include_str!("deterministic_execution.wgsl");

/// Physical object-identity decoder owned by one exact maintained execution.
///
/// Compact GPU words are not renderer-semantic object identities. Code zero and every unknown code
/// decode to `None`; a non-zero code is meaningful only through this exact immutable codebook.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderObjectIdentityDecoder {
    objects_by_code: Vec<RenderObjectId>,
}

impl RenderObjectIdentityDecoder {
    pub fn decode(&self, code: u32) -> Option<RenderObjectId> {
        let index = usize::try_from(code.checked_sub(1)?).ok()?;
        self.objects_by_code.get(index).copied()
    }
}

/// Renderer-private correlation between one admitted output and the three observations required by
/// RR566-EVAL-001. The IDs are process-local RunenGPU correlation values bound to one exact
/// `GpuSubmission`; they are not semantic identity and are never exposed as public renderer state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DeterministicVerificationReadbacks {
    output_index: usize,
    canonical_output: GpuReadbackId,
    definedness: GpuReadbackId,
    status: GpuReadbackId,
}

impl DeterministicVerificationReadbacks {
    pub(super) const fn output_index(self) -> usize {
        self.output_index
    }

    pub(super) const fn canonical_output(self) -> GpuReadbackId {
        self.canonical_output
    }

    pub(super) const fn definedness(self) -> GpuReadbackId {
        self.definedness
    }

    pub(super) const fn status(self) -> GpuReadbackId {
        self.status
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeterministicObservationIntent {
    Ordinary,
    Verify,
}

impl DeterministicObservationIntent {
    const fn requires_private_readback(self) -> bool {
        matches!(self, Self::Verify)
    }
}

/// One exact ordinary maintained execution after RunenGPU accepted its authored work.
///
/// This is not a second submission lifecycle or a render session. Physical completion/failure stays
/// entirely on [`GpuSubmission`]. The retained admitted plan and decoder only preserve renderer
/// correlation that must remain bound to that exact submission. Ordinary execution contains no
/// verification-readback state and therefore cannot be upgraded after submission.
#[derive(Debug, Clone)]
pub struct SubmittedDeterministicRender {
    admitted: AdmittedDeterministicRender,
    submission: GpuSubmission,
    object_identity_decoder: RenderObjectIdentityDecoder,
}

impl SubmittedDeterministicRender {
    pub const fn admitted(&self) -> &AdmittedDeterministicRender {
        &self.admitted
    }

    pub const fn submission(&self) -> &GpuSubmission {
        &self.submission
    }

    pub const fn object_identity_decoder(&self) -> &RenderObjectIdentityDecoder {
        &self.object_identity_decoder
    }
}

/// Private proof witness for a submission whose verification intent was selected before lowering.
///
/// This wraps the ordinary submitted-execution value rather than introducing a second submission
/// lifecycle. Its only additional state is the private correlation required to locate the three
/// observations that were authored into that same exact `GpuSubmission`.
#[derive(Debug, Clone)]
pub(super) struct DeterministicVerificationSubmission {
    submitted: SubmittedDeterministicRender,
    readbacks: Vec<DeterministicVerificationReadbacks>,
}

impl DeterministicVerificationSubmission {
    pub(super) const fn submitted(&self) -> &SubmittedDeterministicRender {
        &self.submitted
    }

    pub(super) fn readbacks(&self) -> &[DeterministicVerificationReadbacks] {
        &self.readbacks
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderDeterministicLoweringError {
    ContextAffinityChanged {
        admitted: GpuContextAffinity,
        actual: GpuContextAffinity,
    },
    OutputCorrelationChanged {
        output_index: usize,
    },
    MissingObjectState {
        output_index: usize,
        object_id: RenderObjectId,
    },
    MissingSurfaceInput {
        output_index: usize,
        object_id: RenderObjectId,
        representation_id: RenderRepresentationId,
    },
    NonInvertibleObjectTransform {
        output_index: usize,
        object_id: RenderObjectId,
    },
    MissingMaterial {
        output_index: usize,
        object_id: RenderObjectId,
    },
    UnsupportedOutput {
        output_index: usize,
    },
    MissingBytesPerRowAlignment,
    InvalidBytesPerRowAlignment {
        alignment: u64,
    },
    SizeOverflow {
        field: &'static str,
    },
    HostAllocation {
        field: &'static str,
    },
    NumericRealization {
        field: &'static str,
    },
    RunenGpuAuthoring {
        stage: &'static str,
        detail: String,
    },
}

impl fmt::Display for RenderDeterministicLoweringError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContextAffinityChanged { .. } => formatter.write_str(
                "maintained deterministic execution context differs from admitted RunenGPU context",
            ),
            Self::OutputCorrelationChanged { output_index } => write!(
                formatter,
                "maintained deterministic output correlation changed at output {output_index}"
            ),
            Self::MissingObjectState {
                output_index,
                object_id,
            } => write!(
                formatter,
                "output {output_index} object {object_id:?} has no retained spatial state"
            ),
            Self::MissingSurfaceInput {
                output_index,
                object_id,
                representation_id,
            } => write!(
                formatter,
                "output {output_index} object {object_id:?} representation {representation_id:?} has no maintained surface input"
            ),
            Self::NonInvertibleObjectTransform {
                output_index,
                object_id,
            } => write!(
                formatter,
                "output {output_index} object {object_id:?} has no finite invertible evaluator transform"
            ),
            Self::MissingMaterial {
                output_index,
                object_id,
            } => write!(
                formatter,
                "radiance output {output_index} object {object_id:?} has no admitted material"
            ),
            Self::UnsupportedOutput { output_index } => write!(
                formatter,
                "output {output_index} is outside the maintained deterministic evaluator contract"
            ),
            Self::MissingBytesPerRowAlignment => {
                formatter.write_str("RunenGPU did not expose a texture bytes-per-row alignment")
            }
            Self::InvalidBytesPerRowAlignment { alignment } => write!(
                formatter,
                "RunenGPU exposed unusable texture bytes-per-row alignment {alignment}"
            ),
            Self::SizeOverflow { field } => {
                write!(
                    formatter,
                    "{field} exceeds maintained physical indexing limits"
                )
            }
            Self::HostAllocation { field } => {
                write!(formatter, "host allocation failed for {field}")
            }
            Self::NumericRealization { field } => write!(
                formatter,
                "{field} cannot be represented by the maintained finite f32 evaluator"
            ),
            Self::RunenGpuAuthoring { stage, detail } => {
                write!(formatter, "RunenGPU {stage} authoring failed: {detail}")
            }
        }
    }
}

impl Error for RenderDeterministicLoweringError {}

#[derive(Debug)]
pub enum RenderDeterministicExecutionError {
    Lowering(RenderDeterministicLoweringError),
    Submission(GpuWorkSubmissionError),
}

impl fmt::Display for RenderDeterministicExecutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lowering(error) => write!(formatter, "deterministic lowering failed: {error}"),
            Self::Submission(error) => write!(formatter, "RunenGPU submission failed: {error}"),
        }
    }
}

impl Error for RenderDeterministicExecutionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Lowering(error) => Some(error),
            Self::Submission(error) => Some(error),
        }
    }
}

impl From<RenderDeterministicLoweringError> for RenderDeterministicExecutionError {
    fn from(value: RenderDeterministicLoweringError) -> Self {
        Self::Lowering(value)
    }
}

struct LoweredDeterministicRender {
    work_set: RenderWorkSet,
    object_identity_decoder: RenderObjectIdentityDecoder,
    verification_readbacks: Vec<DeterministicVerificationReadbacks>,
}

struct LoweredDeterministicOutput {
    fragment: GpuWorkFragment,
    verification_readbacks: Option<DeterministicVerificationReadbacks>,
}

struct PackedOutput {
    input_words: Vec<u32>,
    sample_count: u32,
    output_byte_len: u64,
    texture_row_bytes: Option<u32>,
}

struct VerificationReadbackOperations {
    correlation: DeterministicVerificationReadbacks,
    canonical_output: GpuReadbackOperation,
    definedness: GpuReadbackOperation,
    status: GpuReadbackOperation,
}

/// Submit one ordinary maintained deterministic render invocation.
///
/// Outputs are lowered independently from their exact semantic output/observation correlation, but
/// every resulting fragment is submitted together through exactly one `GpuContext::submit_work`.
/// Ordinary execution authors no CPU readback operations.
pub async fn submit_deterministic_render(
    admitted: AdmittedDeterministicRender,
    context: &GpuContext,
) -> Result<SubmittedDeterministicRender, RenderDeterministicExecutionError> {
    let lowered =
        lower_deterministic_render(&admitted, context, DeterministicObservationIntent::Ordinary)?;
    debug_assert!(lowered.verification_readbacks.is_empty());
    submit_lowered_deterministic_render(
        admitted,
        context,
        lowered.work_set,
        lowered.object_identity_decoder,
    )
    .await
}

/// Submit the exact maintained deterministic path with renderer-private same-submission readbacks.
///
/// Static verifier eligibility is owned by `deterministic_verification` and must be established
/// before this function is called. The returned private witness wraps the same ordinary submitted
/// execution plus only the readback correlation authored before that submission.
pub(super) async fn submit_deterministic_render_for_verification(
    admitted: AdmittedDeterministicRender,
    context: &GpuContext,
) -> Result<DeterministicVerificationSubmission, RenderDeterministicExecutionError> {
    let lowered =
        lower_deterministic_render(&admitted, context, DeterministicObservationIntent::Verify)?;
    let verification_readbacks = lowered.verification_readbacks;
    let submitted = submit_lowered_deterministic_render(
        admitted,
        context,
        lowered.work_set,
        lowered.object_identity_decoder,
    )
    .await?;
    Ok(DeterministicVerificationSubmission {
        submitted,
        readbacks: verification_readbacks,
    })
}

async fn submit_lowered_deterministic_render(
    admitted: AdmittedDeterministicRender,
    context: &GpuContext,
    work_set: RenderWorkSet,
    object_identity_decoder: RenderObjectIdentityDecoder,
) -> Result<SubmittedDeterministicRender, RenderDeterministicExecutionError> {
    let submission = context
        .submit_work(
            "RunenRender maintained deterministic execution",
            work_set.fragments().iter().cloned(),
        )
        .await
        .map_err(RenderDeterministicExecutionError::Submission)?;
    Ok(SubmittedDeterministicRender {
        admitted,
        submission,
        object_identity_decoder,
    })
}

fn lower_deterministic_render(
    maintained: &AdmittedDeterministicRender,
    context: &GpuContext,
    intent: DeterministicObservationIntent,
) -> Result<LoweredDeterministicRender, RenderDeterministicLoweringError> {
    let admitted = maintained.admitted();
    if admitted.environment().affinity() != context.affinity() {
        return Err(RenderDeterministicLoweringError::ContextAffinityChanged {
            admitted: admitted.environment().affinity(),
            actual: context.affinity(),
        });
    }

    let object_identity_decoder = build_object_identity_decoder(admitted)?;
    let object_codes = object_identity_decoder
        .objects_by_code
        .iter()
        .copied()
        .enumerate()
        .map(|(index, object_id)| {
            let code = u32::try_from(index + 1).map_err(|_| {
                RenderDeterministicLoweringError::SizeOverflow {
                    field: "object identity codebook",
                }
            })?;
            Ok((object_id, code))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;

    let mut resources = GpuResourceScope::new();
    let mut fragments = Vec::new();
    fragments
        .try_reserve_exact(admitted.outputs().len())
        .map_err(|_| RenderDeterministicLoweringError::HostAllocation {
            field: "maintained output fragments",
        })?;
    let mut verification_readbacks = Vec::new();
    if intent.requires_private_readback() {
        verification_readbacks
            .try_reserve_exact(admitted.outputs().len())
            .map_err(|_| RenderDeterministicLoweringError::HostAllocation {
                field: "verification readback correlation",
            })?;
    }
    for output in admitted.outputs() {
        let lowered = lower_output(
            admitted,
            output.output_index(),
            &object_codes,
            context,
            &mut resources,
            intent,
        )?;
        fragments.push(lowered.fragment);
        if let Some(readbacks) = lowered.verification_readbacks {
            verification_readbacks.push(readbacks);
        }
    }

    Ok(LoweredDeterministicRender {
        work_set: RenderWorkSet::from_lowering(admitted, fragments),
        object_identity_decoder,
        verification_readbacks,
    })
}

fn build_object_identity_decoder(
    admitted: &AdmittedRenderPlan,
) -> Result<RenderObjectIdentityDecoder, RenderDeterministicLoweringError> {
    let objects = admitted
        .outputs()
        .iter()
        .flat_map(|output| output.object_representations())
        .map(|object| object.object_id())
        .collect::<BTreeSet<_>>();
    let object_count = u64::try_from(objects.len()).map_err(|_| {
        RenderDeterministicLoweringError::SizeOverflow {
            field: "object identity codebook",
        }
    })?;
    if object_count > u64::from(u32::MAX) {
        return Err(RenderDeterministicLoweringError::SizeOverflow {
            field: "object identity codebook",
        });
    }
    Ok(RenderObjectIdentityDecoder {
        objects_by_code: objects.into_iter().collect(),
    })
}

fn lower_output(
    admitted: &AdmittedRenderPlan,
    output_index: usize,
    object_codes: &BTreeMap<RenderObjectId, u32>,
    context: &GpuContext,
    resources: &mut GpuResourceScope,
    intent: DeterministicObservationIntent,
) -> Result<LoweredDeterministicOutput, RenderDeterministicLoweringError> {
    let admitted_output = admitted
        .outputs()
        .iter()
        .find(|output| output.output_index() == output_index)
        .ok_or(RenderDeterministicLoweringError::OutputCorrelationChanged { output_index })?;
    let requested = admitted
        .plan()
        .request()
        .outputs()
        .get(output_index)
        .copied()
        .ok_or(RenderDeterministicLoweringError::OutputCorrelationChanged { output_index })?;
    if requested.observation_index() != admitted_output.observation_index() {
        return Err(RenderDeterministicLoweringError::OutputCorrelationChanged { output_index });
    }
    let observation = admitted
        .plan()
        .request()
        .observations()
        .get(requested.observation_index())
        .copied()
        .ok_or(RenderDeterministicLoweringError::OutputCorrelationChanged { output_index })?;

    let packed = pack_output(
        admitted,
        admitted_output,
        requested.spec().value(),
        observation,
        object_codes,
        context,
    )?;
    let sample_byte_len = u64::from(packed.sample_count)
        .checked_mul(WORD_BYTES)
        .ok_or(RenderDeterministicLoweringError::SizeOverflow {
            field: "definedness/status byte length",
        })?;
    let input_payload = PreparedGpuData::<TransferData>::ordinary_pod_transfer(
        format!("RunenRender output {output_index} packed semantic input"),
        &packed.input_words,
    )
    .map_err(|error| gpu_authoring("semantic-input preparation", error))?;

    let input = resources
        .buffer(
            GpuBufferDescriptor::ordinary_owned(
                format!("RunenRender output {output_index} packed input"),
                GpuResourceLifetime::Transient,
                GpuReconstruction::SourceBacked,
                input_payload.layout().byte_len(),
                [GpuBufferUsage::Storage, GpuBufferUsage::CopyDestination],
                GpuBufferInitialization::Uninitialized,
            )
            .map_err(|error| gpu_authoring("input-buffer descriptor", error))?,
        )
        .map_err(|error| gpu_authoring("input-buffer allocation", error))?;
    let canonical_output = resources
        .buffer(
            GpuBufferDescriptor::ordinary_owned(
                format!("RunenRender output {output_index} canonical words"),
                GpuResourceLifetime::Transient,
                GpuReconstruction::SourceBacked,
                packed.output_byte_len,
                [
                    GpuBufferUsage::Storage,
                    GpuBufferUsage::CopySource,
                    GpuBufferUsage::CopyDestination,
                ],
                GpuBufferInitialization::Uninitialized,
            )
            .map_err(|error| gpu_authoring("canonical-output descriptor", error))?,
        )
        .map_err(|error| gpu_authoring("canonical-output allocation", error))?;
    let definedness = resources
        .buffer(
            GpuBufferDescriptor::ordinary_owned(
                format!("RunenRender output {output_index} definedness"),
                GpuResourceLifetime::Transient,
                GpuReconstruction::SourceBacked,
                sample_byte_len,
                [
                    GpuBufferUsage::Storage,
                    GpuBufferUsage::CopySource,
                    GpuBufferUsage::CopyDestination,
                ],
                GpuBufferInitialization::Uninitialized,
            )
            .map_err(|error| gpu_authoring("definedness descriptor", error))?,
        )
        .map_err(|error| gpu_authoring("definedness allocation", error))?;
    let status = resources
        .buffer(
            GpuBufferDescriptor::ordinary_owned(
                format!("RunenRender output {output_index} evaluator status"),
                GpuResourceLifetime::Transient,
                GpuReconstruction::SourceBacked,
                sample_byte_len,
                [
                    GpuBufferUsage::Storage,
                    GpuBufferUsage::CopySource,
                    GpuBufferUsage::CopyDestination,
                ],
                GpuBufferInitialization::Uninitialized,
            )
            .map_err(|error| gpu_authoring("status descriptor", error))?,
        )
        .map_err(|error| gpu_authoring("status allocation", error))?;

    let input_upload = GpuUploadOperation::whole_buffer(&input, input_payload)
        .map_err(|error| gpu_authoring("input upload", error))?;
    let output_clear = GpuClearOperation::buffer_zero(
        GpuBufferRegion::whole(&canonical_output)
            .map_err(|error| gpu_authoring("canonical-output clear region", error))?,
    )
    .map_err(|error| gpu_authoring("canonical-output clear", error))?;
    let definedness_clear = GpuClearOperation::buffer_zero(
        GpuBufferRegion::whole(&definedness)
            .map_err(|error| gpu_authoring("definedness clear region", error))?,
    )
    .map_err(|error| gpu_authoring("definedness clear", error))?;
    let status_clear = GpuClearOperation::buffer_zero(
        GpuBufferRegion::whole(&status)
            .map_err(|error| gpu_authoring("status clear region", error))?,
    )
    .map_err(|error| gpu_authoring("status clear", error))?;

    let [source] =
        admit_static_wgsl_sources([("runenrender.maintained.deterministic", 1, MAINTAINED_WGSL)])
            .map_err(|error| gpu_authoring("maintained WGSL admission", error))?;
    let pipeline = GpuComputePipelineDescriptor::ordinary(source, "main")
        .map_err(|error| gpu_authoring("compute-pipeline descriptor", error))?;
    let runtime_bindings = pipeline
        .runtime_bindings([
            GpuRuntimeBindingValue::whole_buffer(0, 0, &input),
            GpuRuntimeBindingValue::whole_buffer(0, 1, &canonical_output),
            GpuRuntimeBindingValue::whole_buffer(0, 2, &definedness),
            GpuRuntimeBindingValue::whole_buffer(0, 3, &status),
        ])
        .map_err(|error| gpu_authoring("compute runtime bindings", error))?;
    let compute = GpuComputeOperation::new(
        pipeline,
        runtime_bindings,
        GpuDispatchIntent::direct(GpuDispatchSize::new(
            packed.sample_count.div_ceil(WORKGROUP_SIZE),
            1,
            1,
        )),
    )
    .map_err(|error| gpu_authoring("compute operation", error))?;

    let destination_copy = match admitted_output.binding().destination() {
        RenderOutputDestination::ScalarBuffer(destination) => GpuCopyOperation::buffer_to_buffer(
            GpuBufferRegion::whole(&canonical_output)
                .map_err(|error| gpu_authoring("scalar source region", error))?,
            GpuBufferRegion::whole(destination)
                .map_err(|error| gpu_authoring("scalar destination region", error))?,
        )
        .map_err(|error| gpu_authoring("scalar destination copy", error))?,
        RenderOutputDestination::SampleLatticeTexture(destination) => {
            let row_bytes = packed.texture_row_bytes.ok_or(
                RenderDeterministicLoweringError::OutputCorrelationChanged { output_index },
            )?;
            let source = GpuBufferTextureLayout::new(&canonical_output, 0, row_bytes, 0)
                .map_err(|error| gpu_authoring("lattice source layout", error))?;
            let destination = GpuTextureCopyRegion::whole_base_mip(destination)
                .map_err(|error| gpu_authoring("lattice destination region", error))?;
            GpuCopyOperation::buffer_to_texture(source, destination)
                .map_err(|error| gpu_authoring("lattice destination copy", error))?
        }
    };

    let verification = if intent.requires_private_readback() {
        let canonical_readback = GpuReadbackOperation::ordinary(
            GpuBufferRegion::whole(&canonical_output)
                .map_err(|error| gpu_authoring("canonical-output readback region", error))?
                .into(),
        )
        .map_err(|error| gpu_authoring("canonical-output readback", error))?;
        let definedness_readback = GpuReadbackOperation::ordinary(
            GpuBufferRegion::whole(&definedness)
                .map_err(|error| gpu_authoring("definedness readback region", error))?
                .into(),
        )
        .map_err(|error| gpu_authoring("definedness readback", error))?;
        let status_readback = GpuReadbackOperation::ordinary(
            GpuBufferRegion::whole(&status)
                .map_err(|error| gpu_authoring("status readback region", error))?
                .into(),
        )
        .map_err(|error| gpu_authoring("status readback", error))?;
        Some(VerificationReadbackOperations {
            correlation: DeterministicVerificationReadbacks {
                output_index,
                canonical_output: canonical_readback.id(),
                definedness: definedness_readback.id(),
                status: status_readback.id(),
            },
            canonical_output: canonical_readback,
            definedness: definedness_readback,
            status: status_readback,
        })
    } else {
        None
    };
    let verification_readbacks = verification.as_ref().map(|readbacks| readbacks.correlation);

    let fragment = GpuWorkFragment::build(
        format!("RunenRender maintained output {output_index}"),
        |work| {
            work.operation("upload deterministic semantic input", input_upload)?;
            work.operation("clear canonical output", output_clear)?;
            work.operation("clear semantic definedness", definedness_clear)?;
            work.operation("clear evaluator status", status_clear)?;
            work.compute("evaluate deterministic output", compute)?;
            work.operation(
                "copy canonical output to admitted destination",
                destination_copy,
            )?;
            if let Some(readbacks) = verification {
                work.operation(
                    "read back private canonical output",
                    readbacks.canonical_output,
                )?;
                work.operation(
                    "read back private semantic definedness",
                    readbacks.definedness,
                )?;
                work.operation("read back private evaluator status", readbacks.status)?;
            }
            Ok(())
        },
    )
    .map_err(|error| gpu_authoring("work-fragment construction", error))?;

    Ok(LoweredDeterministicOutput {
        fragment,
        verification_readbacks,
    })
}

fn pack_output(
    admitted: &AdmittedRenderPlan,
    admitted_output: &super::admission::RenderAdmittedOutput,
    value: RenderOutputValue,
    observation: RenderObservationSpec,
    object_codes: &BTreeMap<RenderObjectId, u32>,
    context: &GpuContext,
) -> Result<PackedOutput, RenderDeterministicLoweringError> {
    let output_index = admitted_output.output_index();
    let topology = admitted.plan().request().outputs()[output_index]
        .spec()
        .topology();
    let (sample_count, width, height, row_stride_words, output_byte_len, texture_row_bytes) =
        if let Some((width, height)) = topology.sample_lattice_dimensions() {
            let sample_count = width.checked_mul(height).ok_or(
                RenderDeterministicLoweringError::SizeOverflow {
                    field: "lattice sample count",
                },
            )?;
            let logical_row_bytes = u64::from(width).checked_mul(WORD_BYTES).ok_or(
                RenderDeterministicLoweringError::SizeOverflow {
                    field: "lattice logical row bytes",
                },
            )?;
            let alignment = context
                .device_facts()
                .device_limits()
                .alignments()
                .bytes_per_row
                .ok_or(RenderDeterministicLoweringError::MissingBytesPerRowAlignment)?;
            let row_bytes = align_up(logical_row_bytes, alignment)?;
            if row_bytes % WORD_BYTES != 0 {
                return Err(
                    RenderDeterministicLoweringError::InvalidBytesPerRowAlignment { alignment },
                );
            }
            let row_stride_words = u32::try_from(row_bytes / WORD_BYTES).map_err(|_| {
                RenderDeterministicLoweringError::SizeOverflow {
                    field: "lattice row stride",
                }
            })?;
            let output_words = row_stride_words.checked_mul(height).ok_or(
                RenderDeterministicLoweringError::SizeOverflow {
                    field: "canonical lattice word count",
                },
            )?;
            let output_byte_len = u64::from(output_words).checked_mul(WORD_BYTES).ok_or(
                RenderDeterministicLoweringError::SizeOverflow {
                    field: "canonical lattice byte length",
                },
            )?;
            let row_bytes = u32::try_from(row_bytes).map_err(|_| {
                RenderDeterministicLoweringError::SizeOverflow {
                    field: "lattice row bytes",
                }
            })?;
            (
                sample_count,
                width,
                height,
                row_stride_words,
                output_byte_len,
                Some(row_bytes),
            )
        } else {
            (1, 1, 1, 1, WORD_BYTES, None)
        };

    let (output_kind, wavelength) = match value {
        RenderOutputValue::Radiance { representation } => {
            (OUTPUT_RADIANCE, Some(representation.wavelength_meters()))
        }
        RenderOutputValue::Distance {
            convention: RenderDistanceConvention::ObservationForwardDepth,
        } => (OUTPUT_FORWARD_DEPTH, None),
        RenderOutputValue::ObjectIdentity => (OUTPUT_OBJECT_IDENTITY, None),
        RenderOutputValue::Distance { .. } => {
            return Err(RenderDeterministicLoweringError::UnsupportedOutput { output_index });
        }
    };

    let (observation_kind, transform, tan_half_fov, aspect_ratio) = match observation {
        RenderObservationSpec::Perspective(observation) => (
            OBSERVATION_PERSPECTIVE,
            observation.observation_to_scene(),
            Some((observation.vertical_field_of_view_radians() * 0.5).tan()),
            Some(observation.aspect_ratio()),
        ),
        RenderObservationSpec::Probe(observation) => (
            OBSERVATION_PROBE,
            observation.observation_to_scene(),
            None,
            None,
        ),
    };

    let mut geometry = Vec::new();
    geometry
        .try_reserve_exact(admitted_output.object_representations().len())
        .map_err(|_| RenderDeterministicLoweringError::HostAllocation {
            field: "maintained geometry records",
        })?;
    for object in admitted_output.object_representations() {
        let representation_id = object.representation().representation_id();
        let input = admitted.surface_semantic_input(representation_id).ok_or(
            RenderDeterministicLoweringError::MissingSurfaceInput {
                output_index,
                object_id: object.object_id(),
                representation_id,
            },
        )?;
        geometry.push((
            object.object_id(),
            representation_id,
            input.execution_view(),
        ));
    }
    geometry.sort_by_key(|(object_id, representation_id, _)| (*object_id, *representation_id));

    let emitters = if let Some(wavelength) = wavelength {
        matching_emitters(admitted, wavelength)?
    } else {
        Vec::new()
    };
    let emitter_offset = HEADER_WORDS
        .checked_add(geometry.len().checked_mul(GEOMETRY_WORDS).ok_or(
            RenderDeterministicLoweringError::SizeOverflow {
                field: "geometry input words",
            },
        )?)
        .ok_or(RenderDeterministicLoweringError::SizeOverflow {
            field: "emitter input offset",
        })?;
    let total_words = emitter_offset
        .checked_add(emitters.len().checked_mul(EMITTER_WORDS).ok_or(
            RenderDeterministicLoweringError::SizeOverflow {
                field: "emitter input words",
            },
        )?)
        .ok_or(RenderDeterministicLoweringError::SizeOverflow {
            field: "packed input words",
        })?;
    let mut words = Vec::new();
    words.try_reserve_exact(total_words).map_err(|_| {
        RenderDeterministicLoweringError::HostAllocation {
            field: "packed maintained semantic input",
        }
    })?;
    words.resize(total_words, 0_u32);

    words[0] = sample_count;
    words[1] = width;
    words[2] = height;
    words[3] = row_stride_words;
    words[4] = u32::try_from(geometry.len()).map_err(|_| {
        RenderDeterministicLoweringError::SizeOverflow {
            field: "geometry count",
        }
    })?;
    words[5] = u32::try_from(emitters.len()).map_err(|_| {
        RenderDeterministicLoweringError::SizeOverflow {
            field: "emitter count",
        }
    })?;
    words[6] = output_kind;
    words[7] = observation_kind;
    pack_observation(&mut words, transform, tan_half_fov, aspect_ratio)?;
    words[23] = u32::try_from(emitter_offset).map_err(|_| {
        RenderDeterministicLoweringError::SizeOverflow {
            field: "emitter input offset",
        }
    })?;

    for (index, (object_id, _, input)) in geometry.into_iter().enumerate() {
        let base = HEADER_WORDS + index * GEOMETRY_WORDS;
        let state = admitted.plan().scene().object_state(object_id).ok_or(
            RenderDeterministicLoweringError::MissingObjectState {
                output_index,
                object_id,
            },
        )?;
        let transform = RenderCompiledObjectTransform::compile(state.spatial()).map_err(
            |RenderCompiledObjectTransformError::NonInvertibleObjectTransform| {
                RenderDeterministicLoweringError::NonInvertibleObjectTransform {
                    output_index,
                    object_id,
                }
            },
        )?;
        words[base + 1] = *object_codes
            .get(&object_id)
            .ok_or(RenderDeterministicLoweringError::OutputCorrelationChanged { output_index })?;
        words[base + 2] = if output_kind == OUTPUT_RADIANCE {
            let material = admitted
                .plan()
                .scene()
                .object_participation(object_id)
                .and_then(|participation| participation.material_assignment())
                .ok_or(RenderDeterministicLoweringError::MissingMaterial {
                    output_index,
                    object_id,
                })?;
            f32_bits(material.material().reflectance(), "diffuse reflectance")?
        } else {
            0
        };
        pack_invertible_matrix3(
            &mut words,
            base + 4,
            transform.scene_to_local_units_row_major(),
            "object scene-to-local transform",
        )?;
        pack_vec3(&mut words, base + 13, transform.translation_scene())?;
        pack_matrix3(
            &mut words,
            base + 16,
            transform.normal_local_to_scene_row_major(),
        )?;
        match input {
            RenderSurfaceSemanticInputView::Sphere {
                center_local_units,
                radius_local_units,
            } => {
                words[base] = SHAPE_SPHERE;
                pack_vec3(&mut words, base + 25, center_local_units)?;
                words[base + 28] =
                    positive_f32_bits(radius_local_units, "surface-input sphere radius")?;
            }
            RenderSurfaceSemanticInputView::Plane {
                point_local_units,
                normal_local,
            } => {
                words[base] = SHAPE_PLANE;
                pack_vec3(&mut words, base + 25, point_local_units)?;
                pack_vec3(&mut words, base + 28, normal_local)?;
            }
        }
    }

    for (index, emitter) in emitters.into_iter().enumerate() {
        let base = emitter_offset + index * EMITTER_WORDS;
        pack_vec3(&mut words, base, emitter.direction_to_source_scene())?;
        words[base + 3] = f32_bits(
            emitter.spectral_irradiance_w_m3(),
            "directional-emitter spectral irradiance",
        )?;
    }

    Ok(PackedOutput {
        input_words: words,
        sample_count,
        output_byte_len,
        texture_row_bytes,
    })
}

fn matching_emitters(
    admitted: &AdmittedRenderPlan,
    wavelength_meters: f64,
) -> Result<Vec<super::appearance::RenderDirectionalEmitter>, RenderDeterministicLoweringError> {
    let mut emitters = Vec::new();
    emitters
        .try_reserve_exact(admitted.plan().scene().len())
        .map_err(|_| RenderDeterministicLoweringError::HostAllocation {
            field: "directional-emitter realization",
        })?;
    for object_id in admitted.plan().scene().object_ids() {
        let Some(emitter) = admitted
            .plan()
            .scene()
            .object_participation(object_id)
            .and_then(|participation| participation.emitter())
        else {
            continue;
        };
        if emitter.wavelength_meters() == wavelength_meters {
            emitters.push(emitter);
        }
    }
    emitters.sort_by(|left, right| {
        left.direction_to_source_scene()
            .map(f64::to_bits)
            .cmp(&right.direction_to_source_scene().map(f64::to_bits))
            .then_with(|| {
                left.spectral_irradiance_w_m3()
                    .to_bits()
                    .cmp(&right.spectral_irradiance_w_m3().to_bits())
            })
    });

    let total = emitters.iter().try_fold(0.0_f64, |total, emitter| {
        let next = total + emitter.spectral_irradiance_w_m3();
        next.is_finite().then_some(next).ok_or(
            RenderDeterministicLoweringError::NumericRealization {
                field: "summed directional-emitter irradiance",
            },
        )
    })?;
    let _ = f32_bits(total, "summed directional-emitter irradiance")?;
    Ok(emitters)
}

fn pack_observation(
    words: &mut [u32],
    transform: super::space_time::RenderAffineTransform3,
    tan_half_fov: Option<f64>,
    aspect_ratio: Option<f64>,
) -> Result<(), RenderDeterministicLoweringError> {
    let matrix = transform.row_major_3x4();
    pack_vec3(words, 8, [matrix[3], matrix[7], matrix[11]])?;
    pack_invertible_matrix3(
        words,
        11,
        [
            matrix[0], matrix[1], matrix[2], matrix[4], matrix[5], matrix[6], matrix[8], matrix[9],
            matrix[10],
        ],
        "observation linear transform",
    )?;
    words[20] = match tan_half_fov {
        Some(value) => positive_f32_bits(value, "perspective tangent half field of view")?,
        None => 0,
    };
    words[21] = match aspect_ratio {
        Some(value) => positive_f32_bits(value, "perspective aspect ratio")?,
        None => 1.0_f32.to_bits(),
    };
    Ok(())
}

fn pack_vec3(
    words: &mut [u32],
    base: usize,
    values: [f64; 3],
) -> Result<(), RenderDeterministicLoweringError> {
    words[base] = f32_bits(values[0], "packed vector component")?;
    words[base + 1] = f32_bits(values[1], "packed vector component")?;
    words[base + 2] = f32_bits(values[2], "packed vector component")?;
    Ok(())
}

fn pack_matrix3(
    words: &mut [u32],
    base: usize,
    values: [f64; 9],
) -> Result<(), RenderDeterministicLoweringError> {
    for (offset, value) in values.into_iter().enumerate() {
        words[base + offset] = f32_bits(value, "packed transform component")?;
    }
    Ok(())
}

fn pack_invertible_matrix3(
    words: &mut [u32],
    base: usize,
    values: [f64; 9],
    field: &'static str,
) -> Result<(), RenderDeterministicLoweringError> {
    let mut physical = [0.0_f32; 9];
    for (slot, value) in physical.iter_mut().zip(values) {
        let narrowed = value as f32;
        if !value.is_finite() || !narrowed.is_finite() {
            return Err(RenderDeterministicLoweringError::NumericRealization { field });
        }
        *slot = narrowed;
    }
    let determinant = physical[0] * (physical[4] * physical[8] - physical[5] * physical[7])
        - physical[1] * (physical[3] * physical[8] - physical[5] * physical[6])
        + physical[2] * (physical[3] * physical[7] - physical[4] * physical[6]);
    if !determinant.is_finite() || determinant == 0.0 {
        return Err(RenderDeterministicLoweringError::NumericRealization { field });
    }
    for (offset, value) in physical.into_iter().enumerate() {
        words[base + offset] = value.to_bits();
    }
    Ok(())
}

fn f32_bits(value: f64, field: &'static str) -> Result<u32, RenderDeterministicLoweringError> {
    let physical = value as f32;
    if !value.is_finite() || !physical.is_finite() {
        return Err(RenderDeterministicLoweringError::NumericRealization { field });
    }
    Ok(physical.to_bits())
}

fn positive_f32_bits(
    value: f64,
    field: &'static str,
) -> Result<u32, RenderDeterministicLoweringError> {
    let physical = value as f32;
    if !value.is_finite() || !physical.is_finite() || physical <= 0.0 {
        return Err(RenderDeterministicLoweringError::NumericRealization { field });
    }
    Ok(physical.to_bits())
}

fn align_up(value: u64, alignment: u64) -> Result<u64, RenderDeterministicLoweringError> {
    if alignment == 0 {
        return Err(RenderDeterministicLoweringError::InvalidBytesPerRowAlignment { alignment });
    }
    let remainder = value % alignment;
    if remainder == 0 {
        Ok(value)
    } else {
        value.checked_add(alignment - remainder).ok_or(
            RenderDeterministicLoweringError::SizeOverflow {
                field: "aligned lattice row bytes",
            },
        )
    }
}

fn gpu_authoring(
    stage: &'static str,
    error: impl fmt::Display,
) -> RenderDeterministicLoweringError {
    RenderDeterministicLoweringError::RunenGpuAuthoring {
        stage,
        detail: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maintained_wgsl_forms_a_canonical_compute_pipeline() {
        let [source] = admit_static_wgsl_sources([(
            "runenrender.maintained.deterministic.test",
            1,
            MAINTAINED_WGSL,
        )])
        .expect("maintained deterministic source admission");
        assert_eq!(source.identity().revision().get(), 1);
        let pipeline = GpuComputePipelineDescriptor::ordinary(source, "main")
            .expect("maintained deterministic WGSL must form a canonical compute pipeline");
        assert_eq!(pipeline.entry_point().as_str(), "main");
    }

    #[test]
    fn physical_identity_decoder_reserves_zero_and_unknown_codes() {
        let decoder = RenderObjectIdentityDecoder {
            objects_by_code: Vec::new(),
        };
        assert_eq!(decoder.decode(0), None);
        assert_eq!(decoder.decode(1), None);
    }

    #[test]
    fn ordinary_observation_intent_never_requests_private_readback() {
        assert!(!DeterministicObservationIntent::Ordinary.requires_private_readback());
        assert!(DeterministicObservationIntent::Verify.requires_private_readback());
    }

    #[test]
    fn row_alignment_is_checked_without_embedding_device_policy() {
        assert_eq!(align_up(12, 4), Ok(12));
        assert_eq!(align_up(12, 8), Ok(16));
        assert_eq!(
            align_up(12, 0),
            Err(RenderDeterministicLoweringError::InvalidBytesPerRowAlignment { alignment: 0 })
        );
    }
}
