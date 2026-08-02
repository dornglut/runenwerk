use super::admission::{
    GpuCandidateAdmissionReport, GpuCandidateEnvironmentEvidence, GpuRejectedCandidateReport,
    evaluate_validated_candidate,
};
use super::descriptor::{GpuContextDescriptor, GpuPowerPreference};
use super::diagnostics::{GpuContextRequestError, GpuContextRequestErrorCategory};
use super::facts::{GpuAdapterFacts, GpuFallbackStatus, GpuPortabilityClass, GpuSoftwareStatus};
use crate::plugins::gpu::{GpuCapabilityFeature, GpuTextureFormat, GpuTextureFormatCapabilities};
use std::num::NonZeroU64;

/// Opaque, nonzero process-local candidate selector disclosed only for exact retries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuCandidateId(NonZeroU64);

impl GpuCandidateId {
    pub const fn is_nonzero(self) -> bool {
        self.0.get() != 0
    }

    pub(crate) const fn from_ordinal(ordinal: u64) -> Self {
        Self(NonZeroU64::new(ordinal).expect("candidate ordinals begin at one"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuCandidateDisposition {
    Accepted(Box<GpuCandidateAdmissionReport>),
    Rejected(Box<GpuRejectedCandidateReport>),
}

impl GpuCandidateDisposition {
    pub const fn id(&self) -> GpuCandidateId {
        match self {
            Self::Accepted(report) => report.id(),
            Self::Rejected(report) => report.id(),
        }
    }

    pub fn adapter(&self) -> &GpuAdapterFacts {
        match self {
            Self::Accepted(report) => report.adapter(),
            Self::Rejected(report) => report.adapter(),
        }
    }

    pub(crate) const fn is_rejected(&self) -> bool {
        matches!(self, Self::Rejected(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GpuCandidateRankEvidence {
    fallback: u8,
    power: u8,
    portability: u8,
    adapter_class: u8,
    backend_preference: u8,
    vendor: Option<u32>,
    device: Option<u32>,
}

impl GpuCandidateRankEvidence {
    pub const fn fallback_priority(&self) -> u8 {
        self.fallback
    }

    pub const fn power_priority(&self) -> u8 {
        self.power
    }

    pub const fn portability_priority(&self) -> u8 {
        self.portability
    }

    pub const fn adapter_class_priority(&self) -> u8 {
        self.adapter_class
    }

    pub const fn backend_preference_priority(&self) -> u8 {
        self.backend_preference
    }

    pub const fn vendor(&self) -> Option<u32> {
        self.vendor
    }

    pub const fn device(&self) -> Option<u32> {
        self.device
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCandidateSelectionEvidence {
    rank: GpuCandidateRankEvidence,
    reason: &'static str,
}

impl GpuCandidateSelectionEvidence {
    pub fn rank(&self) -> &GpuCandidateRankEvidence {
        &self.rank
    }

    pub const fn reason(&self) -> &'static str {
        self.reason
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuCandidateSelectionKind {
    BackendSelectedCandidate,
    DeterministicallyRanked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuContextAdmissionReport {
    pub(crate) selected: GpuCandidateSelectionKind,
    pub(crate) candidate: GpuCandidateAdmissionReport,
    pub(crate) candidate_dispositions: Vec<GpuCandidateDisposition>,
    pub(crate) selection_evidence: GpuCandidateSelectionEvidence,
}

impl GpuContextAdmissionReport {
    pub fn candidate(&self) -> &GpuCandidateAdmissionReport {
        &self.candidate
    }

    pub const fn selection_kind(&self) -> GpuCandidateSelectionKind {
        self.selected
    }

    pub fn candidate_dispositions(&self) -> &[GpuCandidateDisposition] {
        &self.candidate_dispositions
    }

    pub fn candidate_dispositions_are_canonically_ordered(&self) -> bool {
        self.candidate_dispositions
            .windows(2)
            .all(|pair| canonical_disposition_key(&pair[0]) <= canonical_disposition_key(&pair[1]))
    }

    pub fn selection_evidence(&self) -> &GpuCandidateSelectionEvidence {
        &self.selection_evidence
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GpuCandidateInput {
    pub(crate) id: GpuCandidateId,
    pub(crate) adapter: GpuAdapterFacts,
    pub(crate) environment: GpuCandidateEnvironmentEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GpuCandidateSelection {
    pub(crate) candidate: GpuCandidateAdmissionReport,
    pub(crate) dispositions: Vec<GpuCandidateDisposition>,
    pub(crate) evidence: GpuCandidateSelectionEvidence,
}

#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) fn select_candidate(
    descriptor: &GpuContextDescriptor,
    candidates: impl IntoIterator<Item = GpuAdapterFacts>,
    host_compatible: bool,
) -> Result<GpuCandidateSelection, GpuContextRequestError> {
    select_candidate_with_host_evidence(
        descriptor,
        candidates.into_iter().map(|candidate| {
            (
                candidate,
                GpuCandidateEnvironmentEvidence::current_host(host_compatible),
            )
        }),
    )
}

#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) fn select_candidate_with_host_evidence(
    descriptor: &GpuContextDescriptor,
    candidates: impl IntoIterator<Item = (GpuAdapterFacts, GpuCandidateEnvironmentEvidence)>,
) -> Result<GpuCandidateSelection, GpuContextRequestError> {
    let mut candidates = candidates
        .into_iter()
        .map(|(adapter, environment)| GpuCandidateInput {
            id: GpuCandidateId::from_ordinal(1),
            adapter,
            environment,
        })
        .collect::<Vec<_>>();
    canonicalize_candidate_inputs(&mut candidates);
    select_candidate_inputs(descriptor, candidates)
}

pub(crate) fn select_candidate_inputs(
    descriptor: &GpuContextDescriptor,
    candidates: impl IntoIterator<Item = GpuCandidateInput>,
) -> Result<GpuCandidateSelection, GpuContextRequestError> {
    super::admission::validate_descriptor(descriptor)?;
    let mut dispositions = candidates
        .into_iter()
        .map(|candidate| {
            match evaluate_validated_candidate(
                descriptor,
                candidate.id,
                candidate.adapter.clone(),
                candidate.environment,
            ) {
                Ok(report) => GpuCandidateDisposition::Accepted(Box::new(report)),
                Err(error) => {
                    GpuCandidateDisposition::Rejected(Box::new(GpuRejectedCandidateReport {
                        id: candidate.id,
                        adapter: candidate.adapter,
                        category: error.category(),
                        detail: error.detail().map(str::to_owned),
                    }))
                }
            }
        })
        .collect::<Vec<_>>();
    if dispositions.is_empty() {
        return Err(GpuContextRequestError::new(
            GpuContextRequestErrorCategory::NoAdapterAvailable,
            "the backend reported no adapter candidates",
        ));
    }
    dispositions.sort_by_key(canonical_disposition_key);
    let mut admitted = dispositions
        .iter()
        .filter_map(|disposition| match disposition {
            GpuCandidateDisposition::Accepted(report) => Some(report.as_ref().clone()),
            GpuCandidateDisposition::Rejected(_) => None,
        })
        .collect::<Vec<_>>();
    if admitted.is_empty() {
        return Err(GpuContextRequestError::new(
            GpuContextRequestErrorCategory::NoAdmissibleCandidate,
            "no observed adapter satisfied the normalized request",
        )
        .with_candidate_dispositions(dispositions));
    }
    admitted.sort_by_key(|candidate| candidate_rank(descriptor, candidate));
    if let Some(exact) = descriptor.exact_candidate() {
        let Some(candidate) = admitted
            .into_iter()
            .find(|candidate| candidate.id() == exact)
        else {
            return Err(GpuContextRequestError::new(
                GpuContextRequestErrorCategory::NoAdmissibleCandidate,
                "the exact process-local candidate retry is absent or no longer admissible",
            )
            .with_candidate_dispositions(dispositions));
        };
        return Ok(GpuCandidateSelection {
            evidence: GpuCandidateSelectionEvidence {
                rank: candidate_rank(descriptor, &candidate),
                reason: "exact process-local candidate retry",
            },
            candidate,
            dispositions,
        });
    }
    let best = admitted
        .first()
        .cloned()
        .expect("nonempty admitted candidates checked above");
    if admitted.get(1).is_some_and(|second| {
        candidate_rank(descriptor, second) == candidate_rank(descriptor, &best)
    }) {
        return Err(GpuContextRequestError::new(
            GpuContextRequestErrorCategory::AmbiguousAdapterSelection,
            "best candidates remain indistinguishable after normalized ranking",
        )
        .with_candidate_dispositions(dispositions));
    }
    Ok(GpuCandidateSelection {
        evidence: GpuCandidateSelectionEvidence {
            rank: candidate_rank(descriptor, &best),
            reason: "lowest complete normalized rank",
        },
        candidate: best,
        dispositions,
    })
}

#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) fn canonicalize_candidate_inputs(candidates: &mut [GpuCandidateInput]) {
    candidates.sort_by_key(|candidate| {
        canonical_candidate_input_key(&candidate.adapter, candidate.environment)
    });
    for (offset, candidate) in candidates.iter_mut().enumerate() {
        candidate.id = GpuCandidateId::from_ordinal(
            u64::try_from(offset + 1).expect("candidate count fits process-local identifier"),
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct GpuCanonicalCandidateInputKey {
    adapter: GpuCanonicalAdapterKey,
    host_compatible: bool,
    host_compatibility_required: bool,
}

pub(crate) fn canonical_candidate_input_key(
    adapter: &GpuAdapterFacts,
    environment: GpuCandidateEnvironmentEvidence,
) -> GpuCanonicalCandidateInputKey {
    GpuCanonicalCandidateInputKey {
        adapter: canonical_adapter_key(adapter),
        host_compatible: environment.host_compatible,
        host_compatibility_required: environment.host_compatibility_required,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GpuCanonicalFormatCapabilities {
    sampled: bool,
    filterable: bool,
    storage_read: bool,
    storage_write: bool,
    color_attachment: bool,
    depth_stencil: bool,
    copy_source: bool,
    copy_destination: bool,
    block_dimensions: Option<(u32, u32)>,
    block_copy_size: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GpuCanonicalAdapterKey {
    backend: super::descriptor::GpuBackendFamily,
    class: super::descriptor::GpuAdapterClass,
    software: GpuSoftwareStatus,
    fallback: GpuFallbackStatus,
    supported_features: Vec<GpuCapabilityFeature>,
    adapter_limits: (u64, u64, u32, u32, u32),
    supported_formats: Vec<(GpuTextureFormat, GpuCanonicalFormatCapabilities)>,
    alignments: super::facts::GpuAlignmentFacts,
    vendor: Option<u32>,
    device: Option<u32>,
    diagnostic_name: Option<String>,
    profile: super::facts::GpuDeviceRequestProfile,
    profile_supported: bool,
}

fn canonical_adapter_key(adapter: &GpuAdapterFacts) -> GpuCanonicalAdapterKey {
    let limits = adapter.adapter_limits().values();
    GpuCanonicalAdapterKey {
        backend: adapter.backend(),
        class: adapter.class(),
        software: adapter.software(),
        fallback: adapter.fallback(),
        supported_features: adapter.supported().features().collect(),
        adapter_limits: (
            limits.max_uniform_buffer_binding_size(),
            limits.max_storage_buffer_binding_size(),
            limits.max_color_attachments(),
            limits.max_vertex_buffers(),
            limits.max_bindings_per_group(),
        ),
        supported_formats: adapter
            .supported()
            .formats()
            .map(|(format, capabilities)| (format, canonical_format_capabilities(capabilities)))
            .collect(),
        alignments: adapter.alignments(),
        vendor: adapter.vendor(),
        device: adapter.device(),
        diagnostic_name: adapter.diagnostic_name().map(str::to_owned),
        profile: adapter.device_request_profile(),
        profile_supported: adapter.device_request_profile_supported(),
    }
}

fn canonical_format_capabilities(
    capabilities: GpuTextureFormatCapabilities,
) -> GpuCanonicalFormatCapabilities {
    GpuCanonicalFormatCapabilities {
        sampled: capabilities.sampled,
        filterable: capabilities.filterable,
        storage_read: capabilities.storage_read,
        storage_write: capabilities.storage_write,
        color_attachment: capabilities.color_attachment,
        depth_stencil: capabilities.depth_stencil,
        copy_source: capabilities.copy_source,
        copy_destination: capabilities.copy_destination,
        block_dimensions: capabilities.block_dimensions,
        block_copy_size: capabilities.block_copy_size,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GpuCanonicalDispositionKey {
    adapter: GpuCanonicalAdapterKey,
    disposition_category: u8,
    candidate_id: GpuCandidateId,
}

fn canonical_disposition_key(disposition: &GpuCandidateDisposition) -> GpuCanonicalDispositionKey {
    let (adapter, disposition_category) = match disposition {
        GpuCandidateDisposition::Accepted(report) => (report.adapter(), 0),
        GpuCandidateDisposition::Rejected(report) => {
            (report.adapter(), 1 + error_category_rank(report.category()))
        }
    };
    GpuCanonicalDispositionKey {
        adapter: canonical_adapter_key(adapter),
        disposition_category,
        candidate_id: disposition.id(),
    }
}

const fn error_category_rank(category: GpuContextRequestErrorCategory) -> u8 {
    match category {
        GpuContextRequestErrorCategory::NoAdapterAvailable => 0,
        GpuContextRequestErrorCategory::NoAdmissibleCandidate => 1,
        GpuContextRequestErrorCategory::AmbiguousAdapterSelection => 2,
        GpuContextRequestErrorCategory::BackendFamilyForbidden => 3,
        GpuContextRequestErrorCategory::SoftwareFallbackPolicyViolation => 4,
        GpuContextRequestErrorCategory::MandatoryFeatureMissing => 5,
        GpuContextRequestErrorCategory::LimitBelowRequiredMinimum => 6,
        GpuContextRequestErrorCategory::LimitAbovePermittedMaximum => 7,
        GpuContextRequestErrorCategory::UnsupportedFormatRole => 8,
        GpuContextRequestErrorCategory::AlignmentIncompatibility => 9,
        GpuContextRequestErrorCategory::DeviceRequestProfileUnsupported => 10,
        GpuContextRequestErrorCategory::ContradictoryRequest => 11,
        GpuContextRequestErrorCategory::BackendAdapterRequestFailure => 12,
        GpuContextRequestErrorCategory::BackendDeviceRequestFailure => 13,
        GpuContextRequestErrorCategory::TemporaryHostCompatibilityFailure => 14,
        GpuContextRequestErrorCategory::IdentityExhausted => 15,
        GpuContextRequestErrorCategory::InvalidDegradation => 16,
    }
}

fn candidate_rank(
    descriptor: &GpuContextDescriptor,
    candidate: &GpuCandidateAdmissionReport,
) -> GpuCandidateRankEvidence {
    let adapter = candidate.adapter();
    let fallback = match adapter.fallback() {
        GpuFallbackStatus::ConfirmedNotFallback => 0,
        GpuFallbackStatus::Unknown => 1,
        GpuFallbackStatus::ConfirmedFallback => 2,
    };
    let power = match (descriptor.power_preference(), adapter.class()) {
        (GpuPowerPreference::HighPerformance, super::descriptor::GpuAdapterClass::Discrete)
        | (GpuPowerPreference::LowPower, super::descriptor::GpuAdapterClass::Integrated) => 0,
        (_, super::descriptor::GpuAdapterClass::Discrete) => 1,
        (_, super::descriptor::GpuAdapterClass::Integrated) => 2,
        _ => 3,
    };
    let portability = match candidate.portability() {
        GpuPortabilityClass::PortableBaseline => 0,
        GpuPortabilityClass::PortableWithDeclaredExtensions => 1,
        GpuPortabilityClass::BackendSpecialized => 2,
        GpuPortabilityClass::Unsupported => 3,
    };
    let class = match adapter.class() {
        super::descriptor::GpuAdapterClass::Discrete => 0,
        super::descriptor::GpuAdapterClass::Integrated => 1,
        super::descriptor::GpuAdapterClass::Virtual => 2,
        super::descriptor::GpuAdapterClass::Cpu => 3,
        super::descriptor::GpuAdapterClass::Other => 4,
        super::descriptor::GpuAdapterClass::Unknown => 5,
    };
    let backend = descriptor
        .backend_preference()
        .get(&adapter.backend())
        .copied()
        .unwrap_or(u8::MAX);
    GpuCandidateRankEvidence {
        fallback,
        power,
        portability,
        adapter_class: class,
        backend_preference: backend,
        vendor: adapter.vendor(),
        device: adapter.device(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::gpu::{
        GpuAdapterClass, GpuAdapterFacts, GpuAdapterLimits, GpuAlignmentFacts, GpuBackendFamily,
        GpuCapabilities, GpuCapabilityRequirements, GpuContextDescriptor, GpuFallbackStatus,
        GpuLimits, GpuSoftwareStatus,
    };

    fn adapter() -> GpuAdapterFacts {
        GpuAdapterFacts::new(
            GpuBackendFamily::Vulkan,
            GpuAdapterClass::Discrete,
            GpuSoftwareStatus::Hardware,
            GpuFallbackStatus::ConfirmedNotFallback,
            GpuCapabilities::from_normalized_facts(
                [],
                GpuLimits::new(64 * 1024, 128 * 1024 * 1024, 4, 8, 16).unwrap(),
                [],
            ),
            GpuAdapterLimits::new(GpuLimits::new(64 * 1024, 128 * 1024 * 1024, 4, 8, 16).unwrap()),
            GpuAlignmentFacts {
                uniform_dynamic_offset: Some(256),
                storage_dynamic_offset: Some(256),
                copy_buffer_offset: Some(4),
                bytes_per_row: Some(256),
                query_resolve_destination: Some(256),
            },
        )
    }

    #[test]
    fn reports_canonically_order_all_dispositions_and_split_absence_from_rejection() {
        let descriptor = GpuContextDescriptor::new(GpuCapabilityRequirements::new());
        let empty = select_candidate(&descriptor, [], true).unwrap_err();
        assert_eq!(
            empty.category(),
            GpuContextRequestErrorCategory::NoAdapterAvailable
        );

        let only_metal = descriptor
            .clone()
            .with_allowed_backends([GpuBackendFamily::Metal]);
        let rejected = select_candidate(&only_metal, [adapter()], true).unwrap_err();
        assert_eq!(
            rejected.category(),
            GpuContextRequestErrorCategory::NoAdmissibleCandidate
        );
        assert_eq!(rejected.candidate_dispositions().len(), 1);
        assert!(matches!(
            rejected.candidate_dispositions(),
            [GpuCandidateDisposition::Rejected(report)]
                if report.category() == GpuContextRequestErrorCategory::BackendFamilyForbidden
        ));
    }

    #[test]
    fn ambiguity_exposes_process_local_ids_and_exact_retry_never_uses_diagnostic_names() {
        let first = adapter().with_diagnostics("alpha diagnostic".to_owned(), 7, 9);
        let second = adapter().with_diagnostics("beta diagnostic".to_owned(), 7, 9);
        let descriptor = GpuContextDescriptor::new(GpuCapabilityRequirements::new());
        let ambiguous =
            select_candidate(&descriptor, [second.clone(), first.clone()], true).unwrap_err();
        assert_eq!(
            ambiguous.category(),
            GpuContextRequestErrorCategory::AmbiguousAdapterSelection
        );
        let ids = ambiguous
            .candidate_dispositions()
            .iter()
            .map(GpuCandidateDisposition::id)
            .collect::<Vec<_>>();
        assert_eq!(ids.len(), 2);
        assert_ne!(ids[0], ids[1]);
        assert!(ids.iter().all(|id| id.is_nonzero()));

        let retry = select_candidate(
            &descriptor.clone().with_exact_candidate(ids[1]),
            [first, second],
            true,
        )
        .unwrap();
        assert_eq!(retry.candidate.id(), ids[1]);
        assert_eq!(
            retry.evidence.reason(),
            "exact process-local candidate retry"
        );
    }

    #[test]
    fn full_disposition_collection_is_invariant_to_input_order() {
        let accepted = adapter().with_diagnostics("accepted".to_owned(), 1, 1);
        let rejected = GpuAdapterFacts::new(
            GpuBackendFamily::UnknownBackend,
            GpuAdapterClass::Unknown,
            GpuSoftwareStatus::Unknown,
            GpuFallbackStatus::Unknown,
            GpuCapabilities::from_normalized_facts(
                [],
                GpuLimits::new(64 * 1024, 128 * 1024 * 1024, 4, 8, 16).unwrap(),
                [],
            ),
            GpuAdapterLimits::new(GpuLimits::new(64 * 1024, 128 * 1024 * 1024, 4, 8, 16).unwrap()),
            accepted.alignments(),
        )
        .with_diagnostics("rejected".to_owned(), 2, 2);
        let descriptor = GpuContextDescriptor::new(GpuCapabilityRequirements::new());
        let forward =
            select_candidate(&descriptor, [rejected.clone(), accepted.clone()], true).unwrap();
        let reverse = select_candidate(&descriptor, [accepted, rejected], true).unwrap();
        assert_eq!(forward.dispositions, reverse.dispositions);
        assert!(
            GpuContextAdmissionReport {
                selected: GpuCandidateSelectionKind::DeterministicallyRanked,
                candidate: forward.candidate.clone(),
                candidate_dispositions: forward.dispositions.clone(),
                selection_evidence: forward.evidence.clone(),
            }
            .candidate_dispositions_are_canonically_ordered()
        );
    }
}
