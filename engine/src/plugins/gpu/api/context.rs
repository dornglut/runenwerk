//! Backend-neutral context admission contracts.

use super::{
    GpuCapabilities, GpuCapabilityFeature, GpuCapabilityRequirement, GpuCapabilityRequirements,
    GpuLimits, GpuTextureFormat,
};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};

const MAX_DIAGNOSTIC_BYTES: usize = 256;
static NEXT_CONTEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuBackendFamily {
    Vulkan,
    Metal,
    Direct3D12,
    OpenGl,
    BrowserWebGpu,
    UnknownBackend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuAdapterClass {
    Discrete,
    Integrated,
    Virtual,
    Cpu,
    Other,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuSoftwareStatus {
    Software,
    Hardware,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuPortabilityClass {
    PortableBaseline,
    PortableWithDeclaredExtensions,
    BackendSpecialized,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuPowerPreference {
    HighPerformance,
    LowPower,
    NoPreference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuSoftwareFallbackPolicy {
    Allow,
    Require,
    Forbid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuPortabilityPolicy {
    AllowBackendSpecialized,
    RequirePortableBaseline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuAlignmentKind {
    UniformDynamicOffset,
    StorageDynamicOffset,
    CopyBufferOffset,
    BytesPerRow,
    QueryResolveDestination,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuLimitKind {
    MaxUniformBufferBindingSize,
    MaxStorageBufferBindingSize,
    MaxColorAttachments,
    MaxVertexBuffers,
    MaxBindingsPerGroup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuFormatRole {
    Sampled,
    Filterable,
    StorageRead,
    StorageWrite,
    ColorAttachment,
    DepthStencil,
    CopySource,
    CopyDestination,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuLimitConstraint {
    pub minimum: Option<u64>,
    pub maximum: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuAlignmentFacts {
    pub uniform_dynamic_offset: Option<u64>,
    pub storage_dynamic_offset: Option<u64>,
    pub copy_buffer_offset: Option<u64>,
    pub bytes_per_row: Option<u64>,
    pub query_resolve_destination: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuAdapterFacts {
    backend: GpuBackendFamily,
    class: GpuAdapterClass,
    software: GpuSoftwareStatus,
    fallback_requested: bool,
    diagnostic_name: Option<String>,
    vendor: Option<u32>,
    device: Option<u32>,
    supported: GpuCapabilities,
    alignments: GpuAlignmentFacts,
}

impl GpuAdapterFacts {
    pub fn new(
        backend: GpuBackendFamily,
        class: GpuAdapterClass,
        software: GpuSoftwareStatus,
        fallback_requested: bool,
        supported: GpuCapabilities,
        alignments: GpuAlignmentFacts,
    ) -> Self {
        Self {
            backend,
            class,
            software,
            fallback_requested,
            diagnostic_name: None,
            vendor: None,
            device: None,
            supported,
            alignments,
        }
    }

    pub(crate) fn with_diagnostics(mut self, name: String, vendor: u32, device: u32) -> Self {
        self.diagnostic_name = sanitized_diagnostic(name);
        self.vendor = Some(vendor);
        self.device = Some(device);
        self
    }

    pub const fn backend(&self) -> GpuBackendFamily {
        self.backend
    }
    pub const fn class(&self) -> GpuAdapterClass {
        self.class
    }
    pub const fn software(&self) -> GpuSoftwareStatus {
        self.software
    }
    pub const fn fallback_requested(&self) -> bool {
        self.fallback_requested
    }
    pub fn diagnostic_name(&self) -> Option<&str> {
        self.diagnostic_name.as_deref()
    }
    pub const fn vendor(&self) -> Option<u32> {
        self.vendor
    }
    pub const fn device(&self) -> Option<u32> {
        self.device
    }
    pub fn supported(&self) -> &GpuCapabilities {
        &self.supported
    }
    pub const fn alignments(&self) -> GpuAlignmentFacts {
        self.alignments
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuAdmittedDeviceFacts {
    enabled: BTreeSet<GpuCapabilityFeature>,
    effective_limits: GpuLimits,
}

impl GpuAdmittedDeviceFacts {
    pub fn enabled_features(&self) -> impl ExactSizeIterator<Item = GpuCapabilityFeature> + '_ {
        self.enabled.iter().copied()
    }
    pub fn is_enabled(&self, feature: GpuCapabilityFeature) -> bool {
        self.enabled.contains(&feature)
    }
    pub const fn effective_limits(&self) -> GpuLimits {
        self.effective_limits
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuContextId(NonZeroU64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuDeviceGeneration(NonZeroU64);

impl GpuDeviceGeneration {
    pub const fn first() -> Self {
        Self(NonZeroU64::MIN)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuContextAffinity {
    context: GpuContextId,
    generation: GpuDeviceGeneration,
}

impl GpuContextAffinity {
    pub const fn context(&self) -> GpuContextId {
        self.context
    }
    pub const fn generation(&self) -> GpuDeviceGeneration {
        self.generation
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuContextAffinityError {
    ForeignContext,
    StaleGeneration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuContextDescriptor {
    label: Option<String>,
    provenance: Option<String>,
    power_preference: GpuPowerPreference,
    fallback_policy: GpuSoftwareFallbackPolicy,
    allowed_backends: BTreeSet<GpuBackendFamily>,
    allowed_adapter_classes: BTreeSet<GpuAdapterClass>,
    portability_policy: GpuPortabilityPolicy,
    requirements: GpuCapabilityRequirements,
    limits: BTreeMap<GpuLimitKind, GpuLimitConstraint>,
    format_roles: BTreeSet<(GpuTextureFormat, GpuFormatRole)>,
    alignments: BTreeMap<GpuAlignmentKind, u64>,
}

impl GpuContextDescriptor {
    pub fn new(requirements: GpuCapabilityRequirements) -> Self {
        Self {
            label: None,
            provenance: None,
            power_preference: GpuPowerPreference::NoPreference,
            fallback_policy: GpuSoftwareFallbackPolicy::Allow,
            allowed_backends: BTreeSet::new(),
            allowed_adapter_classes: BTreeSet::new(),
            portability_policy: GpuPortabilityPolicy::AllowBackendSpecialized,
            requirements,
            limits: BTreeMap::new(),
            format_roles: BTreeSet::new(),
            alignments: BTreeMap::new(),
        }
    }
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = sanitized_diagnostic(label.into());
        self
    }
    pub fn with_provenance(mut self, provenance: impl Into<String>) -> Self {
        self.provenance = sanitized_diagnostic(provenance.into());
        self
    }
    pub const fn with_power_preference(mut self, preference: GpuPowerPreference) -> Self {
        self.power_preference = preference;
        self
    }
    pub const fn with_fallback_policy(mut self, policy: GpuSoftwareFallbackPolicy) -> Self {
        self.fallback_policy = policy;
        self
    }
    pub fn with_allowed_backends(
        mut self,
        backends: impl IntoIterator<Item = GpuBackendFamily>,
    ) -> Self {
        self.allowed_backends = backends.into_iter().collect();
        self
    }
    pub fn with_allowed_adapter_classes(
        mut self,
        classes: impl IntoIterator<Item = GpuAdapterClass>,
    ) -> Self {
        self.allowed_adapter_classes = classes.into_iter().collect();
        self
    }
    pub const fn with_portability_policy(mut self, policy: GpuPortabilityPolicy) -> Self {
        self.portability_policy = policy;
        self
    }
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    pub fn provenance(&self) -> Option<&str> {
        self.provenance.as_deref()
    }
    pub const fn power_preference(&self) -> GpuPowerPreference {
        self.power_preference
    }
    pub const fn fallback_policy(&self) -> GpuSoftwareFallbackPolicy {
        self.fallback_policy
    }
    pub fn requirements(&self) -> &GpuCapabilityRequirements {
        &self.requirements
    }
    /// Merges independently authored normalized requests without treating
    /// diagnostics as semantic authority.
    pub fn merge(&self, other: &Self) -> Result<Self, GpuContextRequestError> {
        let requirements = self
            .requirements
            .merge(&other.requirements)
            .map_err(|error| {
                GpuContextRequestError::new(
                    GpuContextRequestErrorCategory::ContradictoryRequest,
                    error.to_string(),
                )
            })?;
        let power_preference =
            merge_power_preference(self.power_preference, other.power_preference)?;
        let fallback_policy = merge_fallback_policy(self.fallback_policy, other.fallback_policy)?;
        let allowed_backends = merge_allowlist(&self.allowed_backends, &other.allowed_backends)?;
        let allowed_adapter_classes = merge_allowlist(
            &self.allowed_adapter_classes,
            &other.allowed_adapter_classes,
        )?;
        let mut merged = Self::new(requirements)
            .with_power_preference(power_preference)
            .with_fallback_policy(fallback_policy)
            .with_allowed_backends(allowed_backends)
            .with_allowed_adapter_classes(allowed_adapter_classes)
            .with_portability_policy(match (self.portability_policy, other.portability_policy) {
                (GpuPortabilityPolicy::RequirePortableBaseline, _)
                | (_, GpuPortabilityPolicy::RequirePortableBaseline) => {
                    GpuPortabilityPolicy::RequirePortableBaseline
                }
                _ => GpuPortabilityPolicy::AllowBackendSpecialized,
            });
        for descriptor in [self, other] {
            for (&kind, constraint) in &descriptor.limits {
                if let Some(minimum) = constraint.minimum {
                    merged = merged.require_limit(kind, minimum);
                }
                if let Some(maximum) = constraint.maximum {
                    merged = merged.permit_limit(kind, maximum);
                }
            }
            for &(format, role) in &descriptor.format_roles {
                merged = merged.require_format_role(format, role);
            }
            for (&kind, &maximum) in &descriptor.alignments {
                merged = merged.require_alignment(kind, maximum);
            }
        }
        validate_descriptor(&merged)?;
        Ok(merged)
    }
    /// Compares only request authority, intentionally excluding diagnostics.
    pub fn semantically_eq(&self, other: &Self) -> bool {
        self.power_preference == other.power_preference
            && self.fallback_policy == other.fallback_policy
            && self.allowed_backends == other.allowed_backends
            && self.allowed_adapter_classes == other.allowed_adapter_classes
            && self.portability_policy == other.portability_policy
            && self.requirements == other.requirements
            && self.limits == other.limits
            && self.format_roles == other.format_roles
            && self.alignments == other.alignments
    }
    pub fn require_limit(mut self, kind: GpuLimitKind, minimum: u64) -> Self {
        let entry = self.limits.entry(kind).or_insert(GpuLimitConstraint {
            minimum: None,
            maximum: None,
        });
        entry.minimum = Some(entry.minimum.unwrap_or(0).max(minimum));
        self
    }
    pub fn permit_limit(mut self, kind: GpuLimitKind, maximum: u64) -> Self {
        let entry = self.limits.entry(kind).or_insert(GpuLimitConstraint {
            minimum: None,
            maximum: None,
        });
        entry.maximum = Some(
            entry
                .maximum
                .map_or(maximum, |current| current.min(maximum)),
        );
        self
    }
    pub fn require_format_role(mut self, format: GpuTextureFormat, role: GpuFormatRole) -> Self {
        self.format_roles.insert((format, role));
        self
    }
    pub fn require_alignment(mut self, kind: GpuAlignmentKind, maximum: u64) -> Self {
        self.alignments
            .entry(kind)
            .and_modify(|current| *current = (*current).min(maximum))
            .or_insert(maximum);
        self
    }
}

fn merge_power_preference(
    left: GpuPowerPreference,
    right: GpuPowerPreference,
) -> Result<GpuPowerPreference, GpuContextRequestError> {
    match (left, right) {
        (GpuPowerPreference::NoPreference, value) | (value, GpuPowerPreference::NoPreference) => {
            Ok(value)
        }
        (left, right) if left == right => Ok(left),
        _ => Err(GpuContextRequestError::new(
            GpuContextRequestErrorCategory::ContradictoryRequest,
            "power preferences conflict",
        )),
    }
}

fn merge_fallback_policy(
    left: GpuSoftwareFallbackPolicy,
    right: GpuSoftwareFallbackPolicy,
) -> Result<GpuSoftwareFallbackPolicy, GpuContextRequestError> {
    match (left, right) {
        (GpuSoftwareFallbackPolicy::Allow, value) | (value, GpuSoftwareFallbackPolicy::Allow) => {
            Ok(value)
        }
        (left, right) if left == right => Ok(left),
        _ => Err(GpuContextRequestError::new(
            GpuContextRequestErrorCategory::ContradictoryRequest,
            "software fallback policies conflict",
        )),
    }
}

fn merge_allowlist<T: Ord + Copy>(
    left: &BTreeSet<T>,
    right: &BTreeSet<T>,
) -> Result<BTreeSet<T>, GpuContextRequestError> {
    let merged = if left.is_empty() {
        right.clone()
    } else if right.is_empty() {
        left.clone()
    } else {
        left.intersection(right).copied().collect()
    };
    if !left.is_empty() && !right.is_empty() && merged.is_empty() {
        return Err(GpuContextRequestError::new(
            GpuContextRequestErrorCategory::ContradictoryRequest,
            "allowlists have no common candidate",
        ));
    }
    Ok(merged)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuContextRequestErrorCategory {
    NoCandidate,
    AmbiguousAdapterSelection,
    BackendFamilyForbidden,
    SoftwareFallbackPolicyViolation,
    MandatoryFeatureMissing,
    LimitBelowRequiredMinimum,
    LimitAbovePermittedMaximum,
    UnsupportedFormatRole,
    AlignmentIncompatibility,
    ContradictoryRequest,
    BackendAdapterRequestFailure,
    BackendDeviceRequestFailure,
    TemporaryHostCompatibilityFailure,
    IdentityExhausted,
    InvalidDegradation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuContextRequestError {
    category: GpuContextRequestErrorCategory,
    detail: Option<String>,
    candidate_dispositions: Vec<GpuCandidateDisposition>,
}

impl GpuContextRequestError {
    pub(crate) fn new(category: GpuContextRequestErrorCategory, detail: impl Into<String>) -> Self {
        Self {
            category,
            detail: sanitized_diagnostic(detail.into()),
            candidate_dispositions: Vec::new(),
        }
    }
    pub const fn category(&self) -> GpuContextRequestErrorCategory {
        self.category
    }
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
    pub fn candidate_dispositions(&self) -> &[GpuCandidateDisposition] {
        &self.candidate_dispositions
    }
    fn with_candidate_dispositions(mut self, dispositions: Vec<GpuCandidateDisposition>) -> Self {
        self.candidate_dispositions = dispositions;
        self
    }
}

impl fmt::Display for GpuContextRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GPU context request failed: {:?}", self.category)
    }
}
impl std::error::Error for GpuContextRequestError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuDegradationRecord {
    pub feature: GpuCapabilityFeature,
    pub fallback: super::GpuPreferredFallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCandidateAdmissionReport {
    adapter: GpuAdapterFacts,
    enabled_features: BTreeSet<GpuCapabilityFeature>,
    degradations: Vec<GpuDegradationRecord>,
    portability: GpuPortabilityClass,
    effective_limits: GpuLimits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuCandidateDisposition {
    Accepted(Box<GpuCandidateAdmissionReport>),
    Rejected(GpuContextRequestError),
}

impl GpuCandidateDisposition {
    pub fn adapter(&self) -> Option<&GpuAdapterFacts> {
        match self {
            Self::Accepted(report) => Some(report.adapter()),
            Self::Rejected(_) => None,
        }
    }
}

impl GpuCandidateAdmissionReport {
    pub fn adapter(&self) -> &GpuAdapterFacts {
        &self.adapter
    }
    pub fn enabled_features(&self) -> impl ExactSizeIterator<Item = GpuCapabilityFeature> + '_ {
        self.enabled_features.iter().copied()
    }
    pub fn degradations(&self) -> &[GpuDegradationRecord] {
        &self.degradations
    }
    pub const fn portability(&self) -> GpuPortabilityClass {
        self.portability
    }
    pub const fn effective_limits(&self) -> GpuLimits {
        self.effective_limits
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
}

#[derive(Debug)]
pub struct GpuContext {
    pub(crate) id: GpuContextId,
    pub(crate) generation: GpuDeviceGeneration,
    pub(crate) adapter: GpuAdapterFacts,
    pub(crate) device: GpuAdmittedDeviceFacts,
    pub(crate) report: GpuContextAdmissionReport,
    pub(crate) backend: crate::plugins::gpu::backend::WgpuContextState,
}

impl GpuContext {
    pub async fn request(descriptor: GpuContextDescriptor) -> Result<Self, GpuContextRequestError> {
        crate::plugins::gpu::backend::request_headless(descriptor).await
    }
    pub const fn id(&self) -> GpuContextId {
        self.id
    }
    pub const fn generation(&self) -> GpuDeviceGeneration {
        self.generation
    }
    pub const fn affinity(&self) -> GpuContextAffinity {
        GpuContextAffinity {
            context: self.id,
            generation: self.generation,
        }
    }
    pub fn adapter_facts(&self) -> &GpuAdapterFacts {
        &self.adapter
    }
    pub fn device_facts(&self) -> &GpuAdmittedDeviceFacts {
        &self.device
    }
    pub fn admission_report(&self) -> &GpuContextAdmissionReport {
        &self.report
    }
    pub fn validate_affinity(
        &self,
        affinity: GpuContextAffinity,
    ) -> Result<(), GpuContextAffinityError> {
        validate_affinity(self.affinity(), affinity)
    }
}

fn validate_affinity(
    expected: GpuContextAffinity,
    actual: GpuContextAffinity,
) -> Result<(), GpuContextAffinityError> {
    if actual.context != expected.context {
        Err(GpuContextAffinityError::ForeignContext)
    } else if actual.generation != expected.generation {
        Err(GpuContextAffinityError::StaleGeneration)
    } else {
        Ok(())
    }
}

pub(crate) fn allocate_context_id() -> Result<GpuContextId, GpuContextRequestError> {
    let value = NEXT_CONTEXT_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            (current != 0).then_some(if current == u64::MAX { 0 } else { current + 1 })
        })
        .map_err(|_| {
            GpuContextRequestError::new(
                GpuContextRequestErrorCategory::IdentityExhausted,
                "context identifier allocator exhausted",
            )
        })?;
    Ok(GpuContextId(
        NonZeroU64::new(value).expect("allocator never returns zero"),
    ))
}

pub(crate) fn evaluate_candidate(
    descriptor: &GpuContextDescriptor,
    adapter: GpuAdapterFacts,
    host_compatible: bool,
) -> Result<GpuCandidateAdmissionReport, GpuContextRequestError> {
    validate_descriptor(descriptor)?;
    evaluate_validated_candidate(descriptor, adapter, host_compatible)
}

/// Validates caller-provided normalized authority before a backend terminal exists.
pub(crate) fn validate_descriptor(
    descriptor: &GpuContextDescriptor,
) -> Result<(), GpuContextRequestError> {
    for (kind, constraint) in &descriptor.limits {
        if constraint.minimum.is_some_and(|minimum| minimum == 0)
            || constraint.maximum.is_some_and(|maximum| maximum == 0)
            || matches!((constraint.minimum, constraint.maximum), (Some(minimum), Some(maximum)) if minimum > maximum)
        {
            return Err(GpuContextRequestError::new(
                GpuContextRequestErrorCategory::ContradictoryRequest,
                format!("invalid limit constraint for {kind:?}"),
            ));
        }
    }
    if descriptor.alignments.values().any(|maximum| *maximum == 0) {
        return Err(GpuContextRequestError::new(
            GpuContextRequestErrorCategory::ContradictoryRequest,
            "alignment requirements must be nonzero",
        ));
    }
    Ok(())
}

fn evaluate_validated_candidate(
    descriptor: &GpuContextDescriptor,
    adapter: GpuAdapterFacts,
    host_compatible: bool,
) -> Result<GpuCandidateAdmissionReport, GpuContextRequestError> {
    if !descriptor.allowed_backends.is_empty()
        && !descriptor.allowed_backends.contains(&adapter.backend)
    {
        return Err(GpuContextRequestError::new(
            GpuContextRequestErrorCategory::BackendFamilyForbidden,
            "adapter backend is not allowed",
        ));
    }
    if !descriptor.allowed_adapter_classes.is_empty()
        && !descriptor.allowed_adapter_classes.contains(&adapter.class)
    {
        return Err(GpuContextRequestError::new(
            GpuContextRequestErrorCategory::NoCandidate,
            "adapter class is not allowed",
        ));
    }
    match descriptor.fallback_policy {
        GpuSoftwareFallbackPolicy::Require if !adapter.fallback_requested => {
            return Err(GpuContextRequestError::new(
                GpuContextRequestErrorCategory::SoftwareFallbackPolicyViolation,
                "fallback adapter was required",
            ));
        }
        GpuSoftwareFallbackPolicy::Forbid
            if adapter.fallback_requested || adapter.software == GpuSoftwareStatus::Software =>
        {
            return Err(GpuContextRequestError::new(
                GpuContextRequestErrorCategory::SoftwareFallbackPolicyViolation,
                "software or fallback adapter is forbidden",
            ));
        }
        _ => {}
    }
    let mut enabled = BTreeSet::new();
    let mut degradations = Vec::new();
    for requirement in descriptor.requirements.iter() {
        match requirement {
            GpuCapabilityRequirement::Required(feature) => {
                let supported = adapter.supported.supports(feature)
                    && (feature != GpuCapabilityFeature::Presentation || host_compatible);
                if !supported {
                    return Err(GpuContextRequestError::new(
                        if feature == GpuCapabilityFeature::Presentation {
                            GpuContextRequestErrorCategory::TemporaryHostCompatibilityFailure
                        } else {
                            GpuContextRequestErrorCategory::MandatoryFeatureMissing
                        },
                        format!("mandatory {feature:?} is unsupported"),
                    ));
                }
                enabled.insert(feature);
            }
            GpuCapabilityRequirement::Preferred { feature, fallback } => {
                let supported = adapter.supported.supports(feature)
                    && (feature != GpuCapabilityFeature::Presentation || host_compatible);
                if supported {
                    enabled.insert(feature);
                } else {
                    degradations.push(GpuDegradationRecord { feature, fallback });
                }
            }
            GpuCapabilityRequirement::Disabled(feature) if adapter.supported.supports(feature) => {}
            GpuCapabilityRequirement::Disabled(_) => {}
        }
    }
    for &(format, role) in &descriptor.format_roles {
        let supported = adapter
            .supported
            .format(format)
            .is_some_and(|facts| match role {
                GpuFormatRole::Sampled => facts.sampled,
                GpuFormatRole::Filterable => facts.filterable,
                GpuFormatRole::StorageRead => facts.storage_read,
                GpuFormatRole::StorageWrite => facts.storage_write,
                GpuFormatRole::ColorAttachment => facts.color_attachment,
                GpuFormatRole::DepthStencil => facts.depth_stencil,
                GpuFormatRole::CopySource => facts.copy_source,
                GpuFormatRole::CopyDestination => facts.copy_destination,
            });
        if !supported {
            return Err(GpuContextRequestError::new(
                GpuContextRequestErrorCategory::UnsupportedFormatRole,
                format!("{format:?} lacks {role:?}"),
            ));
        }
    }
    for (&kind, &maximum) in &descriptor.alignments {
        if adapter_alignment(adapter.alignments, kind).is_none_or(|actual| actual > maximum) {
            return Err(GpuContextRequestError::new(
                GpuContextRequestErrorCategory::AlignmentIncompatibility,
                format!("{kind:?} alignment is incompatible"),
            ));
        }
    }
    for (&kind, constraint) in &descriptor.limits {
        let supported = limit_value(adapter.supported.limits(), kind);
        if constraint
            .minimum
            .is_some_and(|minimum| supported < minimum)
        {
            return Err(GpuContextRequestError::new(
                GpuContextRequestErrorCategory::LimitBelowRequiredMinimum,
                format!("{kind:?} is below the required minimum"),
            ));
        }
    }
    let portability = match descriptor.portability_policy {
        GpuPortabilityPolicy::RequirePortableBaseline
            if adapter.backend == GpuBackendFamily::UnknownBackend =>
        {
            return Err(GpuContextRequestError::new(
                GpuContextRequestErrorCategory::NoCandidate,
                "portable baseline requires a known backend family",
            ));
        }
        GpuPortabilityPolicy::RequirePortableBaseline => GpuPortabilityClass::PortableBaseline,
        GpuPortabilityPolicy::AllowBackendSpecialized
            if adapter.backend == GpuBackendFamily::UnknownBackend =>
        {
            GpuPortabilityClass::BackendSpecialized
        }
        GpuPortabilityPolicy::AllowBackendSpecialized => GpuPortabilityClass::PortableBaseline,
    };
    let effective_limits = effective_limits(descriptor, adapter.supported.limits());
    Ok(GpuCandidateAdmissionReport {
        adapter,
        enabled_features: enabled,
        degradations,
        portability,
        effective_limits,
    })
}

pub(crate) fn admitted_device_facts(
    candidate: &GpuCandidateAdmissionReport,
) -> GpuAdmittedDeviceFacts {
    GpuAdmittedDeviceFacts {
        enabled: candidate.enabled_features.clone(),
        effective_limits: candidate.effective_limits,
    }
}

fn adapter_alignment(facts: GpuAlignmentFacts, kind: GpuAlignmentKind) -> Option<u64> {
    match kind {
        GpuAlignmentKind::UniformDynamicOffset => facts.uniform_dynamic_offset,
        GpuAlignmentKind::StorageDynamicOffset => facts.storage_dynamic_offset,
        GpuAlignmentKind::CopyBufferOffset => facts.copy_buffer_offset,
        GpuAlignmentKind::BytesPerRow => facts.bytes_per_row,
        GpuAlignmentKind::QueryResolveDestination => facts.query_resolve_destination,
    }
}

fn limit_value(limits: GpuLimits, kind: GpuLimitKind) -> u64 {
    match kind {
        GpuLimitKind::MaxUniformBufferBindingSize => limits.max_uniform_buffer_binding_size(),
        GpuLimitKind::MaxStorageBufferBindingSize => limits.max_storage_buffer_binding_size(),
        GpuLimitKind::MaxColorAttachments => u64::from(limits.max_color_attachments()),
        GpuLimitKind::MaxVertexBuffers => u64::from(limits.max_vertex_buffers()),
        GpuLimitKind::MaxBindingsPerGroup => u64::from(limits.max_bindings_per_group()),
    }
}

fn effective_limits(descriptor: &GpuContextDescriptor, supported: GpuLimits) -> GpuLimits {
    let value = |kind| {
        descriptor
            .limits
            .get(&kind)
            .map(|constraint| {
                constraint
                    .minimum
                    .unwrap_or_else(|| limit_value(supported, kind))
                    .min(constraint.maximum.unwrap_or(u64::MAX))
            })
            .unwrap_or_else(|| limit_value(supported, kind))
    };
    GpuLimits::from_validated_adapter_facts(
        value(GpuLimitKind::MaxUniformBufferBindingSize),
        value(GpuLimitKind::MaxStorageBufferBindingSize),
        value(GpuLimitKind::MaxColorAttachments) as u32,
        value(GpuLimitKind::MaxVertexBuffers) as u32,
        value(GpuLimitKind::MaxBindingsPerGroup) as u32,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GpuCandidateSelection {
    pub(crate) candidate: GpuCandidateAdmissionReport,
    pub(crate) dispositions: Vec<GpuCandidateDisposition>,
}

pub(crate) fn select_candidate(
    descriptor: &GpuContextDescriptor,
    candidates: impl IntoIterator<Item = GpuAdapterFacts>,
    host_compatible: bool,
) -> Result<GpuCandidateSelection, GpuContextRequestError> {
    validate_descriptor(descriptor)?;
    let dispositions = candidates
        .into_iter()
        .map(
            |candidate| match evaluate_candidate(descriptor, candidate, host_compatible) {
                Ok(report) => GpuCandidateDisposition::Accepted(Box::new(report)),
                Err(error) => GpuCandidateDisposition::Rejected(error),
            },
        )
        .collect::<Vec<_>>();
    let mut admitted = dispositions
        .iter()
        .filter_map(|disposition| match disposition {
            GpuCandidateDisposition::Accepted(report) => Some(report.as_ref().clone()),
            GpuCandidateDisposition::Rejected(_) => None,
        })
        .collect::<Vec<_>>();
    admitted.sort_by_key(|candidate| candidate_rank(descriptor, candidate));
    let Some(best) = admitted.first().cloned() else {
        return Err(GpuContextRequestError::new(
            GpuContextRequestErrorCategory::NoCandidate,
            "no candidate satisfied the normalized request",
        )
        .with_candidate_dispositions(dispositions));
    };
    if admitted.get(1).is_some_and(|second| {
        candidate_rank(descriptor, second) == candidate_rank(descriptor, &best)
    }) {
        return Err(GpuContextRequestError::new(
            GpuContextRequestErrorCategory::AmbiguousAdapterSelection,
            "best candidates remain indistinguishable",
        )
        .with_candidate_dispositions(dispositions));
    }
    Ok(GpuCandidateSelection {
        candidate: best,
        dispositions,
    })
}

type CandidateRank = (u8, u8, u8, u8, u8, Option<u32>, Option<u32>, Option<String>);

fn candidate_rank(
    descriptor: &GpuContextDescriptor,
    candidate: &GpuCandidateAdmissionReport,
) -> CandidateRank {
    let adapter = candidate.adapter();
    let fallback =
        u8::from(adapter.software() == GpuSoftwareStatus::Software || adapter.fallback_requested());
    let power = match (descriptor.power_preference, adapter.class()) {
        (GpuPowerPreference::HighPerformance, GpuAdapterClass::Discrete)
        | (GpuPowerPreference::LowPower, GpuAdapterClass::Integrated) => 0,
        (_, GpuAdapterClass::Discrete) => 1,
        (_, GpuAdapterClass::Integrated) => 2,
        _ => 3,
    };
    let portability = match candidate.portability() {
        GpuPortabilityClass::PortableBaseline => 0,
        GpuPortabilityClass::PortableWithDeclaredExtensions => 1,
        GpuPortabilityClass::BackendSpecialized => 2,
        GpuPortabilityClass::Unsupported => 3,
    };
    let class = match adapter.class() {
        GpuAdapterClass::Discrete => 0,
        GpuAdapterClass::Integrated => 1,
        GpuAdapterClass::Virtual => 2,
        GpuAdapterClass::Cpu => 3,
        GpuAdapterClass::Other => 4,
        GpuAdapterClass::Unknown => 5,
    };
    let backend = match adapter.backend() {
        GpuBackendFamily::Vulkan => 0,
        GpuBackendFamily::Metal => 1,
        GpuBackendFamily::Direct3D12 => 2,
        GpuBackendFamily::OpenGl => 3,
        GpuBackendFamily::BrowserWebGpu => 4,
        GpuBackendFamily::UnknownBackend => 5,
    };
    (
        fallback,
        power,
        portability,
        class,
        backend,
        adapter.vendor(),
        adapter.device(),
        adapter.diagnostic_name().map(str::to_owned),
    )
}

fn sanitized_diagnostic(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.chars().take(MAX_DIAGNOSTIC_BYTES).collect())
}

#[cfg(test)]
pub(crate) fn reset_context_id_allocator_for_tests(next: u64) {
    NEXT_CONTEXT_ID.store(next, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::gpu::{
        GpuCapabilityRequirement, GpuPreferredFallback, GpuTextureFormat,
        GpuTextureFormatCapabilities,
    };

    fn adapter() -> GpuAdapterFacts {
        let limits = GpuLimits::new(4, 4, 4, 4, 4).unwrap();
        GpuAdapterFacts::new(
            GpuBackendFamily::Vulkan,
            GpuAdapterClass::Discrete,
            GpuSoftwareStatus::Hardware,
            false,
            GpuCapabilities::from_normalized_facts(
                [GpuCapabilityFeature::Compute, GpuCapabilityFeature::Copy],
                limits,
                [(
                    GpuTextureFormat::Rgba8Unorm,
                    GpuTextureFormatCapabilities::none(),
                )],
            ),
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
    fn identity_is_nonzero_unique_and_exhaustion_does_not_wrap() {
        reset_context_id_allocator_for_tests(1);
        let first = allocate_context_id().unwrap();
        let second = allocate_context_id().unwrap();
        assert_ne!(first, second);
        reset_context_id_allocator_for_tests(u64::MAX);
        assert!(allocate_context_id().is_ok());
        assert!(
            matches!(allocate_context_id(), Err(error) if error.category() == GpuContextRequestErrorCategory::IdentityExhausted)
        );
    }

    #[test]
    fn affinity_rejects_foreign_and_stale_values() {
        reset_context_id_allocator_for_tests(1);
        let one = allocate_context_id().unwrap();
        let two = allocate_context_id().unwrap();
        let generation = GpuDeviceGeneration::first();
        assert_eq!(
            validate_affinity(
                GpuContextAffinity {
                    context: one,
                    generation
                },
                GpuContextAffinity {
                    context: two,
                    generation
                }
            ),
            Err(GpuContextAffinityError::ForeignContext)
        );
        assert_eq!(
            validate_affinity(
                GpuContextAffinity {
                    context: one,
                    generation,
                },
                GpuContextAffinity {
                    context: one,
                    generation: GpuDeviceGeneration(NonZeroU64::new(2).unwrap()),
                }
            ),
            Err(GpuContextAffinityError::StaleGeneration)
        );
    }

    #[test]
    fn preferred_missing_feature_degrades_once_without_enabling_unrelated_features() {
        let mut requirements = GpuCapabilityRequirements::new();
        requirements
            .insert(GpuCapabilityRequirement::Preferred {
                feature: GpuCapabilityFeature::TimestampQuery,
                fallback: GpuPreferredFallback::DisableInstrumentation,
            })
            .unwrap();
        let report =
            evaluate_candidate(&GpuContextDescriptor::new(requirements), adapter(), true).unwrap();
        assert_eq!(report.degradations().len(), 1);
        assert!(
            !report
                .enabled_features()
                .any(|feature| feature == GpuCapabilityFeature::TimestampQuery)
        );
    }

    #[test]
    fn pure_admission_normalizes_limits_formats_and_alignments_before_backend_access() {
        let mut facts = GpuTextureFormatCapabilities::none();
        facts.copy_destination = true;
        let supported = GpuCapabilities::from_normalized_facts(
            [GpuCapabilityFeature::Compute],
            GpuLimits::new(2, 2, 2, 2, 2).unwrap(),
            [(GpuTextureFormat::Rgba8Unorm, facts)],
        );
        let candidate = GpuAdapterFacts::new(
            GpuBackendFamily::Vulkan,
            GpuAdapterClass::Discrete,
            GpuSoftwareStatus::Hardware,
            false,
            supported,
            adapter().alignments(),
        );
        let descriptor = GpuContextDescriptor::new(GpuCapabilityRequirements::new())
            .require_limit(GpuLimitKind::MaxUniformBufferBindingSize, 1)
            .permit_limit(GpuLimitKind::MaxUniformBufferBindingSize, 1)
            .require_format_role(GpuTextureFormat::Rgba8Unorm, GpuFormatRole::CopyDestination)
            .require_alignment(GpuAlignmentKind::BytesPerRow, 256);
        assert_eq!(
            evaluate_candidate(&descriptor, candidate, true)
                .unwrap()
                .effective_limits()
                .max_uniform_buffer_binding_size(),
            1
        );
        let contradiction = GpuContextDescriptor::new(GpuCapabilityRequirements::new())
            .require_limit(GpuLimitKind::MaxVertexBuffers, 2)
            .permit_limit(GpuLimitKind::MaxVertexBuffers, 1);
        assert!(matches!(
            evaluate_candidate(&contradiction, adapter(), true),
            Err(error) if error.category() == GpuContextRequestErrorCategory::ContradictoryRequest
        ));
    }

    #[test]
    fn candidate_selection_is_order_independent_and_rejects_equal_best_candidates() {
        let discrete = adapter();
        let integrated = GpuAdapterFacts::new(
            GpuBackendFamily::Vulkan,
            GpuAdapterClass::Integrated,
            GpuSoftwareStatus::Unknown,
            false,
            discrete.supported().clone(),
            discrete.alignments(),
        );
        let descriptor = GpuContextDescriptor::new(GpuCapabilityRequirements::new())
            .with_power_preference(GpuPowerPreference::HighPerformance);
        assert_eq!(
            select_candidate(&descriptor, [integrated.clone(), discrete.clone()], true)
                .unwrap()
                .candidate
                .adapter()
                .class(),
            GpuAdapterClass::Discrete
        );
        assert_eq!(
            select_candidate(&descriptor, [discrete.clone(), integrated], true)
                .unwrap()
                .candidate
                .adapter()
                .class(),
            GpuAdapterClass::Discrete
        );
        assert!(matches!(
            select_candidate(&descriptor, [discrete.clone(), discrete], true),
            Err(error) if error.category() == GpuContextRequestErrorCategory::AmbiguousAdapterSelection
        ));
    }

    #[test]
    fn descriptor_merge_is_order_independent_and_ignores_diagnostics() {
        let left = GpuContextDescriptor::new(GpuCapabilityRequirements::new())
            .with_label("first caller")
            .require_limit(GpuLimitKind::MaxVertexBuffers, 1)
            .permit_limit(GpuLimitKind::MaxVertexBuffers, 4)
            .require_alignment(GpuAlignmentKind::BytesPerRow, 512);
        let right = GpuContextDescriptor::new(GpuCapabilityRequirements::new())
            .with_provenance("second caller")
            .require_limit(GpuLimitKind::MaxVertexBuffers, 2)
            .permit_limit(GpuLimitKind::MaxVertexBuffers, 3)
            .require_alignment(GpuAlignmentKind::BytesPerRow, 256);
        let merged_left = left.merge(&right).unwrap();
        let merged_right = right.merge(&left).unwrap();
        assert!(merged_left.semantically_eq(&merged_right));
        let report = evaluate_candidate(&merged_left, adapter(), true).unwrap();
        assert_eq!(report.effective_limits().max_vertex_buffers(), 2);
        assert!(matches!(
            left.require_limit(GpuLimitKind::MaxVertexBuffers, 5)
                .merge(&right),
            Err(error) if error.category() == GpuContextRequestErrorCategory::ContradictoryRequest
        ));
    }

    #[test]
    fn rejected_candidates_are_retained_in_structured_no_candidate_outcome() {
        let descriptor = GpuContextDescriptor::new(GpuCapabilityRequirements::new())
            .with_allowed_backends([GpuBackendFamily::Metal]);
        let error = select_candidate(&descriptor, [adapter()], true).unwrap_err();
        assert_eq!(
            error.category(),
            GpuContextRequestErrorCategory::NoCandidate
        );
        assert_eq!(error.candidate_dispositions().len(), 1);
        assert!(matches!(
            error.candidate_dispositions(),
            [GpuCandidateDisposition::Rejected(rejection)]
                if rejection.category() == GpuContextRequestErrorCategory::BackendFamilyForbidden
        ));
    }
}
