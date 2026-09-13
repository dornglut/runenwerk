//! Private static eligibility, same-submission correlation, physical observation normalization, and
//! conservative semantic verification for RR566-EVAL-001 verified-result formation.
//!
//! This module deliberately does not narrow maintained deterministic execution. The ordinary
//! evaluator accepts every request already proven by `AdmittedDeterministicRender`; this gate asks a
//! separate question: whether the exact admitted semantics lie inside the first bounded domain for
//! which RunenRender is allowed to establish requested finite-evaluation fidelity. Verified
//! submission proves renderer-private readbacks are correlated to one exact accepted RunenGPU
//! submission; observation normalization then removes physical padding; the private semantic
//! verifier finally consumes that exact submission and fails closed on unsupported/ambiguous
//! arithmetic or branch evidence. FORM-001 result formation remains a later owner-controlled step.

mod numeric;
mod observation;
mod semantic;

use observation::observe_completed_deterministic_verification;

use super::deterministic_admission::AdmittedDeterministicRender;
use super::deterministic_execution::{
    self, DeterministicVerificationSubmission, RenderDeterministicExecutionError,
    SubmittedDeterministicRender,
};
use super::request::RenderObservationSpec;
use super::scene::RenderObjectId;
use super::space_time::{RenderAffineTransform3, RenderHandedness, RenderObjectSpatialState};
use runen_gpu::GpuContext;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

/// Canonical f64 threshold selected by the accepted first verification domain.
///
/// This is only a static domain gate. The dynamic semantic verifier must not treat this value,
/// `std::f64::consts::PI`, or host transcendental functions as exact mathematical reference
/// authority.
const CERTIFIED_MAX_FULL_FOV_RADIANS: f64 = std::f64::consts::FRAC_PI_2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RenderDeterministicVerificationEligibilityError {
    PerspectiveFieldOfViewUnsupported { observation_index: usize },
    ObservationLinearBasisUnsupported { observation_index: usize },
    SelectedObjectStateMissing { object_id: RenderObjectId },
    ObjectLocalScaleUnsupported { object_id: RenderObjectId },
    ObjectHandednessUnsupported { object_id: RenderObjectId },
    ObjectLinearBasisUnsupported { object_id: RenderObjectId },
}

impl fmt::Display for RenderDeterministicVerificationEligibilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PerspectiveFieldOfViewUnsupported { observation_index } => write!(
                formatter,
                "observation {observation_index} lies outside the certified perspective field-of-view domain"
            ),
            Self::ObservationLinearBasisUnsupported { observation_index } => write!(
                formatter,
                "observation {observation_index} requires identity linear basis for verified result formation"
            ),
            Self::SelectedObjectStateMissing { object_id } => write!(
                formatter,
                "selected object {object_id:?} has no retained state for verified result formation"
            ),
            Self::ObjectLocalScaleUnsupported { object_id } => write!(
                formatter,
                "selected object {object_id:?} requires unit local-space scale for verified result formation"
            ),
            Self::ObjectHandednessUnsupported { object_id } => write!(
                formatter,
                "selected object {object_id:?} requires right-handed local space for verified result formation"
            ),
            Self::ObjectLinearBasisUnsupported { object_id } => write!(
                formatter,
                "selected object {object_id:?} requires identity linear basis with translation only for verified result formation"
            ),
        }
    }
}

impl Error for RenderDeterministicVerificationEligibilityError {}

#[derive(Debug)]
pub(super) enum RenderDeterministicVerifiedSubmissionError {
    Eligibility(RenderDeterministicVerificationEligibilityError),
    Execution(RenderDeterministicExecutionError),
    ReadbackCardinality {
        expected: usize,
        actual: usize,
    },
    OutputCorrelationChanged {
        expected_output_index: usize,
        actual_output_index: usize,
    },
    DuplicateReadbackCorrelation {
        output_index: usize,
        channel: &'static str,
    },
    MissingSubmissionReadback {
        output_index: usize,
        channel: &'static str,
    },
}

impl fmt::Display for RenderDeterministicVerifiedSubmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eligibility(error) => {
                write!(formatter, "verification eligibility failed: {error}")
            }
            Self::Execution(error) => write!(
                formatter,
                "verified deterministic submission failed: {error}"
            ),
            Self::ReadbackCardinality { expected, actual } => write!(
                formatter,
                "verified deterministic submission retained {actual} output readback correlations for {expected} admitted outputs"
            ),
            Self::OutputCorrelationChanged {
                expected_output_index,
                actual_output_index,
            } => write!(
                formatter,
                "verified deterministic readback correlation changed from output {expected_output_index} to {actual_output_index}"
            ),
            Self::DuplicateReadbackCorrelation {
                output_index,
                channel,
            } => write!(
                formatter,
                "output {output_index} {channel} readback reused a correlation identity"
            ),
            Self::MissingSubmissionReadback {
                output_index,
                channel,
            } => write!(
                formatter,
                "output {output_index} {channel} readback is not owned by the exact returned RunenGPU submission"
            ),
        }
    }
}

impl Error for RenderDeterministicVerifiedSubmissionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Eligibility(error) => Some(error),
            Self::Execution(error) => Some(error),
            Self::ReadbackCardinality { .. }
            | Self::OutputCorrelationChanged { .. }
            | Self::DuplicateReadbackCorrelation { .. }
            | Self::MissingSubmissionReadback { .. } => None,
        }
    }
}

impl From<RenderDeterministicVerificationEligibilityError>
    for RenderDeterministicVerifiedSubmissionError
{
    fn from(value: RenderDeterministicVerificationEligibilityError) -> Self {
        Self::Eligibility(value)
    }
}

impl From<RenderDeterministicExecutionError> for RenderDeterministicVerifiedSubmissionError {
    fn from(value: RenderDeterministicExecutionError) -> Self {
        Self::Execution(value)
    }
}

/// Dynamic RR566-EVAL-001 failure after one exact verified submission has been authored.
///
/// Static-domain rejection remains the eligibility error above. `Inconclusive` is reserved for a
/// semantic branch the conservative interval proof cannot uniquely establish. Physical mismatch and
/// tolerance mismatch mean the completed observation contradicts the certified semantic result.
#[derive(Debug)]
pub(super) enum RenderDeterministicVerificationError {
    Eligibility(RenderDeterministicVerificationEligibilityError),
    ObservationNormalization {
        detail: String,
    },
    Correlation {
        output_index: usize,
        sample_index: Option<usize>,
        detail: &'static str,
    },
    Inconclusive {
        output_index: usize,
        sample_index: Option<usize>,
        detail: &'static str,
    },
    PhysicalMismatch {
        output_index: usize,
        sample_index: Option<usize>,
        detail: &'static str,
    },
    ToleranceMismatch {
        output_index: usize,
        sample_index: usize,
    },
}

impl fmt::Display for RenderDeterministicVerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eligibility(error) => {
                write!(formatter, "verification eligibility failed: {error}")
            }
            Self::ObservationNormalization { detail } => write!(
                formatter,
                "same-submission observation normalization failed: {detail}"
            ),
            Self::Correlation {
                output_index,
                sample_index,
                detail,
            } => format_verification_location(
                formatter,
                "correlation failed",
                *output_index,
                *sample_index,
                detail,
            ),
            Self::Inconclusive {
                output_index,
                sample_index,
                detail,
            } => format_verification_location(
                formatter,
                "verification is inconclusive",
                *output_index,
                *sample_index,
                detail,
            ),
            Self::PhysicalMismatch {
                output_index,
                sample_index,
                detail,
            } => format_verification_location(
                formatter,
                "physical observation mismatch",
                *output_index,
                *sample_index,
                detail,
            ),
            Self::ToleranceMismatch {
                output_index,
                sample_index,
            } => write!(
                formatter,
                "output {output_index} sample {sample_index} does not satisfy the requested semantic tolerance"
            ),
        }
    }
}

impl Error for RenderDeterministicVerificationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Eligibility(error) => Some(error),
            Self::ObservationNormalization { .. }
            | Self::Correlation { .. }
            | Self::Inconclusive { .. }
            | Self::PhysicalMismatch { .. }
            | Self::ToleranceMismatch { .. } => None,
        }
    }
}

fn format_verification_location(
    formatter: &mut fmt::Formatter<'_>,
    category: &'static str,
    output_index: usize,
    sample_index: Option<usize>,
    detail: &'static str,
) -> fmt::Result {
    if let Some(sample_index) = sample_index {
        write!(
            formatter,
            "output {output_index} sample {sample_index} {category}: {detail}"
        )
    } else {
        write!(formatter, "output {output_index} {category}: {detail}")
    }
}

/// Execution-scoped proof that one exact verified submission satisfied RR566-EVAL-001.
///
/// The private verification submission is owned, not projected into detached output-index tokens.
/// This keeps later FORM-001 evidence bound to the exact admitted semantics, physical decoder, and
/// RunenGPU submission that were actually observed. GPU/readback identity still does not become
/// public `RenderResult` identity.
#[derive(Debug)]
pub(super) struct VerifiedDeterministicRender {
    verification: DeterministicVerificationSubmission,
}

impl VerifiedDeterministicRender {
    pub(super) const fn submitted(&self) -> &SubmittedDeterministicRender {
        self.verification.submitted()
    }
}

/// Prove only the static RR566-EVAL-001 subset that is knowable before one verified submission.
///
/// Instant shutters and ideal-ray sampling are already invariants of `AdmittedDeterministicRender`.
/// This gate therefore adds only verifier-specific restrictions; failure here must never make the
/// same admitted invocation illegal for ordinary maintained execution.
pub(super) fn ensure_deterministic_verification_eligible(
    maintained: &AdmittedDeterministicRender,
) -> Result<(), RenderDeterministicVerificationEligibilityError> {
    let admitted = maintained.admitted();
    for (observation_index, observation) in admitted
        .plan()
        .request()
        .observations()
        .iter()
        .copied()
        .enumerate()
    {
        validate_observation(observation_index, observation)?;
    }

    let mut selected_objects = BTreeSet::new();
    for output in admitted.outputs() {
        for object in output.object_representations() {
            selected_objects.insert(object.object_id());
        }
    }

    for object_id in selected_objects {
        let state = admitted.plan().scene().object_state(object_id).ok_or(
            RenderDeterministicVerificationEligibilityError::SelectedObjectStateMissing {
                object_id,
            },
        )?;
        validate_object_spatial_state(object_id, state.spatial())?;
    }

    Ok(())
}

/// Select verified-result intent before submission, then prove that every private observation
/// correlation resolves back into that exact returned `GpuSubmission`.
///
/// This function intentionally stops before readback readiness or semantic comparison. A successful
/// return proves only the static EVAL-001 domain gate plus same-submission correlation for canonical
/// payload, semantic-definedness, and evaluator-status observations.
pub(super) async fn submit_deterministic_render_for_verified_formation(
    maintained: AdmittedDeterministicRender,
    context: &GpuContext,
) -> Result<DeterministicVerificationSubmission, RenderDeterministicVerifiedSubmissionError> {
    ensure_deterministic_verification_eligible(&maintained)?;
    let verification =
        deterministic_execution::submit_deterministic_render_for_verification(maintained, context)
            .await?;

    let submitted = verification.submitted();
    let expected_outputs = submitted.admitted().admitted().outputs();
    let readbacks = verification.readbacks();
    if readbacks.len() != expected_outputs.len() {
        return Err(
            RenderDeterministicVerifiedSubmissionError::ReadbackCardinality {
                expected: expected_outputs.len(),
                actual: readbacks.len(),
            },
        );
    }

    let mut correlation_ids = BTreeSet::new();
    for (expected_output, readbacks) in expected_outputs.iter().zip(readbacks) {
        if readbacks.output_index() != expected_output.output_index() {
            return Err(
                RenderDeterministicVerifiedSubmissionError::OutputCorrelationChanged {
                    expected_output_index: expected_output.output_index(),
                    actual_output_index: readbacks.output_index(),
                },
            );
        }

        for (channel, id) in [
            ("canonical-output", readbacks.canonical_output()),
            ("definedness", readbacks.definedness()),
            ("evaluator-status", readbacks.status()),
        ] {
            if !correlation_ids.insert(id) {
                return Err(
                    RenderDeterministicVerifiedSubmissionError::DuplicateReadbackCorrelation {
                        output_index: readbacks.output_index(),
                        channel,
                    },
                );
            }
            if submitted.submission().readback(id).is_none() {
                return Err(
                    RenderDeterministicVerifiedSubmissionError::MissingSubmissionReadback {
                        output_index: readbacks.output_index(),
                        channel,
                    },
                );
            }
        }
    }

    Ok(verification)
}

/// Consume one completed same-submission observation set and establish the first conservative
/// deterministic finite-evaluation witness.
///
/// Success does not yet form `RenderResult`; it returns an execution-scoped owner-controlled witness
/// that later FORM-001 wiring can consume without accepting detached output-index assertions.
pub(super) fn verify_completed_deterministic_render(
    verification: DeterministicVerificationSubmission,
) -> Result<VerifiedDeterministicRender, RenderDeterministicVerificationError> {
    ensure_deterministic_verification_eligible(verification.submitted().admitted())
        .map_err(RenderDeterministicVerificationError::Eligibility)?;
    let observations =
        observe_completed_deterministic_verification(&verification).map_err(|error| {
            RenderDeterministicVerificationError::ObservationNormalization {
                detail: format!("{error:?}"),
            }
        })?;
    semantic::verify_completed_semantics(&verification, &observations)?;
    Ok(VerifiedDeterministicRender { verification })
}

fn validate_observation(
    observation_index: usize,
    observation: RenderObservationSpec,
) -> Result<(), RenderDeterministicVerificationEligibilityError> {
    let transform = match observation {
        RenderObservationSpec::Perspective(perspective) => {
            if perspective.vertical_field_of_view_radians() > CERTIFIED_MAX_FULL_FOV_RADIANS {
                return Err(
                    RenderDeterministicVerificationEligibilityError::PerspectiveFieldOfViewUnsupported {
                        observation_index,
                    },
                );
            }
            perspective.observation_to_scene()
        }
        RenderObservationSpec::Probe(probe) => probe.observation_to_scene(),
    };

    if !has_identity_linear_basis(transform) {
        return Err(
            RenderDeterministicVerificationEligibilityError::ObservationLinearBasisUnsupported {
                observation_index,
            },
        );
    }
    Ok(())
}

fn validate_object_spatial_state(
    object_id: RenderObjectId,
    spatial: &RenderObjectSpatialState,
) -> Result<(), RenderDeterministicVerificationEligibilityError> {
    let local_space = spatial.local_space();
    if local_space.meters_per_unit() != 1.0 {
        return Err(
            RenderDeterministicVerificationEligibilityError::ObjectLocalScaleUnsupported {
                object_id,
            },
        );
    }
    if local_space.handedness() != RenderHandedness::Right {
        return Err(
            RenderDeterministicVerificationEligibilityError::ObjectHandednessUnsupported {
                object_id,
            },
        );
    }
    if !has_identity_linear_basis(spatial.local_to_scene()) {
        return Err(
            RenderDeterministicVerificationEligibilityError::ObjectLinearBasisUnsupported {
                object_id,
            },
        );
    }
    Ok(())
}

fn has_identity_linear_basis(transform: RenderAffineTransform3) -> bool {
    let matrix = transform.row_major_3x4();
    [
        matrix[0], matrix[1], matrix[2], matrix[4], matrix[5], matrix[6], matrix[8], matrix[9],
        matrix[10],
    ] == [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::request::{
        RenderPerspectiveObservation, RenderProbeObservation, RenderSamplingSupport,
    };
    use crate::plugins::render::space_time::{
        RenderSemanticValueError, RenderSpaceSpec, RenderSpatialCoverage, RenderTimeInterval,
        RenderTimePoint,
    };

    fn instant() -> RenderTimeInterval {
        RenderTimeInterval::instant(RenderTimePoint::from_seconds(0.0).expect("time"))
    }

    fn translated_identity(x: f64, y: f64, z: f64) -> RenderAffineTransform3 {
        RenderAffineTransform3::from_row_major_3x4([
            1.0, 0.0, 0.0, x, 0.0, 1.0, 0.0, y, 0.0, 0.0, 1.0, z,
        ])
        .expect("translated identity")
    }

    fn rotated_basis() -> RenderAffineTransform3 {
        RenderAffineTransform3::from_row_major_3x4([
            0.0, -1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
        ])
        .expect("rotation")
    }

    #[test]
    fn observation_gate_accepts_translation_but_not_non_identity_linear_basis() {
        let perspective = RenderObservationSpec::Perspective(
            RenderPerspectiveObservation::new(
                translated_identity(1.0, 2.0, 3.0),
                std::f64::consts::FRAC_PI_4,
                1.0,
                instant(),
                RenderSamplingSupport::ideal_ray(),
            )
            .expect("perspective"),
        );
        assert_eq!(validate_observation(0, perspective), Ok(()));

        let probe = RenderObservationSpec::Probe(
            RenderProbeObservation::new(
                rotated_basis(),
                instant(),
                RenderSamplingSupport::ideal_ray(),
            )
            .expect("probe"),
        );
        assert_eq!(
            validate_observation(2, probe),
            Err(
                RenderDeterministicVerificationEligibilityError::ObservationLinearBasisUnsupported {
                    observation_index: 2,
                }
            )
        );
    }

    #[test]
    fn perspective_gate_rejects_fov_outside_first_certified_domain() {
        let observation = RenderObservationSpec::Perspective(
            RenderPerspectiveObservation::new(
                RenderAffineTransform3::identity(),
                std::f64::consts::FRAC_PI_2 * 1.5,
                1.0,
                instant(),
                RenderSamplingSupport::ideal_ray(),
            )
            .expect("semantically legal wider perspective"),
        );
        assert_eq!(
            validate_observation(1, observation),
            Err(
                RenderDeterministicVerificationEligibilityError::PerspectiveFieldOfViewUnsupported {
                    observation_index: 1,
                }
            )
        );
    }

    #[test]
    fn object_gate_accepts_unit_right_handed_translation_only_state() {
        let spatial = RenderObjectSpatialState::new(
            RenderSpaceSpec::new(1.0, RenderHandedness::Right).expect("space"),
            translated_identity(4.0, 5.0, 6.0),
            RenderSpatialCoverage::unbounded(),
        );
        let mut store = crate::plugins::render::scene::RenderSceneStore::new();
        let object_id = store.allocate_object_id().expect("object id");
        assert_eq!(validate_object_spatial_state(object_id, &spatial), Ok(()));
    }

    #[test]
    fn object_gate_rejects_scale_handedness_and_linear_basis_independently() {
        let mut store = crate::plugins::render::scene::RenderSceneStore::new();
        let object_id = store.allocate_object_id().expect("object id");

        let scaled = RenderObjectSpatialState::new(
            RenderSpaceSpec::new(2.0, RenderHandedness::Right).expect("space"),
            RenderAffineTransform3::identity(),
            RenderSpatialCoverage::unbounded(),
        );
        assert_eq!(
            validate_object_spatial_state(object_id, &scaled),
            Err(
                RenderDeterministicVerificationEligibilityError::ObjectLocalScaleUnsupported {
                    object_id,
                }
            )
        );

        let left_handed = RenderObjectSpatialState::new(
            RenderSpaceSpec::new(1.0, RenderHandedness::Left).expect("space"),
            RenderAffineTransform3::identity(),
            RenderSpatialCoverage::unbounded(),
        );
        assert_eq!(
            validate_object_spatial_state(object_id, &left_handed),
            Err(
                RenderDeterministicVerificationEligibilityError::ObjectHandednessUnsupported {
                    object_id,
                }
            )
        );

        let rotated = RenderObjectSpatialState::new(
            RenderSpaceSpec::new(1.0, RenderHandedness::Right).expect("space"),
            rotated_basis(),
            RenderSpatialCoverage::unbounded(),
        );
        assert_eq!(
            validate_object_spatial_state(object_id, &rotated),
            Err(
                RenderDeterministicVerificationEligibilityError::ObjectLinearBasisUnsupported {
                    object_id,
                }
            )
        );
    }

    #[test]
    fn semantic_transform_constructors_remain_the_finiteness_authority() {
        assert_eq!(
            RenderAffineTransform3::from_row_major_3x4([
                1.0,
                0.0,
                0.0,
                f64::INFINITY,
                0.0,
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
            ]),
            Err(RenderSemanticValueError::NonFinite {
                field: "affine_transform"
            })
        );
    }
}
