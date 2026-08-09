use super::{
    GpuCapabilityAdmissionCause, GpuCapabilityAdmissionError, GpuCapabilityRequirementCause,
    GpuCapabilityRequirementError,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuCapabilityFeature {
    Compute,
    RenderPipeline,
    Copy,
    IndirectDraw,
    StorageTexture,
    TextureBindingArray,
    BufferBindingArray,
    StorageResourceBindingArray,
    UniformBufferBindingArray,
    DepthAttachment,
    TimestampQuery,
    Presentation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuPreferredFallback {
    ContinueWithoutFeature,
    DisableInstrumentation,
    SelectAlternativeWork,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuCapabilityRequirement {
    Required(GpuCapabilityFeature),
    Preferred {
        feature: GpuCapabilityFeature,
        fallback: GpuPreferredFallback,
    },
    Disabled(GpuCapabilityFeature),
}

impl GpuCapabilityRequirement {
    pub const fn feature(self) -> GpuCapabilityFeature {
        match self {
            Self::Required(feature) | Self::Preferred { feature, .. } | Self::Disabled(feature) => {
                feature
            }
        }
    }
}

/// Deterministically merged capability requirements.
///
/// ```
/// use engine::plugins::gpu::{
///     GpuCapabilityFeature, GpuCapabilityRequirement, GpuCapabilityRequirements,
/// };
///
/// let mut requirements = GpuCapabilityRequirements::new();
/// requirements
///     .insert(GpuCapabilityRequirement::Required(
///         GpuCapabilityFeature::Compute,
///     ))?;
/// assert!(requirements.get(GpuCapabilityFeature::Compute).is_some());
/// # Ok::<(), engine::plugins::gpu::GpuCapabilityRequirementError>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GpuCapabilityRequirements {
    entries: BTreeMap<GpuCapabilityFeature, GpuCapabilityRequirement>,
}

impl GpuCapabilityRequirements {
    pub const fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        requirement: GpuCapabilityRequirement,
    ) -> Result<(), GpuCapabilityRequirementError> {
        let feature = requirement.feature();
        let Some(existing) = self.entries.get(&feature).copied() else {
            self.entries.insert(feature, requirement);
            return Ok(());
        };
        let merged = merge_requirement(existing, requirement).map_err(|cause| {
            GpuCapabilityRequirementError::Invalid {
                operation: "merge GPU capability requirement",
                label: format!("{feature:?}"),
                cause,
                correction: "remove the conflicting consumer constraint or choose one preferred fallback",
            }
        })?;
        self.entries.insert(feature, merged);
        Ok(())
    }

    pub fn get(&self, feature: GpuCapabilityFeature) -> Option<GpuCapabilityRequirement> {
        self.entries.get(&feature).copied()
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = GpuCapabilityRequirement> + '_ {
        self.entries.values().copied()
    }

    pub fn merge(&self, other: &Self) -> Result<Self, GpuCapabilityRequirementError> {
        let mut merged = self.clone();
        for requirement in other.iter() {
            merged.insert(requirement)?;
        }
        Ok(merged)
    }
}

fn merge_requirement(
    left: GpuCapabilityRequirement,
    right: GpuCapabilityRequirement,
) -> Result<GpuCapabilityRequirement, GpuCapabilityRequirementCause> {
    use GpuCapabilityRequirement::{Disabled, Preferred, Required};
    match (left, right) {
        (Required(feature), Required(_))
        | (Required(feature), Preferred { .. })
        | (Preferred { feature, .. }, Required(_)) => Ok(Required(feature)),
        (
            Preferred {
                feature,
                fallback: left,
            },
            Preferred {
                fallback: right, ..
            },
        ) if left == right => Ok(Preferred {
            feature,
            fallback: left,
        }),
        (Preferred { .. }, Preferred { .. }) => {
            Err(GpuCapabilityRequirementCause::AmbiguousPreferredFallback)
        }
        (Disabled(feature), Disabled(_)) => Ok(Disabled(feature)),
        (Required(_), Disabled(_))
        | (Disabled(_), Required(_))
        | (Preferred { .. }, Disabled(_))
        | (Disabled(_), Preferred { .. }) => {
            Err(GpuCapabilityRequirementCause::ConflictingStrength)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuCapabilityProfile {
    ComputeBaseline,
    OffscreenGraphicsBaseline,
    DesktopPresentationBaseline,
}

/// A profile produces ordinary requirements that can be inspected and merged.
///
/// ```
/// use engine::plugins::gpu::{GpuCapabilityFeature, GpuCapabilityProfile};
/// let requirements = GpuCapabilityProfile::ComputeBaseline.requirements();
/// assert!(requirements.get(GpuCapabilityFeature::Compute).is_some());
/// ```
impl GpuCapabilityProfile {
    pub fn requirements(self) -> GpuCapabilityRequirements {
        let entries = match self {
            Self::ComputeBaseline => [
                (
                    GpuCapabilityFeature::Compute,
                    GpuCapabilityRequirement::Required(GpuCapabilityFeature::Compute),
                ),
                (
                    GpuCapabilityFeature::Copy,
                    GpuCapabilityRequirement::Required(GpuCapabilityFeature::Copy),
                ),
            ]
            .into_iter()
            .collect(),
            Self::OffscreenGraphicsBaseline => [
                (
                    GpuCapabilityFeature::Copy,
                    GpuCapabilityRequirement::Required(GpuCapabilityFeature::Copy),
                ),
                (
                    GpuCapabilityFeature::RenderPipeline,
                    GpuCapabilityRequirement::Required(GpuCapabilityFeature::RenderPipeline),
                ),
            ]
            .into_iter()
            .collect(),
            Self::DesktopPresentationBaseline => [
                (
                    GpuCapabilityFeature::Copy,
                    GpuCapabilityRequirement::Required(GpuCapabilityFeature::Copy),
                ),
                (
                    GpuCapabilityFeature::Presentation,
                    GpuCapabilityRequirement::Required(GpuCapabilityFeature::Presentation),
                ),
                (
                    GpuCapabilityFeature::RenderPipeline,
                    GpuCapabilityRequirement::Required(GpuCapabilityFeature::RenderPipeline),
                ),
            ]
            .into_iter()
            .collect(),
        };
        GpuCapabilityRequirements { entries }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuTextureFormat {
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Bgra8Unorm,
    Bgra8UnormSrgb,
    R32Uint,
    Depth32Float,
}

impl GpuTextureFormat {
    pub const fn bytes_per_texel(self) -> u32 {
        4
    }

    pub const fn is_depth(self) -> bool {
        matches!(self, Self::Depth32Float)
    }

    pub const fn is_srgb(self) -> bool {
        matches!(self, Self::Rgba8UnormSrgb | Self::Bgra8UnormSrgb)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuTextureFormatCapabilities {
    pub sampled: bool,
    pub filterable: bool,
    pub storage_read: bool,
    pub storage_write: bool,
    pub color_attachment: bool,
    pub depth_stencil: bool,
    pub copy_source: bool,
    pub copy_destination: bool,
    /// Texture-block dimensions are absent when the backend cannot report them.
    pub block_dimensions: Option<(u32, u32)>,
    /// Copy bytes per texture block are absent when the backend cannot report them.
    pub block_copy_size: Option<u32>,
}

impl GpuTextureFormatCapabilities {
    pub const fn none() -> Self {
        Self {
            sampled: false,
            filterable: false,
            storage_read: false,
            storage_write: false,
            color_attachment: false,
            depth_stencil: false,
            copy_source: false,
            copy_destination: false,
            block_dimensions: None,
            block_copy_size: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuLimits {
    max_uniform_buffer_binding_size: u64,
    max_storage_buffer_binding_size: u64,
    max_color_attachments: u32,
    max_vertex_buffers: u32,
    max_bindings_per_group: u32,
}

impl GpuLimits {
    pub fn new(
        max_uniform_buffer_binding_size: u64,
        max_storage_buffer_binding_size: u64,
        max_color_attachments: u32,
        max_vertex_buffers: u32,
        max_bindings_per_group: u32,
    ) -> Result<Self, GpuCapabilityAdmissionError> {
        if max_uniform_buffer_binding_size == 0
            || max_storage_buffer_binding_size == 0
            || max_color_attachments == 0
            || max_vertex_buffers == 0
            || max_bindings_per_group == 0
        {
            return Err(GpuCapabilityAdmissionError::Rejected {
                operation: "construct normalized GPU limits",
                label: "GPU limits".to_string(),
                cause: GpuCapabilityAdmissionCause::InvalidLimit,
                correction: "provide nonzero normalized limits",
            });
        }
        Ok(Self {
            max_uniform_buffer_binding_size,
            max_storage_buffer_binding_size,
            max_color_attachments,
            max_vertex_buffers,
            max_bindings_per_group,
        })
    }

    pub const fn max_uniform_buffer_binding_size(self) -> u64 {
        self.max_uniform_buffer_binding_size
    }
    pub const fn max_storage_buffer_binding_size(self) -> u64 {
        self.max_storage_buffer_binding_size
    }
    pub const fn max_color_attachments(self) -> u32 {
        self.max_color_attachments
    }
    pub const fn max_vertex_buffers(self) -> u32 {
        self.max_vertex_buffers
    }
    pub const fn max_bindings_per_group(self) -> u32 {
        self.max_bindings_per_group
    }

    pub(crate) const fn from_validated_adapter_facts(
        max_uniform_buffer_binding_size: u64,
        max_storage_buffer_binding_size: u64,
        max_color_attachments: u32,
        max_vertex_buffers: u32,
        max_bindings_per_group: u32,
    ) -> Self {
        Self {
            max_uniform_buffer_binding_size,
            max_storage_buffer_binding_size,
            max_color_attachments,
            max_vertex_buffers,
            max_bindings_per_group,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GpuCapabilities {
    features: BTreeSet<GpuCapabilityFeature>,
    limits: GpuLimits,
    formats: BTreeMap<GpuTextureFormat, GpuTextureFormatCapabilities>,
}

impl GpuCapabilities {
    pub fn from_normalized_facts(
        features: impl IntoIterator<Item = GpuCapabilityFeature>,
        limits: GpuLimits,
        formats: impl IntoIterator<Item = (GpuTextureFormat, GpuTextureFormatCapabilities)>,
    ) -> Self {
        Self {
            features: features.into_iter().collect(),
            limits,
            formats: formats.into_iter().collect(),
        }
    }

    pub fn supports(&self, feature: GpuCapabilityFeature) -> bool {
        self.features.contains(&feature)
    }

    /// Generic logical capability limits. G4A additionally exposes typed adapter,
    /// device, and workload facts so this value is never published as device truth.
    pub const fn limits(&self) -> GpuLimits {
        self.limits
    }

    pub fn format(&self, format: GpuTextureFormat) -> Option<GpuTextureFormatCapabilities> {
        self.formats.get(&format).copied()
    }

    pub fn features(&self) -> impl ExactSizeIterator<Item = GpuCapabilityFeature> + '_ {
        self.features.iter().copied()
    }

    pub fn formats(
        &self,
    ) -> impl ExactSizeIterator<Item = (GpuTextureFormat, GpuTextureFormatCapabilities)> + '_ {
        self.formats.iter().map(|(format, facts)| (*format, *facts))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuPreferredAvailability {
    pub feature: GpuCapabilityFeature,
    pub available: bool,
    pub enabled: bool,
    pub fallback: GpuPreferredFallback,
}

#[derive(Debug, Clone)]
pub struct GpuCapabilityAdmission {
    granted_required: Vec<GpuCapabilityFeature>,
    preferred: Vec<GpuPreferredAvailability>,
    verified_disabled: Vec<GpuCapabilityFeature>,
    diagnostics: Vec<String>,
}

impl PartialEq for GpuCapabilityAdmission {
    fn eq(&self, other: &Self) -> bool {
        self.granted_required == other.granted_required
            && self.preferred == other.preferred
            && self.verified_disabled == other.verified_disabled
    }
}

impl Eq for GpuCapabilityAdmission {}

impl GpuCapabilityAdmission {
    pub fn evaluate(
        label: impl Into<String>,
        requirements: &GpuCapabilityRequirements,
        capabilities: &GpuCapabilities,
        enabled_features: impl IntoIterator<Item = GpuCapabilityFeature>,
    ) -> Result<Self, GpuCapabilityAdmissionError> {
        let label = label.into();
        let enabled_features = enabled_features.into_iter().collect::<BTreeSet<_>>();
        if let Some(feature) = enabled_features
            .iter()
            .find(|feature| !capabilities.supports(**feature))
        {
            return Err(GpuCapabilityAdmissionError::Rejected {
                operation: "admit GPU capability requirements",
                label: format!("{label}::{feature:?}"),
                cause: GpuCapabilityAdmissionCause::EnabledUnavailable,
                correction: "enable only normalized features reported by the admitted backend",
            });
        }
        let mut granted_required = Vec::new();
        let mut preferred = Vec::new();
        let mut verified_disabled = Vec::new();
        let mut diagnostics = Vec::new();
        for requirement in requirements.iter() {
            match requirement {
                GpuCapabilityRequirement::Required(feature) => {
                    if !capabilities.supports(feature) {
                        return Err(GpuCapabilityAdmissionError::Rejected {
                            operation: "admit GPU capability requirements",
                            label,
                            cause: GpuCapabilityAdmissionCause::RequiredUnavailable,
                            correction: "select a capable backend or remove the required workload",
                        });
                    }
                    if !enabled_features.contains(&feature) {
                        return Err(GpuCapabilityAdmissionError::Rejected {
                            operation: "admit GPU capability requirements",
                            label,
                            cause: GpuCapabilityAdmissionCause::RequiredNotEnabled,
                            correction: "enable the required feature during backend admission",
                        });
                    }
                    granted_required.push(feature);
                }
                GpuCapabilityRequirement::Preferred { feature, fallback } => {
                    let available = capabilities.supports(feature);
                    let enabled = enabled_features.contains(&feature);
                    preferred.push(GpuPreferredAvailability {
                        feature,
                        available,
                        enabled,
                        fallback,
                    });
                    if !available {
                        diagnostics.push(format!(
                            "preferred {feature:?} unavailable; apply {fallback:?}"
                        ));
                    } else if !enabled {
                        diagnostics.push(format!(
                            "preferred {feature:?} not enabled; apply {fallback:?}"
                        ));
                    }
                }
                GpuCapabilityRequirement::Disabled(feature) => {
                    if enabled_features.contains(&feature) {
                        return Err(GpuCapabilityAdmissionError::Rejected {
                            operation: "admit GPU capability requirements",
                            label,
                            cause: GpuCapabilityAdmissionCause::DisabledEnabled,
                            correction: "disable the feature path before admission",
                        });
                    }
                    verified_disabled.push(feature);
                }
            }
        }
        Ok(Self {
            granted_required,
            preferred,
            verified_disabled,
            diagnostics,
        })
    }

    pub fn granted_required(&self) -> &[GpuCapabilityFeature] {
        &self.granted_required
    }
    pub fn preferred(&self) -> &[GpuPreferredAvailability] {
        &self.preferred
    }
    pub fn verified_disabled(&self) -> &[GpuCapabilityFeature] {
        &self.verified_disabled
    }
    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insertion_lookup_iteration_and_merge_are_deterministic() {
        let mut left = GpuCapabilityRequirements::new();
        left.insert(GpuCapabilityRequirement::Preferred {
            feature: GpuCapabilityFeature::TimestampQuery,
            fallback: GpuPreferredFallback::DisableInstrumentation,
        })
        .unwrap();
        let mut right = GpuCapabilityRequirements::new();
        right
            .insert(GpuCapabilityRequirement::Required(
                GpuCapabilityFeature::Compute,
            ))
            .unwrap();

        let left_right = left.merge(&right).unwrap();
        let right_left = right.merge(&left).unwrap();
        assert_eq!(left_right, right_left);
        assert_eq!(
            left_right.iter().collect::<Vec<_>>(),
            right_left.iter().collect::<Vec<_>>()
        );
        assert_eq!(
            left_right.get(GpuCapabilityFeature::Compute),
            Some(GpuCapabilityRequirement::Required(
                GpuCapabilityFeature::Compute
            ))
        );
    }

    #[test]
    fn conflicts_are_explicit() {
        let mut requirements = GpuCapabilityRequirements::new();
        requirements
            .insert(GpuCapabilityRequirement::Required(
                GpuCapabilityFeature::Compute,
            ))
            .unwrap();
        let error = requirements
            .insert(GpuCapabilityRequirement::Disabled(
                GpuCapabilityFeature::Compute,
            ))
            .unwrap_err();
        assert!(matches!(
            error,
            GpuCapabilityRequirementError::Invalid {
                cause: GpuCapabilityRequirementCause::ConflictingStrength,
                ..
            }
        ));

        let mut preferred = GpuCapabilityRequirements::new();
        preferred
            .insert(GpuCapabilityRequirement::Preferred {
                feature: GpuCapabilityFeature::TimestampQuery,
                fallback: GpuPreferredFallback::DisableInstrumentation,
            })
            .unwrap();
        let mut alternative = GpuCapabilityRequirements::new();
        alternative
            .insert(GpuCapabilityRequirement::Preferred {
                feature: GpuCapabilityFeature::TimestampQuery,
                fallback: GpuPreferredFallback::ContinueWithoutFeature,
            })
            .unwrap();
        assert!(matches!(
            preferred.merge(&alternative),
            Err(GpuCapabilityRequirementError::Invalid {
                cause: GpuCapabilityRequirementCause::AmbiguousPreferredFallback,
                ..
            })
        ));
        assert!(matches!(
            alternative.merge(&preferred),
            Err(GpuCapabilityRequirementError::Invalid {
                cause: GpuCapabilityRequirementCause::AmbiguousPreferredFallback,
                ..
            })
        ));
    }

    #[test]
    fn compatible_strength_merge_is_commutative() {
        let mut required = GpuCapabilityRequirements::new();
        required
            .insert(GpuCapabilityRequirement::Required(
                GpuCapabilityFeature::Compute,
            ))
            .unwrap();
        let mut preferred = GpuCapabilityRequirements::new();
        preferred
            .insert(GpuCapabilityRequirement::Preferred {
                feature: GpuCapabilityFeature::Compute,
                fallback: GpuPreferredFallback::SelectAlternativeWork,
            })
            .unwrap();

        assert_eq!(required.merge(&preferred), preferred.merge(&required));
        assert_eq!(
            required
                .merge(&preferred)
                .unwrap()
                .get(GpuCapabilityFeature::Compute),
            Some(GpuCapabilityRequirement::Required(
                GpuCapabilityFeature::Compute
            ))
        );
    }

    #[test]
    fn profiles_are_ordinary_requirements() {
        let requirements = GpuCapabilityProfile::ComputeBaseline.requirements();
        assert_eq!(requirements.iter().len(), 2);
        assert!(matches!(
            requirements.get(GpuCapabilityFeature::Compute),
            Some(GpuCapabilityRequirement::Required(_))
        ));
    }

    #[test]
    fn portable_baseline_profiles_do_not_require_native_binding_arrays() {
        let binding_array_features = [
            GpuCapabilityFeature::TextureBindingArray,
            GpuCapabilityFeature::BufferBindingArray,
            GpuCapabilityFeature::StorageResourceBindingArray,
            GpuCapabilityFeature::UniformBufferBindingArray,
        ];

        for profile in [
            GpuCapabilityProfile::ComputeBaseline,
            GpuCapabilityProfile::OffscreenGraphicsBaseline,
            GpuCapabilityProfile::DesktopPresentationBaseline,
        ] {
            let requirements = profile.requirements();
            for feature in binding_array_features {
                assert!(
                    requirements.get(feature).is_none(),
                    "{profile:?} must not require native binding-array feature {feature:?}"
                );
            }
        }
    }

    #[test]
    fn array_features_remain_unavailable_without_context_enablement() {
        let capabilities = GpuCapabilities::from_normalized_facts(
            [GpuCapabilityFeature::TextureBindingArray],
            GpuLimits::new(1, 1, 1, 1, 1).unwrap(),
            [],
        );
        let mut requirements = GpuCapabilityRequirements::new();
        requirements
            .insert(GpuCapabilityRequirement::Required(
                GpuCapabilityFeature::TextureBindingArray,
            ))
            .unwrap();

        assert!(matches!(
            GpuCapabilityAdmission::evaluate("array program", &requirements, &capabilities, []),
            Err(GpuCapabilityAdmissionError::Rejected {
                cause: GpuCapabilityAdmissionCause::RequiredNotEnabled,
                ..
            })
        ));
    }

    #[test]
    fn admission_distinguishes_availability_from_enablement() {
        let capabilities = GpuCapabilities::from_normalized_facts(
            [
                GpuCapabilityFeature::Compute,
                GpuCapabilityFeature::TimestampQuery,
            ],
            GpuLimits::new(1, 1, 1, 1, 1).unwrap(),
            [],
        );
        let mut requirements = GpuCapabilityRequirements::new();
        requirements
            .insert(GpuCapabilityRequirement::Required(
                GpuCapabilityFeature::Compute,
            ))
            .unwrap();
        requirements
            .insert(GpuCapabilityRequirement::Disabled(
                GpuCapabilityFeature::TimestampQuery,
            ))
            .unwrap();

        let admission = GpuCapabilityAdmission::evaluate(
            "compute",
            &requirements,
            &capabilities,
            [GpuCapabilityFeature::Compute],
        )
        .unwrap();
        assert_eq!(
            admission.verified_disabled(),
            &[GpuCapabilityFeature::TimestampQuery]
        );
        assert!(matches!(
            GpuCapabilityAdmission::evaluate(
                "compute",
                &requirements,
                &capabilities,
                [
                    GpuCapabilityFeature::Compute,
                    GpuCapabilityFeature::TimestampQuery,
                ],
            ),
            Err(GpuCapabilityAdmissionError::Rejected {
                cause: GpuCapabilityAdmissionCause::DisabledEnabled,
                ..
            })
        ));
        assert!(matches!(
            GpuCapabilityAdmission::evaluate("compute", &requirements, &capabilities, []),
            Err(GpuCapabilityAdmissionError::Rejected {
                cause: GpuCapabilityAdmissionCause::RequiredNotEnabled,
                ..
            })
        ));
    }

    #[test]
    fn admission_diagnostics_do_not_change_semantic_equality() {
        let facts = GpuPreferredAvailability {
            feature: GpuCapabilityFeature::TimestampQuery,
            available: false,
            enabled: false,
            fallback: GpuPreferredFallback::DisableInstrumentation,
        };
        let first = GpuCapabilityAdmission {
            granted_required: vec![GpuCapabilityFeature::Compute],
            preferred: vec![facts],
            verified_disabled: vec![GpuCapabilityFeature::Presentation],
            diagnostics: vec!["first diagnostic wording".to_string()],
        };
        let second = GpuCapabilityAdmission {
            granted_required: vec![GpuCapabilityFeature::Compute],
            preferred: vec![facts],
            verified_disabled: vec![GpuCapabilityFeature::Presentation],
            diagnostics: vec!["different diagnostic wording".to_string()],
        };

        assert_eq!(first, second);
        assert_ne!(first.diagnostics(), second.diagnostics());
    }
}
