from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected exactly one occurrence, found {count}: {old!r}")
    p.write_text(text.replace(old, new, 1))


replace_once(
    "Cargo.toml",
    'runen-gpu = { git = "https://github.com/dornglut/runen-gpu", rev = "77c7c8d5ad6922b6f46c6b25e31b1a224c1314a4" }',
    'runen-gpu = { git = "https://github.com/dornglut/runen-gpu", rev = "31649491e9e7746da8e128ad984d2be315961640" }',
)

replace_once(
    "engine/src/plugins/render/mod.rs",
    "pub mod deterministic_admission;\npub mod deterministic_execution;",
    "pub mod deterministic_admission;\nmod deterministic_carrier;\npub mod deterministic_execution;",
)

replace_once(
    "engine/src/plugins/render/deterministic_execution.rs",
    "use super::admission::{AdmittedRenderPlan, RenderOutputDestination};",
    "mod capture;\n\npub use capture::{\n    RenderCapturedDeterministicRadiance, RenderDeterministicRadianceCaptureError,\n};\n\nuse super::admission::{AdmittedRenderPlan, RenderOutputDestination};",
)

replace_once(
    "engine/src/plugins/render/deterministic_verification/observation.rs",
    "use super::super::deterministic_execution::DeterministicVerificationSubmission;\nuse super::super::request::RenderResultTopology;",
    "use super::super::deterministic_carrier::{WORD_BYTES, decode_word};\nuse super::super::deterministic_execution::DeterministicVerificationSubmission;\nuse super::super::request::RenderResultTopology;",
)
replace_once(
    "engine/src/plugins/render/deterministic_verification/observation.rs",
    "\nconst WORD_BYTES: usize = 4;\n",
    "\n",
)
replace_once(
    "engine/src/plugins/render/deterministic_verification/observation.rs",
    "\nfn decode_word(bytes: &[u8]) -> u32 {\n    debug_assert_eq!(bytes.len(), WORD_BYTES);\n    u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])\n}\n",
    "\n",
)

replace_once(
    "engine/src/plugins/render/deterministic_verification/semantic.rs",
    "use super::super::deterministic_execution::{\n    DeterministicVerificationSubmission, RenderObjectIdentityDecoder,\n};",
    "use super::super::deterministic_carrier::decode_finite_f32;\nuse super::super::deterministic_execution::{\n    DeterministicVerificationSubmission, RenderObjectIdentityDecoder,\n};",
)
replace_once(
    "engine/src/plugins/render/deterministic_verification/semantic.rs",
    "    let observed = f64::from(f32::from_bits(canonical_word));\n    if !numeric_value_satisfies_tolerance(observed, reference, tolerance) {",
    "    let observed = decode_finite_f32(canonical_word).ok_or_else(|| {\n        physical_mismatch_error(\n            output_index,\n            Some(sample_index),\n            \"numeric carrier decoded to a non-finite maintained value\",\n        )\n    })?;\n    let observed = f64::from(observed);\n    if !numeric_value_satisfies_tolerance(observed, reference, tolerance) {",
)

Path("engine/src/plugins/render/deterministic_carrier.rs").write_text('''//! Private physical carrier helpers for the maintained deterministic evaluator.
//!
//! These helpers centralize the current one-word transport realization without promoting it to
//! renderer-semantic identity or a generic public codec contract.

pub(super) const WORD_BYTES: usize = size_of::<u32>();

pub(super) fn decode_word(bytes: &[u8]) -> u32 {
    debug_assert_eq!(bytes.len(), WORD_BYTES);
    let bytes: [u8; WORD_BYTES] = bytes
        .try_into()
        .expect("maintained deterministic word decoding requires one exact carrier word");
    u32::from_ne_bytes(bytes)
}

pub(super) fn decode_finite_f32(word: u32) -> Option<f32> {
    let value = f32::from_bits(word);
    value.is_finite().then_some(value)
}
''')

capture_dir = Path("engine/src/plugins/render/deterministic_execution")
capture_dir.mkdir(exist_ok=True)
(capture_dir / "capture.rs").write_text('''//! Product-safe observation of separately captured maintained deterministic radiance output.
//!
//! RunenRender owns interpretation of its maintained physical carrier. RunenGPU owns only generic
//! readback transport, while products remain responsible for visualization and artifact policy.
//! This boundary observes already-verified output; it does not mint or replace `RenderResult`.

use super::{DeterministicVerificationState, SubmittedDeterministicRender};
use crate::plugins::render::admission::RenderOutputDestination;
use crate::plugins::render::deterministic_carrier::{WORD_BYTES, decode_finite_f32, decode_word};
use crate::plugins::render::request::{
    RenderOutputValue, RenderRadiometricRepresentation, RenderResultTopology,
};
use runen_gpu::{
    GpuOpaqueContentContinuity, GpuReadback, GpuReadbackStatus, GpuResourceRef,
    GpuRetainedResourceContinuity, GpuSubmissionFailureKind, GpuTextureAspect,
    GpuTextureCopyRegion, GpuTextureFormat, GpuTransferRegion,
};
use std::error::Error;
use std::fmt;

/// Finite maintained radiance samples captured from one exact verified deterministic output.
///
/// Samples are row-major for the retained semantic 2D lattice. The values remain observations of
/// the maintained execution and never become a second semantic-result authority.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderCapturedDeterministicRadiance {
    output_index: usize,
    topology: RenderResultTopology,
    representation: RenderRadiometricRepresentation,
    samples: Vec<f32>,
}

impl RenderCapturedDeterministicRadiance {
    pub const fn output_index(&self) -> usize {
        self.output_index
    }

    pub const fn topology(&self) -> RenderResultTopology {
        self.topology
    }

    pub const fn representation(&self) -> RenderRadiometricRepresentation {
        self.representation
    }

    pub fn samples(&self) -> &[f32] {
        &self.samples
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderDeterministicRadianceCaptureError {
    VerificationNotFormed,
    OutputMissing { output_index: usize },
    OutputNotRadiance { output_index: usize },
    OutputTopologyUnsupported { output_index: usize },
    OutputDestinationUnsupported { output_index: usize },
    CarrierFormatMismatch { output_index: usize },
    ContinuityResourceMismatch { output_index: usize },
    ContinuityNotEstablished { output_index: usize },
    ContinuityWriterChanged { output_index: usize },
    ReadbackSourceMismatch { output_index: usize },
    ReadbackPending { output_index: usize },
    ReadbackFailed {
        output_index: usize,
        kind: GpuSubmissionFailureKind,
    },
    ReadbackFormatMismatch { output_index: usize },
    ReadbackLayoutMismatch { output_index: usize },
    SizeOverflow { output_index: usize },
    HostAllocation { output_index: usize },
    NonFiniteSample {
        output_index: usize,
        sample_index: usize,
    },
}

impl fmt::Display for RenderDeterministicRadianceCaptureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output_index = match self {
            Self::VerificationNotFormed => {
                return formatter.write_str(
                    "deterministic radiance capture requires successful verified result formation",
                );
            }
            Self::OutputMissing { output_index }
            | Self::OutputNotRadiance { output_index }
            | Self::OutputTopologyUnsupported { output_index }
            | Self::OutputDestinationUnsupported { output_index }
            | Self::CarrierFormatMismatch { output_index }
            | Self::ContinuityResourceMismatch { output_index }
            | Self::ContinuityNotEstablished { output_index }
            | Self::ContinuityWriterChanged { output_index }
            | Self::ReadbackSourceMismatch { output_index }
            | Self::ReadbackPending { output_index }
            | Self::ReadbackFailed { output_index, .. }
            | Self::ReadbackFormatMismatch { output_index }
            | Self::ReadbackLayoutMismatch { output_index }
            | Self::SizeOverflow { output_index }
            | Self::HostAllocation { output_index }
            | Self::NonFiniteSample { output_index, .. } => output_index,
        };
        match self {
            Self::OutputMissing { .. } => write!(formatter, "deterministic output {output_index} is absent"),
            Self::OutputNotRadiance { .. } => write!(formatter, "deterministic output {output_index} is not radiance"),
            Self::OutputTopologyUnsupported { .. } => write!(formatter, "deterministic radiance output {output_index} is not a maintained 2D lattice"),
            Self::OutputDestinationUnsupported { .. } => write!(formatter, "deterministic radiance output {output_index} is not bound to a maintained lattice texture"),
            Self::CarrierFormatMismatch { .. } => write!(formatter, "deterministic radiance output {output_index} no longer uses the maintained carrier format"),
            Self::ContinuityResourceMismatch { .. } => write!(formatter, "retained continuity does not describe deterministic radiance output {output_index}"),
            Self::ContinuityNotEstablished { .. } => write!(formatter, "deterministic radiance output {output_index} has no coherent completed retained content"),
            Self::ContinuityWriterChanged { .. } => write!(formatter, "deterministic radiance output {output_index} was written after the verified renderer submission"),
            Self::ReadbackSourceMismatch { .. } => write!(formatter, "readback source does not exactly cover deterministic radiance output {output_index}"),
            Self::ReadbackPending { .. } => write!(formatter, "radiance readback for output {output_index} is still pending"),
            Self::ReadbackFailed { kind, .. } => write!(formatter, "radiance readback for output {output_index} failed ({kind:?})"),
            Self::ReadbackFormatMismatch { .. } => write!(formatter, "radiance readback format does not match output {output_index}"),
            Self::ReadbackLayoutMismatch { .. } => write!(formatter, "radiance readback layout does not match output {output_index}"),
            Self::SizeOverflow { .. } => write!(formatter, "radiance output {output_index} exceeds maintained capture indexing limits"),
            Self::HostAllocation { .. } => write!(formatter, "host allocation failed while observing radiance output {output_index}"),
            Self::NonFiniteSample { sample_index, .. } => write!(formatter, "radiance output {output_index} sample {sample_index} is non-finite"),
            Self::VerificationNotFormed => unreachable!(),
        }
    }
}

impl Error for RenderDeterministicRadianceCaptureError {}

impl SubmittedDeterministicRender {
    /// Interpret one separately captured exact maintained radiance lattice after verified result
    /// formation has already succeeded.
    ///
    /// The caller authors and progresses the public RunenGPU readback. This method accepts it only
    /// when retained-content continuity proves the renderer submission remains the last completed
    /// writer and the readback exactly covers that same product-owned output texture.
    pub fn interpret_captured_radiance(
        &self,
        output_index: usize,
        continuity: &GpuRetainedResourceContinuity,
        readback: &GpuReadback,
    ) -> Result<RenderCapturedDeterministicRadiance, RenderDeterministicRadianceCaptureError> {
        if !matches!(self.verification, DeterministicVerificationState::Formed) {
            return Err(RenderDeterministicRadianceCaptureError::VerificationNotFormed);
        }

        let admitted = self.admitted.admitted();
        let output = admitted
            .outputs()
            .iter()
            .find(|output| output.output_index() == output_index)
            .ok_or(RenderDeterministicRadianceCaptureError::OutputMissing { output_index })?;
        let requested = admitted
            .plan()
            .request()
            .outputs()
            .get(output_index)
            .copied()
            .ok_or(RenderDeterministicRadianceCaptureError::OutputMissing { output_index })?;
        let RenderOutputValue::Radiance { representation } = requested.spec().value() else {
            return Err(RenderDeterministicRadianceCaptureError::OutputNotRadiance { output_index });
        };
        let topology = requested.spec().topology();
        let Some((width, height)) = topology.sample_lattice_dimensions() else {
            return Err(RenderDeterministicRadianceCaptureError::OutputTopologyUnsupported {
                output_index,
            });
        };
        let RenderOutputDestination::SampleLatticeTexture(texture) = output.binding().destination()
        else {
            return Err(RenderDeterministicRadianceCaptureError::OutputDestinationUnsupported {
                output_index,
            });
        };
        if texture.descriptor().format() != GpuTextureFormat::R32Uint {
            return Err(RenderDeterministicRadianceCaptureError::CarrierFormatMismatch {
                output_index,
            });
        }

        let destination_resource: GpuResourceRef = texture.clone().into();
        if continuity.resource() != &destination_resource {
            return Err(RenderDeterministicRadianceCaptureError::ContinuityResourceMismatch {
                output_index,
            });
        }
        match continuity.opaque_content() {
            GpuOpaqueContentContinuity::Established {
                last_completed_write,
            } if last_completed_write == self.submission.id() => {}
            GpuOpaqueContentContinuity::Established { .. } => {
                return Err(RenderDeterministicRadianceCaptureError::ContinuityWriterChanged {
                    output_index,
                });
            }
            GpuOpaqueContentContinuity::Unestablished | GpuOpaqueContentContinuity::Unknown => {
                return Err(RenderDeterministicRadianceCaptureError::ContinuityNotEstablished {
                    output_index,
                });
            }
        }

        let expected_region = GpuTextureCopyRegion::whole_base_mip(texture, GpuTextureAspect::Color)
            .map_err(|_| RenderDeterministicRadianceCaptureError::ReadbackSourceMismatch {
                output_index,
            })?;
        if readback.source() != &GpuTransferRegion::Texture(expected_region) {
            return Err(RenderDeterministicRadianceCaptureError::ReadbackSourceMismatch {
                output_index,
            });
        }

        let bytes = match readback.status() {
            GpuReadbackStatus::Pending => {
                return Err(RenderDeterministicRadianceCaptureError::ReadbackPending {
                    output_index,
                });
            }
            GpuReadbackStatus::Failed(failure) => {
                return Err(RenderDeterministicRadianceCaptureError::ReadbackFailed {
                    output_index,
                    kind: failure.kind(),
                });
            }
            GpuReadbackStatus::Ready(bytes) => bytes,
        };
        if bytes.texture_format() != Some(GpuTextureFormat::R32Uint) {
            return Err(RenderDeterministicRadianceCaptureError::ReadbackFormatMismatch {
                output_index,
            });
        }

        let word_bytes = u64::try_from(WORD_BYTES).map_err(|_| {
            RenderDeterministicRadianceCaptureError::SizeOverflow { output_index }
        })?;
        let row_bytes = u64::from(width)
            .checked_mul(word_bytes)
            .ok_or(RenderDeterministicRadianceCaptureError::SizeOverflow { output_index })?;
        let sample_count = u64::from(width)
            .checked_mul(u64::from(height))
            .ok_or(RenderDeterministicRadianceCaptureError::SizeOverflow { output_index })?;
        let expected_byte_len = sample_count
            .checked_mul(word_bytes)
            .ok_or(RenderDeterministicRadianceCaptureError::SizeOverflow { output_index })?;
        let layout = bytes.layout();
        if layout.byte_len() != expected_byte_len
            || layout.alignment() != word_bytes
            || layout.stride() != row_bytes
            || layout.element_count() != u64::from(height)
            || u64::try_from(bytes.as_bytes().len()).ok() != Some(expected_byte_len)
        {
            return Err(RenderDeterministicRadianceCaptureError::ReadbackLayoutMismatch {
                output_index,
            });
        }

        let sample_count = usize::try_from(sample_count).map_err(|_| {
            RenderDeterministicRadianceCaptureError::SizeOverflow { output_index }
        })?;
        let mut samples = Vec::new();
        samples.try_reserve_exact(sample_count).map_err(|_| {
            RenderDeterministicRadianceCaptureError::HostAllocation { output_index }
        })?;
        for (sample_index, chunk) in bytes.as_bytes().chunks_exact(WORD_BYTES).enumerate() {
            let word = decode_word(chunk);
            let value = decode_finite_f32(word).ok_or(
                RenderDeterministicRadianceCaptureError::NonFiniteSample {
                    output_index,
                    sample_index,
                },
            )?;
            samples.push(value);
        }
        if samples.len() != sample_count {
            return Err(RenderDeterministicRadianceCaptureError::ReadbackLayoutMismatch {
                output_index,
            });
        }

        Ok(RenderCapturedDeterministicRadiance {
            output_index,
            topology,
            representation,
            samples,
        })
    }
}
''')
