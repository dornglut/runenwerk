//! Owner-controlled semantic planning and R5 admission for the maintained deterministic renderer.
//!
//! A caller-created `RenderMethodContract` cannot become maintained execution authority merely by
//! reusing the renderer-local method ID. RunenRender selects the exact method contract and admits
//! the resulting ordinary plan before applying evaluator/physical-realization compatibility.

use super::admission::{
    AdmittedRenderPlan, RenderExecutionAdmissionFailure, RenderOutputBinding,
    RenderOutputDestination, RenderRepresentationAvailabilityFact,
    admit_render_plan_with_surface_inputs,
};
use super::maintained_method::maintained_deterministic_method;
use super::request::{RenderObservationSpec, RenderRequest};
use super::scene::RenderSceneSnapshot;
use super::semantic_plan::{RenderPlanningFailure, plan_render};
use super::surface_input::RenderSurfaceSemanticInputBinding;
use runen_gpu::{
    GpuBufferUsage, GpuCapabilityFeature, GpuContext, GpuTextureFormat, GpuTextureUsage,
};
use std::error::Error;
use std::fmt;

const SCALAR_CARRIER_BYTES: u64 = 4;
const LATTICE_CARRIER_FORMAT: GpuTextureFormat = GpuTextureFormat::R32Uint;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderDeterministicCompatibilityError {
    ObservationShutterNotInstant { observation_index: usize },
    ObservationSamplingSupportUnsupported { observation_index: usize },
    CopyCapabilityUnsupported,
    CopyCapabilityNotEnabled,
    LatticeCarrierFormatUnsupported,
    LatticeCarrierCopyDestinationUnsupported,
    ScalarDestinationSize {
        output_index: usize,
        expected_size_bytes: u64,
        actual_size_bytes: u64,
    },
    ScalarDestinationNotCopyDestination { output_index: usize },
    LatticeDestinationFormat {
        output_index: usize,
        expected: GpuTextureFormat,
        actual: GpuTextureFormat,
    },
    LatticeDestinationNotCopyDestination { output_index: usize },
}

impl fmt::Display for RenderDeterministicCompatibilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ObservationShutterNotInstant { observation_index } => write!(
                formatter,
                "observation {observation_index} requires an instant shutter"
            ),
            Self::ObservationSamplingSupportUnsupported { observation_index } => write!(
                formatter,
                "observation {observation_index} requires ideal-ray sampling support"
            ),
            Self::CopyCapabilityUnsupported => {
                formatter.write_str("RunenGPU copy capability is unsupported")
            }
            Self::CopyCapabilityNotEnabled => {
                formatter.write_str("RunenGPU copy capability is not enabled")
            }
            Self::LatticeCarrierFormatUnsupported => write!(
                formatter,
                "{LATTICE_CARRIER_FORMAT:?} lattice carrier is unsupported"
            ),
            Self::LatticeCarrierCopyDestinationUnsupported => write!(
                formatter,
                "{LATTICE_CARRIER_FORMAT:?} lacks copy-destination support"
            ),
            Self::ScalarDestinationSize {
                output_index,
                expected_size_bytes,
                actual_size_bytes,
            } => write!(
                formatter,
                "scalar output {output_index} requires {expected_size_bytes} destination bytes, got {actual_size_bytes}"
            ),
            Self::ScalarDestinationNotCopyDestination { output_index } => write!(
                formatter,
                "scalar output {output_index} destination lacks CopyDestination usage"
            ),
            Self::LatticeDestinationFormat {
                output_index,
                expected,
                actual,
            } => write!(
                formatter,
                "lattice output {output_index} requires {expected:?}, got {actual:?}"
            ),
            Self::LatticeDestinationNotCopyDestination { output_index } => write!(
                formatter,
                "lattice output {output_index} destination lacks CopyDestination usage"
            ),
        }
    }
}

impl Error for RenderDeterministicCompatibilityError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderDeterministicAdmissionFailure {
    Planning(RenderPlanningFailure),
    Admission(RenderExecutionAdmissionFailure),
    Compatibility(RenderDeterministicCompatibilityError),
}

impl fmt::Display for RenderDeterministicAdmissionFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Planning(error) => {
                write!(formatter, "deterministic render planning failed: {error}")
            }
            Self::Admission(error) => {
                write!(formatter, "deterministic render admission failed: {error}")
            }
            Self::Compatibility(error) => {
                write!(formatter, "deterministic renderer compatibility failed: {error}")
            }
        }
    }
}

impl Error for RenderDeterministicAdmissionFailure {}

/// R5-admitted work for the exact RunenRender-owned maintained deterministic method.
///
/// The wrapper carries no submission/session identity. Construction additionally proves that the
/// admitted request and physical destinations fit the currently maintained evaluator/carrier
/// contract; RunenGPU still owns graph preparation, realization, submission, and execution failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedDeterministicRender {
    admitted: AdmittedRenderPlan,
}

impl AdmittedDeterministicRender {
    pub const fn admitted(&self) -> &AdmittedRenderPlan {
        &self.admitted
    }
}

pub fn admit_deterministic_render(
    scene: &RenderSceneSnapshot,
    request: &RenderRequest,
    semantic_inputs: &[RenderSurfaceSemanticInputBinding],
    availability: &[RenderRepresentationAvailabilityFact],
    output_bindings: &[RenderOutputBinding],
    context: &GpuContext,
) -> Result<AdmittedDeterministicRender, RenderDeterministicAdmissionFailure> {
    let method = maintained_deterministic_method();
    let plan = plan_render(scene, request, std::slice::from_ref(&method))
        .map_err(RenderDeterministicAdmissionFailure::Planning)?;
    let admitted = admit_render_plan_with_surface_inputs(
        &plan,
        semantic_inputs,
        availability,
        output_bindings,
        context,
    )
    .map_err(RenderDeterministicAdmissionFailure::Admission)?;
    validate_maintained_compatibility(&admitted, context)
        .map_err(RenderDeterministicAdmissionFailure::Compatibility)?;
    Ok(AdmittedDeterministicRender { admitted })
}

fn validate_maintained_compatibility(
    admitted: &AdmittedRenderPlan,
    context: &GpuContext,
) -> Result<(), RenderDeterministicCompatibilityError> {
    for (observation_index, observation) in admitted
        .plan()
        .request()
        .observations()
        .iter()
        .enumerate()
    {
        validate_observation(observation_index, *observation)?;
    }

    let capabilities = context.adapter_facts().supported();
    if !capabilities.supports(GpuCapabilityFeature::Copy) {
        return Err(RenderDeterministicCompatibilityError::CopyCapabilityUnsupported);
    }
    if !context.device_facts().is_enabled(GpuCapabilityFeature::Copy) {
        return Err(RenderDeterministicCompatibilityError::CopyCapabilityNotEnabled);
    }

    if admitted.outputs().iter().any(|output| {
        matches!(
            output.binding().destination(),
            RenderOutputDestination::SampleLatticeTexture(_)
        )
    }) {
        let Some(format_capabilities) = capabilities.format(LATTICE_CARRIER_FORMAT) else {
            return Err(RenderDeterministicCompatibilityError::LatticeCarrierFormatUnsupported);
        };
        if !format_capabilities.copy_destination {
            return Err(
                RenderDeterministicCompatibilityError::LatticeCarrierCopyDestinationUnsupported,
            );
        }
    }

    for output in admitted.outputs() {
        validate_destination(output.output_index(), output.binding().destination())?;
    }
    Ok(())
}

fn validate_observation(
    observation_index: usize,
    observation: RenderObservationSpec,
) -> Result<(), RenderDeterministicCompatibilityError> {
    let shutter = observation.shutter();
    if shutter.start() != shutter.end() {
        return Err(RenderDeterministicCompatibilityError::ObservationShutterNotInstant {
            observation_index,
        });
    }

    let sampling_support = match observation {
        RenderObservationSpec::Perspective(observation) => observation.sampling_support(),
        RenderObservationSpec::Probe(observation) => observation.sampling_support(),
    };
    if !sampling_support.is_ideal_ray() {
        return Err(
            RenderDeterministicCompatibilityError::ObservationSamplingSupportUnsupported {
                observation_index,
            },
        );
    }
    Ok(())
}

fn validate_destination(
    output_index: usize,
    destination: &RenderOutputDestination,
) -> Result<(), RenderDeterministicCompatibilityError> {
    match destination {
        RenderOutputDestination::ScalarBuffer(buffer) => {
            let descriptor = buffer.descriptor();
            if descriptor.size_bytes() != SCALAR_CARRIER_BYTES {
                return Err(RenderDeterministicCompatibilityError::ScalarDestinationSize {
                    output_index,
                    expected_size_bytes: SCALAR_CARRIER_BYTES,
                    actual_size_bytes: descriptor.size_bytes(),
                });
            }
            if !descriptor.usages().contains(GpuBufferUsage::CopyDestination) {
                return Err(
                    RenderDeterministicCompatibilityError::ScalarDestinationNotCopyDestination {
                        output_index,
                    },
                );
            }
        }
        RenderOutputDestination::SampleLatticeTexture(texture) => {
            let descriptor = texture.descriptor();
            if descriptor.format() != LATTICE_CARRIER_FORMAT {
                return Err(RenderDeterministicCompatibilityError::LatticeDestinationFormat {
                    output_index,
                    expected: LATTICE_CARRIER_FORMAT,
                    actual: descriptor.format(),
                });
            }
            if !descriptor.usages().contains(GpuTextureUsage::CopyDestination) {
                return Err(
                    RenderDeterministicCompatibilityError::LatticeDestinationNotCopyDestination {
                        output_index,
                    },
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::method::{
        RenderAbstractExecutionRequirement, RenderMethodContract, RenderMethodId,
        RenderMethodOutputContract, RenderMethodOutputGuarantee, RenderMethodOutputKind,
        RenderObservationKind, RenderSpectralRadianceSupport,
    };
    use crate::plugins::render::request::{
        RenderDistanceConvention, RenderPerspectiveObservation, RenderProbeObservation,
        RenderSamplingSupport,
    };
    use crate::plugins::render::space_time::{
        RenderAffineTransform3, RenderTimeInterval, RenderTimePoint,
    };
    use runen_gpu::{
        GpuBufferDescriptor, GpuBufferInitialization, GpuBufferUsage, GpuReconstruction,
        GpuResourceLifetime, GpuTextureDescriptor, GpuTextureFormat, GpuTextureInitialization,
        GpuTextureUsage, GpuWorkResourceIdAllocator,
    };

    fn time(seconds: f64) -> RenderTimePoint {
        RenderTimePoint::from_seconds(seconds).expect("finite deterministic-admission test time")
    }

    fn instant() -> RenderTimeInterval {
        RenderTimeInterval::instant(time(0.0))
    }

    #[test]
    fn same_method_id_is_not_the_maintained_method_authority() {
        let spectral = RenderSpectralRadianceSupport::new(500e-9, 600e-9).expect("range");
        let impostor = RenderMethodContract::new(
            RenderMethodId::new(1).expect("id"),
            vec![
                RenderMethodOutputContract::new(
                    RenderObservationKind::Probe,
                    RenderMethodOutputKind::Radiance { spectral },
                    RenderMethodOutputGuarantee::Exact,
                    vec![],
                    false,
                )
                .expect("output"),
            ],
            vec![RenderAbstractExecutionRequirement::GeneralParallelWork],
        )
        .expect("impostor contract");
        let maintained = maintained_deterministic_method();
        assert_eq!(impostor.id(), maintained.id());
        assert_ne!(impostor, maintained);
        assert!(
            maintained
                .output_contracts()
                .iter()
                .any(|contract| matches!(
                    contract.output_kind(),
                    RenderMethodOutputKind::Distance {
                        convention: RenderDistanceConvention::ObservationForwardDepth
                    }
                ))
        );
    }

    #[test]
    fn maintained_evaluator_rejects_noninstant_and_nonideal_observations() {
        let interval =
            RenderTimeInterval::new(time(0.0), time(1.0)).expect("ordered test interval");
        let noninstant = RenderObservationSpec::Probe(
            RenderProbeObservation::new(
                RenderAffineTransform3::identity(),
                interval,
                RenderSamplingSupport::ideal_ray(),
            )
            .expect("valid probe"),
        );
        assert_eq!(
            validate_observation(3, noninstant),
            Err(RenderDeterministicCompatibilityError::ObservationShutterNotInstant {
                observation_index: 3
            })
        );

        let cone = RenderObservationSpec::Perspective(
            RenderPerspectiveObservation::new(
                RenderAffineTransform3::identity(),
                std::f64::consts::FRAC_PI_4,
                1.0,
                instant(),
                RenderSamplingSupport::cone(0.01).expect("valid cone"),
            )
            .expect("valid perspective"),
        );
        assert_eq!(
            validate_observation(4, cone),
            Err(
                RenderDeterministicCompatibilityError::ObservationSamplingSupportUnsupported {
                    observation_index: 4
                }
            )
        );
    }

    #[test]
    fn maintained_destination_contract_is_one_word_and_copy_destination() {
        let mut allocator = GpuWorkResourceIdAllocator::new();

        let valid_scalar = allocator
            .allocate_buffer_handle(
                GpuBufferDescriptor::ordinary_owned(
                    "valid maintained scalar",
                    GpuResourceLifetime::Transient,
                    GpuReconstruction::SourceBacked,
                    4,
                    [GpuBufferUsage::CopyDestination],
                    GpuBufferInitialization::Uninitialized,
                )
                .expect("valid scalar descriptor"),
            )
            .expect("scalar handle");
        assert_eq!(
            validate_destination(0, &RenderOutputDestination::ScalarBuffer(valid_scalar)),
            Ok(())
        );

        let oversized_scalar = allocator
            .allocate_buffer_handle(
                GpuBufferDescriptor::ordinary_owned(
                    "oversized maintained scalar",
                    GpuResourceLifetime::Transient,
                    GpuReconstruction::SourceBacked,
                    8,
                    [GpuBufferUsage::CopyDestination],
                    GpuBufferInitialization::Uninitialized,
                )
                .expect("valid oversized descriptor"),
            )
            .expect("oversized scalar handle");
        assert_eq!(
            validate_destination(
                1,
                &RenderOutputDestination::ScalarBuffer(oversized_scalar)
            ),
            Err(RenderDeterministicCompatibilityError::ScalarDestinationSize {
                output_index: 1,
                expected_size_bytes: 4,
                actual_size_bytes: 8,
            })
        );

        let storage_only_scalar = allocator
            .allocate_buffer_handle(
                GpuBufferDescriptor::ordinary_owned(
                    "storage-only maintained scalar",
                    GpuResourceLifetime::Transient,
                    GpuReconstruction::SourceBacked,
                    4,
                    [GpuBufferUsage::Storage],
                    GpuBufferInitialization::Uninitialized,
                )
                .expect("valid storage-only scalar descriptor"),
            )
            .expect("storage-only scalar handle");
        assert_eq!(
            validate_destination(
                2,
                &RenderOutputDestination::ScalarBuffer(storage_only_scalar)
            ),
            Err(
                RenderDeterministicCompatibilityError::ScalarDestinationNotCopyDestination {
                    output_index: 2
                }
            )
        );

        let wrong_format = allocator
            .allocate_texture_handle(
                GpuTextureDescriptor::ordinary_owned_2d(
                    "wrong maintained lattice format",
                    GpuResourceLifetime::Transient,
                    GpuReconstruction::SourceBacked,
                    2,
                    2,
                    GpuTextureFormat::Rgba8Unorm,
                    [GpuTextureUsage::CopyDestination],
                    GpuTextureInitialization::Uninitialized,
                )
                .expect("valid wrong-format descriptor"),
            )
            .expect("wrong-format texture handle");
        assert_eq!(
            validate_destination(
                3,
                &RenderOutputDestination::SampleLatticeTexture(wrong_format)
            ),
            Err(RenderDeterministicCompatibilityError::LatticeDestinationFormat {
                output_index: 3,
                expected: GpuTextureFormat::R32Uint,
                actual: GpuTextureFormat::Rgba8Unorm,
            })
        );

        let storage_only_lattice = allocator
            .allocate_texture_handle(
                GpuTextureDescriptor::ordinary_owned_2d(
                    "storage-only maintained lattice",
                    GpuResourceLifetime::Transient,
                    GpuReconstruction::SourceBacked,
                    2,
                    2,
                    GpuTextureFormat::R32Uint,
                    [GpuTextureUsage::StorageWrite],
                    GpuTextureInitialization::Uninitialized,
                )
                .expect("valid storage-only lattice descriptor"),
            )
            .expect("storage-only lattice handle");
        assert_eq!(
            validate_destination(
                4,
                &RenderOutputDestination::SampleLatticeTexture(storage_only_lattice)
            ),
            Err(
                RenderDeterministicCompatibilityError::LatticeDestinationNotCopyDestination {
                    output_index: 4
                }
            )
        );
    }
}
