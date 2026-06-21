//! App-owned mounted-content liveness and fallback policy.

use std::collections::BTreeMap;

use ui_composition::{
    ContentLiveness, ContentProjectionFallback, MountedUnitDefinition, MountedUnitId,
    select_content_projection_fallback,
};

use super::{
    DrawingCompositionDiagnosticCode as Code, DrawingCompositionDiagnosticRecord as Record,
    DrawingCompositionDiagnosticStage as Stage, DrawingCompositionDiagnosticSubject as Subject,
    DrawingCompositionRejection, DrawingCompositionRuntime, DrawingMountedUnitExtensionV1,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawingCompositionContentState {
    liveness: BTreeMap<MountedUnitId, ContentLiveness>,
}

impl DrawingCompositionContentState {
    pub fn resolved(runtime: &DrawingCompositionRuntime) -> Self {
        Self {
            liveness: runtime
                .composition()
                .definition()
                .mounted_units()
                .iter()
                .map(|unit| (unit.id, ContentLiveness::Resolved))
                .collect(),
        }
    }

    pub fn liveness(&self, mounted_unit: MountedUnitId) -> Option<ContentLiveness> {
        self.liveness.get(&mounted_unit).copied()
    }

    pub fn observations(&self) -> &BTreeMap<MountedUnitId, ContentLiveness> {
        &self.liveness
    }

    pub fn set_liveness(
        &mut self,
        runtime: &DrawingCompositionRuntime,
        mounted_unit: MountedUnitId,
        liveness: ContentLiveness,
    ) -> Result<(), DrawingCompositionRejection> {
        if runtime
            .composition()
            .snapshot()
            .mounted_unit(mounted_unit)
            .is_none()
        {
            return Err(DrawingCompositionRejection::single(Record::error(
                Code::ContentUnitUnknown,
                Stage::Content,
                Subject::MountedUnit(mounted_unit),
                "Update liveness only for a mounted unit in the active Draw composition.",
            )));
        }
        self.liveness.insert(mounted_unit, liveness);
        Ok(())
    }
}

pub fn select_drawing_content_fallback(
    unit: &MountedUnitDefinition,
    extension: &DrawingMountedUnitExtensionV1,
    liveness: ContentLiveness,
    neutral_placeholder_available: bool,
    host_accepts_hide: bool,
) -> Result<ContentProjectionFallback, DrawingCompositionRejection> {
    select_content_projection_fallback(
        liveness,
        extension
            .unavailable_projection
            .app_projection_available(),
        neutral_placeholder_available,
        unit.unavailable_policy(),
        host_accepts_hide,
    )
    .ok_or_else(|| {
        DrawingCompositionRejection::single(
            Record::error(
                Code::ContentFallbackExhausted,
                Stage::Content,
                Subject::MountedUnit(unit.id),
                "Provide a Draw unavailable projection or neutral placeholder, or explicitly permit hiding.",
            )
            .with_context("liveness", liveness_name(liveness)),
        )
    })
}

pub fn unavailable_content_diagnostic(
    mounted_unit: MountedUnitId,
    liveness: ContentLiveness,
    fallback: ContentProjectionFallback,
) -> Option<Record> {
    if matches!(liveness, ContentLiveness::Resolved) {
        return None;
    }
    Some(
        Record::warning(
            Code::ContentUnavailable,
            Stage::Content,
            Subject::MountedUnit(mounted_unit),
            "Draw content is unavailable; project the selected deterministic fallback without changing composition structure.",
        )
        .with_context("fallback", fallback_name(fallback))
        .with_context("liveness", liveness_name(liveness)),
    )
}

pub const fn liveness_name(liveness: ContentLiveness) -> &'static str {
    match liveness {
        ContentLiveness::Resolved => "resolved",
        ContentLiveness::Missing => "missing",
        ContentLiveness::Loading => "loading",
        ContentLiveness::Suspended => "suspended",
        ContentLiveness::Denied => "denied",
        ContentLiveness::UnsupportedProfile => "unsupported_profile",
        ContentLiveness::Crashed => "crashed",
    }
}

pub const fn fallback_name(fallback: ContentProjectionFallback) -> &'static str {
    match fallback {
        ContentProjectionFallback::ResolvedContent => "resolved_content",
        ContentProjectionFallback::AppProvidedUnavailable => "app_provided_unavailable",
        ContentProjectionFallback::NeutralDiagnosticPlaceholder => "neutral_diagnostic_placeholder",
        ContentProjectionFallback::Hidden => "hidden",
    }
}
