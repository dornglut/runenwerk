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
pub use super::deterministic_execution::RenderObjectIdentityDecoder;
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
};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderAdmissionErrorKind {
    Planning,
    Admission,
    Compatibility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderAdmissionError {
    inner: RenderDeterministicAdmissionFailure,
}

impl RenderAdmissionError {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderExecutionErrorKind {
    Lowering,
    Submission,
}

#[derive(Debug)]
pub struct RenderExecutionError {
    inner: RenderDeterministicExecutionError,
}

impl RenderExecutionError {
    pub fn kind(&self) -> RenderExecutionErrorKind {
        match &self.inner {
            RenderDeterministicExecutionError::Lowering(_) => RenderExecutionErrorKind::Lowering,
            RenderDeterministicExecutionError::Submission(_) => {
                RenderExecutionErrorKind::Submission
            }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderVerifiedSubmissionErrorKind {
    Eligibility,
    Execution,
    Correlation,
}

#[derive(Debug)]
pub struct RenderVerifiedSubmissionError {
    inner: RenderDeterministicVerifiedSubmissionError,
}

impl RenderVerifiedSubmissionError {
    pub fn kind(&self) -> RenderVerifiedSubmissionErrorKind {
        match &self.inner {
            RenderDeterministicVerifiedSubmissionError::Eligibility { .. } => {
                RenderVerifiedSubmissionErrorKind::Eligibility
            }
            RenderDeterministicVerifiedSubmissionError::Execution(_) => {
                RenderVerifiedSubmissionErrorKind::Execution
            }
            RenderDeterministicVerifiedSubmissionError::Correlation { .. } => {
                RenderVerifiedSubmissionErrorKind::Correlation
            }
        }
    }
}

impl fmt::Display for RenderVerifiedSubmissionError {
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

impl Error for RenderVerifiedSubmissionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.inner {
            RenderDeterministicVerifiedSubmissionError::Execution(error) => Some(error),
            RenderDeterministicVerifiedSubmissionError::Eligibility { .. }
            | RenderDeterministicVerifiedSubmissionError::Correlation { .. } => None,
        }
    }
}

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

#[derive(Debug)]
pub struct RenderResultFormationError {
    inner: RenderDeterministicResultFormationError,
}

impl RenderResultFormationError {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedRender {
    inner: AdmittedDeterministicRender,
}

impl AdmittedRender {
    pub const fn admitted_plan(&self) -> &AdmittedRenderPlan {
        self.inner.admitted()
    }
}

#[derive(Debug, Clone)]
pub struct PreparedRender {
    inner: PreparedDeterministicRender,
}

impl PreparedRender {
    pub const fn admitted_plan(&self) -> &AdmittedRenderPlan {
        self.inner.admitted().admitted()
    }

    pub const fn work_set(&self) -> &RenderWorkSet {
        self.inner.work_set()
    }

    pub fn radiance_outputs(
        &self,
    ) -> impl ExactSizeIterator<Item = PreparedRadianceOutput<'_>> + '_ {
        self.inner
            .radiance_outputs()
            .iter()
            .map(|inner| PreparedRadianceOutput { inner })
    }

    pub fn radiance_output(&self, output_index: usize) -> Option<PreparedRadianceOutput<'_>> {
        self.inner
            .radiance_output(output_index)
            .map(|inner| PreparedRadianceOutput { inner })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PreparedRadianceOutput<'a> {
    inner: &'a PreparedDeterministicRadianceOutput,
}

impl PreparedRadianceOutput<'_> {
    pub const fn output_index(&self) -> usize {
        self.inner.output_index()
    }

    pub fn resource(&self) -> &GpuResourceRef {
        self.inner.resource()
    }

    pub fn texture(&self) -> Option<&GpuTextureHandle> {
        self.inner.texture()
    }

    pub fn export_relationship(&self) -> &GpuExportRelationship {
        self.inner.export_relationship()
    }

    pub fn import(&self, provenance: GpuResourceProvenance) -> GpuWorkImport {
        self.inner.import(provenance)
    }
}

pub struct RenderRadianceCaptureRequest {
    inner: RenderDeterministicRadianceCaptureRequest,
}

impl RenderRadianceCaptureRequest {
    pub const fn output_index(&self) -> usize {
        self.inner.output_index()
    }

    pub fn source(&self) -> &GpuTransferRegion {
        self.inner.source()
    }

    pub const fn readback_id(&self) -> GpuReadbackId {
        self.inner.readback_id()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderCapturedRadiance {
    inner: RenderCapturedDeterministicRadiance,
}

impl RenderCapturedRadiance {
    pub const fn output_index(&self) -> usize {
        self.inner.output_index()
    }

    pub const fn topology(&self) -> RenderResultTopology {
        self.inner.topology()
    }

    pub const fn representation(&self) -> RenderRadiometricRepresentation {
        self.inner.representation()
    }

    pub fn samples(&self) -> &[f32] {
        self.inner.samples()
    }
}

#[derive(Debug)]
pub struct SubmittedRender {
    inner: SubmittedDeterministicRender,
}

impl SubmittedRender {
    pub const fn admitted_plan(&self) -> &AdmittedRenderPlan {
        self.inner.admitted().admitted()
    }

    pub fn submission_status(&self) -> GpuSubmissionStatus {
        self.inner.submission_status()
    }

    pub const fn object_identity_decoder(&self) -> &RenderObjectIdentityDecoder {
        self.inner.object_identity_decoder()
    }

    pub fn try_form_result(&mut self) -> Result<Option<RenderResult>, RenderResultFormationError> {
        self.inner
            .try_form_verified_result()
            .map_err(|inner| RenderResultFormationError { inner })
    }

    pub fn request_radiance_capture(
        &self,
        output_index: usize,
    ) -> Result<RenderRadianceCaptureRequest, RenderRadianceCaptureRequestError> {
        self.inner
            .request_deterministic_radiance_capture(output_index)
            .map(|inner| RenderRadianceCaptureRequest { inner })
            .map_err(|inner| RenderRadianceCaptureRequestError { inner })
    }

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

pub fn prepare_render(
    admitted: AdmittedRender,
    context: &GpuContext,
) -> Result<PreparedRender, RenderExecutionError> {
    prepare_deterministic_render(admitted.inner, context)
        .map(|inner| PreparedRender { inner })
        .map_err(|inner| RenderExecutionError { inner })
}

pub async fn submit_render(
    admitted: AdmittedRender,
    context: &GpuContext,
) -> Result<SubmittedRender, RenderExecutionError> {
    submit_deterministic_render(admitted.inner, context)
        .await
        .map(|inner| SubmittedRender { inner })
        .map_err(|inner| RenderExecutionError { inner })
}

pub async fn submit_render_for_result(
    admitted: AdmittedRender,
    context: &GpuContext,
) -> Result<SubmittedRender, RenderVerifiedSubmissionError> {
    submit_deterministic_render_for_verified_result(admitted.inner, context)
        .await
        .map(|inner| SubmittedRender { inner })
        .map_err(|inner| RenderVerifiedSubmissionError { inner })
}
