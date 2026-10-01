//! Curated ordinary public surface for the maintained RunenRender path.
//!
//! This module gives ordinary consumers progressive disclosure over the existing semantic
//! scene/request -> planning -> admission -> lowering -> RunenGPU authority. It does not introduce
//! a second planner, renderer, submission lifecycle, or output-value authority. The maintained
//! deterministic implementation remains an internal method realization behind these names.

use super::admission::{
    AdmittedRenderPlan, RenderOutputBinding, RenderRepresentationAvailabilityFact,
};
use super::deterministic_admission::{
    AdmittedDeterministicRender, RenderDeterministicAdmissionFailure,
    admit_deterministic_render_with_semantic_inputs,
};
use super::deterministic_capture::{
    RenderCapturedDeterministicRadiance, RenderDeterministicRadianceCaptureError,
    RenderDeterministicRadianceCaptureRequest, RenderDeterministicRadianceCaptureRequestError,
};
pub use super::deterministic_execution::{
    RenderObjectIdentityDecoder, RenderRequestedCoveragePreparation, RenderTemporalExecutionEvidence,
};
use super::deterministic_execution::{
    PreparedDeterministicRadianceOutput, PreparedDeterministicRender,
    RenderDeterministicExecutionError, RenderDeterministicResultFormationError,
    RenderDeterministicVerifiedSubmissionError, SubmittedDeterministicRender,
    prepare_deterministic_render, submit_deterministic_render,
    submit_deterministic_render_for_verified_result,
};
use super::field_input::RenderFieldSemanticInputBinding;
use super::lowering::RenderWorkSet;
use super::render_result::RenderResult;
use super::request::{RenderRadiometricRepresentation, RenderRequest, RenderResultTopology};
use super::scene::RenderSceneSnapshot;
use super::surface_input::RenderSurfaceSemanticInputBinding;
use runen_gpu::{
    GpuContext, GpuExportRelationship, GpuReadbackId, GpuResourceProvenance, GpuResourceRef,
    GpuSubmission, GpuSubmissionStatus, GpuTextureHandle, GpuTransferRegion, GpuWorkImport,
    GpuWorkSubmissionError,
};
use std::error::Error;
use std::fmt;

/// High-level category for failure before maintained execution is prepared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderAdmissionErrorKind {
    Planning,
    Admission,
    Compatibility,
}

/// Failure while planning, semantically admitting, or checking maintained-method compatibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderAdmissionError {
    inner: RenderDeterministicAdmissionFailure,
}

impl RenderAdmissionError {
    /// Owner-oriented failure category without implementation-specific error names.
    pub fn kind(&self) -> RenderAdmissionErrorKind {
        match &self.inner {
            RenderDeterministicAdmissionFailure::Planning(_) => RenderAdmissionErrorKind::Planning,
            RenderDeterministicAdmissionFailure::Admission(_) => {
                RenderAdmissionErrorKind::Admission
            }
            RenderDeterministicAdmissionFailure::Compatibility(_) => {
                RenderAdmissionErrorKind::Compatibility
            }
        }
    }
}

impl fmt::Display for RenderAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            RenderDeterministicAdmissionFailure::Planning(error) => {
                write!(formatter, "render planning failed: {error}")
            }
            RenderDeterministicAdmissionFailure::Admission(error) => {
                write!(formatter, "render admission failed: {error}")
            }
            RenderDeterministicAdmissionFailure::Compatibility(error) => {
                write!(
                    formatter,
                    "maintained renderer compatibility failed: {error}"
                )
            }
        }
    }
}

impl Error for RenderAdmissionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.inner {
            RenderDeterministicAdmissionFailure::Planning(error) => Some(error),
            RenderDeterministicAdmissionFailure::Admission(error) => Some(error),
            RenderDeterministicAdmissionFailure::Compatibility(error) => Some(error),
        }
    }
}

/// High-level category for ordinary maintained execution failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderExecutionErrorKind {
    Lowering,
    Submission,
}

/// Failure while lowering admitted renderer meaning or submitting it through public RunenGPU.
#[derive(Debug)]
pub struct RenderExecutionError {
    inner: RenderDeterministicExecutionError,
}

impl RenderExecutionError {
    /// Owner-oriented failure category.
    pub fn kind(&self) -> RenderExecutionErrorKind {
        match &self.inner {
            RenderDeterministicExecutionError::Lowering(_) => RenderExecutionErrorKind::Lowering,
            RenderDeterministicExecutionError::Submission(_) => {
                RenderExecutionErrorKind::Submission
            }
        }
    }

    /// Stable public RunenGPU submission failure when execution reached physical submission.
    pub const fn submission_error(&self) -> Option<&GpuWorkSubmissionError> {
        match &self.inner {
            RenderDeterministicExecutionError::Submission(error) => Some(error),
            RenderDeterministicExecutionError::Lowering(_) => None,
        }
    }
}

impl fmt::Display for RenderExecutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            RenderDeterministicExecutionError::Lowering(error) => {
                write!(formatter, "render lowering failed: {error}")
            }
            RenderDeterministicExecutionError::Submission(error) => {
                write!(formatter, "RunenGPU submission failed: {error}")
            }
        }
    }
}

impl Error for RenderExecutionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.inner {
            RenderDeterministicExecutionError::Lowering(error) => Some(error),
            RenderDeterministicExecutionError::Submission(error) => Some(error),
        }
    }
}

/// High-level category for a submission that requested semantic result formation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderResultSubmissionErrorKind {
    Eligibility,
    Execution,
    Correlation,
}

/// Failure while selecting result-verification intent or authoring its exact submission.
#[derive(Debug)]
pub struct RenderResultSubmissionError {
    inner: RenderDeterministicVerifiedSubmissionError,
}

impl RenderResultSubmissionError {
    /// Owner-oriented failure category.
    pub fn kind(&self) -> RenderResultSubmissionErrorKind {
        match &self.inner {
            RenderDeterministicVerifiedSubmissionError::Eligibility { .. } => {
                RenderResultSubmissionErrorKind::Eligibility
            }
            RenderDeterministicVerifiedSubmissionError::Execution(_) => {
                RenderResultSubmissionErrorKind::Execution
            }
            RenderDeterministicVerifiedSubmissionError::Correlation { .. } => {
                RenderResultSubmissionErrorKind::Correlation
            }
        }
    }
}

impl fmt::Display for RenderResultSubmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            RenderDeterministicVerifiedSubmissionError::Eligibility { detail } => {
                write!(
                    formatter,
                    "render result verification is outside the certified domain: {detail}"
                )
            }
            RenderDeterministicVerifiedSubmissionError::Execution(error) => error.fmt(formatter),
            RenderDeterministicVerifiedSubmissionError::Correlation { detail } => {
                write!(
                    formatter,
                    "render verification correlation failed: {detail}"
                )
            }
        }
    }
}

impl Error for RenderResultSubmissionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.inner {
            RenderDeterministicVerifiedSubmissionError::Execution(error) => Some(error),
            RenderDeterministicVerifiedSubmissionError::Eligibility { .. }
            | RenderDeterministicVerifiedSubmissionError::Correlation { .. } => None,
        }
    }
}

/// High-level category while polling semantic result formation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderResultFormationErrorKind {
    VerificationNotRequested,
    ResultAlreadyFormed,
    SubmissionFailed,
    ReadbackCorrelationLost,
    ReadbackFailed,
    VerificationRejected,
    FormationFailed,
}

/// Failure while forming semantic provenance from a result-capable submission.
#[derive(Debug)]
pub struct RenderResultFormationError {
    inner: RenderDeterministicResultFormationError,
}

impl RenderResultFormationError {
    /// Owner-oriented failure category.
    pub fn kind(&self) -> RenderResultFormationErrorKind {
        match &self.inner {
            RenderDeterministicResultFormationError::VerificationNotRequested => {
                RenderResultFormationErrorKind::VerificationNotRequested
            }
            RenderDeterministicResultFormationError::ResultAlreadyFormed => {
                RenderResultFormationErrorKind::ResultAlreadyFormed
            }
            RenderDeterministicResultFormationError::SubmissionFailed { .. } => {
                RenderResultFormationErrorKind::SubmissionFailed
            }
            RenderDeterministicResultFormationError::ReadbackCorrelationLost { .. } => {
                RenderResultFormationErrorKind::ReadbackCorrelationLost
            }
            RenderDeterministicResultFormationError::ReadbackFailed { .. } => {
                RenderResultFormationErrorKind::ReadbackFailed
            }
            RenderDeterministicResultFormationError::VerificationRejected { .. } => {
                RenderResultFormationErrorKind::VerificationRejected
            }
            RenderDeterministicResultFormationError::ResultFormation { .. } => {
                RenderResultFormationErrorKind::FormationFailed
            }
        }
    }
}

impl fmt::Display for RenderResultFormationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            RenderDeterministicResultFormationError::VerificationNotRequested => formatter
                .write_str("this render was submitted without semantic-result verification"),
            RenderDeterministicResultFormationError::ResultAlreadyFormed => {
                formatter.write_str("semantic result evidence was already formed from this render")
            }
            RenderDeterministicResultFormationError::SubmissionFailed { kind } => {
                write!(
                    formatter,
                    "RunenGPU submission failed before result formation: {kind:?}"
                )
            }
            RenderDeterministicResultFormationError::ReadbackCorrelationLost {
                output_index,
                channel,
            } => write!(
                formatter,
                "output {output_index} {channel} verification readback lost exact-submission correlation"
            ),
            RenderDeterministicResultFormationError::ReadbackFailed {
                output_index,
                channel,
                kind,
            } => write!(
                formatter,
                "output {output_index} {channel} verification readback failed: {kind:?}"
            ),
            RenderDeterministicResultFormationError::VerificationRejected { detail } => {
                write!(
                    formatter,
                    "finite-evaluation verification rejected result formation: {detail}"
                )
            }
            RenderDeterministicResultFormationError::ResultFormation { detail } => {
                write!(
                    formatter,
                    "renderer-owned result formation failed: {detail}"
                )
            }
        }
    }
}

impl Error for RenderResultFormationError {}

/// Failure to mint a product-owned readback correlation for one formed radiance output.
#[derive(Debug)]
pub struct RenderRadianceCaptureRequestError {
    inner: RenderDeterministicRadianceCaptureRequestError,
}

impl fmt::Display for RenderRadianceCaptureRequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(formatter)
    }
}

impl Error for RenderRadianceCaptureRequestError {}

/// Failure to correlate or interpret one completed product-owned radiance readback.
#[derive(Debug)]
pub struct RenderRadianceCaptureError {
    inner: RenderDeterministicRadianceCaptureError,
}

impl fmt::Display for RenderRadianceCaptureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(formatter)
    }
}

impl Error for RenderRadianceCaptureError {}

/// Maintained invocation after semantic planning, binding, and execution admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedRender {
    inner: AdmittedDeterministicRender,
}

impl AdmittedRender {
    /// Exact admitted semantic plan retained by this invocation.
    pub const fn admitted_plan(&self) -> &AdmittedRenderPlan {
        self.inner.admitted()
    }
}

/// Renderer-authored public RunenGPU work prepared from one admitted invocation.
#[derive(Debug, Clone)]
pub struct PreparedRender {
    inner: PreparedDeterministicRender,
}

impl PreparedRender {
    /// Exact admitted semantic plan from which this work was lowered.
    pub const fn admitted_plan(&self) -> &AdmittedRenderPlan {
        self.inner.admitted().admitted()
    }

    /// Backend-neutral public RunenGPU work authored by the renderer.
    pub const fn work_set(&self) -> &RenderWorkSet {
        self.inner.work_set()
    }

    /// Prepared radiance outputs available for product composition.
    pub fn radiance_outputs(
        &self,
    ) -> impl ExactSizeIterator<Item = PreparedRadianceOutput<'_>> + '_ {
        self.inner
            .radiance_outputs()
            .iter()
            .map(|inner| PreparedRadianceOutput { inner })
    }

    /// Prepared radiance output for one requested output index, when applicable.
    pub fn radiance_output(&self, output_index: usize) -> Option<PreparedRadianceOutput<'_>> {
        self.inner
            .radiance_output(output_index)
            .map(|inner| PreparedRadianceOutput { inner })
    }
}

/// Borrowed correlation for one prepared radiance destination and its public RunenGPU export.
#[derive(Debug, Clone, Copy)]
pub struct PreparedRadianceOutput<'a> {
    inner: &'a PreparedDeterministicRadianceOutput,
}

impl PreparedRadianceOutput<'_> {
    /// Requested output index correlated to this prepared destination.
    pub const fn output_index(&self) -> usize {
        self.inner.output_index()
    }

    /// Public RunenGPU resource receiving the renderer output.
    pub fn resource(&self) -> &GpuResourceRef {
        self.inner.resource()
    }

    /// Texture handle when this destination is texture-backed.
    pub fn texture(&self) -> Option<&GpuTextureHandle> {
        self.inner.texture()
    }

    /// Exact producer/consumer relationship authored for composition.
    pub fn export_relationship(&self) -> &GpuExportRelationship {
        self.inner.export_relationship()
    }

    /// Form a public RunenGPU import of this renderer-authored output.
    pub fn import(&self, provenance: GpuResourceProvenance) -> GpuWorkImport {
        self.inner.import(provenance)
    }
}

/// One-shot correlation for a product-owned public RunenGPU radiance readback.
pub struct RenderRadianceCaptureRequest {
    inner: RenderDeterministicRadianceCaptureRequest,
}

impl RenderRadianceCaptureRequest {
    /// Requested output index correlated to this capture.
    pub const fn output_index(&self) -> usize {
        self.inner.output_index()
    }

    /// Exact public RunenGPU transfer source the product should read.
    pub fn source(&self) -> &GpuTransferRegion {
        self.inner.source()
    }

    /// Fresh readback correlation identity for the product submission.
    pub const fn readback_id(&self) -> GpuReadbackId {
        self.inner.readback_id()
    }
}

/// Finite maintained radiance samples interpreted from a product-owned readback.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderCapturedRadiance {
    inner: RenderCapturedDeterministicRadiance,
}

impl RenderCapturedRadiance {
    /// Requested output index whose retained destination was observed.
    pub const fn output_index(&self) -> usize {
        self.inner.output_index()
    }

    /// Semantic sample topology of the captured values.
    pub const fn topology(&self) -> RenderResultTopology {
        self.inner.topology()
    }

    /// Semantic radiometric representation of the captured values.
    pub const fn representation(&self) -> RenderRadiometricRepresentation {
        self.inner.representation()
    }

    /// Row-major finite maintained radiance samples.
    pub fn samples(&self) -> &[f32] {
        self.inner.samples()
    }
}

/// One ordinary maintained render already submitted to RunenGPU.
///
/// This state exposes submission and provenance inspection only. Ordinary submission deliberately
/// authors no semantic-result verification readbacks.
#[derive(Debug)]
pub struct SubmittedRender {
    inner: SubmittedDeterministicRender,
}

impl SubmittedRender {
    /// Exact semantic admission that produced this submission.
    pub const fn admitted_plan(&self) -> &AdmittedRenderPlan {
        self.inner.admitted().admitted()
    }

    /// Current public RunenGPU lifecycle status.
    pub fn submission_status(&self) -> GpuSubmissionStatus {
        self.inner.submission_status()
    }

    /// Decoder for execution-local object-identity carrier values, when requested.
    pub const fn object_identity_decoder(&self) -> &RenderObjectIdentityDecoder {
        self.inner.object_identity_decoder()
    }
}

/// One maintained render submitted with semantic-result verification enabled.
///
/// Callers poll result formation after progressing the public RunenGPU context. Product readback
/// remains a separate optional submission even after semantic result formation succeeds.
#[derive(Debug)]
pub struct SubmittedRenderForResult {
    inner: SubmittedDeterministicRender,
}

impl SubmittedRenderForResult {
    /// Exact semantic admission that produced this submission.
    pub const fn admitted_plan(&self) -> &AdmittedRenderPlan {
        self.inner.admitted().admitted()
    }

    /// Current public RunenGPU lifecycle status.
    pub fn submission_status(&self) -> GpuSubmissionStatus {
        self.inner.submission_status()
    }

    /// Decoder for execution-local object-identity carrier values, when requested.
    pub const fn object_identity_decoder(&self) -> &RenderObjectIdentityDecoder {
        self.inner.object_identity_decoder()
    }

    /// Poll semantic result formation without blocking or driving RunenGPU progress.
    pub fn try_form_result(&mut self) -> Result<Option<RenderResult>, RenderResultFormationError> {
        self.inner
            .try_form_verified_result()
            .map_err(|inner| RenderResultFormationError { inner })
    }

    /// Mint one fresh product-owned readback correlation for a formed radiance output.
    pub fn request_radiance_capture(
        &self,
        output_index: usize,
    ) -> Result<RenderRadianceCaptureRequest, RenderRadianceCaptureRequestError> {
        self.inner
            .request_deterministic_radiance_capture(output_index)
            .map(|inner| RenderRadianceCaptureRequest { inner })
            .map_err(|inner| RenderRadianceCaptureRequestError { inner })
    }

    /// Interpret one completed product-owned RunenGPU readback through the maintained carrier.
    pub fn capture_radiance(
        &self,
        request: RenderRadianceCaptureRequest,
        context: &GpuContext,
        product_submission: &GpuSubmission,
    ) -> Result<RenderCapturedRadiance, RenderRadianceCaptureError> {
        self.inner
            .capture_deterministic_radiance(request.inner, context, product_submission)
            .map(|inner| RenderCapturedRadiance { inner })
            .map_err(|inner| RenderRadianceCaptureError { inner })
    }
}

/// Plan and admit one ordinary invocation of the maintained renderer.
///
/// Callers provide semantic scene/request/input facts, physical output destinations, and a public
/// RunenGPU context. Method selection, planning, binding admission, and maintained compatibility
/// remain inside RunenRender.
pub fn admit_render(
    scene: &RenderSceneSnapshot,
    request: &RenderRequest,
    surface_inputs: &[RenderSurfaceSemanticInputBinding],
    field_inputs: &[RenderFieldSemanticInputBinding],
    availability: &[RenderRepresentationAvailabilityFact],
    output_bindings: &[RenderOutputBinding],
    context: &GpuContext,
) -> Result<AdmittedRender, RenderAdmissionError> {
    admit_deterministic_render_with_semantic_inputs(
        scene,
        request,
        surface_inputs,
        field_inputs,
        availability,
        output_bindings,
        context,
    )
    .map(|inner| AdmittedRender { inner })
    .map_err(|inner| RenderAdmissionError { inner })
}

/// Lower one admitted ordinary render into composable public RunenGPU work without submitting it.
///
/// No CPU readback is authored by this path.
pub fn prepare_render(
    admitted: AdmittedRender,
    context: &GpuContext,
) -> Result<PreparedRender, RenderExecutionError> {
    prepare_deterministic_render(admitted.inner, context)
        .map(|inner| PreparedRender { inner })
        .map_err(|inner| RenderExecutionError { inner })
}

/// Lower and submit one ordinary maintained render through public RunenGPU.
///
/// No semantic-result verification or CPU readback is requested.
pub async fn submit_render(
    admitted: AdmittedRender,
    context: &GpuContext,
) -> Result<SubmittedRender, RenderExecutionError> {
    submit_deterministic_render(admitted.inner, context)
        .await
        .map(|inner| SubmittedRender { inner })
        .map_err(|inner| RenderExecutionError { inner })
}

/// Lower and submit one maintained render with private semantic-result verification enabled.
///
/// The returned type is distinct from SubmittedRender so result formation cannot be requested
/// accidentally from an ordinary readback-free submission.
pub async fn submit_render_for_result(
    admitted: AdmittedRender,
    context: &GpuContext,
) -> Result<SubmittedRenderForResult, RenderResultSubmissionError> {
    submit_deterministic_render_for_verified_result(admitted.inner, context)
        .await
        .map(|inner| SubmittedRenderForResult { inner })
        .map_err(|inner| RenderResultSubmissionError { inner })
}
