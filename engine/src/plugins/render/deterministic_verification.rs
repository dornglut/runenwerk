//! Private static eligibility gate for RR566-EVAL-001 verified-result formation.
//!
//! This module deliberately does not narrow maintained deterministic execution. The ordinary
//! evaluator accepts every request already proven by `AdmittedDeterministicRender`; this gate asks a
//! separate question: whether the exact admitted semantics lie inside the first bounded domain for
//! which RunenRender is allowed to attempt conservative finite-evaluation verification.

use super::deterministic_admission::AdmittedDeterministicRender;
use super::request::RenderObservationSpec;
use super::scene::RenderObjectId;
use super::space_time::{
    RenderAffineTransform3, RenderHandedness, RenderObjectSpatialState,
};
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
    PerspectiveFieldOfViewUnsupported {
        observation_index: usize,
    },
    ObservationLinearBasisUnsupported {
        observation_index: usize,
    },
    SelectedObjectStateMissing {
        object_id: RenderObjectId,
    },
    ObjectLocalScaleUnsupported {
        object_id: RenderObjectId,
    },
    ObjectHandednessUnsupported {
        object_id: RenderObjectId,
    },
    ObjectLinearBasisUnsupported {
        object_id: RenderObjectId,
    },
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
        let state = admitted
            .plan()
            .scene()
            .object_state(object_id)
            .ok_or(
                RenderDeterministicVerificationEligibilityError::SelectedObjectStateMissing {
                    object_id,
                },
            )?;
        validate_object_spatial_state(object_id, state.spatial())?;
    }

    Ok(())
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
