#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderScaleVisibilityCapabilityStatus {
    Supported,
    Unsupported,
}

impl RenderScaleVisibilityCapabilityStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderScaleVisibilityCapabilities {
    pub storage_compaction: RenderScaleVisibilityCapabilityStatus,
    pub indirect_submission: RenderScaleVisibilityCapabilityStatus,
}

impl RenderScaleVisibilityCapabilities {
    pub fn supported() -> Self {
        Self {
            storage_compaction: RenderScaleVisibilityCapabilityStatus::Supported,
            indirect_submission: RenderScaleVisibilityCapabilityStatus::Supported,
        }
    }

    pub fn supports_gpu_driven_submission(self) -> bool {
        self.storage_compaction == RenderScaleVisibilityCapabilityStatus::Supported
            && self.indirect_submission == RenderScaleVisibilityCapabilityStatus::Supported
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderScaleVisibilityConfig {
    pub frustum_extent: [f32; 3],
    pub min_screen_size_px: f32,
    pub near_lod_screen_size_px: f32,
    pub medium_lod_screen_size_px: f32,
    pub max_visible_candidates: usize,
}

impl Default for RenderScaleVisibilityConfig {
    fn default() -> Self {
        Self {
            frustum_extent: [1.0, 1.0, 1.0],
            min_screen_size_px: 1.0,
            near_lod_screen_size_px: 96.0,
            medium_lod_screen_size_px: 24.0,
            max_visible_candidates: 65_536,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderScaleVisibilityCandidate {
    pub product_id: u64,
    pub cache_id: String,
    pub center: [f32; 3],
    pub radius: f32,
    pub screen_size_px: f32,
    pub resident_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderScaleCullingReason {
    Visible,
    OutsideFrustum,
    BelowScreenSize,
    InvalidBounds,
    CompactionBudgetExceeded,
}

impl RenderScaleCullingReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Visible => "visible",
            Self::OutsideFrustum => "outside_frustum",
            Self::BelowScreenSize => "below_screen_size",
            Self::InvalidBounds => "invalid_bounds",
            Self::CompactionBudgetExceeded => "compaction_budget_exceeded",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderScaleLodBand {
    Near,
    Medium,
    Far,
    Culled,
}

impl RenderScaleLodBand {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Near => "near",
            Self::Medium => "medium",
            Self::Far => "far",
            Self::Culled => "culled",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderScaleVisibilityRecord {
    pub product_id: u64,
    pub cache_id: String,
    pub culling_reason: String,
    pub lod_band: String,
    pub submitted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderScaleVisibilityDiagnostic {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderScaleVisibilityInspection {
    pub resident_count: usize,
    pub visible_count: usize,
    pub culled_count: usize,
    pub compacted_count: usize,
    pub submitted_draw_count: usize,
    pub indirect_command_count: usize,
    pub storage_compaction_status: String,
    pub indirect_submission_status: String,
    pub diagnostics: Vec<RenderScaleVisibilityDiagnostic>,
    pub records: Vec<RenderScaleVisibilityRecord>,
}

pub fn inspect_render_scale_visibility(
    candidates: &[RenderScaleVisibilityCandidate],
    config: RenderScaleVisibilityConfig,
    capabilities: RenderScaleVisibilityCapabilities,
) -> RenderScaleVisibilityInspection {
    let mut diagnostics = Vec::new();
    if capabilities.storage_compaction == RenderScaleVisibilityCapabilityStatus::Unsupported {
        diagnostics.push(RenderScaleVisibilityDiagnostic {
            code: "storage_compaction_unsupported".to_string(),
            message: "renderer scale visibility cannot compact visible candidates on this backend"
                .to_string(),
        });
    }
    if capabilities.indirect_submission == RenderScaleVisibilityCapabilityStatus::Unsupported {
        diagnostics.push(RenderScaleVisibilityDiagnostic {
            code: "indirect_submission_unsupported".to_string(),
            message:
                "renderer scale visibility cannot emit indirect submission commands on this backend"
                    .to_string(),
        });
    }

    let mut visible_indices = Vec::new();
    let mut records = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let reason = culling_reason(candidate, config);
        let lod_band = lod_band(candidate, config, reason);
        if reason == RenderScaleCullingReason::Visible {
            visible_indices.push(records.len());
        }
        records.push(RenderScaleVisibilityRecord {
            product_id: candidate.product_id,
            cache_id: candidate.cache_id.clone(),
            culling_reason: reason.as_str().to_string(),
            lod_band: lod_band.as_str().to_string(),
            submitted: false,
        });
    }

    let visible_count = visible_indices.len();
    let compacted_count = visible_count.min(config.max_visible_candidates);
    if visible_count > compacted_count {
        diagnostics.push(RenderScaleVisibilityDiagnostic {
            code: "visible_compaction_budget_exceeded".to_string(),
            message: format!(
                "renderer scale visibility compacted {compacted_count} of {visible_count} visible candidates"
            ),
        });
        for index in &visible_indices[compacted_count..] {
            records[*index].culling_reason = RenderScaleCullingReason::CompactionBudgetExceeded
                .as_str()
                .to_string();
            records[*index].lod_band = RenderScaleLodBand::Culled.as_str().to_string();
        }
    }

    let submitted_draw_count = if capabilities.supports_gpu_driven_submission() {
        for index in &visible_indices[..compacted_count] {
            records[*index].submitted = true;
        }
        compacted_count
    } else {
        0
    };
    let indirect_command_count = usize::from(submitted_draw_count > 0);

    RenderScaleVisibilityInspection {
        resident_count: candidates.len(),
        visible_count,
        culled_count: candidates.len().saturating_sub(visible_count),
        compacted_count,
        submitted_draw_count,
        indirect_command_count,
        storage_compaction_status: capabilities.storage_compaction.as_str().to_string(),
        indirect_submission_status: capabilities.indirect_submission.as_str().to_string(),
        diagnostics,
        records,
    }
}

fn culling_reason(
    candidate: &RenderScaleVisibilityCandidate,
    config: RenderScaleVisibilityConfig,
) -> RenderScaleCullingReason {
    if !candidate.radius.is_finite()
        || !candidate.screen_size_px.is_finite()
        || candidate.radius < 0.0
        || candidate.center.iter().any(|value| !value.is_finite())
    {
        return RenderScaleCullingReason::InvalidBounds;
    }
    if candidate.screen_size_px < config.min_screen_size_px {
        return RenderScaleCullingReason::BelowScreenSize;
    }
    let outside = candidate
        .center
        .iter()
        .zip(config.frustum_extent)
        .any(|(center, extent)| center.abs() - candidate.radius > extent);
    if outside {
        return RenderScaleCullingReason::OutsideFrustum;
    }
    RenderScaleCullingReason::Visible
}

fn lod_band(
    candidate: &RenderScaleVisibilityCandidate,
    config: RenderScaleVisibilityConfig,
    reason: RenderScaleCullingReason,
) -> RenderScaleLodBand {
    if reason != RenderScaleCullingReason::Visible {
        return RenderScaleLodBand::Culled;
    }
    if candidate.screen_size_px >= config.near_lod_screen_size_px {
        RenderScaleLodBand::Near
    } else if candidate.screen_size_px >= config.medium_lod_screen_size_px {
        RenderScaleLodBand::Medium
    } else {
        RenderScaleLodBand::Far
    }
}
